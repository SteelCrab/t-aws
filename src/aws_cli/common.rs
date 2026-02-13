use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::{Mutex, OnceLock};
use tokio::runtime::Runtime;

static REGION: Mutex<Option<String>> = Mutex::new(None);

pub fn set_region(region: &str) {
    if let Ok(mut r) = REGION.lock() {
        *r = Some(region.to_string());
    }
}

#[derive(Debug, Clone)]
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
        return None;
    }

    let service = args[0];
    let operation = args[1];

    get_runtime().block_on(async {
        let config = get_sdk_config().await;

        match service {
            "ec2" => run_ec2_request(&config, operation, args).await,
            "ecr" => run_ecr_request(&config, operation, args).await,
            "elbv2" => run_elbv2_request(&config, operation, args).await,
            "iam" => run_iam_request(&config, operation, args).await,
            "sts" => run_sts_request(&config, operation).await,
            _ => None,
        }
    })
}

pub fn check_aws_login() -> Result<String, String> {
    get_runtime().block_on(async {
        let config = get_sdk_config().await;
        let client = aws_sdk_sts::Client::new(&config);

        match client.get_caller_identity().send().await {
            Ok(output) => {
                let account = output.account().unwrap_or_default();
                let arn = output.arn().unwrap_or_default();
                Ok(format!("{} ({})", account, arn))
            }
            Err(e) => Err(format!("AWS 로그인 필요: {}", e)),
        }
    })
}

fn arg_value<'a>(args: &'a [&str], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find_map(|pair| (pair[0] == flag).then_some(pair[1]))
}

fn parse_filter_value(raw: &str, expected_name: &str) -> Option<String> {
    let mut name = None;
    let mut values: Option<String> = None;
    let segments = raw.split(',').collect::<Vec<_>>();
    let mut i = 0;

    while i < segments.len() {
        let segment = segments[i];
        if let Some((k, v)) = segment.split_once('=') {
            match k {
                "Name" => name = Some(v),
                "Values" => {
                    let mut combined = v.to_string();
                    let mut j = i + 1;
                    while j < segments.len() && !segments[j].contains('=') {
                        if !segments[j].is_empty() {
                            combined.push(',');
                            combined.push_str(segments[j]);
                        }
                        j += 1;
                    }
                    values = Some(combined);
                    i = j;
                    continue;
                }
                _ => {}
            }
        }
        i += 1;
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

    let output = req.send().await.ok()?;

    if let Some(query) = arg_value(args, "--query") {
        if query == "Reservations[*].Instances[*].[InstanceId,State.Name,Tags]" {
            let mut by_reservation = Vec::new();

            for reservation in output.reservations() {
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

        if query == "Reservations[0].Instances[0].BlockDeviceMappings" {
            let mappings = output
                .reservations()
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
    }

    let reservations = output
        .reservations()
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

    let volumes = output
        .volumes()
        .iter()
        .map(|volume| {
            json!({
                "VolumeId": volume.volume_id().unwrap_or_default(),
                "Size": volume.size().unwrap_or_default(),
                "VolumeType": volume.volume_type().map(|v| v.as_str()).unwrap_or("unknown"),
                "Iops": volume.iops().map(i64::from).unwrap_or_default(),
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

    let output = req.send().await.ok()?;

    if let Some(query) = arg_value(args, "--query")
        && query == "Vpcs[*].[VpcId,Tags]"
    {
        let mut rows = output
            .vpcs()
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

    let mut vpcs = output
        .vpcs()
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

    let output = req.send().await.ok()?;

    let mut subnets = output
        .subnets()
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

    let output = req.send().await.ok()?;

    if let Some(query) = arg_value(args, "--query")
        && query == "InternetGateways[*].[InternetGatewayId,Tags,Attachments]"
    {
        let rows = output
            .internet_gateways()
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

    let gateways = output
        .internet_gateways()
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

    let output = req.send().await.ok()?;

    let nat_gateways = output
        .nat_gateways()
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

    let output = req.send().await.ok()?;

    if let Some(query) = arg_value(args, "--query")
        && query == "RouteTables[*].[RouteTableId,Tags,Routes,Associations]"
    {
        let rows = output
            .route_tables()
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

    let route_tables = output
        .route_tables()
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

    let addresses = output
        .addresses()
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

    let mut groups = output
        .security_groups()
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

    let images = output
        .images()
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

    let mut repositories = output
        .repositories()
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

    let image_details = output
        .image_details()
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
    let mut load_balancers = output
        .load_balancers()
        .iter()
        .map(lb_to_json)
        .collect::<Vec<_>>();

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

    let mut listeners = output
        .listeners()
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

    let mut target_groups = output
        .target_groups()
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

    let mut descriptions = output
        .target_health_descriptions()
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

    let mut attached_policies = output
        .attached_policies()
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

    let mut policy_names = output
        .policy_names()
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

    value_to_json_string(json!({
        "PolicyDocument": parse_policy_json(Some(output.policy_document()))
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
    let profile_name = std::env::var("AWS_PROFILE").unwrap_or_else(|_| "default".to_string());

    let mut config_loader =
        aws_config::defaults(aws_config::BehaviorVersion::latest()).profile_name(&profile_name);

    // Get region from REGION mutex if set
    if let Ok(r) = REGION.lock()
        && let Some(ref region_str) = *r
    {
        config_loader = config_loader.region(aws_config::Region::new(region_str.clone()));
    }

    config_loader.load().await
}
