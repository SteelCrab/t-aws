use crate::aws_cli::common::{AwsResource, get_sdk_config};
use aws_sdk_autoscaling::Client;
// use tokio::runtime::Runtime;

#[derive(Debug, Clone)]
pub struct ScalingPolicy {
    pub name: String,
    pub policy_type: String,
    pub adjustment_type: Option<String>,
    pub scaling_adjustment: Option<i32>,
    pub cooldown: Option<i32>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct AsgDetail {
    pub name: String,
    pub arn: String,
    pub launch_template_name: Option<String>,
    pub launch_template_id: Option<String>,
    pub launch_config_name: Option<String>,
    pub min_size: i32,
    pub max_size: i32,
    pub desired_capacity: i32,
    pub default_cooldown: i32,
    pub availability_zones: Vec<String>,
    pub target_group_arns: Vec<String>,
    pub health_check_type: String,
    pub health_check_grace_period: i32,
    pub instances: Vec<String>,
    pub created_time: String,
    pub scaling_policies: Vec<ScalingPolicy>,
    pub tags: Vec<(String, String)>,
}

impl AsgDetail {
    pub fn to_markdown(&self) -> String {
        let mut lines = vec![
            format!("## Auto Scaling Group ({})\n", self.name),
            "| 항목 | 값 |".to_string(),
            "|:---|:---|".to_string(),
            format!("| 이름 | {} |", self.name),
        ];

        // Launch Template or Config
        if let Some(ref lt_name) = self.launch_template_name {
            if let Some(ref lt_id) = self.launch_template_id {
                lines.push(format!("| 시작 템플릿 | {} (`{}`) |", lt_name, lt_id));
            } else {
                lines.push(format!("| 시작 템플릿 | {} |", lt_name));
            }
        } else if let Some(ref lc_name) = self.launch_config_name {
            lines.push(format!("| 시작 구성 | {} |", lc_name));
        }

        lines.push(format!("| 최소 크기 | {} |", self.min_size));
        lines.push(format!("| 최대 크기 | {} |", self.max_size));
        lines.push(format!("| 원하는 용량 | {} |", self.desired_capacity));
        lines.push(format!("| 기본 쿨다운 | {}초 |", self.default_cooldown));
        lines.push(format!("| 헬스 체크 유형 | {} |", self.health_check_type));
        lines.push(format!(
            "| 헬스 체크 유예 기간 | {}초 |",
            self.health_check_grace_period
        ));
        lines.push(format!("| 생성일 | {} |", self.created_time));

        // Availability Zones
        if !self.availability_zones.is_empty() {
            lines.push(String::new());
            lines.push("### 가용 영역\n".to_string());
            for az in &self.availability_zones {
                lines.push(format!("- {}", az));
            }
        }

        // Instances
        if !self.instances.is_empty() {
            lines.push(String::new());
            lines.push(format!("### 인스턴스 ({} 개)\n", self.instances.len()));
            lines.push("| 인스턴스 ID |".to_string());
            lines.push("|:---|".to_string());
            for inst in &self.instances {
                lines.push(format!("| `{}` |", inst));
            }
        }

        // Target Groups
        if !self.target_group_arns.is_empty() {
            lines.push(String::new());
            lines.push("### 대상 그룹\n".to_string());
            for tg in &self.target_group_arns {
                let tg_name = tg.split('/').nth(1).unwrap_or(tg);
                lines.push(format!("- {}", tg_name));
            }
        }

        // Scaling Policies
        if !self.scaling_policies.is_empty() {
            lines.push(String::new());
            lines.push("### 조정 정책\n".to_string());
            lines.push("| 이름 | 유형 | 조정 유형 | 조정 값 | 쿨다운 |".to_string());
            lines.push("|:---|:---|:---|:---|:---|".to_string());
            for policy in &self.scaling_policies {
                let adj_type = policy.adjustment_type.as_deref().unwrap_or("-");
                let adj_val = policy
                    .scaling_adjustment
                    .map(|v| v.to_string())
                    .unwrap_or("-".to_string());
                let cooldown = policy
                    .cooldown
                    .map(|v| format!("{}초", v))
                    .unwrap_or("-".to_string());
                lines.push(format!(
                    "| {} | {} | {} | {} | {} |",
                    policy.name, policy.policy_type, adj_type, adj_val, cooldown
                ));
            }
        }

        // Tags
        if !self.tags.is_empty() {
            lines.push(String::new());
            lines.push("### 태그\n".to_string());
            lines.push("| 키 | 값 |".to_string());
            lines.push("|:---|:---|".to_string());
            for (key, value) in &self.tags {
                if key != "Name" {
                    lines.push(format!("| {} | {} |", key, value));
                }
            }
        }

        lines.push(String::new());
        lines.join("\n")
    }
}

use crate::aws_cli::common::get_runtime;

/// List all Auto Scaling Groups using AWS SDK
pub fn list_auto_scaling_groups() -> Vec<AwsResource> {
    get_runtime().block_on(list_auto_scaling_groups_async())
}

async fn list_auto_scaling_groups_async() -> Vec<AwsResource> {
    let config = get_sdk_config().await;
    let client = Client::new(&config);

    let result = client.describe_auto_scaling_groups().send().await;

    match result {
        Ok(output) => output
            .auto_scaling_groups()
            .iter()
            .map(|asg| {
                let name = asg
                    .auto_scaling_group_name()
                    .unwrap_or_default()
                    .to_string();
                let arn = asg.auto_scaling_group_arn().unwrap_or_default().to_string();
                let desired = asg.desired_capacity().unwrap_or(0);
                let min = asg.min_size().unwrap_or(0);
                let max = asg.max_size().unwrap_or(0);
                AwsResource {
                    name: name.clone(),
                    id: name.clone(),
                    state: format!("Desired: {} (Min: {}, Max: {})", desired, min, max),
                    az: String::new(),
                    cidr: arn,
                }
            })
            .collect(),
        Err(e) => {
            tracing::error!("Error listing auto scaling groups: {:?}", e);
            eprintln!("Error listing auto scaling groups: {:?}", e);
            Vec::new()
        }
    }
}

/// Get Auto Scaling Group detail using AWS SDK
pub fn get_asg_detail(asg_name: &str) -> Option<AsgDetail> {
    get_runtime().block_on(get_asg_detail_async(asg_name))
}

async fn get_asg_detail_async(asg_name: &str) -> Option<AsgDetail> {
    let config = get_sdk_config().await;
    let client = Client::new(&config);

    // Get ASG details
    let result = client
        .describe_auto_scaling_groups()
        .auto_scaling_group_names(asg_name)
        .send()
        .await
        .ok()?;

    let asg = result.auto_scaling_groups().first()?;

    let name = asg
        .auto_scaling_group_name()
        .unwrap_or_default()
        .to_string();
    let arn = asg.auto_scaling_group_arn().unwrap_or_default().to_string();

    // Launch Template
    let (launch_template_name, launch_template_id) = if let Some(lt) = asg.launch_template() {
        (
            lt.launch_template_name().map(|s| s.to_string()),
            lt.launch_template_id().map(|s| s.to_string()),
        )
    } else if let Some(mip) = asg.mixed_instances_policy() {
        if let Some(lt_spec) = mip
            .launch_template()
            .and_then(|lt| lt.launch_template_specification())
        {
            (
                lt_spec.launch_template_name().map(|s| s.to_string()),
                lt_spec.launch_template_id().map(|s| s.to_string()),
            )
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let launch_config_name = asg.launch_configuration_name().map(|s| s.to_string());

    let min_size = asg.min_size();
    let max_size = asg.max_size();
    let desired_capacity = asg.desired_capacity();
    let default_cooldown = asg.default_cooldown();
    let health_check_type = asg.health_check_type().unwrap_or("EC2").to_string();
    let health_check_grace_period = asg.health_check_grace_period().unwrap_or(0);

    let created_time = asg
        .created_time()
        .map(|dt| {
            dt.fmt(aws_sdk_autoscaling::primitives::DateTimeFormat::DateTime)
                .unwrap_or_default()
                .split('T')
                .next()
                .unwrap_or("")
                .to_string()
        })
        .unwrap_or_default();

    let availability_zones = asg
        .availability_zones()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let target_group_arns = asg
        .target_group_arns()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let instances: Vec<String> = asg
        .instances()
        .iter()
        .filter_map(|i| i.instance_id().map(|s| s.to_string()))
        .collect();

    let tags: Vec<(String, String)> = asg
        .tags()
        .iter()
        .map(|t| {
            (
                t.key().unwrap_or_default().to_string(),
                t.value().unwrap_or_default().to_string(),
            )
        })
        .collect();

    // Get Scaling Policies
    let scaling_policies = get_scaling_policies_async(&client, &name).await;

    Some(AsgDetail {
        name,
        arn,
        launch_template_name,
        launch_template_id,
        launch_config_name,
        min_size: min_size.unwrap_or(0),
        max_size: max_size.unwrap_or(0),
        desired_capacity: desired_capacity.unwrap_or(0),
        default_cooldown: default_cooldown.unwrap_or(300),
        availability_zones,
        target_group_arns,
        health_check_type,
        health_check_grace_period,
        instances,
        created_time,
        scaling_policies,
        tags,
    })
}

async fn get_scaling_policies_async(client: &Client, asg_name: &str) -> Vec<ScalingPolicy> {
    let result = client
        .describe_policies()
        .auto_scaling_group_name(asg_name)
        .send()
        .await;

    match result {
        Ok(output) => output
            .scaling_policies()
            .iter()
            .map(|p| ScalingPolicy {
                name: p.policy_name().unwrap_or_default().to_string(),
                policy_type: p.policy_type().unwrap_or_default().to_string(),
                adjustment_type: p.adjustment_type().map(|s| s.to_string()),
                scaling_adjustment: p.scaling_adjustment(),
                cooldown: p.cooldown(),
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}
