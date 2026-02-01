use crate::aws_cli::common::{
    AwsResource, extract_json_value, extract_tags, parse_name_tag, run_aws_cli,
};
use base64::{Engine as _, engine::general_purpose};
use serde::Deserialize;

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
    pub launch_time: String,
    pub tags: Vec<(String, String)>,
    pub volumes: Vec<VolumeDetail>,
    pub user_data: Option<String>,
}

impl Ec2Detail {
    pub fn to_markdown(&self) -> String {
        let display_name = if self.name.is_empty() || self.name == self.instance_id {
            format!("NULL - {}", self.instance_id)
        } else {
            format!("{} - {}", self.name, self.instance_id)
        };
        let mut lines = vec![
            format!("## EC2 인스턴스 ({})\n", display_name),
            "| 항목 | 값 |".to_string(),
            "|:---|:---|".to_string(),
            format!("| 이름 | {} |", display_name),
            format!("| 상태 | {} |", self.state),
        ];

        for (key, value) in &self.tags {
            if key != "Name" {
                lines.push(format!("| 태그-{} | {} |", key, value));
            }
        }

        lines.push(format!("| AMI | {} |", self.ami));
        lines.push(format!("| 인스턴스 유형 | {} |", self.instance_type));
        lines.push(format!("| 플랫폼 | {} |", self.platform));
        lines.push(format!("| 아키텍처 | {} |", self.architecture));
        lines.push(format!("| 키 페어 | {} |", self.key_pair));
        lines.push(format!("| VPC | {} |", self.vpc));
        lines.push(format!("| 서브넷 | {} |", self.subnet));
        lines.push(format!("| 가용 영역 | {} |", self.az));
        lines.push(format!("| 프라이빗 IP | {} |", self.private_ip));

        if self.public_ip != "-" && !self.public_ip.is_empty() {
            lines.push(format!("| 퍼블릭 IP | {} |", self.public_ip));
        }

        lines.push(format!(
            "| 보안 그룹 | {} |",
            self.security_groups.join(", ")
        ));

        let ebs_str = if self.ebs_optimized {
            "활성화"
        } else {
            "비활성화"
        };
        lines.push(format!("| EBS 최적화 | {} |", ebs_str));
        lines.push(format!("| 모니터링 | {} |", self.monitoring));

        if let Some(ref role) = self.iam_role {
            lines.push(format!("| IAM 역할 | {} |", role));
        }

        if !self.launch_time.is_empty() {
            lines.push(format!("| 시작 시간 | {} |", self.launch_time));
        }

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

    // Platform (Linux if not specified, Windows otherwise)
    let platform = extract_json_value(json, "Platform").unwrap_or_else(|| "Linux".to_string());
    let architecture =
        extract_json_value(json, "Architecture").unwrap_or_else(|| "x86_64".to_string());

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

    let az = extract_json_value(json, "AvailabilityZone").unwrap_or_default();
    let public_ip = extract_json_value(json, "PublicIpAddress").unwrap_or_else(|| "-".to_string());
    let private_ip = extract_json_value(json, "PrivateIpAddress").unwrap_or_default();
    let state = extract_state(json);

    // EBS Optimized
    let ebs_optimized = json.contains("\"EbsOptimized\": true");

    // Monitoring
    let monitoring = if json.contains("\"State\": \"enabled\"") {
        "활성화".to_string()
    } else {
        "비활성화".to_string()
    };

    // IAM Role (from IamInstanceProfile)
    let iam_role = extract_json_value(json, "Arn")
        .and_then(|arn| arn.split('/').next_back().map(|s| s.to_string()));

    // Launch Time
    let launch_time = extract_json_value(json, "LaunchTime").unwrap_or_default();

    let tags = extract_tags(json);
    let name = tags
        .iter()
        .find(|(k, _)| k == "Name")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();

    let security_groups = extract_security_groups(json);
    let volumes = get_instance_volumes(instance_id);
    let user_data = get_instance_user_data(instance_id);

    Some(Ec2Detail {
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
        launch_time,
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
        let mut search_start = 0;
        while let Some(device_pos) = json[search_start..].find("\"DeviceName\": \"") {
            let device_start = search_start + device_pos + 15;
            if let Some(device_end) = json[device_start..].find('"') {
                let device_name = json[device_start..device_start + device_end].to_string();

                if let Some(vol_pos) = json[device_start..].find("\"VolumeId\": \"") {
                    let vol_start = device_start + vol_pos + 13;
                    if let Some(vol_end) = json[vol_start..].find('"') {
                        let volume_id = json[vol_start..vol_start + vol_end].to_string();

                        let delete_on_term = json
                            [device_start..device_start + 500.min(json.len() - device_start)]
                            .contains("\"DeleteOnTermination\": true");

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

    if let Ok(response) = serde_json::from_str::<UserDataResponse>(&output)
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

pub fn get_subnet_name(subnet_id: &str) -> String {
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
