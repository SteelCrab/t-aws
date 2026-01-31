use serde::Deserialize;
use std::process::Command;
use std::sync::Mutex;

static REGION: Mutex<Option<String>> = Mutex::new(None);

pub fn set_region(region: &str) {
    if let Ok(mut r) = REGION.lock() {
        *r = Some(region.to_string());
    }
}

fn get_region_args() -> Vec<String> {
    if let Ok(r) = REGION.lock()
        && let Some(ref region) = *r
    {
        return vec!["--region".to_string(), region.clone()];
    }
    Vec::new()
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

// Serde structures for AWS API responses
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SubnetResponse {
    subnets: Vec<Subnet>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Subnet {
    subnet_id: String,
    vpc_id: String,
    cidr_block: String,
    availability_zone: String,
    state: String,
    #[serde(default)]
    tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Tag {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NatGatewayResponse {
    nat_gateways: Vec<NatGateway>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NatGateway {
    nat_gateway_id: String,
    subnet_id: String,
    state: String,
    #[serde(default)]
    tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SecurityGroupsResponse {
    security_groups: Vec<SecurityGroupInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SecurityGroupInfo {
    group_id: String,
    group_name: String,
    description: String,
    vpc_id: String,
    #[serde(default)]
    ip_permissions: Vec<IpPermission>,
    #[serde(default)]
    ip_permissions_egress: Vec<IpPermission>,
    #[serde(default)]
    tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct IpPermission {
    ip_protocol: String,
    #[serde(default)]
    from_port: Option<i32>,
    #[serde(default)]
    to_port: Option<i32>,
    #[serde(default)]
    ip_ranges: Vec<IpRange>,
    #[serde(default)]
    ipv6_ranges: Vec<Ipv6Range>,
    #[serde(default)]
    user_id_group_pairs: Vec<UserIdGroupPair>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct IpRange {
    cidr_ip: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Ipv6Range {
    cidr_ipv6: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UserIdGroupPair {
    group_id: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LoadBalancersResponse {
    load_balancers: Vec<LoadBalancerInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LoadBalancerInfo {
    load_balancer_name: String,
    load_balancer_arn: String,
    #[serde(rename = "DNSName")]
    dns_name: String,
    #[serde(rename = "Type")]
    lb_type: String,
    scheme: String,
    #[serde(rename = "VpcId")]
    vpc_id: String,
    #[serde(default)]
    availability_zones: Vec<AvailabilityZone>,
    #[serde(default)]
    security_groups: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AvailabilityZone {
    zone_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ListenersResponse {
    listeners: Vec<ListenerData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ListenerData {
    port: i32,
    protocol: String,
    #[serde(default)]
    default_actions: Vec<DefaultAction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DefaultAction {
    #[serde(rename = "Type")]
    action_type: String,
    #[serde(default)]
    target_group_arn: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TargetGroupsResponse {
    target_groups: Vec<TargetGroupData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TargetGroupData {
    target_group_name: String,
    target_group_arn: String,
    protocol: String,
    port: i32,
    target_type: String,
    health_check_protocol: String,
    health_check_path: String,
    healthy_threshold_count: i32,
    unhealthy_threshold_count: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TargetHealthResponse {
    target_health_descriptions: Vec<TargetHealthDescription>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TargetHealthDescription {
    target: Target,
    #[allow(dead_code)]
    health_check_port: String,
    target_health: TargetHealth,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Target {
    id: String,
    port: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TargetHealth {
    state: String,
}

// ECR Serde structures
#[derive(Debug, Deserialize)]
struct EcrRepositoriesResponse {
    repositories: Vec<EcrRepository>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EcrRepository {
    repository_name: String,
    repository_uri: String,
    #[serde(default)]
    image_tag_mutability: String,
    #[serde(default)]
    encryption_configuration: Option<EcrEncryptionConfiguration>,
    #[serde(default)]
    created_at: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EcrEncryptionConfiguration {
    encryption_type: String,
    #[serde(default)]
    kms_key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EcrImagesResponse {
    image_details: Vec<EcrImageDetail>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EcrImageDetail {
    #[allow(dead_code)]
    image_digest: String,
}

fn run_aws_cli(args: &[&str]) -> Option<String> {
    use std::io::Write;

    let region_args = get_region_args();
    let mut cmd = Command::new("aws");
    cmd.args(args);
    for arg in &region_args {
        cmd.arg(arg);
    }

    // 디버그: 실행할 명령 로깅
    let cmd_str = format!("aws {} {}", args.join(" "), region_args.join(" "));
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/emd_debug.log")
    {
        let _ = writeln!(f, "[START] {}", cmd_str);
    }

    let output = cmd.output().ok()?;

    // 디버그: 결과 로깅
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/awsmd_debug.log")
    {
        let _ = writeln!(
            f,
            "[END] {} - success: {}, stdout_len: {}",
            cmd_str,
            output.status.success(),
            output.stdout.len()
        );
    }

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

pub fn check_aws_login() -> Result<String, String> {
    let output = Command::new("aws")
        .args(["sts", "get-caller-identity", "--output", "json"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let json = String::from_utf8_lossy(&o.stdout);
            let account = extract_json_value(&json, "Account").unwrap_or_default();
            let arn = extract_json_value(&json, "Arn").unwrap_or_default();
            Ok(format!("{} ({})", account, arn))
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            Err(format!(
                "AWS 로그인 필요: {}",
                err.lines().next().unwrap_or("")
            ))
        }
        Err(e) => Err(format!("AWS CLI 실행 실패: {}", e)),
    }
}

fn extract_json_value(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\": \"", key);
    if let Some(start) = json.find(&pattern) {
        let offset = start + pattern.len();
        if let Some(end) = json[offset..].find('"') {
            return Some(json[offset..offset + end].to_string());
        }
    }
    None
}

fn parse_name_tag(tags_json: &str) -> String {
    if let Some(start) = tags_json.find("\"Key\": \"Name\"")
        && let Some(value_start) = tags_json[start..].find("\"Value\": \"")
    {
        let offset = start + value_start + 10;
        if let Some(end) = tags_json[offset..].find('"') {
            return tags_json[offset..offset + end].to_string();
        }
    }
    // Try reverse order
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

pub fn list_vpcs() -> Vec<AwsResource> {
    let output = match run_aws_cli(&[
        "ec2",
        "describe-vpcs",
        "--query",
        "Vpcs[*].[VpcId,Tags]",
        "--output",
        "json",
    ]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    parse_resources_from_json(&output, "vpc-")
}

pub fn list_subnets(vpc_id: &str) -> Vec<AwsResource> {
    // 필터 없이 전체 조회 후 코드에서 필터링 (AWS CLI 필터 문제 회피)
    let output = match run_aws_cli(&["ec2", "describe-subnets", "--output", "json"]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    // serde_json으로 빠르게 파싱
    let response: SubnetResponse = match serde_json::from_str(&output) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    // VPC ID로 필터링하고 AwsResource로 변환
    response
        .subnets
        .into_iter()
        .filter(|s| s.vpc_id == vpc_id)
        .map(|s| {
            let name = s
                .tags
                .iter()
                .find(|t| t.key == "Name")
                .map(|t| t.value.clone())
                .unwrap_or_else(|| s.subnet_id.clone());

            AwsResource {
                name,
                id: s.subnet_id,
                state: s.state,
                az: s.availability_zone,
                cidr: s.cidr_block,
            }
        })
        .collect()
}

pub fn list_instances() -> Vec<AwsResource> {
    let output = match run_aws_cli(&[
        "ec2",
        "describe-instances",
        "--query",
        "Reservations[*].Instances[*].[InstanceId,State.Name,Tags]",
        "--output",
        "json",
    ]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    parse_instance_resources(&output)
}

fn parse_resources_from_json(json: &str, prefix: &str) -> Vec<AwsResource> {
    let mut resources = Vec::new();

    // Simple parsing - find all IDs with the prefix
    let mut search_start = 0;
    while let Some(pos) = json[search_start..].find(prefix) {
        let start = search_start + pos;
        // Find the end of the ID (until quote)
        if let Some(end) = json[start..].find('"') {
            let id = &json[start..start + end];
            if id.starts_with(prefix) && !id.contains(' ') {
                // Find associated tags
                let section_end = json[start..]
                    .find(']')
                    .map(|p| start + p)
                    .unwrap_or(json.len());
                let tag_start = start; // Assuming tags are right after the ID in the query
                let tag_end = section_end; // Assuming tags are within the same block
                let tags_json = &json[tag_start..tag_end];
                let name = parse_name_tag(tags_json);

                // After tags, structure is: "az", "state", "cidr"
                // This logic is specific to the `describe-subnets` query format.
                // For `describe-vpcs`, there's no AZ, State, CIDR directly after tags.
                // So, we'll keep them empty for general `parse_resources_from_json`
                // and let `parse_subnet_resources` handle the specific extraction.
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

pub fn parse_route_tables(json: &str) -> Vec<RouteTableDetail> {
    let mut details = Vec::new();
    let mut search_start = 0;

    // Structure: ["rtb-id", [Tags], [Routes], [Associations]]
    while let Some(pos) = json[search_start..].find("rtb-") {
        let start = search_start + pos;
        if let Some(end) = json[start..].find('"') {
            let id = &json[start..start + end];

            // Basic validation
            if !id.starts_with("rtb-") {
                search_start = start + end;
                continue;
            }

            // Find Tag block
            let tag_start_offset = json[start..].find('[').unwrap_or_default();
            let tag_start = start + tag_start_offset;
            let tag_end_offset = json[tag_start..].find(']').unwrap_or_default();
            let tag_end = tag_start + tag_end_offset + 1;
            let tags_json = &json[tag_start..tag_end];
            let name = parse_name_tag(tags_json);

            // Find Routes block (after tags)
            let route_start_offset = json[tag_end..].find('[').unwrap_or_default();
            let route_start = tag_end + route_start_offset;
            let route_end_offset = find_balanced_bracket_end(&json[route_start..]).unwrap_or(0);
            let route_end = route_start + route_end_offset;
            let routes_json = &json[route_start..route_end];
            let routes = extract_routes(routes_json);

            // Find Associations block (after routes)
            let assoc_start_offset = json[route_end..].find('[').unwrap_or_default();
            let assoc_start = route_end + assoc_start_offset;
            let assoc_end_offset = find_balanced_bracket_end(&json[assoc_start..]).unwrap_or(0);
            let assoc_end = assoc_start + assoc_end_offset;
            let assocs_json = &json[assoc_start..assoc_end];
            let associations = extract_associations(assocs_json);

            details.push(RouteTableDetail {
                name: if name.is_empty() {
                    id.to_string()
                } else {
                    name
                },
                id: id.to_string(),
                routes,
                associations,
            });

            search_start = assoc_end;
        } else {
            break;
        }
    }
    details
}

fn find_balanced_bracket_end(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        if c == '[' {
            depth += 1;
        } else if c == ']' {
            depth -= 1;
            if depth == 0 {
                return Some(i + 1);
            }
        }
    }
    None
}

fn extract_routes(json: &str) -> Vec<String> {
    let mut routes = Vec::new();
    let mut search_start = 0;

    // Pattern: "DestinationCidrBlock": "..." ... "GatewayId": "..."
    // We want to extract objects basically.
    // Iterating by looking for DestinationCidrBlock
    while let Some(pos) = json[search_start..].find("\"DestinationCidrBlock\": \"") {
        let block_start = search_start + pos;
        // Find end of this route object to limit search
        let obj_end = if let Some(end) = json[block_start..].find('}') {
            block_start + end
        } else {
            json.len()
        };

        let section = &json[block_start..obj_end];

        let dest = extract_json_value(section, "DestinationCidrBlock").unwrap_or_default();
        let gateway = extract_json_value(section, "GatewayId").unwrap_or_default();
        let nat = extract_json_value(section, "NatGatewayId").unwrap_or_default();
        let target = if !gateway.is_empty() { gateway } else { nat };

        routes.push(format!("{} -> {}", dest, target));

        search_start = obj_end;
    }
    routes
}

fn extract_associations(json: &str) -> Vec<String> {
    let mut assocs = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = json[search_start..].find("\"SubnetId\": \"") {
        let block_start = search_start + pos;
        let obj_end = if let Some(end) = json[block_start..].find('}') {
            block_start + end
        } else {
            json.len()
        };

        let section = &json[block_start..obj_end];
        let subnet_id = extract_json_value(section, "SubnetId").unwrap_or_default();

        let name = get_subnet_name(&subnet_id);
        assocs.push(format!("{} ({})", name, subnet_id));

        search_start = obj_end;
    }
    assocs
}

fn parse_instance_resources(json: &str) -> Vec<AwsResource> {
    let mut resources = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = json[search_start..].find("i-") {
        let start = search_start + pos;
        if let Some(end) = json[start..].find('"') {
            let id = &json[start..start + end];
            // Validate it's an instance ID (not i- in some other context)
            if id.starts_with("i-")
                && id.len() > 3
                && id.chars().skip(2).all(|c| c.is_alphanumeric())
            {
                // Find the section for this instance
                let section_start = json[..start].rfind('[').unwrap_or(0);
                let section_end = json[start..]
                    .find(']')
                    .map(|p| start + p)
                    .unwrap_or(json.len());
                let section = &json[section_start..section_end];

                let name = parse_name_tag(section);
                let state = extract_state(section);

                let display_name = if name.is_empty() {
                    id.to_string()
                } else {
                    name
                };

                resources.push(AwsResource {
                    name: format!("{} - {} - {}", display_name, id, state),
                    id: id.to_string(),
                    state: state.clone(),
                    az: String::new(),
                    cidr: String::new(),
                });
            }
            search_start = start + end;
        } else {
            break;
        }
    }

    // Remove duplicates
    resources.dedup_by(|a, b| a.id == b.id);
    resources
}

fn extract_state(json: &str) -> String {
    for state in [
        "running",
        "stopped",
        "pending",
        "terminated",
        "stopping",
        "shutting-down",
    ] {
        if json.contains(state) {
            return state.to_string();
        }
    }
    "unknown".to_string()
}

#[derive(Debug, Clone)]
pub struct VolumeDetail {
    pub device_name: String,
    pub volume_id: String,
    pub size_gb: i64,
    pub volume_type: String,
    pub iops: Option<i64>,
    pub encrypted: bool,
    pub delete_on_termination: bool,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Ec2Detail {
    pub name: String,
    pub instance_id: String,
    pub instance_type: String,
    pub ami: String,
    pub key_pair: String,
    pub vpc: String,
    pub subnet: String,
    pub public_ip: String,
    pub private_ip: String,
    pub security_groups: Vec<String>,
    pub state: String,
    pub tags: Vec<(String, String)>,
    pub volumes: Vec<VolumeDetail>,
    pub user_data: Option<String>,
}

pub fn get_instance_detail(instance_id: &str) -> Option<Ec2Detail> {
    let output = run_aws_cli(&[
        "ec2",
        "describe-instances",
        "--instance-ids",
        instance_id,
        "--output",
        "json",
    ])?;

    let json = &output;

    let instance_type = extract_json_value(json, "InstanceType").unwrap_or_default();
    let ami_id = extract_json_value(json, "ImageId").unwrap_or_default();
    let ami = if !ami_id.is_empty() {
        get_ami_name(&ami_id)
    } else {
        String::new()
    };
    let key_pair = extract_json_value(json, "KeyName").unwrap_or_else(|| "-".to_string());
    let vpc_id = extract_json_value(json, "VpcId").unwrap_or_default();
    let vpc = if !vpc_id.is_empty() {
        get_vpc_name(&vpc_id)
    } else {
        String::new()
    };

    let subnet_id = extract_json_value(json, "SubnetId").unwrap_or_default();
    let subnet = if !subnet_id.is_empty() {
        get_subnet_name(&subnet_id)
    } else {
        String::new()
    };
    let public_ip = extract_json_value(json, "PublicIpAddress").unwrap_or_else(|| "-".to_string());
    let private_ip = extract_json_value(json, "PrivateIpAddress").unwrap_or_default();
    let state = extract_state(json);

    let tags = extract_tags(json);
    let name = tags
        .iter()
        .find(|(k, _)| k == "Name")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| instance_id.to_string());

    let security_groups = extract_security_groups(json);
    let volumes = get_instance_volumes(instance_id);
    let user_data = get_instance_user_data(instance_id);

    Some(Ec2Detail {
        name,
        instance_id: instance_id.to_string(),
        instance_type,
        ami,
        key_pair,
        vpc,
        subnet,
        public_ip,
        private_ip,
        security_groups,
        state,
        tags,
        volumes,
        user_data,
    })
}

fn get_instance_volumes(instance_id: &str) -> Vec<VolumeDetail> {
    let output = run_aws_cli(&[
        "ec2",
        "describe-instances",
        "--instance-ids",
        instance_id,
        "--query",
        "Reservations[0].Instances[0].BlockDeviceMappings",
        "--output",
        "json",
    ]);

    let mut volumes = Vec::new();

    if let Some(json) = output {
        // Parse block device mappings
        let mut search_start = 0;
        while let Some(device_pos) = json[search_start..].find("\"DeviceName\": \"") {
            let device_start = search_start + device_pos + 15;
            if let Some(device_end) = json[device_start..].find('"') {
                let device_name = json[device_start..device_start + device_end].to_string();

                // Find volume ID
                if let Some(vol_pos) = json[device_start..].find("\"VolumeId\": \"") {
                    let vol_start = device_start + vol_pos + 13;
                    if let Some(vol_end) = json[vol_start..].find('"') {
                        let volume_id = json[vol_start..vol_start + vol_end].to_string();

                        // Find DeleteOnTermination
                        let delete_on_term = json
                            [device_start..device_start + 500.min(json.len() - device_start)]
                            .contains("\"DeleteOnTermination\": true");

                        // Get volume details
                        if let Some(vol_detail) =
                            get_volume_detail(&volume_id, &device_name, delete_on_term)
                        {
                            volumes.push(vol_detail);
                        }
                    }
                }
                search_start = device_start + device_end;
            } else {
                break;
            }
        }
    }
    volumes
}

fn get_volume_detail(
    volume_id: &str,
    device_name: &str,
    delete_on_termination: bool,
) -> Option<VolumeDetail> {
    let output = run_aws_cli(&[
        "ec2",
        "describe-volumes",
        "--volume-ids",
        volume_id,
        "--output",
        "json",
    ])?;

    let json = &output;
    let size_str = extract_json_value(json, "Size").unwrap_or_default();
    let size_gb = size_str.parse::<i64>().unwrap_or(0);
    let volume_type =
        extract_json_value(json, "VolumeType").unwrap_or_else(|| "unknown".to_string());
    let iops_str = extract_json_value(json, "Iops");
    let iops = iops_str.and_then(|s| s.parse::<i64>().ok());
    let encrypted = json.contains("\"Encrypted\": true");

    Some(VolumeDetail {
        device_name: device_name.to_string(),
        volume_id: volume_id.to_string(),
        size_gb,
        volume_type,
        iops,
        encrypted,
        delete_on_termination,
    })
}

fn get_instance_user_data(instance_id: &str) -> Option<String> {
    let output = run_aws_cli(&[
        "ec2",
        "describe-instance-attribute",
        "--instance-id",
        instance_id,
        "--attribute",
        "userData",
        "--output",
        "json",
    ])?;

    // Extract base64 encoded user data
    if let Some(value_pos) = output.find("\"Value\": \"") {
        let start = value_pos + 10;
        if let Some(end) = output[start..].find('"') {
            let base64_data = &output[start..start + end];
            if !base64_data.is_empty() {
                // Decode base64
                use std::process::Command;
                let decode_output = Command::new("bash")
                    .arg("-c")
                    .arg(format!("echo '{}' | base64 -d 2>/dev/null", base64_data))
                    .output()
                    .ok()?;

                if decode_output.status.success() {
                    let decoded = String::from_utf8_lossy(&decode_output.stdout).to_string();
                    if !decoded.trim().is_empty() {
                        return Some(decoded);
                    }
                }
            }
        }
    }
    None
}

fn extract_tags(json: &str) -> Vec<(String, String)> {
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

fn extract_security_groups(json: &str) -> Vec<String> {
    let mut sgs = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = json[search_start..].find("\"GroupName\": \"") {
        let start = search_start + pos + 14;
        if let Some(end) = json[start..].find('"') {
            let sg = json[start..start + end].to_string();
            if !sgs.contains(&sg) {
                sgs.push(sg);
            }
            search_start = start + end;
        } else {
            break;
        }
    }
    sgs
}

impl Ec2Detail {
    pub fn to_markdown(&self) -> String {
        let mut lines = vec![
            format!("## EC2 인스턴스 ({})\n", self.name),
            "| 항목 | 값 |".to_string(),
            "|:---|:---|".to_string(),
            format!("| 이름 | {} |", self.name),
            format!("| 상태 | {} |", self.state),
        ];

        for (key, value) in &self.tags {
            if key != "Name" {
                lines.push(format!("| 태그-{} | {} |", key, value));
            }
        }

        lines.push(format!("| AMI | {} |", self.ami));
        lines.push(format!("| 인스턴스 유형 | {} |", self.instance_type));
        lines.push(format!("| 키 페어 | {} |", self.key_pair));
        lines.push(format!("| VPC | {} |", self.vpc));
        lines.push(format!("| 서브넷 | {} |", self.subnet));
        lines.push(format!("| 프라이빗 IP | {} |", self.private_ip));
        lines.push(format!("| 퍼블릭 IP | {} |", self.public_ip));
        lines.push(format!(
            "| 보안 그룹 | {} |",
            self.security_groups.join(", ")
        ));

        // 스토리지 섹션
        if !self.volumes.is_empty() {
            lines.push(String::new());
            lines.push("### 스토리지\n".to_string());
            lines.push(
                "| 디바이스 | 볼륨 ID | 크기 | 유형 | IOPS | 암호화 | 종료 시 삭제 |".to_string(),
            );
            lines.push("|:---|:---|---:|:---|---:|:---:|:---:|".to_string());

            for vol in &self.volumes {
                let iops_str = vol
                    .iops
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "-".to_string());
                let encrypted_str = if vol.encrypted { "✓" } else { "-" };
                let delete_str = if vol.delete_on_termination {
                    "✓"
                } else {
                    "-"
                };
                lines.push(format!(
                    "| {} | {} | {} GB | {} | {} | {} | {} |",
                    vol.device_name,
                    vol.volume_id,
                    vol.size_gb,
                    vol.volume_type,
                    iops_str,
                    encrypted_str,
                    delete_str
                ));
            }
        }

        // 사용자 데이터 섹션
        if let Some(ref user_data) = self.user_data {
            lines.push(String::new());
            lines.push("### 사용자 데이터\n".to_string());
            lines.push("```bash".to_string());
            lines.push(user_data.clone());
            lines.push("```".to_string());
        }

        lines.join("\n") + "\n"
    }
}

fn get_vpc_name(vpc_id: &str) -> String {
    let output = run_aws_cli(&[
        "ec2",
        "describe-vpcs",
        "--vpc-ids",
        vpc_id,
        "--output",
        "json",
    ]);

    if let Some(json) = output {
        let name = parse_name_tag(&json);
        if !name.is_empty() {
            return name;
        }
    }
    vpc_id.to_string()
}

fn get_subnet_name(subnet_id: &str) -> String {
    let output = run_aws_cli(&[
        "ec2",
        "describe-subnets",
        "--subnet-ids",
        subnet_id,
        "--output",
        "json",
    ]);

    if let Some(json) = output {
        let name = parse_name_tag(&json);
        if !name.is_empty() {
            return name;
        }
    }
    subnet_id.to_string()
}

fn get_ami_name(ami_id: &str) -> String {
    let output = run_aws_cli(&[
        "ec2",
        "describe-images",
        "--image-ids",
        ami_id,
        "--output",
        "json",
    ]);

    if let Some(json) = output {
        let name = parse_name_tag(&json);
        if !name.is_empty() {
            return name;
        }
    }
    ami_id.to_string()
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct NetworkDetail {
    pub name: String,
    pub id: String,
    pub cidr: String,
    pub state: String,
    pub subnets: Vec<AwsResource>,
    pub igws: Vec<AwsResource>,
    pub nats: Vec<AwsResource>,
    pub route_tables: Vec<RouteTableDetail>,
    pub eips: Vec<EipDetail>,
    pub dns_support: bool,
    pub dns_hostnames: bool,
    pub tags: Vec<(String, String)>,
}

pub fn get_network_detail(vpc_id: &str) -> Option<NetworkDetail> {
    let output = run_aws_cli(&[
        "ec2",
        "describe-vpcs",
        "--vpc-ids",
        vpc_id,
        "--output",
        "json",
    ])?;

    let json = &output;
    let cidr = extract_json_value(json, "CidrBlock").unwrap_or_default();
    let state = extract_state(json);
    let tags = extract_tags(json);

    let name = tags
        .iter()
        .find(|(k, _)| k == "Name")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| vpc_id.to_string());

    let subnets = list_subnets(vpc_id);
    let igws = list_internet_gateways(vpc_id);
    let nats = list_nat_gateways(vpc_id);
    let route_tables = list_route_tables(vpc_id);
    let eips = list_eips();

    let dns_support = get_vpc_attribute(vpc_id, "enableDnsSupport");
    let dns_hostnames = get_vpc_attribute(vpc_id, "enableDnsHostnames");

    Some(NetworkDetail {
        name,
        id: vpc_id.to_string(),
        cidr,
        state,
        subnets,
        igws,
        nats,
        route_tables,
        eips,
        dns_support,
        dns_hostnames,
        tags,
    })
}

impl NetworkDetail {
    pub fn to_markdown(&self) -> String {
        let mut lines = vec![format!("## Network ({})\n", self.name)];

        lines.push("| 항목 | 값 |".to_string());
        lines.push("|:---|:---|".to_string());
        lines.push(format!("| 이름 | {} |", self.name));
        lines.push(format!("| CIDR | {} |", self.cidr));
        lines.push(format!("| 상태 | {} |", self.state));
        lines.push(format!("| DNS 지원 | {} |", self.dns_support));
        lines.push(format!("| DNS 호스트 이름 | {} |", self.dns_hostnames));

        for (key, value) in &self.tags {
            if key != "Name" {
                lines.push(format!("| 태그-{} | {} |", key, value));
            }
        }

        if !self.subnets.is_empty() {
            lines.push("\n### 서브넷".to_string());
            lines.push("| 이름 | CIDR | AZ | 상태 |".to_string());
            lines.push("|:---|:---|:---|:---|".to_string());
            for subnet in &self.subnets {
                lines.push(format!(
                    "| {} | {} | {} | {} |",
                    subnet.name, subnet.cidr, subnet.az, subnet.state
                ));
            }
        }

        if !self.igws.is_empty() {
            lines.push("\n### 인터넷 게이트웨이".to_string());
            lines.push("| 이름 |".to_string());
            lines.push("|:---|".to_string());
            for igw in &self.igws {
                lines.push(format!("| {} |", igw.name));
            }
        }

        if !self.nats.is_empty() {
            lines.push("\n### NAT 게이트웨이".to_string());
            lines.push("| 이름 | 위치 (서브넷) |".to_string());
            lines.push("|:---|:---|".to_string());
            for nat in &self.nats {
                let nat_subnet_id = &nat.az; // SubnetId가 az 필드에 저장됨
                let subnet_name = self
                    .subnets
                    .iter()
                    .find(|s| &s.id == nat_subnet_id)
                    .map(|s| s.name.as_str())
                    .unwrap_or("unknown");
                lines.push(format!("| {} | {} |", nat.name, subnet_name));
            }
        }

        if !self.route_tables.is_empty() {
            lines.push("\n### 라우팅 테이블".to_string());
            for rt in &self.route_tables {
                lines.push(format!("\n#### {}", rt.name));

                if !rt.routes.is_empty() {
                    lines.push("\n**라우트:**".to_string());
                    lines.push("| 라우트 |".to_string());
                    lines.push("|:---|".to_string());
                    for route in &rt.routes {
                        lines.push(format!("| {} |", route));
                    }
                }

                if !rt.associations.is_empty() {
                    lines.push("\n**연결된 서브넷:**".to_string());
                    lines.push("| 서브넷 |".to_string());
                    lines.push("|:---|".to_string());
                    for assoc in &rt.associations {
                        lines.push(format!("| {} |", assoc));
                    }
                }
            }
        }

        if !self.eips.is_empty() {
            lines.push("\n### Elastic IPs".to_string());
            lines.push("| 이름 | Public IP | 연결 |".to_string());
            lines.push("|:---|:---|:---|".to_string());
            for eip in &self.eips {
                let assoc = if !eip.instance_id.is_empty() {
                    format!("Instance: {}", eip.instance_id)
                } else if !eip.private_ip.is_empty() {
                    format!("Private IP: {}", eip.private_ip)
                } else {
                    "-".to_string()
                };
                lines.push(format!("| {} | {} | {} |", eip.name, eip.public_ip, assoc));
            }
        }

        // Mermaid 다이어그램 추가
        lines.push("\n### 네트워크 구성도".to_string());
        lines.push(self.generate_mermaid());

        lines.join("\n") + "\n"
    }

    fn generate_mermaid(&self) -> String {
        use std::collections::HashMap;

        // 이름을 Mermaid ID로 변환하는 헬퍼 함수
        let to_node_id = |name: &str| -> String {
            name.to_lowercase()
                .replace("-", "_")
                .replace(" ", "_")
                .replace("(", "")
                .replace(")", "")
        };

        let mut diagram = String::from("\n```mermaid\ngraph TD\n");

        // Internet
        diagram.push_str("    Internet((\"☁️ Internet\"))\n");
        diagram.push_str("    style Internet fill:#fff9c4,stroke:#f57f17\n\n");

        diagram.push_str(&format!(
            "    subgraph VPC[\"{} ({})\"]\n",
            self.name, self.cidr
        ));
        diagram.push_str("    style VPC fill:#e1f5fe,stroke:#01579b\n");

        // IGW
        for igw in &self.igws {
            let igw_node_id = to_node_id(&igw.name);
            diagram.push_str(&format!("    {}[\"🌐 {}\"]\n", igw_node_id, igw.name));
            diagram.push_str(&format!(
                "    style {} fill:#fff3e0,stroke:#e65100\n",
                igw_node_id
            ));
        }

        // NAT의 서브넷 정보 맵 생성
        let mut nat_by_subnet: HashMap<String, &AwsResource> = HashMap::new();
        for nat in &self.nats {
            let nat_subnet_id = &nat.az; // SubnetId가 az 필드에 저장됨
            nat_by_subnet.insert(nat_subnet_id.clone(), nat);
        }

        // ID->Name 매핑 생성 (라우팅 테이블 처리용)
        let mut id_to_name: HashMap<String, String> = HashMap::new();
        for igw in &self.igws {
            id_to_name.insert(igw.id.clone(), igw.name.clone());
        }
        for nat in &self.nats {
            id_to_name.insert(nat.id.clone(), nat.name.clone());
        }
        for subnet in &self.subnets {
            id_to_name.insert(subnet.id.clone(), subnet.name.clone());
        }

        // Subnets grouped by AZ
        let mut subnets_by_az: HashMap<String, Vec<&AwsResource>> = HashMap::new();
        for s in &self.subnets {
            let az = if s.az.is_empty() {
                "unknown".to_string()
            } else {
                s.az.clone()
            };
            subnets_by_az.entry(az).or_default().push(s);
        }

        for (az, subnets) in &subnets_by_az {
            let az_clean = az.replace("-", "_");
            diagram.push_str(&format!("\n        subgraph {}[\"📍 {}\"]\n", az_clean, az));
            diagram.push_str(&format!(
                "        style {} fill:#f3e5f5,stroke:#4a148c,stroke-dasharray: 5 5\n",
                az_clean
            ));

            for subnet in subnets {
                let subnet_node_id = to_node_id(&subnet.name);
                diagram.push_str(&format!(
                    "            {}[\"{}\\n{}\"]\n",
                    subnet_node_id, subnet.name, subnet.cidr
                ));
                diagram.push_str(&format!(
                    "            style {} fill:#e8f5e9,stroke:#1b5e20\n",
                    subnet_node_id
                ));

                // 이 서브넷에 NAT가 있으면 표시
                if let Some(nat) = nat_by_subnet.get(&subnet.id) {
                    let nat_node_id = to_node_id(&nat.name);
                    diagram.push_str(&format!(
                        "            {}[\"🔀 {}\"]\n",
                        nat_node_id, nat.name
                    ));
                    diagram.push_str(&format!(
                        "            style {} fill:#ffecb3,stroke:#ff6f00\n",
                        nat_node_id
                    ));
                }
            }
            diagram.push_str("        end\n");
        }

        diagram.push('\n');

        // Internet <-> IGW 연결
        if let Some(igw) = self.igws.first() {
            let igw_node_id = to_node_id(&igw.name);
            diagram.push_str(&format!("    Internet <--> {}\n", igw_node_id));
        }

        // 라우팅 테이블 기반 연결선
        for rt in &self.route_tables {
            // 0.0.0.0/0 라우트의 타겟 찾기 (IGW 또는 NAT)
            let mut target_id: Option<String> = None;
            for route in &rt.routes {
                if route.starts_with("0.0.0.0/0") {
                    // "0.0.0.0/0 -> igw-xxx" 또는 "0.0.0.0/0 -> nat-xxx"에서 타겟 추출
                    if let Some(arrow_pos) = route.find("->") {
                        let target = route[arrow_pos + 2..].trim();
                        if target.starts_with("igw-") || target.starts_with("nat-") {
                            target_id = Some(target.to_string());
                            break;
                        }
                    }
                }
            }

            // 이 라우팅 테이블에 연결된 서브넷들 찾기
            for assoc in &rt.associations {
                // "subnet-name (subnet-xxx)" 형식에서 subnet-xxx 추출
                if let Some(start) = assoc.find("(subnet-")
                    && let Some(end) = assoc[start + 1..].find(')')
                {
                    let subnet_id = &assoc[start + 1..start + 1 + end];

                    // 서브넷 이름 찾기
                    if let Some(subnet_name) = id_to_name.get(subnet_id) {
                        let subnet_node_id = to_node_id(subnet_name);

                        // 타겟 이름 찾기
                        if let Some(ref target_aws_id) = target_id
                            && let Some(target_name) = id_to_name.get(target_aws_id)
                        {
                            let target_node_id = to_node_id(target_name);

                            // NAT로 라우팅: Private Subnet
                            if target_aws_id.starts_with("nat-") {
                                diagram.push_str(&format!(
                                    "    {} -.->|Private| {}\n",
                                    subnet_node_id, target_node_id
                                ));
                            }
                            // IGW로 라우팅: Public Subnet
                            else if target_aws_id.starts_with("igw-") {
                                diagram.push_str(&format!(
                                    "    {} <-->|Public| {}\n",
                                    target_node_id, subnet_node_id
                                ));
                            }
                        }
                    }
                }
            }
        }

        // NAT -> IGW 연결
        for nat in &self.nats {
            if let Some(igw) = self.igws.first() {
                let nat_node_id = to_node_id(&nat.name);
                let igw_node_id = to_node_id(&igw.name);
                diagram.push_str(&format!("    {} ==> {}\n", nat_node_id, igw_node_id));
            }
        }

        diagram.push_str("    end\n```\n");
        diagram
    }
}

fn get_vpc_attribute(vpc_id: &str, attribute: &str) -> bool {
    let output = run_aws_cli(&[
        "ec2",
        "describe-vpc-attribute",
        "--vpc-id",
        vpc_id,
        "--attribute",
        attribute,
        "--output",
        "json",
    ]);

    if let Some(json) = output {
        let key = if attribute == "enableDnsSupport" {
            "EnableDnsSupport"
        } else {
            "EnableDnsHostnames"
        };
        if let Some(pos) = json.find(key) {
            let section = &json[pos..];
            if let Some(val_pos) = section.find("\"Value\": ") {
                let val_start = val_pos + 9;
                if section[val_start..].starts_with("true") {
                    return true;
                }
            }
        }
    }
    false
}

pub fn list_internet_gateways(vpc_id: &str) -> Vec<AwsResource> {
    let filter = format!("Name=attachment.vpc-id,Values={}", vpc_id);
    let output = match run_aws_cli(&[
        "ec2",
        "describe-internet-gateways",
        "--filters",
        &filter,
        "--query",
        "InternetGateways[*].[InternetGatewayId,Tags,Attachments]",
        "--output",
        "json",
    ]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    parse_internet_gateways(&output)
}

fn parse_internet_gateways(json: &str) -> Vec<AwsResource> {
    let mut resources = Vec::new();
    let mut search_start = 0;

    // Pattern: [ "igw-...", [Tags], [Attachments] ]
    while let Some(pos) = json[search_start..].find("igw-") {
        let start = search_start + pos;
        if let Some(end) = json[start..].find('"') {
            let id = &json[start..start + end];
            // Validate
            if !id.starts_with("igw-") {
                search_start = start + end;
                continue;
            }

            // Tags
            let tag_start_offset = json[start..].find('[').unwrap_or_default();
            let tag_start = start + tag_start_offset;
            let tag_end_offset = json[tag_start..].find(']').unwrap_or_default();
            let tag_end = tag_start + tag_end_offset + 1;
            let tags_json = &json[tag_start..tag_end];
            let name = parse_name_tag(tags_json);

            // Attachments
            let att_start_offset = json[tag_end..].find('[').unwrap_or_default();
            let att_start = tag_end + att_start_offset;
            let att_end_offset = json[att_start..].find(']').unwrap_or_default();
            let att_end = att_start + att_end_offset + 1;
            let att_json = &json[att_start..att_end];

            let state = if att_json.contains("\"State\": \"available\"")
                || att_json.contains("\"State\": \"attached\"")
                || att_json.contains("vpc-")
            {
                "attached".to_string()
            } else {
                "detached".to_string()
            };

            resources.push(AwsResource {
                name: format!(
                    "{} ({})",
                    if name.is_empty() {
                        id.to_string()
                    } else {
                        name
                    },
                    state
                ),
                id: id.to_string(),
                state,
                az: String::new(),
                cidr: String::new(), // IGWs do not have a CIDR
            });

            search_start = att_end;
        } else {
            break;
        }
    }
    resources
}

pub fn list_nat_gateways(vpc_id: &str) -> Vec<AwsResource> {
    let filter = format!("Name=vpc-id,Values={}", vpc_id);
    let output = match run_aws_cli(&[
        "ec2",
        "describe-nat-gateways",
        "--filter",
        &filter,
        "--output",
        "json",
    ]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    // serde_json으로 파싱
    let response: NatGatewayResponse = match serde_json::from_str(&output) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    response
        .nat_gateways
        .into_iter()
        .map(|nat| {
            let name = nat
                .tags
                .iter()
                .find(|t| t.key == "Name")
                .map(|t| t.value.clone())
                .unwrap_or_else(|| nat.nat_gateway_id.clone());

            AwsResource {
                name,
                id: nat.nat_gateway_id,
                state: nat.state,
                az: nat.subnet_id, // SubnetId를 az 필드에 저장
                cidr: String::new(),
            }
        })
        .collect()
}

pub fn list_route_tables(vpc_id: &str) -> Vec<RouteTableDetail> {
    let filter = format!("Name=vpc-id,Values={}", vpc_id);
    let output = match run_aws_cli(&[
        "ec2",
        "describe-route-tables",
        "--filters",
        &filter,
        "--query",
        "RouteTables[*].[RouteTableId,Tags,Routes,Associations]",
        "--output",
        "json",
    ]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    parse_route_tables(&output)
}

pub fn list_eips() -> Vec<EipDetail> {
    let output = match run_aws_cli(&[
        "ec2",
        "describe-addresses",
        "--query",
        "Addresses[*].{AllocationId:AllocationId, PublicIp:PublicIp, AssociationId:AssociationId, InstanceId:InstanceId, PrivateIpAddress:PrivateIpAddress, Tags:Tags}",
        "--output",
        "json",
    ]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let json = &output;
    let mut details = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = json[search_start..].find("AllocationId") {
        let block_start = search_start + pos;
        let obj_start = json[..block_start].rfind('{').unwrap_or(0);

        // Find matching closing brace
        // Simple brace counting
        let mut depth = 0;
        let mut obj_end = json.len();
        for (i, c) in json[obj_start..].char_indices() {
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    obj_end = obj_start + i + 1;
                    break;
                }
            }
        }

        let section = &json[obj_start..obj_end];

        // Extract fields
        let public_ip = extract_json_value(section, "PublicIp").unwrap_or_default();
        let instance_id = extract_json_value(section, "InstanceId").unwrap_or_default();
        let private_ip = extract_json_value(section, "PrivateIpAddress").unwrap_or_default();

        // Tags
        let name = parse_name_tag(section);
        let display_name = if name.is_empty() {
            public_ip.clone()
        } else {
            name
        };

        details.push(EipDetail {
            name: display_name,
            public_ip,
            instance_id,
            private_ip,
        });

        search_start = obj_end;
    }

    details
}

#[derive(Debug, Clone)]
pub struct RouteTableDetail {
    pub name: String,
    #[allow(dead_code)]
    pub id: String,
    pub routes: Vec<String>,
    pub associations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EipDetail {
    pub name: String,
    pub public_ip: String,
    pub instance_id: String,
    pub private_ip: String,
}

#[derive(Debug)]
pub struct SecurityGroupDetail {
    pub name: String,
    pub id: String,
    pub description: String,
    pub vpc_id: String,
    pub inbound_rules: Vec<SecurityRule>,
    pub outbound_rules: Vec<SecurityRule>,
}

#[derive(Debug)]
pub struct SecurityRule {
    pub protocol: String,
    pub port_range: String,
    pub source_dest: String,
    pub description: String,
}

#[derive(Debug)]
pub struct LoadBalancerDetail {
    pub name: String,
    pub arn: String,
    pub dns_name: String,
    pub lb_type: String,
    pub scheme: String,
    pub vpc_id: String,
    pub availability_zones: Vec<String>,
    pub security_groups: Vec<String>,
    pub listeners: Vec<ListenerInfo>,
    pub target_groups: Vec<TargetGroupInfo>,
}

#[derive(Debug)]
pub struct ListenerInfo {
    pub port: i32,
    pub protocol: String,
    pub default_action: String,
}

#[derive(Debug)]
pub struct TargetGroupInfo {
    pub name: String,
    #[allow(dead_code)]
    pub arn: String,
    pub protocol: String,
    pub port: i32,
    pub target_type: String,
    pub health_check_protocol: String,
    pub health_check_path: String,
    pub healthy_threshold: i32,
    pub unhealthy_threshold: i32,
    pub targets: Vec<TargetInfo>,
}

#[derive(Debug)]
pub struct TargetInfo {
    pub id: String,
    pub port: i32,
    pub health_status: String,
}

impl LoadBalancerDetail {
    pub fn to_markdown(&self) -> String {
        let mut lines = vec![
            format!("## Load Balancer ({})\n", self.name),
            "| 항목 | 값 |".to_string(),
            "|:---|:---|".to_string(),
            format!("| 이름 | {} |", self.name),
            format!("| DNS 이름 | {} |", self.dns_name),
            format!("| 타입 | {} |", self.lb_type),
            format!("| Scheme | {} |", self.scheme),
            format!("| VPC ID | {} |", self.vpc_id),
        ];

        if !self.availability_zones.is_empty() {
            lines.push(format!(
                "| Availability Zones | {} |",
                self.availability_zones.join(", ")
            ));
        }

        if !self.security_groups.is_empty() {
            lines.push(format!(
                "| Security Groups | {} |",
                self.security_groups.join(", ")
            ));
        }

        if !self.listeners.is_empty() {
            lines.push("\n### Listeners".to_string());
            lines.push("| 포트 | 프로토콜 | 기본 액션 |".to_string());
            lines.push("|:---|:---|:---|".to_string());
            for listener in &self.listeners {
                lines.push(format!(
                    "| {} | {} | {} |",
                    listener.port, listener.protocol, listener.default_action
                ));
            }
        }

        if !self.target_groups.is_empty() {
            lines.push("\n### Target Groups".to_string());
            for tg in &self.target_groups {
                lines.push(format!("\n#### {}", tg.name));
                lines.push("\n**기본 정보:**".to_string());
                lines.push("| 항목 | 값 |".to_string());
                lines.push("|:---|:---|".to_string());
                lines.push(format!("| 프로토콜 | {} |", tg.protocol));
                lines.push(format!("| 포트 | {} |", tg.port));
                lines.push(format!("| Target Type | {} |", tg.target_type));
                lines.push(format!(
                    "| Health Check | {} {} |",
                    tg.health_check_protocol, tg.health_check_path
                ));
                lines.push(format!(
                    "| Threshold | Healthy: {}, Unhealthy: {} |",
                    tg.healthy_threshold, tg.unhealthy_threshold
                ));

                if !tg.targets.is_empty() {
                    lines.push("\n**Targets:**".to_string());
                    lines.push("| Target ID | 포트 | 상태 |".to_string());
                    lines.push("|:---|:---|:---|".to_string());
                    for target in &tg.targets {
                        lines.push(format!(
                            "| {} | {} | {} |",
                            target.id, target.port, target.health_status
                        ));
                    }
                }
            }
        }

        lines.join("\n") + "\n"
    }
}

impl SecurityGroupDetail {
    pub fn to_markdown(&self) -> String {
        let mut lines = vec![
            format!("## Security Group ({})\n", self.name),
            "| 항목 | 값 |".to_string(),
            "|:---|:---|".to_string(),
            format!("| 이름 | {} |", self.name),
            format!("| Group ID | {} |", self.id),
            format!("| 설명 | {} |", self.description),
            format!("| VPC ID | {} |", self.vpc_id),
        ];

        if !self.inbound_rules.is_empty() {
            lines.push("\n### 인바운드 규칙".to_string());
            lines.push("| 프로토콜 | 포트 범위 | 소스 | 설명 |".to_string());
            lines.push("|:---|:---|:---|:---|".to_string());
            for rule in &self.inbound_rules {
                lines.push(format!(
                    "| {} | {} | {} | {} |",
                    rule.protocol, rule.port_range, rule.source_dest, rule.description
                ));
            }
        }

        if !self.outbound_rules.is_empty() {
            lines.push("\n### 아웃바운드 규칙".to_string());
            lines.push("| 프로토콜 | 포트 범위 | 대상 | 설명 |".to_string());
            lines.push("|:---|:---|:---|:---|".to_string());
            for rule in &self.outbound_rules {
                lines.push(format!(
                    "| {} | {} | {} | {} |",
                    rule.protocol, rule.port_range, rule.source_dest, rule.description
                ));
            }
        }

        lines.join("\n") + "\n"
    }
}

/// VPC 기본 정보만 조회 (name, cidr, state, tags)
#[allow(clippy::type_complexity)]
pub fn get_vpc_info(vpc_id: &str) -> Option<(String, String, String, Vec<(String, String)>)> {
    let output = run_aws_cli(&[
        "ec2",
        "describe-vpcs",
        "--vpc-ids",
        vpc_id,
        "--output",
        "json",
    ])?;

    let json = &output;
    let cidr = extract_json_value(json, "CidrBlock").unwrap_or_default();
    let state = extract_state(json);
    let tags = extract_tags(json);

    let name = tags
        .iter()
        .find(|(k, _)| k == "Name")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| vpc_id.to_string());

    Some((name, cidr, state, tags))
}

pub fn get_vpc_dns_support(vpc_id: &str) -> bool {
    get_vpc_attribute(vpc_id, "enableDnsSupport")
}

pub fn get_vpc_dns_hostnames(vpc_id: &str) -> bool {
    get_vpc_attribute(vpc_id, "enableDnsHostnames")
}

pub fn list_security_groups() -> Vec<AwsResource> {
    let output = match run_aws_cli(&["ec2", "describe-security-groups", "--output", "json"]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let response: SecurityGroupsResponse = match serde_json::from_str(&output) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    response
        .security_groups
        .into_iter()
        .map(|sg| {
            let name = sg
                .tags
                .iter()
                .find(|t| t.key == "Name")
                .map(|t| t.value.clone())
                .unwrap_or_else(|| sg.group_name.clone());

            AwsResource {
                name: format!("{} ({})", name, sg.group_name),
                id: sg.group_id,
                state: sg.vpc_id, // vpc_id를 state 필드에 저장
                az: String::new(),
                cidr: String::new(),
            }
        })
        .collect()
}

pub fn get_security_group_detail(sg_id: &str) -> Option<SecurityGroupDetail> {
    let output = run_aws_cli(&[
        "ec2",
        "describe-security-groups",
        "--group-ids",
        sg_id,
        "--output",
        "json",
    ])?;

    let response: SecurityGroupsResponse = serde_json::from_str(&output).ok()?;
    let sg = response.security_groups.first()?;

    let name = sg
        .tags
        .iter()
        .find(|t| t.key == "Name")
        .map(|t| t.value.clone())
        .unwrap_or_else(|| sg.group_name.clone());

    let inbound_rules = parse_security_rules(&sg.ip_permissions);
    let outbound_rules = parse_security_rules(&sg.ip_permissions_egress);

    Some(SecurityGroupDetail {
        name,
        id: sg.group_id.clone(),
        description: sg.description.clone(),
        vpc_id: sg.vpc_id.clone(),
        inbound_rules,
        outbound_rules,
    })
}

fn parse_security_rules(permissions: &[IpPermission]) -> Vec<SecurityRule> {
    let mut rules = Vec::new();

    for perm in permissions {
        let protocol = match perm.ip_protocol.as_str() {
            "-1" => "All".to_string(),
            "tcp" => "TCP".to_string(),
            "udp" => "UDP".to_string(),
            "icmp" => "ICMP".to_string(),
            other => other.to_uppercase(),
        };

        let port_range = if perm.ip_protocol == "-1" {
            "All".to_string()
        } else if let (Some(from), Some(to)) = (perm.from_port, perm.to_port) {
            if from == to {
                from.to_string()
            } else {
                format!("{}-{}", from, to)
            }
        } else {
            "All".to_string()
        };

        // IP Ranges
        for ip_range in &perm.ip_ranges {
            rules.push(SecurityRule {
                protocol: protocol.clone(),
                port_range: port_range.clone(),
                source_dest: ip_range.cidr_ip.clone(),
                description: ip_range
                    .description
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            });
        }

        // IPv6 Ranges
        for ipv6_range in &perm.ipv6_ranges {
            rules.push(SecurityRule {
                protocol: protocol.clone(),
                port_range: port_range.clone(),
                source_dest: ipv6_range.cidr_ipv6.clone(),
                description: ipv6_range
                    .description
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            });
        }

        // Security Group References
        for sg_pair in &perm.user_id_group_pairs {
            rules.push(SecurityRule {
                protocol: protocol.clone(),
                port_range: port_range.clone(),
                source_dest: format!("sg: {}", sg_pair.group_id),
                description: sg_pair
                    .description
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            });
        }
    }

    rules
}

pub fn list_load_balancers() -> Vec<AwsResource> {
    let output = match run_aws_cli(&["elbv2", "describe-load-balancers", "--output", "json"]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let response: LoadBalancersResponse = match serde_json::from_str(&output) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    response
        .load_balancers
        .into_iter()
        .map(|lb| {
            let scheme_text = match lb.scheme.as_str() {
                "internet-facing" => "인터넷 연결",
                "internal" => "내부",
                _ => &lb.scheme,
            };

            AwsResource {
                name: format!("{} ({})", lb.load_balancer_name, scheme_text),
                id: lb.load_balancer_arn,
                state: lb.lb_type,
                az: lb.vpc_id,
                cidr: lb.dns_name,
            }
        })
        .collect()
}

pub fn get_load_balancer_detail(lb_arn: &str) -> Option<LoadBalancerDetail> {
    // Get load balancer info
    let output = run_aws_cli(&[
        "elbv2",
        "describe-load-balancers",
        "--load-balancer-arns",
        lb_arn,
        "--output",
        "json",
    ])?;

    let response: LoadBalancersResponse = serde_json::from_str(&output).ok()?;
    let lb = response.load_balancers.first()?;

    // Get listeners
    let listeners_output = run_aws_cli(&[
        "elbv2",
        "describe-listeners",
        "--load-balancer-arn",
        lb_arn,
        "--output",
        "json",
    ])?;

    let listeners_response: ListenersResponse = serde_json::from_str(&listeners_output).ok()?;
    let listeners: Vec<ListenerInfo> = listeners_response
        .listeners
        .iter()
        .map(|l| {
            let action = l
                .default_actions
                .first()
                .map(|a| a.action_type.clone())
                .unwrap_or_else(|| "-".to_string());

            ListenerInfo {
                port: l.port,
                protocol: l.protocol.clone(),
                default_action: action,
            }
        })
        .collect();

    // Get target groups from listeners
    let mut target_group_arns: Vec<String> = listeners_response
        .listeners
        .iter()
        .flat_map(|l| &l.default_actions)
        .filter_map(|a| a.target_group_arn.clone())
        .collect();

    target_group_arns.dedup();

    let mut target_groups = Vec::new();

    for tg_arn in &target_group_arns {
        if let Some(tg_info) = get_target_group_info(tg_arn) {
            target_groups.push(tg_info);
        }
    }

    let availability_zones: Vec<String> = lb
        .availability_zones
        .iter()
        .map(|az| az.zone_name.clone())
        .collect();

    Some(LoadBalancerDetail {
        name: lb.load_balancer_name.clone(),
        arn: lb.load_balancer_arn.clone(),
        dns_name: lb.dns_name.clone(),
        lb_type: lb.lb_type.clone(),
        scheme: lb.scheme.clone(),
        vpc_id: lb.vpc_id.clone(),
        availability_zones,
        security_groups: lb.security_groups.clone(),
        listeners,
        target_groups,
    })
}

fn get_target_group_info(tg_arn: &str) -> Option<TargetGroupInfo> {
    // Get target group details
    let output = run_aws_cli(&[
        "elbv2",
        "describe-target-groups",
        "--target-group-arns",
        tg_arn,
        "--output",
        "json",
    ])?;

    let response: TargetGroupsResponse = serde_json::from_str(&output).ok()?;
    let tg = response.target_groups.first()?;

    // Get target health
    let health_output = run_aws_cli(&[
        "elbv2",
        "describe-target-health",
        "--target-group-arn",
        tg_arn,
        "--output",
        "json",
    ])?;

    let health_response: TargetHealthResponse = serde_json::from_str(&health_output).ok()?;
    let targets: Vec<TargetInfo> = health_response
        .target_health_descriptions
        .iter()
        .map(|thd| TargetInfo {
            id: thd.target.id.clone(),
            port: thd.target.port,
            health_status: thd.target_health.state.clone(),
        })
        .collect();

    Some(TargetGroupInfo {
        name: tg.target_group_name.clone(),
        arn: tg.target_group_arn.clone(),
        protocol: tg.protocol.clone(),
        port: tg.port,
        target_type: tg.target_type.clone(),
        health_check_protocol: tg.health_check_protocol.clone(),
        health_check_path: tg.health_check_path.clone(),
        healthy_threshold: tg.healthy_threshold_count,
        unhealthy_threshold: tg.unhealthy_threshold_count,
        targets,
    })
}

// ECR functions
#[derive(Debug)]
pub struct EcrDetail {
    pub name: String,
    pub uri: String,
    pub tag_mutability: String,
    pub encryption_type: String,
    pub kms_key: Option<String>,
    pub created_at: String,
    pub image_count: i32,
}

impl EcrDetail {
    pub fn to_markdown(&self) -> String {
        let encryption_display = if self.encryption_type == "KMS" {
            if let Some(ref key) = self.kms_key {
                format!("AWS KMS ({})", key)
            } else {
                "AWS KMS".to_string()
            }
        } else {
            "AES-256".to_string()
        };

        let lines = vec![
            format!("## ECR 레포지토리 ({})\n", self.name),
            "| 항목 | 값 |".to_string(),
            "|:---|:---|".to_string(),
            format!("| 이름 | {} |", self.name),
            format!("| URI | {} |", self.uri),
            format!("| 태그 변경 가능 | {} |", self.tag_mutability),
            format!("| 암호화 | {} |", encryption_display),
            format!("| 이미지 수 | {} |", self.image_count),
            format!("| 생성일 | {} |", self.created_at),
        ];

        lines.join("\n") + "\n"
    }
}

pub fn list_ecr_repositories() -> Vec<AwsResource> {
    let output = match run_aws_cli(&["ecr", "describe-repositories", "--output", "json"]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let response: EcrRepositoriesResponse = match serde_json::from_str(&output) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    response
        .repositories
        .into_iter()
        .map(|repo| {
            let mutability = if repo.image_tag_mutability == "IMMUTABLE" {
                "Immutable"
            } else {
                "Mutable"
            };

            AwsResource {
                name: format!("{} ({})", repo.repository_name, mutability),
                id: repo.repository_name,
                state: repo.image_tag_mutability,
                az: String::new(),
                cidr: repo.repository_uri,
            }
        })
        .collect()
}

pub fn get_ecr_detail(repo_name: &str) -> Option<EcrDetail> {
    // Get repository info
    let output = run_aws_cli(&[
        "ecr",
        "describe-repositories",
        "--repository-names",
        repo_name,
        "--output",
        "json",
    ])?;

    let response: EcrRepositoriesResponse = serde_json::from_str(&output).ok()?;
    let repo = response.repositories.first()?;

    // Get image count
    let images_output = run_aws_cli(&[
        "ecr",
        "describe-images",
        "--repository-name",
        repo_name,
        "--output",
        "json",
    ]);

    let image_count = images_output
        .and_then(|o| serde_json::from_str::<EcrImagesResponse>(&o).ok())
        .map(|r| r.image_details.len() as i32)
        .unwrap_or(0);

    // Parse created_at timestamp
    let created_at = repo
        .created_at
        .map(|ts| {
            let secs = ts as i64;
            chrono::DateTime::from_timestamp(secs, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "-".to_string())
        })
        .unwrap_or_else(|| "-".to_string());

    let (encryption_type, kms_key) = repo
        .encryption_configuration
        .as_ref()
        .map(|enc| (enc.encryption_type.clone(), enc.kms_key.clone()))
        .unwrap_or_else(|| ("AES256".to_string(), None));

    Some(EcrDetail {
        name: repo.repository_name.clone(),
        uri: repo.repository_uri.clone(),
        tag_mutability: repo.image_tag_mutability.clone(),
        encryption_type,
        kms_key,
        created_at,
        image_count,
    })
}
