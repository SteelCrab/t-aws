use crate::aws_cli::asg::{AsgDetail, ScalingPolicy};
use crate::aws_cli::common::{AwsResource, get_runtime, get_sdk_config};
use aws_sdk_autoscaling::Client;

/// List all Auto Scaling Groups using AWS SDK
pub fn list_auto_scaling_groups() -> Vec<AwsResource> {
    get_runtime().block_on(list_auto_scaling_groups_async())
}

async fn list_auto_scaling_groups_async() -> Vec<AwsResource> {
    let config = get_sdk_config().await;
    let client = Client::new(&config);

    let result = client
        .describe_auto_scaling_groups()
        .into_paginator()
        .items()
        .send()
        .try_collect()
        .await;

    match result {
        Ok(groups) => groups.iter().map(map_asg_resource).collect(),
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

    let scaling_policies = get_scaling_policies_async(&client, &name).await;
    Some(map_asg_detail(asg, scaling_policies, Some((name, arn))))
}

async fn get_scaling_policies_async(client: &Client, asg_name: &str) -> Vec<ScalingPolicy> {
    let result = client
        .describe_policies()
        .auto_scaling_group_name(asg_name)
        .send()
        .await;

    match result {
        Ok(output) => map_scaling_policies(output.scaling_policies()),
        Err(_) => Vec::new(),
    }
}

fn map_asg_resource(asg: &aws_sdk_autoscaling::types::AutoScalingGroup) -> AwsResource {
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
}

fn map_asg_detail(
    asg: &aws_sdk_autoscaling::types::AutoScalingGroup,
    scaling_policies: Vec<ScalingPolicy>,
    prefetched_identity: Option<(String, String)>,
) -> AsgDetail {
    let (name, arn) = prefetched_identity.unwrap_or_else(|| {
        (
            asg.auto_scaling_group_name()
                .unwrap_or_default()
                .to_string(),
            asg.auto_scaling_group_arn().unwrap_or_default().to_string(),
        )
    });

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

    AsgDetail {
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
    }
}

fn map_scaling_policies(
    policies: &[aws_sdk_autoscaling::types::ScalingPolicy],
) -> Vec<ScalingPolicy> {
    policies
        .iter()
        .map(|p| ScalingPolicy {
            name: p.policy_name().unwrap_or_default().to_string(),
            policy_type: p.policy_type().unwrap_or_default().to_string(),
            adjustment_type: p.adjustment_type().map(|s| s.to_string()),
            scaling_adjustment: p.scaling_adjustment(),
            cooldown: p.cooldown(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{map_asg_detail, map_asg_resource, map_scaling_policies};

    #[test]
    fn map_asg_resource_formats_capacity_state() {
        let asg = aws_sdk_autoscaling::types::AutoScalingGroup::builder()
            .auto_scaling_group_name("asg-main")
            .auto_scaling_group_arn(
                "arn:aws:autoscaling:ap-northeast-2:123456789012:autoScalingGroup:abcd:autoScalingGroupName/asg-main",
            )
            .desired_capacity(2)
            .min_size(1)
            .max_size(4)
            .build();

        let resource = map_asg_resource(&asg);
        assert_eq!(resource.id, "asg-main");
        assert_eq!(resource.name, "asg-main");
        assert!(resource.state.contains("Desired: 2"));
        assert!(resource.state.contains("Min: 1"));
        assert!(resource.state.contains("Max: 4"));
    }

    #[test]
    fn map_scaling_policies_maps_optional_fields() {
        let policy = aws_sdk_autoscaling::types::ScalingPolicy::builder()
            .policy_name("scale-out")
            .policy_type("SimpleScaling")
            .adjustment_type("ChangeInCapacity")
            .scaling_adjustment(1)
            .cooldown(60)
            .build();

        let mapped = map_scaling_policies(&[policy]);
        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].name, "scale-out");
        assert_eq!(mapped[0].policy_type, "SimpleScaling");
        assert_eq!(
            mapped[0].adjustment_type.as_deref(),
            Some("ChangeInCapacity")
        );
        assert_eq!(mapped[0].scaling_adjustment, Some(1));
        assert_eq!(mapped[0].cooldown, Some(60));
    }

    #[test]
    fn map_asg_detail_uses_launch_template_and_tags() {
        let launch_template = aws_sdk_autoscaling::types::LaunchTemplateSpecification::builder()
            .launch_template_name("lt-main")
            .launch_template_id("lt-1234")
            .build();
        let instance = aws_sdk_autoscaling::types::Instance::builder()
            .instance_id("i-1234")
            .build();
        let tag = aws_sdk_autoscaling::types::TagDescription::builder()
            .key("Env")
            .value("prod")
            .build();

        let asg = aws_sdk_autoscaling::types::AutoScalingGroup::builder()
            .auto_scaling_group_name("asg-main")
            .auto_scaling_group_arn("arn:aws:autoscaling:...:asg-main")
            .launch_template(launch_template)
            .min_size(1)
            .max_size(3)
            .desired_capacity(2)
            .default_cooldown(300)
            .health_check_type("EC2")
            .health_check_grace_period(120)
            .availability_zones("ap-northeast-2a")
            .target_group_arns("arn:aws:elasticloadbalancing:...:targetgroup/tg-main/abcd")
            .instances(instance)
            .tags(tag)
            .build();

        let detail = map_asg_detail(&asg, Vec::new(), None);
        assert_eq!(detail.name, "asg-main");
        assert_eq!(detail.launch_template_name.as_deref(), Some("lt-main"));
        assert_eq!(detail.launch_template_id.as_deref(), Some("lt-1234"));
        assert_eq!(detail.instances, vec!["i-1234".to_string()]);
        assert_eq!(detail.tags, vec![("Env".to_string(), "prod".to_string())]);
    }

    #[test]
    fn map_asg_detail_prefetched_identity_and_defaults_when_fields_missing() {
        let asg = aws_sdk_autoscaling::types::AutoScalingGroup::builder().build();

        let detail = map_asg_detail(
            &asg,
            Vec::new(),
            Some(("prefetched-asg".to_string(), "arn:prefetched".to_string())),
        );

        assert_eq!(detail.name, "prefetched-asg");
        assert_eq!(detail.arn, "arn:prefetched");
        assert_eq!(detail.launch_template_name, None);
        assert_eq!(detail.launch_template_id, None);
        assert_eq!(detail.launch_config_name, None);
        assert_eq!(detail.default_cooldown, 300);
        assert_eq!(detail.health_check_type, "EC2");
        assert_eq!(detail.health_check_grace_period, 0);
        assert_eq!(detail.created_time, "");
    }

    #[test]
    fn map_asg_resource_uses_zero_defaults_when_capacity_absent() {
        let asg = aws_sdk_autoscaling::types::AutoScalingGroup::builder()
            .auto_scaling_group_name("asg-empty")
            .build();
        let resource = map_asg_resource(&asg);

        assert_eq!(resource.id, "asg-empty");
        assert!(resource.state.contains("Desired: 0"));
        assert!(resource.state.contains("Min: 0"));
        assert!(resource.state.contains("Max: 0"));
    }

    #[test]
    fn map_scaling_policies_keeps_none_for_missing_optional_fields() {
        let policy = aws_sdk_autoscaling::types::ScalingPolicy::builder()
            .policy_name("scale-in")
            .policy_type("SimpleScaling")
            .build();

        let mapped = map_scaling_policies(&[policy]);
        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].name, "scale-in");
        assert_eq!(mapped[0].adjustment_type, None);
        assert_eq!(mapped[0].scaling_adjustment, None);
        assert_eq!(mapped[0].cooldown, None);
    }
}
