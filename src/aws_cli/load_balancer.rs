use crate::aws_cli::common::{AwsResource, run_aws_cli};
use crate::i18n::{I18n, Language};
use serde::Deserialize;

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
    ip_address_type: String,
    #[serde(default)]
    availability_zones: Vec<AvailabilityZone>,
    #[serde(default)]
    security_groups: Vec<String>,
    #[serde(default)]
    state: Option<LoadBalancerState>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AvailabilityZone {
    zone_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LoadBalancerState {
    code: String,
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

#[derive(Debug)]
pub struct LoadBalancerDetail {
    pub name: String,
    pub arn: String,
    pub dns_name: String,
    pub lb_type: String,
    pub scheme: String,
    pub vpc_id: String,
    pub ip_address_type: String,
    pub state: String,
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
    pub fn to_markdown(&self, lang: Language) -> String {
        let i18n = I18n::new(lang);
        // Extract LB ID from ARN (last part after the last /)
        let lb_id = self.arn.rsplit('/').next().unwrap_or(&self.arn);
        let display_name = if self.name.is_empty() {
            format!("NULL - {}", lb_id)
        } else {
            format!("{} - {}", self.name, lb_id)
        };
        let mut lines = vec![
            format!("## Load Balancer ({})\n", display_name),
            format!("| {} | {} |", i18n.item(), i18n.value()),
            "|:---|:---|".to_string(),
            format!("| {} | {} |", i18n.md_name(), display_name),
            format!("| {} | {} |", i18n.md_state(), self.state),
            format!("| {} | {} |", i18n.md_dns_name(), self.dns_name),
            format!("| {} | {} |", i18n.md_type(), self.lb_type),
            format!("| {} | {} |", i18n.md_scheme(), self.scheme),
            format!(
                "| {} | {} |",
                i18n.md_ip_address_type(),
                self.ip_address_type
            ),
            format!("| VPC ID | {} |", self.vpc_id),
        ];

        if !self.availability_zones.is_empty() {
            lines.push(format!(
                "| {} | {} |",
                i18n.md_availability_zones(),
                self.availability_zones.join(", ")
            ));
        }

        if !self.security_groups.is_empty() {
            lines.push(format!(
                "| {} | {} |",
                i18n.md_security_groups(),
                self.security_groups.join(", ")
            ));
        }

        if !self.listeners.is_empty() {
            lines.push("\n### Listeners".to_string());
            lines.push(format!(
                "| {} | {} | {} |",
                i18n.md_port(),
                i18n.md_protocol(),
                i18n.md_default_action()
            ));
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
                lines.push(format!("\n**{}**", i18n.md_basic_info()));
                lines.push(format!("| {} | {} |", i18n.item(), i18n.value()));
                lines.push("|:---|:---|".to_string());
                lines.push(format!("| {} | {} |", i18n.md_protocol(), tg.protocol));
                lines.push(format!("| {} | {} |", i18n.md_port(), tg.port));
                lines.push(format!(
                    "| {} | {} |",
                    i18n.md_target_type(),
                    tg.target_type
                ));
                lines.push(format!(
                    "| {} | {} {} |",
                    i18n.md_health_check(),
                    tg.health_check_protocol,
                    tg.health_check_path
                ));
                lines.push(format!(
                    "| {} | {}: {}, {}: {} |",
                    i18n.md_threshold(),
                    i18n.md_healthy(),
                    tg.healthy_threshold,
                    i18n.md_unhealthy(),
                    tg.unhealthy_threshold
                ));

                if !tg.targets.is_empty() {
                    lines.push(format!("\n**{}**", i18n.md_targets()));
                    lines.push(format!(
                        "| Target ID | {} | {} |",
                        i18n.md_port(),
                        i18n.md_state()
                    ));
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

    let ip_address_type = if lb.ip_address_type.is_empty() {
        "ipv4".to_string()
    } else {
        lb.ip_address_type.clone()
    };

    let state = lb
        .state
        .as_ref()
        .map(|s| s.code.clone())
        .unwrap_or_else(|| "unknown".to_string());

    Some(LoadBalancerDetail {
        name: lb.load_balancer_name.clone(),
        arn: lb.load_balancer_arn.clone(),
        dns_name: lb.dns_name.clone(),
        lb_type: lb.lb_type.clone(),
        scheme: lb.scheme.clone(),
        vpc_id: lb.vpc_id.clone(),
        ip_address_type,
        state,
        availability_zones,
        security_groups: lb.security_groups.clone(),
        listeners,
        target_groups,
    })
}

fn get_target_group_info(tg_arn: &str) -> Option<TargetGroupInfo> {
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
