use crate::aws_cli::common::{AwsResource, extract_json_value, extract_tags, parse_name_tag};
use crate::aws_cli::iam::IamRoleDetail;
use crate::i18n::{I18n, Language};
use base64::{Engine as _, engine::general_purpose};
use serde::Deserialize;

#[cfg(not(test))]
mod cli_adapter {
    pub(super) fn run(args: &[&str]) -> Option<String> {
        crate::aws_cli::common::run_aws_cli(args)
    }
}

#[cfg(test)]
mod cli_adapter {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static RESPONSES: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();

    fn key(args: &[&str]) -> String {
        args.join("\x1f")
    }

    pub(super) fn run(args: &[&str]) -> Option<String> {
        RESPONSES
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .ok()
            .and_then(|map| map.get(&key(args)).cloned().flatten())
    }

    pub(super) fn set(args: &[&str], output: Option<&str>) {
        if let Ok(mut map) = RESPONSES.get_or_init(|| Mutex::new(HashMap::new())).lock() {
            map.insert(key(args), output.map(str::to_string));
        }
    }

    pub(super) fn clear() {
        if let Ok(mut map) = RESPONSES.get_or_init(|| Mutex::new(HashMap::new())).lock() {
            map.clear();
        }
    }
}

#[cfg(not(test))]
mod iam_adapter {
    use crate::aws_cli::iam::{IamRoleDetail, get_iam_role_detail};

    pub(super) fn get(role_name: &str) -> Option<IamRoleDetail> {
        get_iam_role_detail(role_name)
    }
}

#[cfg(test)]
mod iam_adapter {
    use crate::aws_cli::iam::IamRoleDetail;
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static RESPONSES: OnceLock<Mutex<HashMap<String, Option<IamRoleDetail>>>> = OnceLock::new();

    pub(super) fn get(role_name: &str) -> Option<IamRoleDetail> {
        RESPONSES
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .ok()
            .and_then(|map| map.get(role_name).cloned().flatten())
    }

    pub(super) fn set(role_name: &str, detail: Option<IamRoleDetail>) {
        if let Ok(mut map) = RESPONSES.get_or_init(|| Mutex::new(HashMap::new())).lock() {
            map.insert(role_name.to_string(), detail);
        }
    }

    pub(super) fn clear() {
        if let Ok(mut map) = RESPONSES.get_or_init(|| Mutex::new(HashMap::new())).lock() {
            map.clear();
        }
    }
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
    pub platform: String,
    pub architecture: String,
    pub key_pair: String,
    pub vpc: String,
    pub subnet: String,
    pub az: String,
    pub public_ip: String,
    pub private_ip: String,
    pub security_groups: Vec<String>,
    pub state: String,
    pub ebs_optimized: bool,
    pub monitoring: String,
    pub iam_role: Option<String>,
    pub iam_role_detail: Option<IamRoleDetail>,
    pub launch_time: String,
    pub tags: Vec<(String, String)>,
    pub volumes: Vec<VolumeDetail>,
    pub user_data: Option<String>,
}

