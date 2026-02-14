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

    let i18n = I18n::new(Language::English);
    parse_load_balancers_output(&output, &i18n).unwrap_or_default()
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

    let listeners_output = run_aws_cli(&[
        "elbv2",
        "describe-listeners",
        "--load-balancer-arn",
        lb_arn,
        "--output",
        "json",
    ])?;

    let (listeners, target_group_arns) = parse_listener_infos(&listeners_output)?;

    let mut target_groups = Vec::new();

    for tg_arn in &target_group_arns {
        if let Some(tg_info) = get_target_group_info(tg_arn) {
            target_groups.push(tg_info);
        }
    }
    parse_load_balancer_detail_output(&output, listeners, target_groups)
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

    let health_output = run_aws_cli(&[
        "elbv2",
        "describe-target-health",
        "--target-group-arn",
        tg_arn,
        "--output",
        "json",
    ])?;

    parse_target_group_info_outputs(&output, &health_output)
}

fn parse_load_balancers_output(output: &str, i18n: &I18n) -> Option<Vec<AwsResource>> {
    let response: LoadBalancersResponse = serde_json::from_str(output).ok()?;
    Some(
        response
            .load_balancers
            .into_iter()
            .map(|lb| {
                let scheme_text = match lb.scheme.as_str() {
                    "internet-facing" => i18n.md_public().to_string(),
                    "internal" => i18n.md_private().to_string(),
                    _ => lb.scheme.clone(),
                };

                AwsResource {
                    name: format!("{} ({})", lb.load_balancer_name, scheme_text),
                    id: lb.load_balancer_arn,
                    state: lb.lb_type,
                    az: lb.vpc_id,
                    cidr: lb.dns_name,
                }
            })
            .collect(),
    )
}

