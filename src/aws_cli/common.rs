use aws_credential_types::provider::ProvideCredentials;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use tokio::runtime::Runtime;

static REGION: Mutex<Option<String>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AwsAuthErrorCode {
    CredentialsProviderMissing,
    CredentialsLoadFailed,
    CallerIdentityFailed,
    Network,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwsAuthError {
    pub code: AwsAuthErrorCode,
    pub detail: String,
}

impl AwsAuthError {
    fn new(code: AwsAuthErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

pub fn set_region(region: &str) {
    if let Ok(mut r) = REGION.lock() {
        *r = Some(region.to_string());
    }
}

pub fn set_aws_profile(profile: &str) {
    let profile = profile.trim();
    if profile.is_empty() {
        unsafe {
            std::env::remove_var("AWS_PROFILE");
            std::env::remove_var("AWS_DEFAULT_PROFILE");
        }
        return;
    }

    unsafe {
        std::env::set_var("AWS_PROFILE", profile);
        std::env::set_var("AWS_DEFAULT_PROFILE", profile);
    }
}

pub fn list_aws_profiles() -> Result<Vec<String>, String> {
    let home = dirs::home_dir()
        .ok_or_else(|| "Home directory not found; cannot read AWS profile files.".to_string())?;
    let mut profiles = BTreeSet::new();
    let mut had_profile_source = false;

    let config_path = home.join(".aws").join("config");
    if config_path.exists() {
        had_profile_source = true;
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read {}: {}", config_path.display(), e))?;
        parse_profile_sections(&content, true, &mut profiles);
    }

    let credentials_path = home.join(".aws").join("credentials");
    if credentials_path.exists() {
        had_profile_source = true;
        let content = std::fs::read_to_string(&credentials_path)
            .map_err(|e| format!("Failed to read {}: {}", credentials_path.display(), e))?;
        parse_profile_sections(&content, false, &mut profiles);
    }

    if !had_profile_source {
        return Ok(Vec::new());
    }

    Ok(profiles.into_iter().collect())
}

fn parse_profile_sections(contents: &str, from_config: bool, profiles: &mut BTreeSet<String>) {
    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if !line.starts_with('[') || !line.ends_with(']') {
            continue;
        }

        let section = line
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim()
            .to_string();
        if section.is_empty() {
            continue;
        }

        if from_config {
            if section == "default" {
                profiles.insert("default".to_string());
                continue;
            }

            if let Some(name) = section.strip_prefix("profile ") {
                let name = name.trim();
                if !name.is_empty() {
                    profiles.insert(name.to_string());
                }
            }
            continue;
        }

        profiles.insert(section);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AwsResource {
    pub name: String,
    pub id: String,
    pub state: String,
    pub az: String,
    pub cidr: String,
}

impl AwsResource {
    pub fn display(&self) -> String {
        if self.name.is_empty() {
            self.id.clone()
        } else {
            format!("{} ({})", self.name, self.id)
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    pub key: String,
    pub value: String,
}

pub fn run_aws_cli(args: &[&str]) -> Option<String> {
    if args.len() < 2 {
        tracing::warn!(
            args_len = args.len(),
            "run_aws_cli called with insufficient arguments"
        );
        return None;
    }

    let service = args[0];
    let operation = args[1];
    let request_args = args.join(" ");
    let started_at = Instant::now();
    let profile_name = std::env::var("AWS_PROFILE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("AWS_DEFAULT_PROFILE")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "default".to_string());

    tracing::info!(
        service = %service,
        operation = %operation,
        args = %request_args,
        "AWS CLI emulation request start"
    );

    let result = get_runtime().block_on(async {
        let config = get_sdk_config().await;
        let region = config
            .region()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        tracing::debug!(
            profile = %profile_name,
            region = %region,
            "AWS SDK config loaded for emulation request"
        );

        if let Some(credentials_provider) = config.credentials_provider()
            && let Err(error) = credentials_provider.provide_credentials().await
        {
            tracing::error!(
                error = %error,
                profile = %profile_name,
                region = %region,
                service = %service,
                operation = %operation,
                args = %request_args,
                "AWS credential provider unavailable; skipping SDK request"
            );
            return None;
        }

        match service {
            "ec2" => run_ec2_request(&config, operation, args).await,
            "ecr" => run_ecr_request(&config, operation, args).await,
            "elbv2" => run_elbv2_request(&config, operation, args).await,
            "iam" => run_iam_request(&config, operation, args).await,
            "sts" => run_sts_request(&config, operation).await,
            _ => {
                tracing::warn!(
                    service = %service,
                    operation = %operation,
                    "Unsupported AWS service requested"
                );
                None
            }
        }
    });

    let elapsed_ms = started_at.elapsed().as_millis();
    if let Some(ref output) = result {
        tracing::debug!(
            service = %service,
            operation = %operation,
            bytes = output.len(),
            elapsed_ms,
            "AWS CLI emulation request success"
        );
    } else {
        tracing::warn!(
            service = %service,
            operation = %operation,
            args = %request_args,
            elapsed_ms,
            "AWS CLI emulation request returned no result"
        );
    }

    result
}

pub fn check_aws_login() -> Result<String, AwsAuthError> {
    let started_at = std::time::Instant::now();
    get_runtime().block_on(async {
        tracing::info!("AWS login check started");
        let config = get_sdk_config().await;
        let region = config
            .region()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let profile = std::env::var("AWS_PROFILE")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                std::env::var("AWS_DEFAULT_PROFILE")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
            })
            .unwrap_or_else(|| "default".to_string());
        let region_from_env = std::env::var("AWS_REGION")
            .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
            .unwrap_or_else(|_| "-".to_string());
        let has_static_credentials = std::env::var("AWS_ACCESS_KEY_ID").is_ok()
            || std::env::var("AWS_SECRET_ACCESS_KEY").is_ok()
            || std::env::var("AWS_SESSION_TOKEN").is_ok();
        tracing::debug!(
            profile = %profile,
            region = %region,
            sdk_region = %region,
            region_from_env = %region_from_env,
            has_static_credentials = has_static_credentials,
            "AWS SDK config loaded for login check"
        );

        let credentials_provider = config.credentials_provider().ok_or_else(|| {
            tracing::warn!(
                profile = %profile,
                region = %region,
                "AWS login check failed: no credential provider found"
            );
            AwsAuthError::new(
                AwsAuthErrorCode::CredentialsProviderMissing,
                "no credential provider found",
            )
        })?;

        if let Err(error) = credentials_provider.provide_credentials().await {
            tracing::warn!(
                error = %error,
                profile = %profile,
                "AWS credential provider validation failed"
            );
            return Err(AwsAuthError::new(
                AwsAuthErrorCode::CredentialsLoadFailed,
                error.to_string(),
            ));
        }
        tracing::debug!("AWS credential provider returned credentials");

        let client = aws_sdk_sts::Client::new(&config);

        match client.get_caller_identity().send().await {
            Ok(output) => {
                let account = output.account().unwrap_or_default();
                let arn = output.arn().unwrap_or_default();
                let elapsed_ms = started_at.elapsed().as_millis();
                tracing::info!(
                    account = %account,
                    arn = %arn,
                    profile = %profile,
                    elapsed_ms = elapsed_ms,
                    "AWS caller identity verified"
                );
                Ok(format!("{} ({})", account, arn))
            }
            Err(e) => {
                let elapsed_ms = started_at.elapsed().as_millis();
                let error_text = e.to_string();
                tracing::warn!(
                    profile = %profile,
                    region = %region,
                    error = %e,
                    elapsed_ms = elapsed_ms,
                    "Caller identity check failed"
                );
                if is_auth_failure_error(&error_text) {
                    Err(AwsAuthError::new(
                        AwsAuthErrorCode::CallerIdentityFailed,
                        error_text,
                    ))
                } else {
                    let code = if is_network_error(&error_text) {
                        AwsAuthErrorCode::Network
                    } else {
                        AwsAuthErrorCode::Unknown
                    };
                    Err(AwsAuthError::new(code, error_text))
                }
            }
        }
    })
}

fn is_auth_failure_error(error_text: &str) -> bool {
    let lower = error_text.to_ascii_lowercase();
    const AUTH_MARKERS: [&str; 11] = [
        "expiredtoken",
        "accessdenied",
        "access denied",
        "invalidclienttokenid",
        "unrecognizedclientexception",
        "signaturedoesnotmatch",
        "security token included in the request is invalid",
        "failed to refresh cached login token",
        "refresh token has expired",
        "token has expired",
        "unauthorized",
    ];
    AUTH_MARKERS.iter().any(|marker| lower.contains(marker))
}

fn is_network_error(error_text: &str) -> bool {
    let lower = error_text.to_ascii_lowercase();
    const NETWORK_MARKERS: [&str; 9] = [
        "could not connect",
        "connection refused",
        "connection reset",
        "timed out",
        "timeout",
        "dns error",
        "name or service not known",
        "dispatch failure",
        "network is unreachable",
    ];
    NETWORK_MARKERS.iter().any(|marker| lower.contains(marker))
}

fn arg_value<'a>(args: &'a [&str], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find_map(|pair| (pair[0] == flag).then_some(pair[1]))
}

fn parse_filter_value(raw: &str, expected_name: &str) -> Option<String> {
    let segments = raw.split(',').collect::<Vec<_>>();
    let mut name = None;
    let mut values: Option<String> = None;
    let mut index = 0;

    while index < segments.len() {
        if let Some((k, v)) = segments[index].split_once('=') {
            match k {
                "Name" => name = Some(v),
                "Values" => {
                    let mut collected_values = vec![v.to_string()];
                    let mut next = index + 1;
                    while next < segments.len() && !segments[next].contains('=') {
                        collected_values.push(segments[next].to_string());
                        next += 1;
                    }
                    values = Some(collected_values.join(","));
                    index = next;
                    continue;
                }
                _ => {}
            }
        }

        index += 1;
    }

    if name == Some(expected_name) {
        values
    } else {
        None
    }
}

fn parse_tags_ec2(tags: &[aws_sdk_ec2::types::Tag]) -> Vec<Value> {
    tags.iter()
        .map(|t| {
            json!({
                "Key": t.key().unwrap_or_default(),
                "Value": t.value().unwrap_or_default()
            })
        })
        .collect()
}

fn parse_tags_iam(tags: &[aws_sdk_iam::types::Tag]) -> Vec<Value> {
    tags.iter()
        .map(|t| {
            json!({
                "Key": t.key(),
                "Value": t.value()
            })
        })
        .collect()
}

fn value_to_json_string(value: Value) -> Option<String> {
    serde_json::to_string(&value).ok()
}

async fn run_ec2_request(
    config: &aws_config::SdkConfig,
    operation: &str,
    args: &[&str],
) -> Option<String> {
    let client = aws_sdk_ec2::Client::new(config);

    match operation {
        "describe-instances" => ec2_describe_instances(&client, args).await,
        "describe-volumes" => ec2_describe_volumes(&client, args).await,
        "describe-instance-attribute" => ec2_describe_instance_attribute(&client, args).await,
        "describe-vpcs" => ec2_describe_vpcs(&client, args).await,
        "describe-subnets" => ec2_describe_subnets(&client, args).await,
        "describe-internet-gateways" => ec2_describe_internet_gateways(&client, args).await,
        "describe-nat-gateways" => ec2_describe_nat_gateways(&client, args).await,
        "describe-route-tables" => ec2_describe_route_tables(&client, args).await,
        "describe-addresses" => ec2_describe_addresses(&client).await,
        "describe-vpc-attribute" => ec2_describe_vpc_attribute(&client, args).await,
        "describe-security-groups" => ec2_describe_security_groups(&client, args).await,
        "describe-images" => ec2_describe_images(&client, args).await,
        _ => None,
    }
}

async fn ec2_describe_instances(client: &aws_sdk_ec2::Client, args: &[&str]) -> Option<String> {
    let mut req = client.describe_instances();

    if let Some(instance_id) = arg_value(args, "--instance-ids") {
        req = req.instance_ids(instance_id);
    }

    let output = match req.send().await {
        Ok(output) => output,
        Err(error) => {
            tracing::error!(
                error = %error,
                args = %args.join(" "),
                "describe-instances API call failed"
            );
            return None;
        }
    };
    ec2_describe_instances_output(output.reservations(), arg_value(args, "--query"))
}

fn ec2_describe_instances_output(
    reservations: &[aws_sdk_ec2::types::Reservation],
    query: Option<&str>,
) -> Option<String> {
    if query == Some("Reservations[*].Instances[*].[InstanceId,State.Name,Tags]") {
        let mut by_reservation = Vec::new();

        for reservation in reservations {
            let mut instances = reservation
                .instances()
                .iter()
                .map(|instance| {
                    let id = instance.instance_id().unwrap_or_default();
                    let state = instance
                        .state()
                        .and_then(|s| s.name())
                        .map(|s| s.as_str())
                        .unwrap_or("unknown");
                    let tags = parse_tags_ec2(instance.tags());

                    json!([id, state, tags])
                })
                .collect::<Vec<_>>();

            instances.sort_by(|a, b| {
                let a_id = a.get(0).and_then(Value::as_str).unwrap_or_default();
                let b_id = b.get(0).and_then(Value::as_str).unwrap_or_default();
                a_id.cmp(b_id)
            });
            by_reservation.push(Value::Array(instances));
        }

        return value_to_json_string(Value::Array(by_reservation));
    }

    if query == Some("Reservations[0].Instances[0].BlockDeviceMappings") {
        let mappings = reservations
            .first()
            .and_then(|r| r.instances().first())
            .map(|instance| {
                instance
                    .block_device_mappings()
                    .iter()
                    .map(|bdm| {
                        let ebs = bdm.ebs();
                        json!({
                            "DeviceName": bdm.device_name().unwrap_or_default(),
                            "Ebs": {
                                "VolumeId": ebs.and_then(|e| e.volume_id()).unwrap_or_default(),
                                "DeleteOnTermination": ebs.and_then(|e| e.delete_on_termination()).unwrap_or(false)
                            }
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        return value_to_json_string(Value::Array(mappings));
    }

    let reservations = reservations
        .iter()
        .map(|reservation| {
            let mut instances = reservation
                .instances()
                .iter()
                .map(|instance| {
                    let tags = parse_tags_ec2(instance.tags());
                    let security_groups = instance
                        .security_groups()
                        .iter()
                        .map(|sg| {
                            json!({
                                "GroupName": sg.group_name().unwrap_or_default(),
                                "GroupId": sg.group_id().unwrap_or_default()
                            })
                        })
                        .collect::<Vec<_>>();
                    let block_device_mappings = instance
                        .block_device_mappings()
                        .iter()
                        .map(|bdm| {
                            let ebs = bdm.ebs();
                            json!({
                                "DeviceName": bdm.device_name().unwrap_or_default(),
                                "Ebs": {
                                    "VolumeId": ebs.and_then(|e| e.volume_id()).unwrap_or_default(),
                                    "DeleteOnTermination": ebs.and_then(|e| e.delete_on_termination()).unwrap_or(false)
                                }
                            })
                        })
                        .collect::<Vec<_>>();

                    json!({
                        "InstanceId": instance.instance_id().unwrap_or_default(),
                        "InstanceType": instance.instance_type().map(|v| v.as_str()).unwrap_or_default(),
                        "ImageId": instance.image_id().unwrap_or_default(),
                        "Platform": instance.platform().map(|v| v.as_str()).unwrap_or("Linux"),
                        "Architecture": instance.architecture().map(|v| v.as_str()).unwrap_or("x86_64"),
                        "KeyName": instance.key_name().unwrap_or("-"),
                        "VpcId": instance.vpc_id().unwrap_or_default(),
                        "SubnetId": instance.subnet_id().unwrap_or_default(),
                        "Placement": {
                            "AvailabilityZone": instance
                                .placement()
                                .and_then(|p| p.availability_zone())
                                .unwrap_or_default()
                        },
                        "PublicIpAddress": instance.public_ip_address().unwrap_or("-"),
                        "PrivateIpAddress": instance.private_ip_address().unwrap_or_default(),
                        "State": {
                            "Name": instance
                                .state()
                                .and_then(|s| s.name())
                                .map(|v| v.as_str())
                                .unwrap_or("unknown")
                        },
                        "Monitoring": {
                            "State": instance
                                .monitoring()
                                .and_then(|m| m.state())
                                .map(|v| v.as_str())
                                .unwrap_or("disabled")
                        },
                        "EbsOptimized": instance.ebs_optimized().unwrap_or(false),
                        "IamInstanceProfile": {
                            "Arn": instance
                                .iam_instance_profile()
                                .and_then(|p| p.arn())
                                .unwrap_or_default()
                        },
                        "LaunchTime": instance
                            .launch_time()
                            .map(|t| t.fmt(aws_sdk_ec2::primitives::DateTimeFormat::DateTime).unwrap_or_default())
                            .unwrap_or_default(),
                        "Tags": tags,
                        "SecurityGroups": security_groups,
                        "BlockDeviceMappings": block_device_mappings
                    })
                })
                .collect::<Vec<_>>();

            instances.sort_by(|a, b| {
                let a_id = a
                    .get("InstanceId")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let b_id = b
                    .get("InstanceId")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                a_id.cmp(b_id)
            });
            json!({ "Instances": instances })
        })
        .collect::<Vec<_>>();

    value_to_json_string(json!({ "Reservations": reservations }))
}

async fn ec2_describe_volumes(client: &aws_sdk_ec2::Client, args: &[&str]) -> Option<String> {
    let volume_id = arg_value(args, "--volume-ids")?;
    let output = client
        .describe_volumes()
        .volume_ids(volume_id)
        .send()
        .await
        .ok()?;
    ec2_describe_volumes_output(output.volumes())
}

fn ec2_describe_volumes_output(volumes: &[aws_sdk_ec2::types::Volume]) -> Option<String> {
    let volumes = volumes
        .iter()
        .map(|volume| {
            json!({
                "VolumeId": volume.volume_id().unwrap_or_default(),
                "Size": volume.size().unwrap_or_default(),
                "VolumeType": volume.volume_type().map(|v| v.as_str()).unwrap_or("unknown"),
                "Iops": volume.iops().unwrap_or_default(),
                "Encrypted": volume.encrypted().unwrap_or(false)
            })
        })
        .collect::<Vec<_>>();

    value_to_json_string(json!({ "Volumes": volumes }))
}

async fn ec2_describe_instance_attribute(
    client: &aws_sdk_ec2::Client,
    args: &[&str],
) -> Option<String> {
    let instance_id = arg_value(args, "--instance-id")?;
    let attribute = arg_value(args, "--attribute")?;

    if attribute != "userData" {
        return None;
    }

    let output = client
        .describe_instance_attribute()
        .instance_id(instance_id)
        .attribute(aws_sdk_ec2::types::InstanceAttributeName::UserData)
        .send()
        .await
        .ok()?;

    value_to_json_string(json!({
        "UserData": {
            "Value": output.user_data().and_then(|u| u.value()).unwrap_or_default()
        }
    }))
}

async fn ec2_describe_vpcs(client: &aws_sdk_ec2::Client, args: &[&str]) -> Option<String> {
    let mut req = client.describe_vpcs();
    if let Some(vpc_id) = arg_value(args, "--vpc-ids") {
        req = req.vpc_ids(vpc_id);
    }

    let output = match req.send().await {
        Ok(output) => output,
        Err(error) => {
            tracing::error!(
                error = %error,
                vpc_id = arg_value(args, "--vpc-ids"),
                query = arg_value(args, "--query"),
                "describe-vpcs API call failed"
            );
            return None;
        }
    };
    ec2_describe_vpcs_output(output.vpcs(), arg_value(args, "--query"))
}

fn ec2_describe_vpcs_output(
    vpcs: &[aws_sdk_ec2::types::Vpc],
    query: Option<&str>,
) -> Option<String> {
    if query == Some("Vpcs[*].[VpcId,Tags]") {
        let mut rows = vpcs
            .iter()
            .map(|vpc| json!([vpc.vpc_id().unwrap_or_default(), parse_tags_ec2(vpc.tags())]))
            .collect::<Vec<_>>();

        rows.sort_by(|a, b| {
            let a_id = a.get(0).and_then(Value::as_str).unwrap_or_default();
            let b_id = b.get(0).and_then(Value::as_str).unwrap_or_default();
            a_id.cmp(b_id)
        });
        return value_to_json_string(Value::Array(rows));
    }

    let mut vpcs = vpcs
        .iter()
        .map(|vpc| {
            json!({
                "VpcId": vpc.vpc_id().unwrap_or_default(),
                "CidrBlock": vpc.cidr_block().unwrap_or_default(),
                "State": vpc.state().map(|s| s.as_str()).unwrap_or("unknown"),
                "Tags": parse_tags_ec2(vpc.tags())
            })
        })
        .collect::<Vec<_>>();

    vpcs.sort_by(|a, b| {
        let a_id = a.get("VpcId").and_then(Value::as_str).unwrap_or_default();
        let b_id = b.get("VpcId").and_then(Value::as_str).unwrap_or_default();
        a_id.cmp(b_id)
    });
    value_to_json_string(json!({ "Vpcs": vpcs }))
}

async fn ec2_describe_subnets(client: &aws_sdk_ec2::Client, args: &[&str]) -> Option<String> {
    let mut req = client.describe_subnets();
    if let Some(subnet_id) = arg_value(args, "--subnet-ids") {
        req = req.subnet_ids(subnet_id);
    }

    let output = match req.send().await {
        Ok(output) => output,
        Err(error) => {
            tracing::error!(
                error = %error,
                subnet_id = arg_value(args, "--subnet-ids"),
                "describe-subnets API call failed"
            );
            return None;
        }
    };
    ec2_describe_subnets_output(output.subnets())
}

fn ec2_describe_subnets_output(subnets: &[aws_sdk_ec2::types::Subnet]) -> Option<String> {
    let mut subnets = subnets
        .iter()
        .map(|subnet| {
            json!({
                "SubnetId": subnet.subnet_id().unwrap_or_default(),
                "VpcId": subnet.vpc_id().unwrap_or_default(),
                "CidrBlock": subnet.cidr_block().unwrap_or_default(),
                "AvailabilityZone": subnet.availability_zone().unwrap_or_default(),
                "State": subnet.state().map(|s| s.as_str()).unwrap_or("unknown"),
                "MapPublicIpOnLaunch": subnet.map_public_ip_on_launch().unwrap_or(false),
                "AvailableIpAddressCount": subnet.available_ip_address_count().unwrap_or_default(),
                "Tags": parse_tags_ec2(subnet.tags())
            })
        })
        .collect::<Vec<_>>();

    subnets.sort_by(|a, b| {
        let a_id = a
            .get("SubnetId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let b_id = b
            .get("SubnetId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        a_id.cmp(b_id)
    });
    value_to_json_string(json!({ "Subnets": subnets }))
}

async fn ec2_describe_internet_gateways(
    client: &aws_sdk_ec2::Client,
    args: &[&str],
) -> Option<String> {
    let mut req = client.describe_internet_gateways();

    if let Some(raw_filter) = arg_value(args, "--filters")
        && let Some(vpc_id) = parse_filter_value(raw_filter, "attachment.vpc-id")
    {
        let filter = aws_sdk_ec2::types::Filter::builder()
            .name("attachment.vpc-id")
            .values(vpc_id)
            .build();
        req = req.filters(filter);
    }

    let output = match req.send().await {
        Ok(output) => output,
        Err(error) => {
            tracing::error!(
                error = %error,
                filter = arg_value(args, "--filters"),
                query = arg_value(args, "--query"),
                "describe-internet-gateways API call failed"
            );
            return None;
        }
    };
    ec2_describe_internet_gateways_output(output.internet_gateways(), arg_value(args, "--query"))
}

fn ec2_describe_internet_gateways_output(
    internet_gateways: &[aws_sdk_ec2::types::InternetGateway],
    query: Option<&str>,
) -> Option<String> {
    if query == Some("InternetGateways[*].[InternetGatewayId,Tags,Attachments]") {
        let rows = internet_gateways
            .iter()
            .map(|igw| {
                let attachments = igw
                    .attachments()
                    .iter()
                    .map(|attachment| {
                        json!({
                            "VpcId": attachment.vpc_id().unwrap_or_default(),
                            "State": attachment.state().map(|s| s.as_str()).unwrap_or("unknown")
                        })
                    })
                    .collect::<Vec<_>>();

                json!([
                    igw.internet_gateway_id().unwrap_or_default(),
                    parse_tags_ec2(igw.tags()),
                    attachments
                ])
            })
            .collect::<Vec<_>>();

        return value_to_json_string(Value::Array(rows));
    }

    let gateways = internet_gateways
        .iter()
        .map(|igw| {
            let attachments = igw
                .attachments()
                .iter()
                .map(|attachment| {
                    json!({
                        "VpcId": attachment.vpc_id().unwrap_or_default(),
                        "State": attachment.state().map(|s| s.as_str()).unwrap_or("unknown")
                    })
                })
                .collect::<Vec<_>>();

            json!({
                "InternetGatewayId": igw.internet_gateway_id().unwrap_or_default(),
                "Tags": parse_tags_ec2(igw.tags()),
                "Attachments": attachments
            })
        })
        .collect::<Vec<_>>();

    value_to_json_string(json!({ "InternetGateways": gateways }))
}

async fn ec2_describe_nat_gateways(client: &aws_sdk_ec2::Client, args: &[&str]) -> Option<String> {
    let mut req = client.describe_nat_gateways();

    let filter_value = arg_value(args, "--filter").or_else(|| arg_value(args, "--filters"));
    if let Some(raw_filter) = filter_value
        && let Some(vpc_id) = parse_filter_value(raw_filter, "vpc-id")
    {
        let filter = aws_sdk_ec2::types::Filter::builder()
            .name("vpc-id")
            .values(vpc_id)
            .build();
        req = req.filter(filter);
    }

    let output = match req.send().await {
        Ok(output) => output,
        Err(error) => {
            tracing::error!(
                error = %error,
                filter = filter_value,
                "describe-nat-gateways API call failed"
            );
            return None;
        }
    };
    ec2_describe_nat_gateways_output(output.nat_gateways())
}

fn ec2_describe_nat_gateways_output(
    nat_gateways: &[aws_sdk_ec2::types::NatGateway],
) -> Option<String> {
    let nat_gateways = nat_gateways
        .iter()
        .map(|nat| {
            let addresses = nat
                .nat_gateway_addresses()
                .iter()
                .map(|address| {
                    json!({
                        "AllocationId": address.allocation_id().unwrap_or_default(),
                        "PublicIp": address.public_ip().unwrap_or_default(),
                        "PrivateIp": address.private_ip().unwrap_or_default()
                    })
                })
                .collect::<Vec<_>>();

            json!({
                "NatGatewayId": nat.nat_gateway_id().unwrap_or_default(),
                "SubnetId": nat.subnet_id().unwrap_or_default(),
                "State": nat.state().map(|s| s.as_str()).unwrap_or("unknown"),
                "ConnectivityType": nat.connectivity_type().map(|v| v.as_str()).unwrap_or_default(),
                "NatGatewayAddresses": addresses,
                "Tags": parse_tags_ec2(nat.tags())
            })
        })
        .collect::<Vec<_>>();

    value_to_json_string(json!({ "NatGateways": nat_gateways }))
}

async fn ec2_describe_route_tables(client: &aws_sdk_ec2::Client, args: &[&str]) -> Option<String> {
    let mut req = client.describe_route_tables();

    if let Some(raw_filter) = arg_value(args, "--filters")
        && let Some(vpc_id) = parse_filter_value(raw_filter, "vpc-id")
    {
        let filter = aws_sdk_ec2::types::Filter::builder()
            .name("vpc-id")
            .values(vpc_id)
            .build();
        req = req.filters(filter);
    }

    let output = match req.send().await {
        Ok(output) => output,
        Err(error) => {
            tracing::error!(
                error = %error,
                filter = arg_value(args, "--filters"),
                query = arg_value(args, "--query"),
                "describe-route-tables API call failed"
            );
            return None;
        }
    };
    ec2_describe_route_tables_output(output.route_tables(), arg_value(args, "--query"))
}

fn ec2_describe_route_tables_output(
    route_tables: &[aws_sdk_ec2::types::RouteTable],
    query: Option<&str>,
) -> Option<String> {
    if query == Some("RouteTables[*].[RouteTableId,Tags,Routes,Associations]") {
        let rows = route_tables
            .iter()
            .map(|rt| {
                let routes = rt
                    .routes()
                    .iter()
                    .map(|route| {
                        json!({
                            "DestinationCidrBlock": route.destination_cidr_block().unwrap_or_default(),
                            "GatewayId": route.gateway_id().unwrap_or_default(),
                            "NatGatewayId": route.nat_gateway_id().unwrap_or_default(),
                            "State": route.state().map(|s| s.as_str()).unwrap_or("active")
                        })
                    })
                    .collect::<Vec<_>>();

                let associations = rt
                    .associations()
                    .iter()
                    .map(|association| {
                        json!({
                            "SubnetId": association.subnet_id().unwrap_or_default()
                        })
                    })
                    .collect::<Vec<_>>();

                json!([
                    rt.route_table_id().unwrap_or_default(),
                    parse_tags_ec2(rt.tags()),
                    routes,
                    associations
                ])
            })
            .collect::<Vec<_>>();

        return value_to_json_string(Value::Array(rows));
    }

    let route_tables = route_tables
        .iter()
        .map(|rt| {
            let routes = rt
                .routes()
                .iter()
                .map(|route| {
                    json!({
                        "DestinationCidrBlock": route.destination_cidr_block().unwrap_or_default(),
                        "GatewayId": route.gateway_id().unwrap_or_default(),
                        "NatGatewayId": route.nat_gateway_id().unwrap_or_default(),
                        "State": route.state().map(|s| s.as_str()).unwrap_or("active")
                    })
                })
                .collect::<Vec<_>>();

            let associations = rt
                .associations()
                .iter()
                .map(|association| {
                    json!({
                        "SubnetId": association.subnet_id().unwrap_or_default()
                    })
                })
                .collect::<Vec<_>>();

            json!({
                "RouteTableId": rt.route_table_id().unwrap_or_default(),
                "Tags": parse_tags_ec2(rt.tags()),
                "Routes": routes,
                "Associations": associations
            })
        })
        .collect::<Vec<_>>();

    value_to_json_string(json!({ "RouteTables": route_tables }))
}

async fn ec2_describe_addresses(client: &aws_sdk_ec2::Client) -> Option<String> {
    let output = client.describe_addresses().send().await.ok()?;
    ec2_describe_addresses_output(output.addresses())
}

fn ec2_describe_addresses_output(addresses: &[aws_sdk_ec2::types::Address]) -> Option<String> {
    let addresses = addresses
        .iter()
        .map(|address| {
            json!({
                "AllocationId": address.allocation_id().unwrap_or_default(),
                "PublicIp": address.public_ip().unwrap_or_default(),
                "AssociationId": address.association_id().unwrap_or_default(),
                "InstanceId": address.instance_id().unwrap_or_default(),
                "PrivateIpAddress": address.private_ip_address().unwrap_or_default(),
                "Tags": parse_tags_ec2(address.tags())
            })
        })
        .collect::<Vec<_>>();

    value_to_json_string(json!({ "Addresses": addresses }))
}

async fn ec2_describe_vpc_attribute(client: &aws_sdk_ec2::Client, args: &[&str]) -> Option<String> {
    let vpc_id = arg_value(args, "--vpc-id")?;
    let attribute = arg_value(args, "--attribute")?;

    let output = client
        .describe_vpc_attribute()
        .vpc_id(vpc_id)
        .attribute(match attribute {
            "enableDnsSupport" => aws_sdk_ec2::types::VpcAttributeName::EnableDnsSupport,
            "enableDnsHostnames" => aws_sdk_ec2::types::VpcAttributeName::EnableDnsHostnames,
            _ => return None,
        })
        .send()
        .await
        .ok()?;

    match attribute {
        "enableDnsSupport" => value_to_json_string(json!({
            "EnableDnsSupport": {
                "Value": output
                    .enable_dns_support()
                    .and_then(|v| v.value())
                    .unwrap_or(false)
            }
        })),
        "enableDnsHostnames" => value_to_json_string(json!({
            "EnableDnsHostnames": {
                "Value": output
                    .enable_dns_hostnames()
                    .and_then(|v| v.value())
                    .unwrap_or(false)
            }
        })),
        _ => None,
    }
}

fn parse_ip_permissions(permissions: &[aws_sdk_ec2::types::IpPermission]) -> Vec<Value> {
    permissions
        .iter()
        .map(|permission| {
            let ip_ranges = permission
                .ip_ranges()
                .iter()
                .map(|range| {
                    json!({
                        "CidrIp": range.cidr_ip().unwrap_or_default(),
                        "Description": range.description().unwrap_or_default()
                    })
                })
                .collect::<Vec<_>>();

            let ipv6_ranges = permission
                .ipv6_ranges()
                .iter()
                .map(|range| {
                    json!({
                        "CidrIpv6": range.cidr_ipv6().unwrap_or_default(),
                        "Description": range.description().unwrap_or_default()
                    })
                })
                .collect::<Vec<_>>();

            let user_id_group_pairs = permission
                .user_id_group_pairs()
                .iter()
                .map(|pair| {
                    json!({
                        "GroupId": pair.group_id().unwrap_or_default(),
                        "Description": pair.description().unwrap_or_default()
                    })
                })
                .collect::<Vec<_>>();

            json!({
                "IpProtocol": permission.ip_protocol().unwrap_or("-1"),
                "FromPort": permission.from_port(),
                "ToPort": permission.to_port(),
                "IpRanges": ip_ranges,
                "Ipv6Ranges": ipv6_ranges,
                "UserIdGroupPairs": user_id_group_pairs
            })
        })
        .collect()
}

async fn ec2_describe_security_groups(
    client: &aws_sdk_ec2::Client,
    args: &[&str],
) -> Option<String> {
    let mut req = client.describe_security_groups();

    if let Some(group_id) = arg_value(args, "--group-ids") {
        req = req.group_ids(group_id);
    }

    let output = req.send().await.ok()?;
    ec2_describe_security_groups_output(output.security_groups())
}

fn ec2_describe_security_groups_output(
    security_groups: &[aws_sdk_ec2::types::SecurityGroup],
) -> Option<String> {
    let mut groups = security_groups
        .iter()
        .map(|group| {
            json!({
                "GroupId": group.group_id().unwrap_or_default(),
                "GroupName": group.group_name().unwrap_or_default(),
                "Description": group.description().unwrap_or_default(),
                "VpcId": group.vpc_id().unwrap_or_default(),
                "IpPermissions": parse_ip_permissions(group.ip_permissions()),
                "IpPermissionsEgress": parse_ip_permissions(group.ip_permissions_egress()),
                "Tags": parse_tags_ec2(group.tags())
            })
        })
        .collect::<Vec<_>>();

    groups.sort_by(|a, b| {
        let a_id = a.get("GroupId").and_then(Value::as_str).unwrap_or_default();
        let b_id = b.get("GroupId").and_then(Value::as_str).unwrap_or_default();
        a_id.cmp(b_id)
    });
    value_to_json_string(json!({ "SecurityGroups": groups }))
}

async fn ec2_describe_images(client: &aws_sdk_ec2::Client, args: &[&str]) -> Option<String> {
    let image_id = arg_value(args, "--image-ids")?;
    let output = client
        .describe_images()
        .image_ids(image_id)
        .send()
        .await
        .ok()?;
    ec2_describe_images_output(output.images())
}

fn ec2_describe_images_output(images: &[aws_sdk_ec2::types::Image]) -> Option<String> {
    let images = images
        .iter()
        .map(|image| {
            let mut tags = parse_tags_ec2(image.tags());
            if !tags
                .iter()
                .any(|tag| tag.get("Key").and_then(Value::as_str) == Some("Name"))
            {
                tags.push(json!({
                    "Key": "Name",
                    "Value": image.name().unwrap_or_default()
                }));
            }

            json!({
                "ImageId": image.image_id().unwrap_or_default(),
                "Name": image.name().unwrap_or_default(),
                "Tags": tags
            })
        })
        .collect::<Vec<_>>();

    value_to_json_string(json!({ "Images": images }))
}

async fn run_ecr_request(
    config: &aws_config::SdkConfig,
    operation: &str,
    args: &[&str],
) -> Option<String> {
    let client = aws_sdk_ecr::Client::new(config);

    match operation {
        "describe-repositories" => ecr_describe_repositories(&client, args).await,
        "describe-images" => ecr_describe_images(&client, args).await,
        _ => None,
    }
}

async fn ecr_describe_repositories(client: &aws_sdk_ecr::Client, args: &[&str]) -> Option<String> {
    let mut req = client.describe_repositories();

    if let Some(repo_name) = arg_value(args, "--repository-names") {
        req = req.repository_names(repo_name);
    }

    let output = req.send().await.ok()?;
    ecr_describe_repositories_output(output.repositories())
}

fn ecr_describe_repositories_output(
    repositories: &[aws_sdk_ecr::types::Repository],
) -> Option<String> {
    let mut repositories = repositories
        .iter()
        .map(|repo| {
            let enc = repo.encryption_configuration();
            json!({
                "repositoryName": repo.repository_name().unwrap_or_default(),
                "repositoryUri": repo.repository_uri().unwrap_or_default(),
                "imageTagMutability": repo.image_tag_mutability().map(|v| v.as_str()).unwrap_or_default(),
                "encryptionConfiguration": {
                    "encryptionType": enc.map(|v| v.encryption_type().as_str()).unwrap_or("AES256"),
                    "kmsKey": enc.and_then(|v| v.kms_key()).unwrap_or_default()
                },
                "createdAt": repo.created_at().map(|t| t.secs() as f64).unwrap_or_default()
            })
        })
        .collect::<Vec<_>>();

    repositories.sort_by(|a, b| {
        let a_name = a
            .get("repositoryName")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let b_name = b
            .get("repositoryName")
            .and_then(Value::as_str)
            .unwrap_or_default();
        a_name.cmp(b_name)
    });
    value_to_json_string(json!({ "repositories": repositories }))
}

async fn ecr_describe_images(client: &aws_sdk_ecr::Client, args: &[&str]) -> Option<String> {
    let repo_name = arg_value(args, "--repository-name")?;

    let output = client
        .describe_images()
        .repository_name(repo_name)
        .send()
        .await
        .ok()?;
    ecr_describe_images_output(output.image_details())
}

fn ecr_describe_images_output(image_details: &[aws_sdk_ecr::types::ImageDetail]) -> Option<String> {
    let image_details = image_details
        .iter()
        .map(|image| {
            json!({
                "imageDigest": image.image_digest().unwrap_or_default()
            })
        })
        .collect::<Vec<_>>();

    value_to_json_string(json!({ "imageDetails": image_details }))
}

async fn run_elbv2_request(
    config: &aws_config::SdkConfig,
    operation: &str,
    args: &[&str],
) -> Option<String> {
    let client = aws_sdk_elasticloadbalancingv2::Client::new(config);

    match operation {
        "describe-load-balancers" => elbv2_describe_load_balancers(&client, args).await,
        "describe-listeners" => elbv2_describe_listeners(&client, args).await,
        "describe-target-groups" => elbv2_describe_target_groups(&client, args).await,
        "describe-target-health" => elbv2_describe_target_health(&client, args).await,
        _ => None,
    }
}

fn lb_to_json(lb: &aws_sdk_elasticloadbalancingv2::types::LoadBalancer) -> Value {
    let availability_zones = lb
        .availability_zones()
        .iter()
        .map(|az| {
            json!({
                "ZoneName": az.zone_name().unwrap_or_default()
            })
        })
        .collect::<Vec<_>>();

    json!({
        "LoadBalancerName": lb.load_balancer_name().unwrap_or_default(),
        "LoadBalancerArn": lb.load_balancer_arn().unwrap_or_default(),
        "DNSName": lb.dns_name().unwrap_or_default(),
        "Type": lb.r#type().map(|v| v.as_str()).unwrap_or_default(),
        "Scheme": lb.scheme().map(|v| v.as_str()).unwrap_or_default(),
        "VpcId": lb.vpc_id().unwrap_or_default(),
        "IpAddressType": lb.ip_address_type().map(|v| v.as_str()).unwrap_or_default(),
        "AvailabilityZones": availability_zones,
        "SecurityGroups": lb.security_groups(),
        "State": {
            "Code": lb.state().and_then(|s| s.code()).map(|v| v.as_str()).unwrap_or("unknown")
        }
    })
}

async fn elbv2_describe_load_balancers(
    client: &aws_sdk_elasticloadbalancingv2::Client,
    args: &[&str],
) -> Option<String> {
    let mut req = client.describe_load_balancers();

    if let Some(arn) = arg_value(args, "--load-balancer-arns") {
        req = req.load_balancer_arns(arn);
    }

    let output = req.send().await.ok()?;
    elbv2_describe_load_balancers_output(output.load_balancers())
}

fn elbv2_describe_load_balancers_output(
    load_balancers: &[aws_sdk_elasticloadbalancingv2::types::LoadBalancer],
) -> Option<String> {
    let mut load_balancers = load_balancers.iter().map(lb_to_json).collect::<Vec<_>>();

    load_balancers.sort_by(|a, b| {
        let a_name = a
            .get("LoadBalancerName")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let b_name = b
            .get("LoadBalancerName")
            .and_then(Value::as_str)
            .unwrap_or_default();
        a_name.cmp(b_name)
    });
    value_to_json_string(json!({ "LoadBalancers": load_balancers }))
}

async fn elbv2_describe_listeners(
    client: &aws_sdk_elasticloadbalancingv2::Client,
    args: &[&str],
) -> Option<String> {
    let lb_arn = arg_value(args, "--load-balancer-arn")?;

    let output = client
        .describe_listeners()
        .load_balancer_arn(lb_arn)
        .send()
        .await
        .ok()?;
    elbv2_describe_listeners_output(output.listeners())
}

fn elbv2_describe_listeners_output(
    listeners: &[aws_sdk_elasticloadbalancingv2::types::Listener],
) -> Option<String> {
    let mut listeners = listeners
        .iter()
        .map(|listener| {
            let default_actions = listener
                .default_actions()
                .iter()
                .map(|action| {
                    json!({
                        "Type": action.r#type().map(|v| v.as_str()).unwrap_or_default(),
                        "TargetGroupArn": action.target_group_arn().unwrap_or_default()
                    })
                })
                .collect::<Vec<_>>();

            json!({
                "Port": listener.port().unwrap_or_default(),
                "Protocol": listener.protocol().map(|v| v.as_str()).unwrap_or_default(),
                "DefaultActions": default_actions
            })
        })
        .collect::<Vec<_>>();

    listeners.sort_by(|a, b| {
        let a_port = a.get("Port").and_then(Value::as_i64).unwrap_or_default();
        let b_port = b.get("Port").and_then(Value::as_i64).unwrap_or_default();
        a_port.cmp(&b_port)
    });
    value_to_json_string(json!({ "Listeners": listeners }))
}

fn target_group_to_json(
    target_group: &aws_sdk_elasticloadbalancingv2::types::TargetGroup,
) -> Value {
    json!({
        "TargetGroupName": target_group.target_group_name().unwrap_or_default(),
        "TargetGroupArn": target_group.target_group_arn().unwrap_or_default(),
        "Protocol": target_group.protocol().map(|v| v.as_str()).unwrap_or_default(),
        "Port": target_group.port().unwrap_or_default(),
        "TargetType": target_group.target_type().map(|v| v.as_str()).unwrap_or_default(),
        "HealthCheckProtocol": target_group.health_check_protocol().map(|v| v.as_str()).unwrap_or_default(),
        "HealthCheckPath": target_group.health_check_path().unwrap_or_default(),
        "HealthyThresholdCount": target_group.healthy_threshold_count().unwrap_or_default(),
        "UnhealthyThresholdCount": target_group.unhealthy_threshold_count().unwrap_or_default()
    })
}

async fn elbv2_describe_target_groups(
    client: &aws_sdk_elasticloadbalancingv2::Client,
    args: &[&str],
) -> Option<String> {
    let mut req = client.describe_target_groups();

    if let Some(lb_arn) = arg_value(args, "--load-balancer-arn") {
        req = req.load_balancer_arn(lb_arn);
    }

    if let Some(tg_arn) = arg_value(args, "--target-group-arns") {
        req = req.target_group_arns(tg_arn);
    }

    let output = req.send().await.ok()?;
    elbv2_describe_target_groups_output(output.target_groups())
}

fn elbv2_describe_target_groups_output(
    target_groups: &[aws_sdk_elasticloadbalancingv2::types::TargetGroup],
) -> Option<String> {
    let mut target_groups = target_groups
        .iter()
        .map(target_group_to_json)
        .collect::<Vec<_>>();

    target_groups.sort_by(|a, b| {
        let a_name = a
            .get("TargetGroupName")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let b_name = b
            .get("TargetGroupName")
            .and_then(Value::as_str)
            .unwrap_or_default();
        a_name.cmp(b_name)
    });
    value_to_json_string(json!({ "TargetGroups": target_groups }))
}

async fn elbv2_describe_target_health(
    client: &aws_sdk_elasticloadbalancingv2::Client,
    args: &[&str],
) -> Option<String> {
    let tg_arn = arg_value(args, "--target-group-arn")?;

    let output = client
        .describe_target_health()
        .target_group_arn(tg_arn)
        .send()
        .await
        .ok()?;
    elbv2_describe_target_health_output(output.target_health_descriptions())
}

fn elbv2_describe_target_health_output(
    descriptions: &[aws_sdk_elasticloadbalancingv2::types::TargetHealthDescription],
) -> Option<String> {
    let mut descriptions = descriptions
        .iter()
        .map(|description| {
            json!({
                "Target": {
                    "Id": description.target().and_then(|t| t.id()).unwrap_or_default(),
                    "Port": description.target().and_then(|t| t.port()).unwrap_or_default()
                },
                "HealthCheckPort": description.health_check_port().unwrap_or("traffic-port"),
                "TargetHealth": {
                    "State": description
                        .target_health()
                        .and_then(|health| health.state())
                        .map(|v| v.as_str())
                        .unwrap_or("unknown")
                }
            })
        })
        .collect::<Vec<_>>();

    descriptions.sort_by(|a, b| {
        let a_id = a
            .get("Target")
            .and_then(|v| v.get("Id"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let b_id = b
            .get("Target")
            .and_then(|v| v.get("Id"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        a_id.cmp(b_id)
    });
    value_to_json_string(json!({ "TargetHealthDescriptions": descriptions }))
}

async fn run_iam_request(
    config: &aws_config::SdkConfig,
    operation: &str,
    args: &[&str],
) -> Option<String> {
    let client = aws_sdk_iam::Client::new(config);

    match operation {
        "get-role" => iam_get_role(&client, args).await,
        "list-attached-role-policies" => iam_list_attached_role_policies(&client, args).await,
        "list-role-policies" => iam_list_role_policies(&client, args).await,
        "get-role-policy" => iam_get_role_policy(&client, args).await,
        _ => None,
    }
}

fn parse_policy_json(policy: Option<&str>) -> Value {
    policy
        .and_then(|document| {
            let decoded = percent_encoding::percent_decode_str(document)
                .decode_utf8()
                .ok()?;
            serde_json::from_str::<Value>(&decoded).ok()
        })
        .unwrap_or_else(|| Value::String(policy.unwrap_or_default().to_string()))
}

async fn iam_get_role(client: &aws_sdk_iam::Client, args: &[&str]) -> Option<String> {
    let role_name = arg_value(args, "--role-name")?;

    let output = client.get_role().role_name(role_name).send().await.ok()?;
    let role = output.role()?;
    iam_get_role_output(role)
}

fn iam_get_role_output(role: &aws_sdk_iam::types::Role) -> Option<String> {
    value_to_json_string(json!({
        "Role": {
            "RoleName": role.role_name(),
            "Arn": role.arn(),
            "AssumeRolePolicyDocument": parse_policy_json(role.assume_role_policy_document()),
            "Tags": parse_tags_iam(role.tags())
        }
    }))
}

async fn iam_list_attached_role_policies(
    client: &aws_sdk_iam::Client,
    args: &[&str],
) -> Option<String> {
    let role_name = arg_value(args, "--role-name")?;

    let output = client
        .list_attached_role_policies()
        .role_name(role_name)
        .send()
        .await
        .ok()?;
    iam_list_attached_role_policies_output(output.attached_policies())
}

fn iam_list_attached_role_policies_output(
    attached_policies: &[aws_sdk_iam::types::AttachedPolicy],
) -> Option<String> {
    let mut attached_policies = attached_policies
        .iter()
        .map(|policy| {
            json!({
                "PolicyName": policy.policy_name().unwrap_or_default(),
                "PolicyArn": policy.policy_arn().unwrap_or_default()
            })
        })
        .collect::<Vec<_>>();

    attached_policies.sort_by(|a, b| {
        let a_name = a
            .get("PolicyName")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let b_name = b
            .get("PolicyName")
            .and_then(Value::as_str)
            .unwrap_or_default();
        a_name.cmp(b_name)
    });
    value_to_json_string(json!({ "AttachedPolicies": attached_policies }))
}

async fn iam_list_role_policies(client: &aws_sdk_iam::Client, args: &[&str]) -> Option<String> {
    let role_name = arg_value(args, "--role-name")?;

    let output = client
        .list_role_policies()
        .role_name(role_name)
        .send()
        .await
        .ok()?;
    iam_list_role_policies_output(output.policy_names())
}

fn iam_list_role_policies_output(policy_names: &[String]) -> Option<String> {
    let mut policy_names = policy_names
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();

    policy_names.sort();
    value_to_json_string(json!({ "PolicyNames": policy_names }))
}

async fn iam_get_role_policy(client: &aws_sdk_iam::Client, args: &[&str]) -> Option<String> {
    let role_name = arg_value(args, "--role-name")?;
    let policy_name = arg_value(args, "--policy-name")?;

    let output = client
        .get_role_policy()
        .role_name(role_name)
        .policy_name(policy_name)
        .send()
        .await
        .ok()?;
    iam_get_role_policy_output(output.policy_document())
}

fn iam_get_role_policy_output(policy_document: &str) -> Option<String> {
    value_to_json_string(json!({
        "PolicyDocument": parse_policy_json(Some(policy_document))
    }))
}

async fn run_sts_request(config: &aws_config::SdkConfig, operation: &str) -> Option<String> {
    if operation != "get-caller-identity" {
        return None;
    }

    let client = aws_sdk_sts::Client::new(config);
    let output = client.get_caller_identity().send().await.ok()?;

    value_to_json_string(json!({
        "Account": output.account().unwrap_or_default(),
        "Arn": output.arn().unwrap_or_default()
    }))
}

pub fn extract_json_value(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\": \"", key);
    if let Some(start) = json.find(&pattern) {
        let offset = start + pattern.len();
        if let Some(end) = json[offset..].find('"') {
            return Some(json[offset..offset + end].to_string());
        }
    }
    None
}

pub fn parse_name_tag(tags_json: &str) -> String {
    if let Some(start) = tags_json.find("\"Key\": \"Name\"")
        && let Some(value_start) = tags_json[start..].find("\"Value\": \"")
    {
        let offset = start + value_start + 10;
        if let Some(end) = tags_json[offset..].find('"') {
            return tags_json[offset..offset + end].to_string();
        }
    }
    if let Some(start) = tags_json.find("\"Value\": \"") {
        let offset = start + 10;
        if let Some(end) = tags_json[offset..].find('"') {
            let value = &tags_json[offset..offset + end];
            if tags_json[offset + end..].contains("\"Key\": \"Name\"") {
                return value.to_string();
            }
        }
    }
    String::new()
}

pub fn extract_tags(json: &str) -> Vec<(String, String)> {
    let mut tags = Vec::new();
    let mut search_start = 0;

    while let Some(key_pos) = json[search_start..].find("\"Key\": \"") {
        let key_start = search_start + key_pos + 8;
        if let Some(key_end) = json[key_start..].find('"') {
            let key = json[key_start..key_start + key_end].to_string();

            if let Some(val_pos) = json[key_start..].find("\"Value\": \"") {
                let val_start = key_start + val_pos + 10;
                if let Some(val_end) = json[val_start..].find('"') {
                    let value = json[val_start..val_start + val_end].to_string();
                    if !tags.iter().any(|(k, _)| k == &key) {
                        tags.push((key, value));
                    }
                }
            }
            search_start = key_start + key_end;
        } else {
            break;
        }
    }
    tags
}

pub fn parse_resources_from_json(json: &str, prefix: &str) -> Vec<AwsResource> {
    let mut resources = Vec::new();

    let mut search_start = 0;
    while let Some(pos) = json[search_start..].find(prefix) {
        let start = search_start + pos;
        if let Some(end) = json[start..].find('"') {
            let id = &json[start..start + end];
            if id.starts_with(prefix) && !id.contains(' ') {
                let section_end = json[start..]
                    .find(']')
                    .map(|p| start + p)
                    .unwrap_or(json.len());
                let tag_start = start;
                let tag_end = section_end;
                let tags_json = &json[tag_start..tag_end];
                let name = parse_name_tag(tags_json);

                resources.push(AwsResource {
                    name,
                    id: id.to_string(),
                    state: String::new(),
                    az: String::new(),
                    cidr: String::new(),
                });
            }
            search_start = start + end;
        } else {
            break;
        }
    }
    resources
}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("Failed to create Tokio runtime"))
}

/// Get AWS SDK config with profile-based credentials and region
pub async fn get_sdk_config() -> aws_config::SdkConfig {
    let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest());

    let profile_name = std::env::var("AWS_PROFILE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("AWS_DEFAULT_PROFILE")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .or_else(|| Some("default".to_string()));

    if let Some(profile_name) = profile_name.as_deref() {
        config_loader = config_loader.profile_name(profile_name);
    }

    let region_env = std::env::var("AWS_REGION")
        .ok()
        .or_else(|| std::env::var("AWS_DEFAULT_REGION").ok());

    if let Ok(region) = REGION.lock() {
        if let Some(ref region_str) = *region {
            config_loader = config_loader.region(aws_config::Region::new(region_str.clone()));
        } else if let Some(region_str) = region_env {
            config_loader = config_loader.region(aws_config::Region::new(region_str));
        }
    }

    if let Ok(region) = REGION.lock()
        && region.is_none()
    {
        config_loader = config_loader.region(aws_config::Region::new("us-east-1"));
    }

    config_loader.load().await
}

#[cfg(test)]
mod tests {
    use super::{
        AwsResource, arg_value, ec2_describe_addresses_output, ec2_describe_images_output,
        ec2_describe_instances_output, ec2_describe_internet_gateways_output,
        ec2_describe_nat_gateways_output, ec2_describe_route_tables_output,
        ec2_describe_security_groups_output, ec2_describe_subnets_output,
        ec2_describe_volumes_output, ec2_describe_vpcs_output, ecr_describe_images_output,
        ecr_describe_repositories_output, elbv2_describe_listeners_output,
        elbv2_describe_load_balancers_output, elbv2_describe_target_groups_output,
        elbv2_describe_target_health_output, extract_json_value, extract_tags,
        iam_get_role_policy_output, iam_list_attached_role_policies_output,
        iam_list_role_policies_output, is_auth_failure_error, is_network_error, lb_to_json,
        list_aws_profiles, parse_filter_value, parse_ip_permissions, parse_name_tag,
        parse_policy_json, parse_resources_from_json, parse_tags_ec2, parse_tags_iam,
        run_ec2_request, run_ecr_request, run_elbv2_request, run_iam_request, run_sts_request,
        set_aws_profile, target_group_to_json, value_to_json_string,
    };
    use serde_json::Value;
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn temp_home(prefix: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        path.push(format!(
            "emd-common-{}-{}-{}",
            prefix,
            std::process::id(),
            nanos
        ));
        path
    }

    fn snapshot_var(name: &str) -> (String, Option<OsString>) {
        (name.to_string(), env::var_os(name))
    }

    fn restore_var(name: &str, value: Option<OsString>) {
        if let Some(v) = value {
            unsafe {
                env::set_var(name, v);
            }
        } else {
            unsafe {
                env::remove_var(name);
            }
        }
    }

    #[test]
    fn auth_failure_detector_matches_known_auth_errors() {
        let message = "AccessDeniedException: The refresh token has expired.";
        assert!(is_auth_failure_error(message));
    }

    #[test]
    fn auth_failure_detector_ignores_endpoint_connectivity_errors() {
        let message =
            "Could not connect to the endpoint URL: https://sts.ap-southeast-1.amazonaws.com/";
        assert!(!is_auth_failure_error(message));
    }

    #[test]
    fn network_error_detector_matches_expected_markers() {
        assert!(is_network_error("Could not connect to endpoint"));
        assert!(is_network_error("connection refused"));
        assert!(is_network_error("request timed out"));
        assert!(!is_network_error("AccessDeniedException: token expired"));
    }

    #[test]
    fn set_aws_profile_updates_and_clears_profile_vars() {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let vars = [
            snapshot_var("AWS_PROFILE"),
            snapshot_var("AWS_DEFAULT_PROFILE"),
        ];

        set_aws_profile("qa");
        assert_eq!(env::var("AWS_PROFILE").ok().as_deref(), Some("qa"));
        assert_eq!(env::var("AWS_DEFAULT_PROFILE").ok().as_deref(), Some("qa"));

        set_aws_profile("   ");
        assert!(env::var("AWS_PROFILE").is_err());
        assert!(env::var("AWS_DEFAULT_PROFILE").is_err());

        for (name, value) in vars {
            restore_var(&name, value);
        }
    }

    #[test]
    fn list_aws_profiles_reads_config_and_credentials_files() {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let original_home = env::var_os("HOME");
        let home = temp_home("profiles");
        let aws_dir = home.join(".aws");
        fs::create_dir_all(&aws_dir).expect("create .aws");

        fs::write(
            aws_dir.join("config"),
            "[default]\nregion=ap-southeast-1\n[profile dev]\n[profile qa]\n",
        )
        .expect("write config");
        fs::write(
            aws_dir.join("credentials"),
            "[default]\naws_access_key_id=x\n[ops]\naws_access_key_id=y\n",
        )
        .expect("write credentials");

        unsafe {
            env::set_var("HOME", &home);
        }

        let profiles = list_aws_profiles().expect("list profiles");
        assert_eq!(profiles, vec!["default", "dev", "ops", "qa"]);

        restore_var("HOME", original_home);
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn request_dispatchers_return_none_for_unsupported_or_missing_required_args() {
        let config = aws_config::SdkConfig::builder()
            .behavior_version(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new("us-east-1"))
            .build();
        super::get_runtime().block_on(async {
            assert!(
                run_ec2_request(&config, "unknown-op", &["ec2", "unknown-op"])
                    .await
                    .is_none()
            );
            assert!(
                run_ec2_request(&config, "describe-volumes", &["ec2", "describe-volumes"])
                    .await
                    .is_none()
            );
            assert!(
                run_ec2_request(
                    &config,
                    "describe-instance-attribute",
                    &["ec2", "describe-instance-attribute"]
                )
                .await
                .is_none()
            );
            assert!(
                run_ec2_request(
                    &config,
                    "describe-vpc-attribute",
                    &["ec2", "describe-vpc-attribute"]
                )
                .await
                .is_none()
            );
            assert!(
                run_ec2_request(&config, "describe-images", &["ec2", "describe-images"])
                    .await
                    .is_none()
            );

            assert!(
                run_ecr_request(&config, "describe-images", &["ecr", "describe-images"])
                    .await
                    .is_none()
            );
            assert!(
                run_elbv2_request(
                    &config,
                    "describe-listeners",
                    &["elbv2", "describe-listeners"]
                )
                .await
                .is_none()
            );
            assert!(
                run_elbv2_request(
                    &config,
                    "describe-target-health",
                    &["elbv2", "describe-target-health"]
                )
                .await
                .is_none()
            );
            assert!(
                run_iam_request(&config, "get-role", &["iam", "get-role"])
                    .await
                    .is_none()
            );
            assert!(
                run_iam_request(
                    &config,
                    "list-attached-role-policies",
                    &["iam", "list-attached-role-policies"]
                )
                .await
                .is_none()
            );
            assert!(
                run_iam_request(
                    &config,
                    "list-role-policies",
                    &["iam", "list-role-policies"]
                )
                .await
                .is_none()
            );
            assert!(
                run_iam_request(&config, "get-role-policy", &["iam", "get-role-policy"])
                    .await
                    .is_none()
            );
            assert!(run_sts_request(&config, "unknown-op").await.is_none());
        });
    }

    #[test]
    fn scenario_lt_filter_parser_multivalue() {
        let raw = "Name=attachment.vpc-id,Values=vpc-11111111,vpc-22222222,vpc-33333333";
        let value = parse_filter_value(raw, "attachment.vpc-id");
        assert_eq!(
            value,
            Some("vpc-11111111,vpc-22222222,vpc-33333333".to_string())
        );
    }

    #[test]
    fn scenario_lt_filter_parser_name_mismatch() {
        let raw = "Name=vpc-id,Values=vpc-11111111,vpc-22222222";
        let value = parse_filter_value(raw, "attachment.vpc-id");
        assert_eq!(value, None);
    }

    #[test]
    fn scenario_lt_policy_document_percent_decode() {
        let encoded = "%7B%22Version%22%3A%222012-10-17%22%2C%22Statement%22%3A%5B%5D%7D";
        let value = parse_policy_json(Some(encoded));
        assert_eq!(
            value.get("Version").and_then(Value::as_str),
            Some("2012-10-17")
        );
    }

    #[test]
    fn parse_policy_json_uses_existing_fallback_for_invalid_utf8() {
        let value = parse_policy_json(Some("%FF"));
        assert_eq!(value, Value::String("%FF".to_string()));
    }

    fn ec2_test_tag(key: &str, value: &str) -> aws_sdk_ec2::types::Tag {
        aws_sdk_ec2::types::Tag::builder()
            .key(key)
            .value(value)
            .build()
    }

    fn sample_instance(
        instance_id: &str,
        name: &str,
        state: aws_sdk_ec2::types::InstanceStateName,
    ) -> aws_sdk_ec2::types::Instance {
        let bdm = aws_sdk_ec2::types::InstanceBlockDeviceMapping::builder()
            .device_name("/dev/xvda")
            .ebs(
                aws_sdk_ec2::types::EbsInstanceBlockDevice::builder()
                    .volume_id("vol-0001")
                    .delete_on_termination(true)
                    .build(),
            )
            .build();

        aws_sdk_ec2::types::Instance::builder()
            .instance_id(instance_id)
            .instance_type(aws_sdk_ec2::types::InstanceType::T3Micro)
            .image_id("ami-0001")
            .platform(aws_sdk_ec2::types::PlatformValues::Windows)
            .architecture(aws_sdk_ec2::types::ArchitectureValues::X8664)
            .key_name("main-key")
            .vpc_id("vpc-1000")
            .subnet_id("subnet-1000")
            .placement(
                aws_sdk_ec2::types::Placement::builder()
                    .availability_zone("ap-northeast-2a")
                    .build(),
            )
            .public_ip_address("3.3.3.3")
            .private_ip_address("10.0.0.20")
            .state(
                aws_sdk_ec2::types::InstanceState::builder()
                    .name(state)
                    .build(),
            )
            .monitoring(
                aws_sdk_ec2::types::Monitoring::builder()
                    .state(aws_sdk_ec2::types::MonitoringState::Enabled)
                    .build(),
            )
            .ebs_optimized(true)
            .iam_instance_profile(
                aws_sdk_ec2::types::IamInstanceProfile::builder()
                    .arn("arn:aws:iam::123456789012:instance-profile/web")
                    .build(),
            )
            .tags(ec2_test_tag("Name", name))
            .security_groups(
                aws_sdk_ec2::types::GroupIdentifier::builder()
                    .group_name("default")
                    .group_id("sg-1111")
                    .build(),
            )
            .block_device_mappings(bdm)
            .build()
    }

    #[test]
    fn ec2_instances_output_supports_query_shapes_and_default_object() {
        let reservation = aws_sdk_ec2::types::Reservation::builder()
            .instances(sample_instance(
                "i-bbbb",
                "web-b",
                aws_sdk_ec2::types::InstanceStateName::Running,
            ))
            .instances(sample_instance(
                "i-aaaa",
                "web-a",
                aws_sdk_ec2::types::InstanceStateName::Stopped,
            ))
            .build();

        let reservations = vec![reservation];

        let query_matrix = ec2_describe_instances_output(
            &reservations,
            Some("Reservations[*].Instances[*].[InstanceId,State.Name,Tags]"),
        )
        .expect("query matrix output");
        let matrix_json: Value = serde_json::from_str(&query_matrix).expect("valid json");
        assert_eq!(matrix_json[0][0][0], "i-aaaa");
        assert_eq!(matrix_json[0][1][0], "i-bbbb");
        assert_eq!(matrix_json[0][0][1], "stopped");

        let bdm_only = ec2_describe_instances_output(
            &reservations,
            Some("Reservations[0].Instances[0].BlockDeviceMappings"),
        )
        .expect("bdm output");
        let bdm_json: Value = serde_json::from_str(&bdm_only).expect("valid json");
        assert_eq!(bdm_json[0]["DeviceName"], "/dev/xvda");
        assert_eq!(bdm_json[0]["Ebs"]["VolumeId"], "vol-0001");

        let default_output =
            ec2_describe_instances_output(&reservations, None).expect("default output");
        let default_json: Value = serde_json::from_str(&default_output).expect("valid json");
        assert_eq!(
            default_json["Reservations"][0]["Instances"][0]["InstanceId"],
            "i-aaaa"
        );
        assert_eq!(
            default_json["Reservations"][0]["Instances"][0]["SecurityGroups"][0]["GroupId"],
            "sg-1111"
        );
    }

    #[test]
    fn ec2_volumes_output_keeps_size_and_iops_numeric() {
        let volumes = vec![
            aws_sdk_ec2::types::Volume::builder()
                .volume_id("vol-1")
                .size(8)
                .volume_type(aws_sdk_ec2::types::VolumeType::Gp3)
                .iops(3000)
                .encrypted(true)
                .build(),
        ];

        let out = ec2_describe_volumes_output(&volumes).expect("volume output");
        let json: Value = serde_json::from_str(&out).expect("valid json");
        assert_eq!(json["Volumes"][0]["Size"], 8);
        assert_eq!(json["Volumes"][0]["Iops"], 3000);
    }

    #[test]
    fn ec2_vpc_output_supports_query_and_default_shape() {
        let vpcs = vec![
            aws_sdk_ec2::types::Vpc::builder()
                .vpc_id("vpc-bbbb")
                .cidr_block("10.1.0.0/16")
                .state(aws_sdk_ec2::types::VpcState::Available)
                .tags(ec2_test_tag("Name", "vpc-b"))
                .build(),
            aws_sdk_ec2::types::Vpc::builder()
                .vpc_id("vpc-aaaa")
                .cidr_block("10.0.0.0/16")
                .state(aws_sdk_ec2::types::VpcState::Available)
                .tags(ec2_test_tag("Name", "vpc-a"))
                .build(),
        ];

        let query_out =
            ec2_describe_vpcs_output(&vpcs, Some("Vpcs[*].[VpcId,Tags]")).expect("query vpcs");
        let query_json: Value = serde_json::from_str(&query_out).expect("valid json");
        assert_eq!(query_json[0][0], "vpc-aaaa");
        assert_eq!(query_json[1][0], "vpc-bbbb");

        let default_out = ec2_describe_vpcs_output(&vpcs, Some("unsupported")).expect("vpcs");
        let default_json: Value = serde_json::from_str(&default_out).expect("valid json");
        assert_eq!(default_json["Vpcs"][0]["VpcId"], "vpc-aaaa");
        assert_eq!(default_json["Vpcs"][1]["CidrBlock"], "10.1.0.0/16");
    }

    #[test]
    fn ec2_subnet_and_nat_gateway_outputs_render_expected_fields() {
        let subnets = vec![
            aws_sdk_ec2::types::Subnet::builder()
                .subnet_id("subnet-1")
                .vpc_id("vpc-1")
                .cidr_block("10.0.1.0/24")
                .availability_zone("ap-northeast-2a")
                .state(aws_sdk_ec2::types::SubnetState::Available)
                .map_public_ip_on_launch(true)
                .available_ip_address_count(251)
                .tags(ec2_test_tag("Name", "public-a"))
                .build(),
        ];
        let subnets_out = ec2_describe_subnets_output(&subnets).expect("subnet output");
        let subnets_json: Value = serde_json::from_str(&subnets_out).expect("valid json");
        assert_eq!(subnets_json["Subnets"][0]["MapPublicIpOnLaunch"], true);
        assert_eq!(subnets_json["Subnets"][0]["AvailableIpAddressCount"], 251);

        let nat_gateways = vec![
            aws_sdk_ec2::types::NatGateway::builder()
                .nat_gateway_id("nat-1")
                .subnet_id("subnet-1")
                .state(aws_sdk_ec2::types::NatGatewayState::Available)
                .connectivity_type(aws_sdk_ec2::types::ConnectivityType::Public)
                .nat_gateway_addresses(
                    aws_sdk_ec2::types::NatGatewayAddress::builder()
                        .allocation_id("eipalloc-1")
                        .public_ip("52.0.0.1")
                        .private_ip("10.0.1.10")
                        .build(),
                )
                .tags(ec2_test_tag("Name", "nat-main"))
                .build(),
        ];
        let nat_out = ec2_describe_nat_gateways_output(&nat_gateways).expect("nat output");
        let nat_json: Value = serde_json::from_str(&nat_out).expect("valid json");
        assert_eq!(nat_json["NatGateways"][0]["NatGatewayId"], "nat-1");
        assert!(nat_json["NatGateways"][0].get("AvailabilityMode").is_none());
    }

    #[test]
    fn ec2_gateway_route_address_and_sg_outputs_match_cli_shape() {
        let igws = vec![
            aws_sdk_ec2::types::InternetGateway::builder()
                .internet_gateway_id("igw-1")
                .tags(ec2_test_tag("Name", "igw-main"))
                .attachments(
                    aws_sdk_ec2::types::InternetGatewayAttachment::builder()
                        .vpc_id("vpc-1")
                        .build(),
                )
                .build(),
        ];
        let igw_query = ec2_describe_internet_gateways_output(
            &igws,
            Some("InternetGateways[*].[InternetGatewayId,Tags,Attachments]"),
        )
        .expect("igw query");
        let igw_query_json: Value = serde_json::from_str(&igw_query).expect("valid json");
        assert_eq!(igw_query_json[0][0], "igw-1");

        let route_tables = vec![
            aws_sdk_ec2::types::RouteTable::builder()
                .route_table_id("rtb-1")
                .tags(ec2_test_tag("Name", "main-rt"))
                .routes(
                    aws_sdk_ec2::types::Route::builder()
                        .destination_cidr_block("0.0.0.0/0")
                        .gateway_id("igw-1")
                        .state(aws_sdk_ec2::types::RouteState::Active)
                        .build(),
                )
                .associations(
                    aws_sdk_ec2::types::RouteTableAssociation::builder()
                        .subnet_id("subnet-1")
                        .build(),
                )
                .build(),
        ];
        let route_query = ec2_describe_route_tables_output(
            &route_tables,
            Some("RouteTables[*].[RouteTableId,Tags,Routes,Associations]"),
        )
        .expect("route query");
        let route_query_json: Value = serde_json::from_str(&route_query).expect("valid json");
        assert_eq!(route_query_json[0][0], "rtb-1");

        let addresses = vec![
            aws_sdk_ec2::types::Address::builder()
                .allocation_id("eipalloc-1")
                .public_ip("52.0.0.10")
                .association_id("eipassoc-1")
                .instance_id("i-1")
                .private_ip_address("10.0.1.11")
                .tags(ec2_test_tag("Name", "edge-eip"))
                .build(),
        ];
        let addr_out = ec2_describe_addresses_output(&addresses).expect("address output");
        let addr_json: Value = serde_json::from_str(&addr_out).expect("valid json");
        assert_eq!(addr_json["Addresses"][0]["AllocationId"], "eipalloc-1");

        let perm = aws_sdk_ec2::types::IpPermission::builder()
            .ip_protocol("tcp")
            .from_port(443)
            .to_port(443)
            .ip_ranges(
                aws_sdk_ec2::types::IpRange::builder()
                    .cidr_ip("0.0.0.0/0")
                    .description("https")
                    .build(),
            )
            .build();
        let sgs = vec![
            aws_sdk_ec2::types::SecurityGroup::builder()
                .group_id("sg-1")
                .group_name("web")
                .description("web access")
                .vpc_id("vpc-1")
                .ip_permissions(perm.clone())
                .ip_permissions_egress(perm)
                .tags(ec2_test_tag("Name", "web-sg"))
                .build(),
        ];
        let sg_out = ec2_describe_security_groups_output(&sgs).expect("sg output");
        let sg_json: Value = serde_json::from_str(&sg_out).expect("valid json");
        assert_eq!(sg_json["SecurityGroups"][0]["GroupId"], "sg-1");
        assert_eq!(
            sg_json["SecurityGroups"][0]["IpPermissions"][0]["FromPort"],
            443
        );
    }

    #[test]
    fn ec2_images_output_adds_name_tag_when_missing() {
        let images = vec![
            aws_sdk_ec2::types::Image::builder()
                .image_id("ami-1")
                .name("base-ami")
                .build(),
        ];
        let out = ec2_describe_images_output(&images).expect("images output");
        let json: Value = serde_json::from_str(&out).expect("valid json");
        assert_eq!(json["Images"][0]["ImageId"], "ami-1");
        let tags = json["Images"][0]["Tags"].as_array().expect("tags");
        assert!(
            tags.iter()
                .any(|t| t["Key"] == "Name" && t["Value"] == "base-ami")
        );
    }

    #[test]
    fn ecr_outputs_follow_expected_object_shapes() {
        let repositories = vec![
            aws_sdk_ecr::types::Repository::builder()
                .repository_name("repo-b")
                .repository_uri("123456789012.dkr.ecr.ap-northeast-2.amazonaws.com/repo-b")
                .build(),
            aws_sdk_ecr::types::Repository::builder()
                .repository_name("repo-a")
                .repository_uri("123456789012.dkr.ecr.ap-northeast-2.amazonaws.com/repo-a")
                .build(),
        ];
        let repos_out = ecr_describe_repositories_output(&repositories).expect("repos output");
        let repos_json: Value = serde_json::from_str(&repos_out).expect("valid json");
        assert_eq!(repos_json["repositories"][0]["repositoryName"], "repo-a");
        assert_eq!(
            repos_json["repositories"][0]["encryptionConfiguration"]["encryptionType"],
            "AES256"
        );

        let images = vec![
            aws_sdk_ecr::types::ImageDetail::builder()
                .image_digest("sha256:abc123")
                .build(),
        ];
        let images_out = ecr_describe_images_output(&images).expect("images output");
        let images_json: Value = serde_json::from_str(&images_out).expect("valid json");
        assert_eq!(
            images_json["imageDetails"][0]["imageDigest"],
            "sha256:abc123"
        );
    }

    #[test]
    fn elbv2_outputs_keep_sorting_and_nested_keys() {
        let lbs = vec![
            aws_sdk_elasticloadbalancingv2::types::LoadBalancer::builder()
                .load_balancer_name("lb-b")
                .load_balancer_arn("arn:aws:elasticloadbalancing:...:loadbalancer/app/lb-b/1")
                .scheme(
                    aws_sdk_elasticloadbalancingv2::types::LoadBalancerSchemeEnum::InternetFacing,
                )
                .build(),
            aws_sdk_elasticloadbalancingv2::types::LoadBalancer::builder()
                .load_balancer_name("lb-a")
                .load_balancer_arn("arn:aws:elasticloadbalancing:...:loadbalancer/app/lb-a/1")
                .scheme(
                    aws_sdk_elasticloadbalancingv2::types::LoadBalancerSchemeEnum::InternetFacing,
                )
                .build(),
        ];
        let lbs_out = elbv2_describe_load_balancers_output(&lbs).expect("lbs output");
        let lbs_json: Value = serde_json::from_str(&lbs_out).expect("valid json");
        assert_eq!(lbs_json["LoadBalancers"][0]["LoadBalancerName"], "lb-a");

        let listeners = vec![
            aws_sdk_elasticloadbalancingv2::types::Listener::builder()
                .port(443)
                .protocol(aws_sdk_elasticloadbalancingv2::types::ProtocolEnum::Https)
                .default_actions(
                    aws_sdk_elasticloadbalancingv2::types::Action::builder()
                        .r#type(aws_sdk_elasticloadbalancingv2::types::ActionTypeEnum::Forward)
                        .target_group_arn("arn:aws:elasticloadbalancing:...:targetgroup/tg-a/1")
                        .build(),
                )
                .build(),
        ];
        let listeners_out = elbv2_describe_listeners_output(&listeners).expect("listeners output");
        let listeners_json: Value = serde_json::from_str(&listeners_out).expect("valid json");
        assert_eq!(listeners_json["Listeners"][0]["Port"], 443);

        let target_groups = vec![
            aws_sdk_elasticloadbalancingv2::types::TargetGroup::builder()
                .target_group_name("tg-a")
                .target_group_arn("arn:aws:elasticloadbalancing:...:targetgroup/tg-a/1")
                .protocol(aws_sdk_elasticloadbalancingv2::types::ProtocolEnum::Http)
                .port(80)
                .build(),
        ];
        let tgs_out = elbv2_describe_target_groups_output(&target_groups).expect("tgs output");
        let tgs_json: Value = serde_json::from_str(&tgs_out).expect("valid json");
        assert_eq!(tgs_json["TargetGroups"][0]["TargetGroupName"], "tg-a");

        let health_descriptions = vec![
            aws_sdk_elasticloadbalancingv2::types::TargetHealthDescription::builder()
                .target(
                    aws_sdk_elasticloadbalancingv2::types::TargetDescription::builder()
                        .id("i-1111")
                        .port(80)
                        .build(),
                )
                .target_health(
                    aws_sdk_elasticloadbalancingv2::types::TargetHealth::builder()
                        .state(
                            aws_sdk_elasticloadbalancingv2::types::TargetHealthStateEnum::Healthy,
                        )
                        .build(),
                )
                .build(),
        ];
        let health_out =
            elbv2_describe_target_health_output(&health_descriptions).expect("health output");
        let health_json: Value = serde_json::from_str(&health_out).expect("valid json");
        assert_eq!(
            health_json["TargetHealthDescriptions"][0]["Target"]["Id"],
            "i-1111"
        );
    }

    #[test]
    fn iam_outputs_sort_and_decode_policy_documents() {
        let attached = vec![
            aws_sdk_iam::types::AttachedPolicy::builder()
                .policy_name("zz-policy")
                .policy_arn("arn:aws:iam::123456789012:policy/zz-policy")
                .build(),
            aws_sdk_iam::types::AttachedPolicy::builder()
                .policy_name("aa-policy")
                .policy_arn("arn:aws:iam::123456789012:policy/aa-policy")
                .build(),
        ];
        let attached_out =
            iam_list_attached_role_policies_output(&attached).expect("attached policies output");
        let attached_json: Value = serde_json::from_str(&attached_out).expect("valid json");
        assert_eq!(
            attached_json["AttachedPolicies"][0]["PolicyName"],
            "aa-policy"
        );

        let names_out =
            iam_list_role_policies_output(&["inline-z".to_string(), "inline-a".to_string()])
                .expect("policy names output");
        let names_json: Value = serde_json::from_str(&names_out).expect("valid json");
        assert_eq!(names_json["PolicyNames"][0], "inline-a");
        assert_eq!(names_json["PolicyNames"][1], "inline-z");

        let encoded = "%7B%22Version%22%3A%222012-10-17%22%7D";
        let policy_out = iam_get_role_policy_output(encoded).expect("policy output");
        let policy_json: Value = serde_json::from_str(&policy_out).expect("valid json");
        assert_eq!(policy_json["PolicyDocument"]["Version"], "2012-10-17");
    }

    #[test]
    fn parse_name_tag_extracts_name_key() {
        let tags = r#"[{"Key": "Name", "Value": "prod-vpc"}, {"Key": "Env", "Value": "prod"}]"#;
        assert_eq!(parse_name_tag(tags), "prod-vpc");
    }

    #[test]
    fn parse_resources_from_json_extracts_prefixed_ids() {
        let payload = r#"
            [
              ["vpc-aaaa1111", [{"Key": "Name", "Value": "main-vpc"}]],
              ["vpc-bbbb2222", [{"Key": "Name", "Value": "shared-vpc"}]]
            ]
        "#;
        let resources = parse_resources_from_json(payload, "vpc-");
        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0].id, "vpc-aaaa1111");
        assert_eq!(resources[1].id, "vpc-bbbb2222");
    }

    #[test]
    fn scenario_lt_catalog_offline_detail_projection() {
        let payload = r#"
            [
              ["lt-aaaa1111", [{"Key": "Name", "Value": "web-template"}]],
              ["lt-bbbb2222", [{"Key": "Name", "Value": "batch-template"}]]
            ]
        "#;
        let resources = parse_resources_from_json(payload, "lt-");
        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0].id, "lt-aaaa1111");
        assert_eq!(resources[1].id, "lt-bbbb2222");
    }

    #[test]
    fn extract_json_value_returns_expected_string() {
        let payload =
            r#"{"RoleName": "demo-role", "Arn": "arn:aws:iam::123456789012:role/demo-role"}"#;
        assert_eq!(
            extract_json_value(payload, "RoleName"),
            Some("demo-role".to_string())
        );
    }

    #[test]
    fn arg_value_extracts_flag_value_pairs() {
        let args = [
            "ec2",
            "describe-vpcs",
            "--vpc-ids",
            "vpc-1234",
            "--output",
            "json",
        ];
        assert_eq!(arg_value(&args, "--vpc-ids"), Some("vpc-1234"));
        assert_eq!(arg_value(&args, "--query"), None);
    }

    #[test]
    fn extract_tags_deduplicates_keys() {
        let payload = r#"
            {
              "Tags": [
                {"Key": "Name", "Value": "main"},
                {"Key": "Env", "Value": "prod"},
                {"Key": "Name", "Value": "ignored"}
              ]
            }
        "#;
        let tags = extract_tags(payload);
        assert_eq!(
            tags,
            vec![
                ("Name".to_string(), "main".to_string()),
                ("Env".to_string(), "prod".to_string())
            ]
        );
    }

    #[test]
    fn parse_name_tag_handles_value_before_key_shape() {
        let payload = r#"{"Value": "vpc-main", "Key": "Name"}"#;
        assert_eq!(parse_name_tag(payload), "vpc-main");
    }

    #[test]
    fn aws_resource_display_prefers_name_when_present() {
        let named = AwsResource {
            name: "web".to_string(),
            id: "i-1234".to_string(),
            state: String::new(),
            az: String::new(),
            cidr: String::new(),
        };
        assert_eq!(named.display(), "web (i-1234)");

        let unnamed = AwsResource {
            name: String::new(),
            id: "i-5678".to_string(),
            state: String::new(),
            az: String::new(),
            cidr: String::new(),
        };
        assert_eq!(unnamed.display(), "i-5678");
    }

    #[test]
    fn parse_filter_value_returns_none_for_mismatched_name() {
        let raw = "Name=tag:Env,Values=prod,staging";
        assert_eq!(parse_filter_value(raw, "vpc-id"), None);
    }

    #[test]
    fn parse_tags_helpers_map_key_value_pairs() {
        let ec2_tags = vec![
            aws_sdk_ec2::types::Tag::builder()
                .key("Name")
                .value("web")
                .build(),
        ];
        let iam_tags = vec![
            aws_sdk_iam::types::Tag::builder()
                .key("Env")
                .value("prod")
                .build()
                .expect("iam tag"),
        ];

        let ec2 = parse_tags_ec2(&ec2_tags);
        let iam = parse_tags_iam(&iam_tags);

        assert_eq!(ec2[0]["Key"], "Name");
        assert_eq!(ec2[0]["Value"], "web");
        assert_eq!(iam[0]["Key"], "Env");
        assert_eq!(iam[0]["Value"], "prod");
    }

    #[test]
    fn parse_ip_permissions_maps_ipv4_ipv6_and_sg_pairs() {
        let perm = aws_sdk_ec2::types::IpPermission::builder()
            .ip_protocol("tcp")
            .from_port(80)
            .to_port(80)
            .ip_ranges(
                aws_sdk_ec2::types::IpRange::builder()
                    .cidr_ip("0.0.0.0/0")
                    .description("web")
                    .build(),
            )
            .ipv6_ranges(
                aws_sdk_ec2::types::Ipv6Range::builder()
                    .cidr_ipv6("::/0")
                    .description("v6")
                    .build(),
            )
            .user_id_group_pairs(
                aws_sdk_ec2::types::UserIdGroupPair::builder()
                    .group_id("sg-1234")
                    .description("peer")
                    .build(),
            )
            .build();

        let mapped = parse_ip_permissions(&[perm]);
        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0]["IpProtocol"], "tcp");
        assert_eq!(mapped[0]["FromPort"], 80);
        assert_eq!(mapped[0]["IpRanges"][0]["CidrIp"], "0.0.0.0/0");
        assert_eq!(mapped[0]["Ipv6Ranges"][0]["CidrIpv6"], "::/0");
        assert_eq!(mapped[0]["UserIdGroupPairs"][0]["GroupId"], "sg-1234");
    }

    #[test]
    fn elbv2_helper_mappers_generate_expected_keys() {
        let lb = aws_sdk_elasticloadbalancingv2::types::LoadBalancer::builder()
            .load_balancer_name("alb-main")
            .load_balancer_arn(
                "arn:aws:elasticloadbalancing:ap-northeast-2:123456789012:loadbalancer/app/alb-main/1234",
            )
            .dns_name("alb.example.com")
            .r#type(aws_sdk_elasticloadbalancingv2::types::LoadBalancerTypeEnum::Application)
            .scheme(aws_sdk_elasticloadbalancingv2::types::LoadBalancerSchemeEnum::InternetFacing)
            .vpc_id("vpc-1111")
            .ip_address_type(
                aws_sdk_elasticloadbalancingv2::types::IpAddressType::Ipv4,
            )
            .availability_zones(
                aws_sdk_elasticloadbalancingv2::types::AvailabilityZone::builder()
                    .zone_name("ap-northeast-2a")
                    .build(),
            )
            .state(
                aws_sdk_elasticloadbalancingv2::types::LoadBalancerState::builder()
                    .code(aws_sdk_elasticloadbalancingv2::types::LoadBalancerStateEnum::Active)
                    .build(),
            )
            .build();

        let lb_json = lb_to_json(&lb);
        assert_eq!(lb_json["LoadBalancerName"], "alb-main");
        assert_eq!(lb_json["State"]["Code"], "active");

        let tg = aws_sdk_elasticloadbalancingv2::types::TargetGroup::builder()
            .target_group_name("tg-main")
            .target_group_arn("arn:aws:elasticloadbalancing:...:targetgroup/tg-main/abcd")
            .protocol(aws_sdk_elasticloadbalancingv2::types::ProtocolEnum::Http)
            .port(80)
            .target_type(aws_sdk_elasticloadbalancingv2::types::TargetTypeEnum::Instance)
            .health_check_protocol(aws_sdk_elasticloadbalancingv2::types::ProtocolEnum::Http)
            .health_check_path("/health")
            .healthy_threshold_count(2)
            .unhealthy_threshold_count(3)
            .build();
        let tg_json = target_group_to_json(&tg);
        assert_eq!(tg_json["TargetGroupName"], "tg-main");
        assert_eq!(tg_json["Port"], 80);
    }

    #[test]
    fn value_to_json_string_serializes_value() {
        let value = serde_json::json!({"ok": true, "count": 2});
        let out = value_to_json_string(value).expect("serialize");
        assert!(out.contains("\"ok\":true"));
        assert!(out.contains("\"count\":2"));
    }
}