impl Ec2Detail {
    pub fn to_markdown(&self, lang: Language) -> String {
        let i18n = I18n::new(lang);
        let display_name = if self.name.is_empty() || self.name == self.instance_id {
            format!("NULL - {}", self.instance_id)
        } else {
            format!("{} - {}", self.name, self.instance_id)
        };
        let mut lines = vec![
            format!("## {} ({})\n", i18n.md_ec2_instance(), display_name),
            format!("| {} | {} |", i18n.item(), i18n.value()),
            "|:---|:---|".to_string(),
            format!("| {} | {} |", i18n.md_name(), display_name),
            format!("| {} | {} |", i18n.md_state(), self.state),
        ];

        for (key, value) in &self.tags {
            if key != "Name" {
                lines.push(format!("| {}-{} | {} |", i18n.tag(), key, value));
            }
        }

        lines.push(format!("| AMI | {} |", self.ami));
        lines.push(format!(
            "| {} | {} |",
            i18n.md_instance_type(),
            self.instance_type
        ));
        lines.push(format!("| {} | {} |", i18n.md_platform(), self.platform));
        lines.push(format!(
            "| {} | {} |",
            i18n.md_architecture(),
            self.architecture
        ));
        lines.push(format!("| {} | {} |", i18n.md_key_pair(), self.key_pair));
        lines.push(format!("| VPC | {} |", self.vpc));
        lines.push(format!("| {} | {} |", i18n.md_subnet(), self.subnet));
        lines.push(format!("| {} | {} |", i18n.md_availability_zone(), self.az));
        lines.push(format!(
            "| {} | {} |",
            i18n.md_private_ip(),
            self.private_ip
        ));

        if self.public_ip != "-" && !self.public_ip.is_empty() {
            lines.push(format!("| {} | {} |", i18n.md_public_ip(), self.public_ip));
        }

        lines.push(format!(
            "| {} | {} |",
            i18n.md_security_groups(),
            self.security_groups.join(", ")
        ));

        let ebs_str = if self.ebs_optimized {
            i18n.md_enabled()
        } else {
            i18n.md_disabled()
        };
        lines.push(format!("| {} | {} |", i18n.md_ebs_optimized(), ebs_str));
        let monitoring_str = if self.monitoring == "Enabled" {
            i18n.md_enabled()
        } else {
            i18n.md_disabled()
        };
        lines.push(format!("| {} | {} |", i18n.md_monitoring(), monitoring_str));

        if let Some(ref role) = self.iam_role {
            lines.push(format!("| {} | {} |", i18n.md_iam_role(), role));
        }

        if !self.launch_time.is_empty() {
            lines.push(format!(
                "| {} | {} |",
                i18n.md_launch_time(),
                self.launch_time
            ));
        }

        // IAM Role Detail section
        if let Some(ref iam_detail) = self.iam_role_detail {
            lines.push(String::new());
            lines.push(format!(
                "### {} ({})\n",
                i18n.md_iam_role_detail(),
                iam_detail.name
            ));

            // Attached Policies
            if !iam_detail.attached_policies.is_empty() {
                lines.push(format!("#### {}\n", i18n.md_attached_policies()));
                lines.push(format!("| {} | ARN |", i18n.md_policy_name()));
                lines.push("|:---|:---|".to_string());
                for policy in &iam_detail.attached_policies {
                    lines.push(format!("| {} | {} |", policy.name, policy.arn));
                }
                lines.push(String::new());
            }

            // Inline Policies
            if !iam_detail.inline_policies.is_empty() {
                lines.push(format!("#### {}\n", i18n.md_inline_policies()));
                for policy in &iam_detail.inline_policies {
                    lines.push(format!("**{}**\n", policy.name));
                    lines.push("```json".to_string());
                    lines.push(policy.document.clone());
                    lines.push("```".to_string());
                    lines.push(String::new());
                }
            }

            // Trust Policy
            if !iam_detail.assume_role_policy.is_empty() {
                lines.push(format!("#### {}\n", i18n.md_trust_policy()));
                lines.push("```json".to_string());
                lines.push(iam_detail.assume_role_policy.clone());
                lines.push("```".to_string());
            }
        }

        // Storage section
        if !self.volumes.is_empty() {
            lines.push(String::new());
            lines.push(format!("### {}\n", i18n.md_storage()));
            lines.push(format!(
                "| {} | Volume ID | {} | {} | IOPS | {} | {} |",
                i18n.md_device(),
                i18n.md_size(),
                i18n.md_type(),
                i18n.md_encrypted(),
                i18n.md_delete_on_termination()
            ));
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

        // User Data section
        if let Some(ref user_data) = self.user_data {
            lines.push(String::new());
            lines.push(format!("### {}\n", i18n.md_user_data()));
            lines.push("```bash".to_string());
            lines.push(user_data.clone());
            lines.push("```".to_string());
        }

        lines.join("\n") + "\n"
    }
}

pub fn list_instances() -> Vec<AwsResource> {
    let output = match cli_adapter::run(&[
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

fn parse_instance_resources(json: &str) -> Vec<AwsResource> {
    let mut resources = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = json[search_start..].find("i-") {
        let start = search_start + pos;
        if let Some(end) = json[start..].find('"') {
            let id = &json[start..start + end];
            // Validate it's an instance ID
            if id.starts_with("i-")
                && id.len() > 3
                && id.chars().skip(2).all(|c| c.is_alphanumeric())
            {
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

pub fn get_instance_detail(instance_id: &str) -> Option<Ec2Detail> {
    let output = cli_adapter::run(&[
        "ec2",
        "describe-instances",
        "--instance-ids",
        instance_id,
        "--output",
        "json",
    ])?;

    let ami_id = extract_json_value(&output, "ImageId").unwrap_or_default();
    let ami = if !ami_id.is_empty() {
        get_ami_name(&ami_id)
    } else {
        String::new()
    };

    let vpc_id = extract_json_value(&output, "VpcId").unwrap_or_default();
    let vpc = if !vpc_id.is_empty() {
        get_vpc_name(&vpc_id)
    } else {
        String::new()
    };

    let subnet_id = extract_json_value(&output, "SubnetId").unwrap_or_default();
    let subnet = if !subnet_id.is_empty() {
        get_subnet_name(&subnet_id)
    } else {
        String::new()
    };
    let volumes = get_instance_volumes(instance_id);
    let user_data = get_instance_user_data(instance_id);

    Some(parse_instance_detail_output(
        instance_id,
        &output,
        ami,
        vpc,
        subnet,
        volumes,
        user_data,
    ))
}

fn parse_instance_detail_output(
    instance_id: &str,
    json: &str,
    ami: String,
    vpc: String,
    subnet: String,
    volumes: Vec<VolumeDetail>,
    user_data: Option<String>,
) -> Ec2Detail {
    let instance_type = extract_json_value(json, "InstanceType").unwrap_or_default();
    let platform = extract_json_value(json, "Platform").unwrap_or_else(|| "Linux".to_string());
    let architecture =
        extract_json_value(json, "Architecture").unwrap_or_else(|| "x86_64".to_string());
    let key_pair = extract_json_value(json, "KeyName").unwrap_or_else(|| "-".to_string());
    let az = extract_json_value(json, "AvailabilityZone").unwrap_or_default();
    let public_ip = extract_json_value(json, "PublicIpAddress").unwrap_or_else(|| "-".to_string());
    let private_ip = extract_json_value(json, "PrivateIpAddress").unwrap_or_default();
    let state = extract_state(json);
    let ebs_optimized = json.contains("\"EbsOptimized\": true");
    let monitoring = if json.contains("\"State\": \"enabled\"") {
        "Enabled".to_string()
    } else {
        "Disabled".to_string()
    };
    let iam_role = extract_json_value(json, "Arn")
        .and_then(|arn| arn.split('/').next_back().map(|s| s.to_string()));
    let iam_role_detail = iam_role
        .as_ref()
        .and_then(|role_name| iam_adapter::get(role_name));
    let launch_time = extract_json_value(json, "LaunchTime").unwrap_or_default();
    let tags = extract_tags(json);
    let name = tags
        .iter()
        .find(|(k, _)| k == "Name")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let security_groups = extract_security_groups(json);

    Ec2Detail {
        name,
        instance_id: instance_id.to_string(),
        instance_type,
        ami,
        platform,
        architecture,
        key_pair,
        vpc,
        subnet,
        az,
        public_ip,
        private_ip,
        security_groups,
        state,
        ebs_optimized,
        monitoring,
        iam_role,
        iam_role_detail,
        launch_time,
        tags,
        volumes,
        user_data,
    }
}

fn get_instance_volumes(instance_id: &str) -> Vec<VolumeDetail> {
    let output = cli_adapter::run(&[
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
        for (device_name, volume_id, delete_on_term) in parse_instance_volume_mappings(&json) {
            if let Some(vol_detail) = get_volume_detail(&volume_id, &device_name, delete_on_term) {
                volumes.push(vol_detail);
            }
        }
    }
    volumes
}

fn parse_instance_volume_mappings(json: &str) -> Vec<(String, String, bool)> {
    let mut mappings = Vec::new();
    let mut search_start = 0;

    while let Some(device_pos) = json[search_start..].find("\"DeviceName\": \"") {
        let device_marker_start = search_start + device_pos;
        let device_start = device_marker_start + 15;
        if let Some(device_end) = json[device_start..].find('"') {
            let device_name = json[device_start..device_start + device_end].to_string();
            let next_device_start = json[device_start..]
                .find("\"DeviceName\": \"")
                .map(|offset| device_start + offset)
                .unwrap_or(json.len());

            if let Some(vol_pos) = json[device_start..next_device_start].find("\"VolumeId\": \"") {
                let vol_start = device_start + vol_pos + 13;
                if let Some(vol_end) = json[vol_start..].find('"') {
                    let volume_id = json[vol_start..vol_start + vol_end].to_string();
                    let delete_on_term = json[device_start..next_device_start]
                        .contains("\"DeleteOnTermination\": true");
                    mappings.push((device_name, volume_id, delete_on_term));
                }
            }

            search_start = next_device_start;
        } else {
            break;
        }
    }

    mappings
}

fn get_volume_detail(
    volume_id: &str,
    device_name: &str,
    delete_on_termination: bool,
) -> Option<VolumeDetail> {
    let output = cli_adapter::run(&[
        "ec2",
        "describe-volumes",
        "--volume-ids",
        volume_id,
        "--output",
        "json",
    ])?;

    Some(parse_volume_detail_output(
        &output,
        volume_id,
        device_name,
        delete_on_termination,
    ))
}

fn parse_volume_detail_output(
    json: &str,
    volume_id: &str,
    device_name: &str,
    delete_on_termination: bool,
) -> VolumeDetail {
    let size_str = extract_json_value(json, "Size").unwrap_or_default();
    let size_gb = size_str.parse::<i64>().unwrap_or(0);
    let volume_type =
        extract_json_value(json, "VolumeType").unwrap_or_else(|| "unknown".to_string());
    let iops_str = extract_json_value(json, "Iops");
    let iops = iops_str.and_then(|s| s.parse::<i64>().ok());
    let encrypted = json.contains("\"Encrypted\": true");

    VolumeDetail {
        device_name: device_name.to_string(),
        volume_id: volume_id.to_string(),
        size_gb,
        volume_type,
        iops,
        encrypted,
        delete_on_termination,
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UserDataResponse {
    user_data: Option<UserDataValue>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UserDataValue {
    value: Option<String>,
}

fn get_instance_user_data(instance_id: &str) -> Option<String> {
    let output = cli_adapter::run(&[
        "ec2",
        "describe-instance-attribute",
        "--instance-id",
        instance_id,
        "--attribute",
        "userData",
        "--output",
        "json",
    ])?;

    parse_user_data_output(&output)
}

fn parse_user_data_output(output: &str) -> Option<String> {
    if let Ok(response) = serde_json::from_str::<UserDataResponse>(output)
        && let Some(user_data) = response.user_data
        && let Some(base64_data) = user_data.value
        && !base64_data.is_empty()
        && let Ok(decoded_bytes) = general_purpose::STANDARD.decode(base64_data)
    {
        let decoded = String::from_utf8_lossy(&decoded_bytes).to_string();
        if !decoded.trim().is_empty() {
            return Some(decoded);
        }
    }
    None
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

fn get_vpc_name(vpc_id: &str) -> String {
    let output = cli_adapter::run(&[
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

pub fn get_subnet_name(subnet_id: &str) -> String {
    let output = cli_adapter::run(&[
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
    let output = cli_adapter::run(&[
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

#[cfg(test)]
mod tests {
    use super::{
        Ec2Detail, VolumeDetail, cli_adapter, extract_security_groups, extract_state,
        get_instance_detail, get_subnet_name, iam_adapter, list_instances,
        parse_instance_detail_output, parse_instance_resources, parse_instance_volume_mappings,
        parse_user_data_output, parse_volume_detail_output,
    };
    use crate::aws_cli::iam::{AttachedPolicy, IamRoleDetail, InlinePolicy};
    use crate::i18n::Language;
    use std::sync::{Mutex, OnceLock};

    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        match LOCK.get_or_init(|| Mutex::new(())).lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn sample_detail() -> Ec2Detail {
        Ec2Detail {
            name: "web-a".to_string(),
            instance_id: "i-0123456789abcdef0".to_string(),
            instance_type: "t3.micro".to_string(),
            ami: "ami-12345678".to_string(),
            platform: "Linux".to_string(),
            architecture: "x86_64".to_string(),
            key_pair: "main-key".to_string(),
            vpc: "vpc-main".to_string(),
            subnet: "subnet-a".to_string(),
            az: "ap-northeast-2a".to_string(),
            public_ip: "1.1.1.1".to_string(),
            private_ip: "10.0.0.10".to_string(),
            security_groups: vec!["sg-web".to_string(), "sg-db".to_string()],
            state: "running".to_string(),
            ebs_optimized: true,
            monitoring: "Enabled".to_string(),
            iam_role: Some("role-web".to_string()),
            iam_role_detail: Some(IamRoleDetail {
                name: "role-web".to_string(),
                arn: "arn:aws:iam::123456789012:role/role-web".to_string(),
                assume_role_policy: "{\"Version\":\"2012-10-17\"}".to_string(),
                attached_policies: vec![AttachedPolicy {
                    name: "ReadOnlyAccess".to_string(),
                    arn: "arn:aws:iam::aws:policy/ReadOnlyAccess".to_string(),
                }],
                inline_policies: vec![InlinePolicy {
                    name: "inline-policy".to_string(),
                    document: "{\"Statement\":[]}".to_string(),
                }],
            }),
            launch_time: "2026-02-13T00:00:00Z".to_string(),
            tags: vec![
                ("Name".to_string(), "web-a".to_string()),
                ("Env".to_string(), "prod".to_string()),
            ],
            volumes: vec![VolumeDetail {
                device_name: "/dev/xvda".to_string(),
                volume_id: "vol-1234".to_string(),
                size_gb: 30,
                volume_type: "gp3".to_string(),
                iops: Some(3000),
                encrypted: true,
                delete_on_termination: true,
            }],
            user_data: Some("#!/bin/bash\necho hello".to_string()),
        }
    }

    #[test]
    fn parse_instance_resources_extracts_ids_and_names() {
        let payload = r#"
            [
              [
                ["i-aaa111", "running", [{"Key":"Name","Value":"web-a"}]],
                ["i-bbb222", "stopped", [{"Key":"Name","Value":"web-b"}]]
              ]
            ]
        "#;
        let resources = parse_instance_resources(payload);
        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0].id, "i-aaa111");
        assert!(resources[0].name.contains("running"));
    }

    #[test]
    fn extract_state_falls_back_to_unknown() {
        assert_eq!(extract_state(r#"{"State":{"Name":"running"}}"#), "running");
        assert_eq!(extract_state(r#"{"State":{"Name":"mystery"}}"#), "unknown");
    }

    #[test]
    fn extract_security_groups_deduplicates_names() {
        let payload = r#"
            {
              "SecurityGroups": [
                {"GroupName": "web"},
                {"GroupName": "db"},
                {"GroupName": "web"}
              ]
            }
        "#;
        let names = extract_security_groups(payload);
        assert_eq!(names, vec!["web".to_string(), "db".to_string()]);
    }

    #[test]
    fn ec2_markdown_renders_extended_sections() {
        let md = sample_detail().to_markdown(Language::English);
        assert!(md.contains("## EC2 Instance"));
        assert!(md.contains("IAM Role Detail"));
        assert!(md.contains("Storage"));
        assert!(md.contains("User Data"));
    }

    #[test]
    fn parse_instance_resources_filters_invalid_and_deduplicates_ids() {
        let payload = r#"
            [
              [
                ["i-valid001", "running", [{"Key":"Name","Value":"api-a"}]],
                ["i-valid001", "running", [{"Key":"Name","Value":"api-a"}]],
                ["i-", "running", [{"Key":"Name","Value":"invalid"}]],
                ["not-an-instance", "running", [{"Key":"Name","Value":"skip"}]]
              ]
            ]
        "#;
        let resources = parse_instance_resources(payload);
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].id, "i-valid001");
    }

    #[test]
    fn extract_security_groups_handles_empty_payload() {
        let names = extract_security_groups("{}");
        assert!(names.is_empty());
    }

    #[test]
    fn ec2_markdown_omits_optional_sections_when_data_is_missing() {
        let detail = Ec2Detail {
            name: String::new(),
            instance_id: "i-0abc".to_string(),
            instance_type: "t3.nano".to_string(),
            ami: "ami-0abc".to_string(),
            platform: "Linux".to_string(),
            architecture: "x86_64".to_string(),
            key_pair: "-".to_string(),
            vpc: "vpc-main".to_string(),
            subnet: "subnet-a".to_string(),
            az: "ap-northeast-2a".to_string(),
            public_ip: "-".to_string(),
            private_ip: "10.0.0.5".to_string(),
            security_groups: vec!["sg-main".to_string()],
            state: "stopped".to_string(),
            ebs_optimized: false,
            monitoring: "Disabled".to_string(),
            iam_role: None,
            iam_role_detail: None,
            launch_time: String::new(),
            tags: vec![],
            volumes: vec![],
            user_data: None,
        };

        let md = detail.to_markdown(Language::English);
        assert!(md.contains("NULL - i-0abc"));
        assert!(!md.contains("IAM Role Detail"));
        assert!(!md.contains("### Storage"));
        assert!(!md.contains("### User Data"));
        assert!(!md.contains("Public IP"));
    }

    #[test]
    fn parse_instance_detail_output_maps_basic_fields_from_fixture_json() {
        let payload = r#"
        {
          "InstanceType": "t3.micro",
          "Platform": "Windows",
          "Architecture": "x86_64",
          "KeyName": "main-key",
          "AvailabilityZone": "ap-northeast-2a",
          "PublicIpAddress": "3.3.3.3",
          "PrivateIpAddress": "10.0.0.20",
          "EbsOptimized": true,
          "LaunchTime": "2026-02-13T00:00:00Z",
          "Tags": [{"Key": "Name", "Value": "web-a"}, {"Key": "Env", "Value": "prod"}],
          "SecurityGroups": [{"GroupName": "sg-web"}],
          "State": {"Name": "running"},
          "Monitoring": {"State": "enabled"}
        }
        "#;

        let detail = parse_instance_detail_output(
            "i-abc",
            payload,
            "ami-name".to_string(),
            "vpc-name".to_string(),
            "subnet-name".to_string(),
            vec![],
            None,
        );
        assert_eq!(detail.instance_id, "i-abc");
        assert_eq!(detail.instance_type, "t3.micro");
        assert_eq!(detail.platform, "Windows");
        assert_eq!(detail.monitoring, "Enabled");
        assert_eq!(detail.name, "web-a");
        assert_eq!(detail.security_groups, vec!["sg-web".to_string()]);
    }

    #[test]
    fn parse_instance_volume_mappings_and_volume_details_handle_fixture_json() {
        let mappings_payload = r#"
        [
          {"DeviceName": "/dev/xvda", "Ebs": {"VolumeId": "vol-1", "DeleteOnTermination": true}},
          {"DeviceName": "/dev/xvdb", "Ebs": {"VolumeId": "vol-2", "DeleteOnTermination": false}}
        ]
        "#;
        let mappings = parse_instance_volume_mappings(mappings_payload);
        assert_eq!(mappings.len(), 2);
        assert_eq!(mappings[0].0, "/dev/xvda");
        assert!(mappings[0].2);
        assert!(!mappings[1].2);

        let volume_payload = r#"{"VolumeId": "vol-1", "Size": "30", "VolumeType": "gp3", "Iops": "3000", "Encrypted": true}"#;
        let volume = parse_volume_detail_output(volume_payload, "vol-1", "/dev/xvda", true);
        assert_eq!(volume.size_gb, 30);
        assert_eq!(volume.volume_type, "gp3");
        assert_eq!(volume.iops, Some(3000));
        assert!(volume.encrypted);
        assert!(volume.delete_on_termination);
    }

    #[test]
    fn parse_user_data_output_decodes_base64_and_rejects_empty_payloads() {
        let payload = r#"{"UserData":{"Value":"IyEvYmluL2Jhc2gKZWNobyBoaQ=="}}"#;
        let decoded = parse_user_data_output(payload).expect("decoded user data");
        assert!(decoded.contains("#!/bin/bash"));
        assert!(decoded.contains("echo hi"));

        let empty_payload = r#"{"UserData":{"Value":""}}"#;
        assert_eq!(parse_user_data_output(empty_payload), None);
    }

    #[test]
    fn top_level_list_instances_uses_mocked_cli() {
        let _guard = test_lock();
        cli_adapter::clear();
        iam_adapter::clear();

        cli_adapter::set(
            &[
                "ec2",
                "describe-instances",
                "--query",
                "Reservations[*].Instances[*].[InstanceId,State.Name,Tags]",
                "--output",
                "json",
            ],
            Some(
                r#"
                [
                  [
                    ["i-abc123", "running", [{"Key": "Name", "Value": "web-a"}]],
                    ["i-def456", "stopped", [{"Key": "Name", "Value": "web-b"}]]
                  ]
                ]
                "#,
            ),
        );

        let resources = list_instances();
        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0].id, "i-abc123");
        assert!(resources[0].name.contains("running"));
    }

    #[test]
    fn top_level_get_instance_detail_uses_mocked_cli_and_iam() {
        let _guard = test_lock();
        cli_adapter::clear();
        iam_adapter::clear();

        cli_adapter::set(
            &[
                "ec2",
                "describe-instances",
                "--instance-ids",
                "i-abc",
                "--output",
                "json",
            ],
            Some(
                r#"
                {
                  "Reservations": [{
                    "Instances": [{
                      "InstanceId": "i-abc",
                      "InstanceType": "t3.micro",
                      "ImageId": "ami-123",
                      "Platform": "Linux",
                      "Architecture": "x86_64",
                      "KeyName": "main-key",
                      "VpcId": "vpc-123",
                      "SubnetId": "subnet-123",
                      "Placement": {"AvailabilityZone": "ap-northeast-2a"},
                      "PublicIpAddress": "1.2.3.4",
                      "PrivateIpAddress": "10.0.0.10",
                      "State": {"Name": "running"},
                      "Monitoring": {"State": "enabled"},
                      "EbsOptimized": true,
                      "IamInstanceProfile": {
                        "Arn": "arn:aws:iam::123456789012:instance-profile/role-web"
                      },
                      "LaunchTime": "2026-02-13T00:00:00Z",
                      "Tags": [
                        {"Key": "Name", "Value": "web-a"},
                        {"Key": "Env", "Value": "prod"}
                      ],
                      "SecurityGroups": [{"GroupName": "sg-web"}]
                    }]
                  }]
                }
                "#,
            ),
        );
        cli_adapter::set(
            &[
                "ec2",
                "describe-images",
                "--image-ids",
                "ami-123",
                "--output",
                "json",
            ],
            Some(
                r#"{"Images":[{"ImageId":"ami-123","Tags":[{"Key": "Name", "Value": "ubuntu"}]}]}"#,
            ),
        );
        cli_adapter::set(
            &[
                "ec2",
                "describe-vpcs",
                "--vpc-ids",
                "vpc-123",
                "--output",
                "json",
            ],
            Some(r#"{"Vpcs":[{"VpcId":"vpc-123","Tags":[{"Key": "Name", "Value": "main-vpc"}]}]}"#),
        );
        cli_adapter::set(
            &[
                "ec2",
                "describe-subnets",
                "--subnet-ids",
                "subnet-123",
                "--output",
                "json",
            ],
            Some(
                r#"{"Subnets":[{"SubnetId":"subnet-123","Tags":[{"Key": "Name", "Value": "public-a"}]}]}"#,
            ),
        );
        cli_adapter::set(
            &[
                "ec2",
                "describe-instances",
                "--instance-ids",
                "i-abc",
                "--query",
                "Reservations[0].Instances[0].BlockDeviceMappings",
                "--output",
                "json",
            ],
            Some(
                r#"
                [
                  {"DeviceName": "/dev/xvda", "Ebs": {"VolumeId": "vol-1", "DeleteOnTermination": true}}
                ]
                "#,
            ),
        );
        cli_adapter::set(
            &[
                "ec2",
                "describe-volumes",
                "--volume-ids",
                "vol-1",
                "--output",
                "json",
            ],
            Some(
                r#"{"Volumes":[{"VolumeId": "vol-1", "Size": "30", "VolumeType": "gp3", "Iops": "3000", "Encrypted": true}]}"#,
            ),
        );
        cli_adapter::set(
            &[
                "ec2",
                "describe-instance-attribute",
                "--instance-id",
                "i-abc",
                "--attribute",
                "userData",
                "--output",
                "json",
            ],
            Some(r#"{"UserData":{"Value":"IyEvYmluL2Jhc2gKZWNobyBoaQ=="}}"#),
        );

        iam_adapter::set(
            "role-web",
            Some(IamRoleDetail {
                name: "role-web".to_string(),
                arn: "arn:aws:iam::123456789012:role/role-web".to_string(),
                assume_role_policy: "{\"Version\":\"2012-10-17\"}".to_string(),
                attached_policies: vec![],
                inline_policies: vec![],
            }),
        );

        let detail = get_instance_detail("i-abc").expect("instance detail");
        assert_eq!(detail.instance_id, "i-abc");
        assert_eq!(detail.name, "web-a");
        assert_eq!(detail.ami, "ubuntu");
        assert_eq!(detail.vpc, "main-vpc");
        assert_eq!(detail.subnet, "public-a");
        assert_eq!(detail.volumes.len(), 1);
        assert_eq!(detail.volumes[0].volume_id, "vol-1");
        assert!(
            detail
                .user_data
                .as_deref()
                .unwrap_or_default()
                .contains("echo hi")
        );
        assert_eq!(detail.iam_role.as_deref(), Some("role-web"));
        assert!(detail.iam_role_detail.is_some());
    }

    #[test]
    fn top_level_get_subnet_name_handles_found_and_missing_cases() {
        let _guard = test_lock();
        cli_adapter::clear();
        iam_adapter::clear();

        cli_adapter::set(
            &[
                "ec2",
                "describe-subnets",
                "--subnet-ids",
                "subnet-777",
                "--output",
                "json",
            ],
            Some(
                r#"{"Subnets":[{"SubnetId":"subnet-777","Tags":[{"Key": "Name", "Value": "app-a"}]}]}"#,
            ),
        );
        assert_eq!(get_subnet_name("subnet-777"), "app-a");
        assert_eq!(get_subnet_name("subnet-999"), "subnet-999");
    }
}
