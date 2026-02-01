use crate::aws_cli::common::{AwsResource, Tag, run_aws_cli};
use serde::Deserialize;

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

impl SecurityGroupDetail {
    pub fn to_markdown(&self) -> String {
        let display_name = if self.name.is_empty() || self.name == self.id {
            format!("NULL - {}", self.id)
        } else {
            format!("{} - {}", self.name, self.id)
        };
        let mut lines = vec![
            format!("## Security Group ({})\n", display_name),
            "| 항목 | 값 |".to_string(),
            "|:---|:---|".to_string(),
            format!("| 이름 | {} |", display_name),
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
                state: sg.vpc_id,
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