fn parse_target_group_info_outputs(
    target_group_output: &str,
    target_health_output: &str,
) -> Option<TargetGroupInfo> {
    let response: TargetGroupsResponse = serde_json::from_str(target_group_output).ok()?;
    let tg = response.target_groups.first()?;

    let health_response: TargetHealthResponse = serde_json::from_str(target_health_output).ok()?;
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

fn parse_listener_infos(listeners_output: &str) -> Option<(Vec<ListenerInfo>, Vec<String>)> {
    let listeners_response: ListenersResponse = serde_json::from_str(listeners_output).ok()?;
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
    target_group_arns.sort();
    target_group_arns.dedup();

    Some((listeners, target_group_arns))
}

fn parse_load_balancer_detail_output(
    output: &str,
    listeners: Vec<ListenerInfo>,
    target_groups: Vec<TargetGroupInfo>,
) -> Option<LoadBalancerDetail> {
    let response: LoadBalancersResponse = serde_json::from_str(output).ok()?;
    let lb = response.load_balancers.first()?;

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

#[cfg(test)]
mod tests {
    use super::{
        ListenerInfo, LoadBalancerDetail, TargetGroupInfo, TargetInfo, parse_listener_infos,
        parse_load_balancer_detail_output, parse_load_balancers_output,
        parse_target_group_info_outputs,
    };
    use crate::i18n::{I18n, Language};

    #[test]
    fn load_balancer_markdown_contains_listener_and_target_group_sections() {
        let detail = LoadBalancerDetail {
            name: "alb-main".to_string(),
            arn: "arn:aws:elasticloadbalancing:ap-northeast-2:123456789012:loadbalancer/app/alb-main/1234".to_string(),
            dns_name: "alb-main.example.com".to_string(),
            lb_type: "application".to_string(),
            scheme: "internet-facing".to_string(),
            vpc_id: "vpc-1111".to_string(),
            ip_address_type: "ipv4".to_string(),
            state: "active".to_string(),
            availability_zones: vec!["ap-northeast-2a".to_string()],
            security_groups: vec!["sg-1234".to_string()],
            listeners: vec![ListenerInfo {
                port: 80,
                protocol: "HTTP".to_string(),
                default_action: "forward".to_string(),
            }],
            target_groups: vec![TargetGroupInfo {
                name: "tg-main".to_string(),
                arn: "arn:aws:elasticloadbalancing:...:targetgroup/tg-main/abcd".to_string(),
                protocol: "HTTP".to_string(),
                port: 80,
                target_type: "instance".to_string(),
                health_check_protocol: "HTTP".to_string(),
                health_check_path: "/health".to_string(),
                healthy_threshold: 2,
                unhealthy_threshold: 3,
                targets: vec![TargetInfo {
                    id: "i-1234".to_string(),
                    port: 80,
                    health_status: "healthy".to_string(),
                }],
            }],
        };

        let md = detail.to_markdown(Language::English);
        assert!(md.contains("## Load Balancer"));
        assert!(md.contains("### Listeners"));
        assert!(md.contains("### Target Groups"));
        assert!(md.contains("healthy"));
    }

    #[test]
    fn parse_load_balancers_output_maps_scheme_text() {
        let payload = r#"
            {
              "LoadBalancers": [
                {
                  "LoadBalancerName": "alb-main",
                  "LoadBalancerArn": "arn:aws:elasticloadbalancing:...:loadbalancer/app/alb-main/1234",
                  "DNSName": "alb-main.example.com",
                  "Type": "application",
                  "Scheme": "internet-facing",
                  "VpcId": "vpc-1111",
                  "AvailabilityZones": [],
                  "SecurityGroups": []
                }
              ]
            }
        "#;

        let i18n = I18n::new(Language::English);
        let lbs = parse_load_balancers_output(payload, &i18n).expect("parse lbs");
        assert_eq!(lbs.len(), 1);
        assert!(lbs[0].name.contains("Public"));
        assert_eq!(lbs[0].state, "application");
    }

    #[test]
    fn parse_target_group_info_outputs_extracts_targets() {
        let tg_payload = r#"
            {
              "TargetGroups": [
                {
                  "TargetGroupName": "tg-main",
                  "TargetGroupArn": "arn:aws:elasticloadbalancing:...:targetgroup/tg-main/1234",
                  "Protocol": "HTTP",
                  "Port": 80,
                  "TargetType": "instance",
                  "HealthCheckProtocol": "HTTP",
                  "HealthCheckPath": "/health",
                  "HealthyThresholdCount": 2,
                  "UnhealthyThresholdCount": 3
                }
              ]
            }
        "#;

        let health_payload = r#"
            {
              "TargetHealthDescriptions": [
                {
                  "Target": {"Id": "i-1234", "Port": 80},
                  "HealthCheckPort": "traffic-port",
                  "TargetHealth": {"State": "healthy"}
                }
              ]
            }
        "#;

        let info = parse_target_group_info_outputs(tg_payload, health_payload).expect("tg info");
        assert_eq!(info.name, "tg-main");
        assert_eq!(info.targets.len(), 1);
        assert_eq!(info.targets[0].id, "i-1234");
    }

    #[test]
    fn parse_listener_infos_extracts_default_action_and_dedups_target_groups() {
        let listeners_payload = r#"
            {
              "Listeners": [
                {
                  "Port": 80,
                  "Protocol": "HTTP",
                  "DefaultActions": [
                    {"Type":"forward","TargetGroupArn":"arn:...:targetgroup/tg-a/1"}
                  ]
                },
                {
                  "Port": 443,
                  "Protocol": "HTTPS",
                  "DefaultActions": [
                    {"Type":"forward","TargetGroupArn":"arn:...:targetgroup/tg-a/1"}
                  ]
                }
              ]
            }
        "#;
        let (listeners, tg_arns) = parse_listener_infos(listeners_payload).expect("listeners");
        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].default_action, "forward");
        assert_eq!(tg_arns.len(), 1);
    }

    #[test]
    fn parse_load_balancer_detail_output_applies_defaults() {
        let lb_payload = r#"
            {
              "LoadBalancers": [
                {
                  "LoadBalancerName": "lb-a",
                  "LoadBalancerArn": "arn:...:loadbalancer/app/lb-a/1",
                  "DNSName": "lb-a.example.com",
                  "Type": "application",
                  "Scheme": "internal",
                  "VpcId": "vpc-1",
                  "AvailabilityZones": [],
                  "SecurityGroups": []
                }
              ]
            }
        "#;
        let detail = parse_load_balancer_detail_output(lb_payload, vec![], vec![]).expect("detail");
        assert_eq!(detail.name, "lb-a");
        assert_eq!(detail.ip_address_type, "ipv4");
        assert_eq!(detail.state, "unknown");
    }

    #[test]
    fn load_balancer_markdown_omits_listener_and_target_sections_when_empty() {
        let detail = LoadBalancerDetail {
            name: String::new(),
            arn: "arn:.../lb-id".to_string(),
            dns_name: "lb.example.com".to_string(),
            lb_type: "application".to_string(),
            scheme: "internal".to_string(),
            vpc_id: "vpc-1".to_string(),
            ip_address_type: "ipv4".to_string(),
            state: "active".to_string(),
            availability_zones: vec![],
            security_groups: vec![],
            listeners: vec![],
            target_groups: vec![],
        };
        let md = detail.to_markdown(Language::English);
        assert!(md.contains("NULL - lb-id"));
        assert!(!md.contains("### Listeners"));
        assert!(!md.contains("### Target Groups"));
    }
}
