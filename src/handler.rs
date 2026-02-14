use crate::app::{App, LoadingTask, REGIONS, SERVICE_KEYS, Screen};
use crate::aws_cli::NetworkDetail;
use crate::blueprint::{BlueprintResource, ResourceType};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

type VpcInfoTuple = (String, String, String, Vec<(String, String)>);

#[cfg(not(test))]
mod aws_adapter {
    use super::VpcInfoTuple;
    use crate::aws_cli;

    pub fn set_region(region: &str) {
        aws_cli::set_region(region);
    }

    pub fn list_instances() -> Vec<aws_cli::AwsResource> {
        aws_cli::list_instances()
    }

    pub fn get_instance_detail(id: &str) -> Option<aws_cli::Ec2Detail> {
        aws_cli::get_instance_detail(id)
    }

    pub fn list_vpcs() -> Vec<aws_cli::AwsResource> {
        aws_cli::list_vpcs()
    }

    pub fn get_network_detail(vpc_id: &str) -> Option<aws_cli::NetworkDetail> {
        aws_cli::get_network_detail(vpc_id)
    }

    pub fn get_vpc_info(vpc_id: &str) -> Option<VpcInfoTuple> {
        aws_cli::get_vpc_info(vpc_id)
    }

    pub fn list_subnets(vpc_id: &str) -> Vec<aws_cli::AwsResource> {
        aws_cli::list_subnets(vpc_id)
    }

    pub fn list_internet_gateways(vpc_id: &str) -> Vec<aws_cli::AwsResource> {
        aws_cli::list_internet_gateways(vpc_id)
    }

    pub fn list_nat_gateways(vpc_id: &str) -> Vec<aws_cli::NatDetail> {
        aws_cli::list_nat_gateways(vpc_id)
    }

    pub fn list_route_tables(vpc_id: &str) -> Vec<aws_cli::RouteTableDetail> {
        aws_cli::list_route_tables(vpc_id)
    }

    pub fn list_eips() -> Vec<aws_cli::EipDetail> {
        aws_cli::list_eips()
    }

    pub fn get_vpc_dns_support(vpc_id: &str) -> bool {
        aws_cli::get_vpc_dns_support(vpc_id)
    }

    pub fn get_vpc_dns_hostnames(vpc_id: &str) -> bool {
        aws_cli::get_vpc_dns_hostnames(vpc_id)
    }

    pub fn list_security_groups() -> Vec<aws_cli::AwsResource> {
        aws_cli::list_security_groups()
    }

    pub fn get_security_group_detail(id: &str) -> Option<aws_cli::SecurityGroupDetail> {
        aws_cli::get_security_group_detail(id)
    }

    pub fn list_load_balancers() -> Vec<aws_cli::AwsResource> {
        aws_cli::list_load_balancers()
    }

    pub fn get_load_balancer_detail(id: &str) -> Option<aws_cli::LoadBalancerDetail> {
        aws_cli::get_load_balancer_detail(id)
    }

    pub fn list_ecr_repositories() -> Vec<aws_cli::AwsResource> {
        aws_cli::ecr::list_ecr_repositories()
    }

    pub fn get_ecr_detail(id: &str) -> Option<aws_cli::EcrDetail> {
        aws_cli::ecr::get_ecr_detail(id)
    }

    pub fn list_auto_scaling_groups() -> Vec<aws_cli::AwsResource> {
        aws_cli::asg::list_auto_scaling_groups()
    }

    pub fn get_asg_detail(name: &str) -> Option<aws_cli::AsgDetail> {
        aws_cli::asg::get_asg_detail(name)
    }
}

#[cfg(test)]
mod aws_adapter {
    use super::VpcInfoTuple;
    use crate::aws_cli;

    fn resource(id: &str, name: &str) -> aws_cli::AwsResource {
        aws_cli::AwsResource {
            name: name.to_string(),
            id: id.to_string(),
            state: "available".to_string(),
            az: "ap-northeast-2a".to_string(),
            cidr: "10.0.0.0/24".to_string(),
        }
    }

    pub fn set_region(_region: &str) {}

    pub fn list_instances() -> Vec<aws_cli::AwsResource> {
        vec![resource("i-test", "ec2-test")]
    }

    pub fn get_instance_detail(id: &str) -> Option<aws_cli::Ec2Detail> {
        Some(aws_cli::Ec2Detail {
            name: format!("ec2-{}", id),
            instance_id: id.to_string(),
            instance_type: "t3.micro".to_string(),
            ami: "ami-test".to_string(),
            platform: "Linux".to_string(),
            architecture: "x86_64".to_string(),
            key_pair: "kp".to_string(),
            vpc: "vpc-test".to_string(),
            subnet: "subnet-test".to_string(),
            az: "ap-northeast-2a".to_string(),
            public_ip: "-".to_string(),
            private_ip: "10.0.0.10".to_string(),
            security_groups: vec!["sg-test".to_string()],
            state: "running".to_string(),
            ebs_optimized: false,
            monitoring: "Disabled".to_string(),
            iam_role: None,
            iam_role_detail: None,
            launch_time: String::new(),
            tags: vec![("Name".to_string(), "ec2-test".to_string())],
            volumes: vec![],
            user_data: None,
        })
    }

    pub fn list_vpcs() -> Vec<aws_cli::AwsResource> {
        vec![resource("vpc-test", "vpc-test")]
    }

    pub fn get_network_detail(vpc_id: &str) -> Option<aws_cli::NetworkDetail> {
        Some(aws_cli::NetworkDetail {
            name: format!("network-{}", vpc_id),
            id: vpc_id.to_string(),
            cidr: "10.0.0.0/16".to_string(),
            state: "available".to_string(),
            subnets: vec![],
            igws: vec![],
            nats: vec![],
            route_tables: vec![],
            eips: vec![],
            dns_support: true,
            dns_hostnames: true,
            tags: vec![],
        })
    }

    pub fn get_vpc_info(vpc_id: &str) -> Option<VpcInfoTuple> {
        Some((
            format!("network-{}", vpc_id),
            "10.0.0.0/16".to_string(),
            "available".to_string(),
            vec![],
        ))
    }

    pub fn list_subnets(_vpc_id: &str) -> Vec<aws_cli::AwsResource> {
        vec![resource("subnet-test", "subnet-test")]
    }

    pub fn list_internet_gateways(_vpc_id: &str) -> Vec<aws_cli::AwsResource> {
        vec![resource("igw-test", "igw-test")]
    }

    pub fn list_nat_gateways(_vpc_id: &str) -> Vec<aws_cli::NatDetail> {
        vec![]
    }

    pub fn list_route_tables(_vpc_id: &str) -> Vec<aws_cli::RouteTableDetail> {
        vec![]
    }

    pub fn list_eips() -> Vec<aws_cli::EipDetail> {
        vec![]
    }

    pub fn get_vpc_dns_support(_vpc_id: &str) -> bool {
        true
    }

    pub fn get_vpc_dns_hostnames(_vpc_id: &str) -> bool {
        false
    }

    pub fn list_security_groups() -> Vec<aws_cli::AwsResource> {
        vec![resource("sg-test", "sg-test")]
    }

    pub fn get_security_group_detail(id: &str) -> Option<aws_cli::SecurityGroupDetail> {
        Some(aws_cli::SecurityGroupDetail {
            name: format!("sg-{}", id),
            id: id.to_string(),
            description: "sg".to_string(),
            vpc_id: "vpc-test".to_string(),
            inbound_rules: vec![],
            outbound_rules: vec![],
        })
    }

    pub fn list_load_balancers() -> Vec<aws_cli::AwsResource> {
        vec![resource("lb-test", "lb-test")]
    }

    pub fn get_load_balancer_detail(id: &str) -> Option<aws_cli::LoadBalancerDetail> {
        Some(aws_cli::LoadBalancerDetail {
            name: format!("lb-{}", id),
            arn: id.to_string(),
            dns_name: "lb.example.com".to_string(),
            lb_type: "application".to_string(),
            scheme: "internal".to_string(),
            vpc_id: "vpc-test".to_string(),
            ip_address_type: "ipv4".to_string(),
            state: "active".to_string(),
            availability_zones: vec![],
            security_groups: vec![],
            listeners: vec![],
            target_groups: vec![],
        })
    }

    pub fn list_ecr_repositories() -> Vec<aws_cli::AwsResource> {
        vec![resource("repo-test", "repo-test")]
    }

    pub fn get_ecr_detail(id: &str) -> Option<aws_cli::EcrDetail> {
        Some(aws_cli::EcrDetail {
            name: id.to_string(),
            uri: "123456789012.dkr.ecr.ap-northeast-2.amazonaws.com/repo-test".to_string(),
            tag_mutability: "MUTABLE".to_string(),
            encryption_type: "AES256".to_string(),
            kms_key: None,
            created_at: "2026-01-01".to_string(),
            image_count: 0,
        })
    }

    pub fn list_auto_scaling_groups() -> Vec<aws_cli::AwsResource> {
        vec![resource("asg-test", "asg-test")]
    }

    pub fn get_asg_detail(name: &str) -> Option<aws_cli::AsgDetail> {
        Some(aws_cli::AsgDetail {
            name: name.to_string(),
            arn: format!("arn:aws:autoscaling:ap-northeast-2:123456789012:autoScalingGroup:{name}"),
            launch_template_name: None,
            launch_template_id: None,
            launch_config_name: None,
            min_size: 1,
            max_size: 1,
            desired_capacity: 1,
            default_cooldown: 0,
            availability_zones: vec![],
            target_group_arns: vec![],
            health_check_type: "EC2".to_string(),
            health_check_grace_period: 0,
            instances: vec![],
            created_time: "2026-01-01T00:00:00Z".to_string(),
            scaling_policies: vec![],
            tags: vec![],
        })
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match &app.screen {
        Screen::Login => handle_login(app, key),
        Screen::BlueprintSelect => handle_blueprint_select(app, key),
        Screen::BlueprintDetail => handle_blueprint_detail(app, key),
        Screen::BlueprintNameInput => handle_blueprint_name_input(app, key),
        Screen::BlueprintPreview => handle_blueprint_preview(app, key),
        Screen::RegionSelect => handle_region_select(app, key),
        Screen::ServiceSelect => handle_service_select(app, key),
        Screen::Ec2Select => handle_ec2_select(app, key),
        Screen::VpcSelect => handle_vpc_select(app, key),
        Screen::SecurityGroupSelect => handle_security_group_select(app, key),
        Screen::LoadBalancerSelect => handle_load_balancer_select(app, key),
        Screen::EcrSelect => handle_ecr_select(app, key),
        Screen::AsgSelect => handle_asg_select(app, key),
        Screen::Preview => handle_preview(app, key),
        Screen::Settings => handle_settings(app, key),
    }
}

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    // 미리보기 화면에서만 마우스 스크롤/드래그 처리
    match &app.screen {
        Screen::Preview | Screen::BlueprintPreview => {
            handle_preview_mouse(app, mouse);
        }
        _ => {}
    }
}

fn handle_preview_mouse(app: &mut App, mouse: MouseEvent) {
    let content_lines = app.preview_content.lines().count() as u16;

    match mouse.kind {
        MouseEventKind::ScrollUp => {
            app.preview_scroll = app.preview_scroll.saturating_sub(3);
        }
        MouseEventKind::ScrollDown => {
            app.preview_scroll = app
                .preview_scroll
                .saturating_add(3)
                .min(content_lines.saturating_sub(1));
        }
        MouseEventKind::Down(_) => {
            app.preview_drag_start = Some((mouse.column, mouse.row));
        }
        MouseEventKind::Up(_) => {
            app.preview_drag_start = None;
        }
        MouseEventKind::Drag(_) => {
            if let Some((_, start_y)) = app.preview_drag_start {
                let current_y = mouse.row;
                if current_y < start_y {
                    // 드래그 위로 -> 스크롤 다운
                    let diff = start_y - current_y;
                    app.preview_scroll = app
                        .preview_scroll
                        .saturating_add(diff)
                        .min(content_lines.saturating_sub(1));
                } else if current_y > start_y {
                    // 드래그 아래로 -> 스크롤 업
                    let diff = current_y - start_y;
                    app.preview_scroll = app.preview_scroll.saturating_sub(diff);
                }
                app.preview_drag_start = Some((mouse.column, mouse.row));
            }
        }
        _ => {}
    }
}

pub fn process_loading(app: &mut App) {
    match app.loading_task.clone() {
        LoadingTask::RefreshEc2 => {
            app.instances = aws_adapter::list_instances();
            app.message = app.i18n.refresh_complete().to_string();
            finish_loading(app);
        }
        LoadingTask::RefreshVpc => {
            app.vpcs = aws_adapter::list_vpcs();
            app.message = app.i18n.refresh_complete().to_string();
            finish_loading(app);
        }
        LoadingTask::RefreshPreview => {
            if app.ec2_detail.is_some() {
                if let Some(new_detail) = aws_adapter::get_instance_detail(
                    app.instances
                        .get(app.selected_index)
                        .map(|i| i.id.as_str())
                        .unwrap_or(""),
                ) {
                    app.preview_content = new_detail.to_markdown(app.settings.language);
                    app.preview_filename = format!("{}.md", new_detail.name);
                    app.ec2_detail = Some(new_detail);
                }
            } else if app.network_detail.is_some() {
                if let Some(new_detail) = aws_adapter::get_network_detail(
                    app.vpcs
                        .get(app.selected_index)
                        .map(|v| v.id.as_str())
                        .unwrap_or(""),
                ) {
                    app.preview_content = new_detail.to_markdown(app.settings.language);
                    app.preview_filename = format!("{}.md", new_detail.name);
                    app.network_detail = Some(new_detail);
                }
            } else if app.sg_detail.is_some()
                && let Some(new_detail) = aws_adapter::get_security_group_detail(
                    app.security_groups
                        .get(app.selected_index)
                        .map(|sg| sg.id.as_str())
                        .unwrap_or(""),
                )
            {
                app.preview_content = new_detail.to_markdown(app.settings.language);
                app.preview_filename = format!("{}.md", new_detail.name);
                app.sg_detail = Some(new_detail);
            } else if app.lb_detail.is_some() {
                if let Some(new_detail) = aws_adapter::get_load_balancer_detail(
                    app.load_balancers
                        .get(app.selected_index)
                        .map(|lb| lb.id.as_str())
                        .unwrap_or(""),
                ) {
                    app.preview_content = new_detail.to_markdown(app.settings.language);
                    app.preview_filename = format!("{}.md", new_detail.name);
                    app.lb_detail = Some(new_detail);
                }
            } else if app.ecr_detail.is_some() {
                if let Some(new_detail) = aws_adapter::get_ecr_detail(
                    app.ecr_repositories
                        .get(app.selected_index)
                        .map(|repo| repo.id.as_str())
                        .unwrap_or(""),
                ) {
                    app.preview_content = new_detail.to_markdown(app.settings.language);
                    app.preview_filename = format!("{}.md", new_detail.name);
                    app.ecr_detail = Some(new_detail);
                }
            } else if app.asg_detail.is_some()
                && let Some(new_detail) = aws_adapter::get_asg_detail(
                    app.auto_scaling_groups
                        .get(app.selected_index)
                        .map(|asg| asg.id.as_str())
                        .unwrap_or(""),
                )
            {
                app.preview_content = new_detail.to_markdown(app.settings.language);
                app.preview_filename = format!("{}.md", new_detail.name);
                app.asg_detail = Some(new_detail);
            }
            app.message = app.i18n.refresh_complete().to_string();
            finish_loading(app);
        }
        LoadingTask::LoadEc2 => {
            app.instances = aws_adapter::list_instances();
            app.selected_index = 0;
            app.screen = Screen::Ec2Select;
            finish_loading(app);
        }
        LoadingTask::LoadEc2Detail(id) => {
            if let Some(detail) = aws_adapter::get_instance_detail(&id) {
                app.preview_content = detail.to_markdown(app.settings.language);
                app.preview_filename = format!("{}.md", detail.name);
                app.ec2_detail = Some(detail);
                app.screen = Screen::Preview;
            }
            finish_loading(app);
        }
        LoadingTask::LoadVpc => {
            app.vpcs = aws_adapter::list_vpcs();
            app.selected_index = 0;
            app.screen = Screen::VpcSelect;
            finish_loading(app);
        }

        LoadingTask::LoadVpcDetail(id, step) => {
            process_vpc_detail_step(app, &id, step);
        }
        LoadingTask::RefreshSecurityGroup => {
            app.security_groups = aws_adapter::list_security_groups();
            app.message = app.i18n.refresh_complete().to_string();
            finish_loading(app);
        }
        LoadingTask::LoadSecurityGroup => {
            app.security_groups = aws_adapter::list_security_groups();
            app.selected_index = 0;
            app.screen = Screen::SecurityGroupSelect;
            finish_loading(app);
        }
        LoadingTask::LoadSecurityGroupDetail(id) => {
            if let Some(detail) = aws_adapter::get_security_group_detail(&id) {
                app.preview_content = detail.to_markdown(app.settings.language);
                app.preview_filename = format!("{}.md", detail.name);
                app.sg_detail = Some(detail);
                app.screen = Screen::Preview;
            }
            finish_loading(app);
        }

        LoadingTask::RefreshLoadBalancer => {
            app.load_balancers = aws_adapter::list_load_balancers();
            app.message = app.i18n.refresh_complete().to_string();
            finish_loading(app);
        }
        LoadingTask::LoadLoadBalancer => {
            app.load_balancers = aws_adapter::list_load_balancers();
            app.selected_index = 0;
            app.screen = Screen::LoadBalancerSelect;
            finish_loading(app);
        }
        LoadingTask::LoadLoadBalancerDetail(id) => {
            if let Some(detail) = aws_adapter::get_load_balancer_detail(&id) {
                app.preview_content = detail.to_markdown(app.settings.language);
                app.preview_filename = format!("{}.md", detail.name);
                app.lb_detail = Some(detail);
                app.screen = Screen::Preview;
            }
            finish_loading(app);
        }

        LoadingTask::RefreshEcr => {
            app.ecr_repositories = aws_adapter::list_ecr_repositories();
            app.message = app.i18n.refresh_complete().to_string();
            finish_loading(app);
        }
        LoadingTask::LoadEcr => {
            app.ecr_repositories = aws_adapter::list_ecr_repositories();
            app.selected_index = 0;
            app.screen = Screen::EcrSelect;
            finish_loading(app);
        }
        LoadingTask::LoadEcrDetail(id) => {
            if let Some(detail) = aws_adapter::get_ecr_detail(&id) {
                app.preview_content = detail.to_markdown(app.settings.language);
                app.preview_filename = format!("{}.md", detail.name);
                app.ecr_detail = Some(detail);
                app.screen = Screen::Preview;
            }
            finish_loading(app);
        }

        LoadingTask::RefreshAsg => {
            app.auto_scaling_groups = aws_adapter::list_auto_scaling_groups();
            app.message = app.i18n.refresh_complete().to_string();
            finish_loading(app);
        }
        LoadingTask::LoadAsg => {
            app.auto_scaling_groups = aws_adapter::list_auto_scaling_groups();
            app.selected_index = 0;
            app.screen = Screen::AsgSelect;
            finish_loading(app);
        }
        LoadingTask::LoadAsgDetail(name) => {
            if let Some(detail) = aws_adapter::get_asg_detail(&name) {
                app.preview_content = detail.to_markdown(app.settings.language);
                app.preview_filename = format!("{}.md", detail.name);
                app.asg_detail = Some(detail);
                app.screen = Screen::Preview;
            }
            finish_loading(app);
        }

        LoadingTask::LoadBlueprintResources(current_index) => {
            process_blueprint_resources(app, current_index);
        }
        LoadingTask::None => {}
    }
}

fn process_blueprint_resources(app: &mut App, current_index: usize) {
    let blueprint = match &app.current_blueprint {
        Some(bp) => bp.clone(),
        None => {
            finish_loading(app);
            return;
        }
    };

    if current_index >= blueprint.resources.len() {
        // All resources loaded, generate table of contents and combine markdown
        let mut toc = vec![format!("## {}\n", app.i18n.toc())];
        for (i, (res, markdown)) in blueprint
            .resources
            .iter()
            .zip(app.blueprint_markdown_parts.iter())
            .enumerate()
        {
            let anchor = format!(
                "{}-{}",
                res.resource_type.display().to_lowercase().replace(" ", "-"),
                res.resource_name.to_lowercase().replace(" ", "-")
            );
            toc.push(format!(
                "- [{}. {} - {}](#{})",
                i + 1,
                res.resource_type.display(),
                res.resource_name,
                anchor
            ));

            // ### 헤더들을 서브 목차로 추가
            for line in markdown.lines() {
                if line.starts_with("### ") {
                    let header = line.trim_start_matches("### ").trim();
                    let sub_anchor = header
                        .to_lowercase()
                        .replace(" ", "-")
                        .replace("(", "")
                        .replace(")", "");
                    toc.push(format!("  - [{}](#{})", header, sub_anchor));
                }
            }
        }
        toc.push("\n".to_string());

        let combined = app.blueprint_markdown_parts.join("\n---\n\n");
        let toc_str = toc.join("\n");
        app.preview_content = format!("# Blueprint: {}\n\n{}{}", blueprint.name, toc_str, combined);
        app.preview_filename = format!("{}.md", blueprint.name);
        app.preview_scroll = 0;
        app.screen = Screen::BlueprintPreview;
        finish_loading(app);
        return;
    }

    let resource = &blueprint.resources[current_index];

    // Set region for this resource
    aws_adapter::set_region(&resource.region);

    // Fetch resource detail and generate markdown
    let failed = app.i18n.query_failed();
    let markdown = match resource.resource_type {
        ResourceType::Ec2 => aws_adapter::get_instance_detail(&resource.resource_id)
            .map(|d| d.to_markdown(app.settings.language))
            .unwrap_or_else(|| {
                format!(
                    "## {}: {} ({})\n",
                    app.i18n.ec2(),
                    resource.resource_name,
                    failed
                )
            }),
        ResourceType::Network => aws_adapter::get_network_detail(&resource.resource_id)
            .map(|d| d.to_markdown(app.settings.language))
            .unwrap_or_else(|| {
                format!(
                    "## {}: {} ({})\n",
                    app.i18n.network(),
                    resource.resource_name,
                    failed
                )
            }),
        ResourceType::SecurityGroup => {
            aws_adapter::get_security_group_detail(&resource.resource_id)
                .map(|d| d.to_markdown(app.settings.language))
                .unwrap_or_else(|| {
                    format!(
                        "## {}: {} ({})\n",
                        app.i18n.security_group(),
                        resource.resource_name,
                        failed
                    )
                })
        }
        ResourceType::LoadBalancer => aws_adapter::get_load_balancer_detail(&resource.resource_id)
            .map(|d| d.to_markdown(app.settings.language))
            .unwrap_or_else(|| {
                format!(
                    "## {}: {} ({})\n",
                    app.i18n.load_balancer(),
                    resource.resource_name,
                    failed
                )
            }),
        ResourceType::Ecr => aws_adapter::get_ecr_detail(&resource.resource_id)
            .map(|d| d.to_markdown(app.settings.language))
            .unwrap_or_else(|| {
                format!(
                    "## {}: {} ({})\n",
                    app.i18n.md_ecr_repository(),
                    resource.resource_name,
                    failed
                )
            }),
        ResourceType::Asg => aws_adapter::get_asg_detail(&resource.resource_id)
            .map(|d| d.to_markdown(app.settings.language))
            .unwrap_or_else(|| {
                format!(
                    "## {}: {} ({})\n",
                    app.i18n.auto_scaling_group(),
                    resource.resource_name,
                    failed
                )
            }),
    };

    app.blueprint_markdown_parts.push(markdown);

    // Move to next resource
    app.loading_task = LoadingTask::LoadBlueprintResources(current_index + 1);
}

fn process_vpc_detail_step(app: &mut App, vpc_id: &str, step: u8) {
    if step > 0 && app.network_detail.is_none() {
        tracing::warn!(
            vpc_id,
            step,
            "Network detail aborting because VPC info is missing"
        );
        app.message = app.i18n.network_detail_unavailable(vpc_id);
        finish_loading(app);
        return;
    }

    match step {
        0 => {
            // Step 0: VPC 기본 정보
            if let Some(info) = aws_adapter::get_vpc_info(vpc_id) {
                tracing::info!(
                    vpc_id,
                    cidr = %info.1,
                    state = %info.2,
                    tag_count = info.3.len(),
                    "Network detail step 0 loaded VPC info"
                );
                app.network_detail = Some(NetworkDetail {
                    name: info.0,
                    id: vpc_id.to_string(),
                    cidr: info.1,
                    state: info.2,
                    tags: info.3,
                    subnets: Vec::new(),
                    igws: Vec::new(),
                    nats: Vec::new(),
                    route_tables: Vec::new(),
                    eips: Vec::new(),
                    dns_support: false,
                    dns_hostnames: false,
                });
                app.loading_progress.vpc_info = true;
            } else {
                tracing::warn!(vpc_id, "Network detail step 0 failed to load VPC info");
                app.message = app.i18n.network_detail_unavailable(vpc_id);
                app.network_detail = None;
                finish_loading(app);
                return;
            }
            app.loading_task = LoadingTask::LoadVpcDetail(vpc_id.to_string(), 1);
        }
        1 => {
            // Step 1: Subnets
            let subnets = aws_adapter::list_subnets(vpc_id);
            tracing::info!(
                vpc_id,
                subnet_count = subnets.len(),
                "Network detail step 1 loaded subnets"
            );
            if subnets.is_empty() {
                tracing::warn!(vpc_id, "Network detail step 1 returned empty subnet list");
            }
            if let Some(ref mut detail) = app.network_detail {
                detail.subnets = subnets;
            }
            app.loading_progress.subnets = true;
            app.loading_task = LoadingTask::LoadVpcDetail(vpc_id.to_string(), 2);
        }
        2 => {
            // Step 2: Internet Gateways
            let igws = aws_adapter::list_internet_gateways(vpc_id);
            tracing::info!(
                vpc_id,
                igw_count = igws.len(),
                "Network detail step 2 loaded internet gateways"
            );
            if igws.is_empty() {
                tracing::warn!(
                    vpc_id,
                    "Network detail step 2 returned empty internet gateway list"
                );
            }
            if let Some(ref mut detail) = app.network_detail {
                detail.igws = igws;
            }
            app.loading_progress.igws = true;
            app.loading_task = LoadingTask::LoadVpcDetail(vpc_id.to_string(), 3);
        }
        3 => {
            // Step 3: NAT Gateways
            let nats = aws_adapter::list_nat_gateways(vpc_id);
            tracing::info!(
                vpc_id,
                nat_count = nats.len(),
                "Network detail step 3 loaded NAT gateways"
            );
            if nats.is_empty() {
                tracing::warn!(
                    vpc_id,
                    "Network detail step 3 returned empty NAT gateway list"
                );
            }
            if let Some(ref mut detail) = app.network_detail {
                detail.nats = nats;
            }
            app.loading_progress.nats = true;
            app.loading_task = LoadingTask::LoadVpcDetail(vpc_id.to_string(), 4);
        }
        4 => {
            // Step 4: Route Tables
            let route_tables = aws_adapter::list_route_tables(vpc_id);
            tracing::info!(
                vpc_id,
                route_table_count = route_tables.len(),
                "Network detail step 4 loaded route tables"
            );
            if route_tables.is_empty() {
                tracing::warn!(
                    vpc_id,
                    "Network detail step 4 returned empty route table list"
                );
            }
            if let Some(ref mut detail) = app.network_detail {
                detail.route_tables = route_tables;
            }
            app.loading_progress.route_tables = true;
            app.loading_task = LoadingTask::LoadVpcDetail(vpc_id.to_string(), 5);
        }
        5 => {
            // Step 5: Elastic IPs
            let eips = aws_adapter::list_eips();
            tracing::info!(
                vpc_id,
                eip_count = eips.len(),
                "Network detail step 5 loaded EIPs"
            );
            if eips.is_empty() {
                tracing::warn!(vpc_id, "Network detail step 5 returned empty EIP list");
            }
            if let Some(ref mut detail) = app.network_detail {
                detail.eips = eips;
            }
            app.loading_progress.eips = true;
            app.loading_task = LoadingTask::LoadVpcDetail(vpc_id.to_string(), 6);
        }
        6 => {
            // Step 6: DNS Attributes
            tracing::debug!(vpc_id, "Network detail step 6 loading DNS attributes");
            if let Some(ref mut detail) = app.network_detail {
                detail.dns_support = aws_adapter::get_vpc_dns_support(vpc_id);
                detail.dns_hostnames = aws_adapter::get_vpc_dns_hostnames(vpc_id);
                tracing::info!(
                    vpc_id,
                    dns_support = detail.dns_support,
                    dns_hostnames = detail.dns_hostnames,
                    "Network detail step 6 loaded DNS attributes"
                );
            }
            app.loading_progress.dns_attrs = true;
            app.loading_task = LoadingTask::LoadVpcDetail(vpc_id.to_string(), 7);
        }
        _ => {
            // 완료: Preview 화면으로 전환
            if let Some(ref detail) = app.network_detail {
                app.preview_content = detail.to_markdown(app.settings.language);
                app.preview_filename = format!("{}.md", detail.name);
            }
            app.screen = Screen::Preview;
            finish_loading(app);
        }
    }
}

fn finish_loading(app: &mut App) {
    app.loading = false;
    app.loading_task = LoadingTask::None;
    app.loading_progress.reset();
}

fn start_loading(app: &mut App, task: LoadingTask) {
    app.loading = true;
    app.loading_progress.reset();
    app.loading_task = task;
    app.message = app.i18n.loading_msg().to_string();
}

fn handle_login(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_profile_index > 0 {
                app.selected_profile_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_profile_index + 1 < app.available_profiles.len() {
                app.selected_profile_index += 1;
            }
        }
        KeyCode::Enter => {
            app.select_current_profile_and_login();
        }
        KeyCode::Char('r') => {
            app.refresh_profiles();
        }
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn handle_blueprint_select(app: &mut App, key: KeyEvent) {
    let list_len = app.blueprint_store.blueprints.len() + 1; // +1 for "새 블루프린터"

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_blueprint_index > 0 {
                app.selected_blueprint_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_blueprint_index < list_len - 1 {
                app.selected_blueprint_index += 1;
            }
        }
        KeyCode::Enter => {
            if app.selected_blueprint_index == 0 {
                // "새 블루프린터" 선택
                app.input_buffer.clear();
                app.screen = Screen::BlueprintNameInput;
            } else {
                // 기존 블루프린터 선택
                let bp_index = app.selected_blueprint_index - 1;
                if let Some(bp) = app.blueprint_store.get_blueprint(bp_index) {
                    app.current_blueprint = Some(bp.clone());
                    app.selected_blueprint_index = bp_index;
                    app.blueprint_resource_index = 0;
                    app.screen = Screen::BlueprintDetail;
                }
            }
        }
        KeyCode::Char('g') => {
            // 선택된 블루프린터의 마크다운 생성
            if app.selected_blueprint_index > 0 {
                let bp_index = app.selected_blueprint_index - 1;
                if let Some(bp) = app.blueprint_store.get_blueprint(bp_index) {
                    if !bp.resources.is_empty() {
                        app.current_blueprint = Some(bp.clone());
                        app.selected_blueprint_index = bp_index;
                        app.blueprint_markdown_parts.clear();
                        start_loading(app, LoadingTask::LoadBlueprintResources(0));
                    } else {
                        app.message = app.i18n.no_resources().to_string();
                    }
                }
            }
        }
        KeyCode::Char('d') => {
            // 블루프린터 삭제
            if app.selected_blueprint_index > 0 {
                let bp_index = app.selected_blueprint_index - 1;
                app.delete_blueprint(bp_index);
                if app.selected_blueprint_index > 0 {
                    app.selected_blueprint_index = 1.min(app.blueprint_store.blueprints.len());
                }
            }
        }
        KeyCode::Char('s') => {
            // 단일 리소스 모드로 전환 (리전 선택)
            app.blueprint_mode = false;
            app.screen = Screen::RegionSelect;
        }
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
            // Switch to Settings tab
            app.selected_tab = 1;
            app.selected_setting = 0;
            app.screen = Screen::Settings;
        }
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn handle_blueprint_detail(app: &mut App, key: KeyEvent) {
    let resource_len = app
        .current_blueprint
        .as_ref()
        .map(|bp| bp.resources.len())
        .unwrap_or(0);

    match key.code {
        KeyCode::Up | KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            // Shift+Up/K: 리소스 위로 이동
            if app.move_resource_up(app.blueprint_resource_index) {
                app.blueprint_resource_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            // Shift+Down/J: 리소스 아래로 이동
            if app.move_resource_down(app.blueprint_resource_index) {
                app.blueprint_resource_index += 1;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.blueprint_resource_index > 0 {
                app.blueprint_resource_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.blueprint_resource_index < resource_len.saturating_sub(1) {
                app.blueprint_resource_index += 1;
            }
        }
        KeyCode::Char('a') => {
            // 리소스 추가 (리전 선택으로 이동)
            app.blueprint_mode = true;
            app.screen = Screen::RegionSelect;
        }
        KeyCode::Char('d') => {
            // 리소스 삭제
            if resource_len > 0 {
                app.remove_resource_from_current_blueprint(app.blueprint_resource_index);
                if app.blueprint_resource_index >= resource_len.saturating_sub(1)
                    && app.blueprint_resource_index > 0
                {
                    app.blueprint_resource_index -= 1;
                }
            }
        }
        KeyCode::Enter | KeyCode::Char('g') => {
            // 마크다운 생성
            if let Some(ref bp) = app.current_blueprint {
                if !bp.resources.is_empty() {
                    app.blueprint_markdown_parts.clear();
                    start_loading(app, LoadingTask::LoadBlueprintResources(0));
                } else {
                    app.message = app.i18n.no_resources().to_string();
                }
            }
        }
        KeyCode::Esc => {
            app.current_blueprint = None;
            app.screen = Screen::BlueprintSelect;
        }
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn handle_blueprint_name_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if !app.input_buffer.trim().is_empty() {
                let name = app.input_buffer.trim().to_string();
                app.create_blueprint(name);
                // 새로 생성된 블루프린터 선택
                let bp_index = app.blueprint_store.blueprints.len() - 1;
                if let Some(bp) = app.blueprint_store.get_blueprint(bp_index) {
                    app.current_blueprint = Some(bp.clone());
                    app.selected_blueprint_index = bp_index;
                    app.blueprint_resource_index = 0;
                    app.screen = Screen::BlueprintDetail;
                }
            }
            app.input_buffer.clear();
        }
        KeyCode::Esc => {
            app.input_buffer.clear();
            app.screen = Screen::BlueprintSelect;
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
        }
        _ => {}
    }
}

fn handle_blueprint_preview(app: &mut App, key: KeyEvent) {
    let content_lines = app.preview_content.lines().count() as u16;

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.preview_scroll = app.preview_scroll.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.preview_scroll = app
                .preview_scroll
                .saturating_add(1)
                .min(content_lines.saturating_sub(1));
        }
        KeyCode::PageUp => {
            app.preview_scroll = app.preview_scroll.saturating_sub(20);
        }
        KeyCode::PageDown => {
            app.preview_scroll = app
                .preview_scroll
                .saturating_add(20)
                .min(content_lines.saturating_sub(1));
        }
        KeyCode::Home => {
            app.preview_scroll = 0;
        }
        KeyCode::End => {
            app.preview_scroll = content_lines.saturating_sub(1);
        }
        KeyCode::Enter | KeyCode::Char('s') => {
            let _ = app.save_file();
        }
        KeyCode::Esc => {
            app.preview_scroll = 0;
            if app.current_blueprint.is_some() {
                // 블루프린터 상세로 돌아가기
                app.screen = Screen::BlueprintDetail;
            } else {
                app.screen = Screen::BlueprintSelect;
            }
        }
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn handle_region_select(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_region > 0 {
                app.selected_region -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_region < REGIONS.len() - 1 {
                app.selected_region += 1;
            }
        }
        KeyCode::Enter => app.select_region(),
        KeyCode::Esc => {
            if app.blueprint_mode {
                app.screen = Screen::BlueprintDetail;
            } else {
                app.screen = Screen::BlueprintSelect;
            }
        }
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn handle_service_select(app: &mut App, key: KeyEvent) {
    let total_items = SERVICE_KEYS.len() + 1; // +1 for exit
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_service > 0 {
                app.selected_service -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_service < total_items - 1 {
                app.selected_service += 1;
            }
        }
        KeyCode::Enter => match app.selected_service {
            0 => start_loading(app, LoadingTask::LoadEc2),
            1 => start_loading(app, LoadingTask::LoadVpc),
            2 => start_loading(app, LoadingTask::LoadSecurityGroup),
            3 => start_loading(app, LoadingTask::LoadLoadBalancer),
            4 => start_loading(app, LoadingTask::LoadEcr),
            5 => start_loading(app, LoadingTask::LoadAsg),
            6 => {
                // Exit
                if app.blueprint_mode {
                    app.screen = Screen::BlueprintDetail;
                } else {
                    app.running = false;
                }
            }
            _ => {}
        },
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
            // Switch to Settings tab
            app.selected_tab = 1;
            app.selected_setting = 0;
            app.screen = Screen::Settings;
        }
        KeyCode::Char('q') => app.running = false,
        KeyCode::Esc => app.screen = Screen::RegionSelect,
        _ => {}
    }
}

fn handle_ec2_select(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_index < app.instances.len().saturating_sub(1) {
                app.selected_index += 1;
            }
        }
        KeyCode::Enter => {
            if app.selected_index < app.instances.len() {
                let inst = &app.instances[app.selected_index];
                if app.blueprint_mode {
                    add_resource_to_blueprint(
                        app,
                        ResourceType::Ec2,
                        inst.id.clone(),
                        inst.name.clone(),
                    );
                } else {
                    start_loading(app, LoadingTask::LoadEc2Detail(inst.id.clone()));
                }
            }
        }
        KeyCode::Char('r') => {
            start_loading(app, LoadingTask::RefreshEc2);
        }
        KeyCode::Esc => app.screen = Screen::ServiceSelect,
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn handle_vpc_select(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_index < app.vpcs.len().saturating_sub(1) {
                app.selected_index += 1;
            }
        }
        KeyCode::Enter => {
            if app.selected_index < app.vpcs.len() {
                let vpc = &app.vpcs[app.selected_index];
                if app.blueprint_mode {
                    add_resource_to_blueprint(
                        app,
                        ResourceType::Network,
                        vpc.id.clone(),
                        vpc.name.clone(),
                    );
                } else {
                    start_loading(app, LoadingTask::LoadVpcDetail(vpc.id.clone(), 0));
                }
            }
        }
        KeyCode::Char('r') => {
            start_loading(app, LoadingTask::RefreshVpc);
        }
        KeyCode::Esc => app.screen = Screen::ServiceSelect,
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn handle_security_group_select(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_index < app.security_groups.len().saturating_sub(1) {
                app.selected_index += 1;
            }
        }
        KeyCode::Enter => {
            if app.selected_index < app.security_groups.len() {
                let sg = &app.security_groups[app.selected_index];
                if app.blueprint_mode {
                    add_resource_to_blueprint(
                        app,
                        ResourceType::SecurityGroup,
                        sg.id.clone(),
                        sg.name.clone(),
                    );
                } else {
                    start_loading(app, LoadingTask::LoadSecurityGroupDetail(sg.id.clone()));
                }
            }
        }
        KeyCode::Char('r') => {
            start_loading(app, LoadingTask::RefreshSecurityGroup);
        }
        KeyCode::Esc => app.screen = Screen::ServiceSelect,
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn handle_load_balancer_select(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_index < app.load_balancers.len().saturating_sub(1) {
                app.selected_index += 1;
            }
        }
        KeyCode::Enter => {
            if app.selected_index < app.load_balancers.len() {
                let lb = &app.load_balancers[app.selected_index];
                if app.blueprint_mode {
                    add_resource_to_blueprint(
                        app,
                        ResourceType::LoadBalancer,
                        lb.id.clone(),
                        lb.name.clone(),
                    );
                } else {
                    start_loading(app, LoadingTask::LoadLoadBalancerDetail(lb.id.clone()));
                }
            }
        }
        KeyCode::Char('r') => {
            start_loading(app, LoadingTask::RefreshLoadBalancer);
        }
        KeyCode::Esc => app.screen = Screen::ServiceSelect,
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn handle_ecr_select(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_index < app.ecr_repositories.len().saturating_sub(1) {
                app.selected_index += 1;
            }
        }
        KeyCode::Enter => {
            if app.selected_index < app.ecr_repositories.len() {
                let repo = &app.ecr_repositories[app.selected_index];
                if app.blueprint_mode {
                    add_resource_to_blueprint(
                        app,
                        ResourceType::Ecr,
                        repo.id.clone(),
                        repo.name.clone(),
                    );
                } else {
                    start_loading(app, LoadingTask::LoadEcrDetail(repo.id.clone()));
                }
            }
        }
        KeyCode::Char('r') => {
            start_loading(app, LoadingTask::RefreshEcr);
        }
        KeyCode::Esc => app.screen = Screen::ServiceSelect,
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn handle_preview(app: &mut App, key: KeyEvent) {
    let content_lines = app.preview_content.lines().count() as u16;

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.preview_scroll = app.preview_scroll.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.preview_scroll = app
                .preview_scroll
                .saturating_add(1)
                .min(content_lines.saturating_sub(1));
        }
        KeyCode::PageUp => {
            app.preview_scroll = app.preview_scroll.saturating_sub(20);
        }
        KeyCode::PageDown => {
            app.preview_scroll = app
                .preview_scroll
                .saturating_add(20)
                .min(content_lines.saturating_sub(1));
        }
        KeyCode::Home => {
            app.preview_scroll = 0;
        }
        KeyCode::End => {
            app.preview_scroll = content_lines.saturating_sub(1);
        }
        KeyCode::Enter | KeyCode::Char('s') => {
            let _ = app.save_file();
        }
        KeyCode::Char('r') => {
            start_loading(app, LoadingTask::RefreshPreview);
        }
        KeyCode::Char('a') => {
            // 블루프린트 모드일 때만 리소스 추가
            if app.blueprint_mode
                && let (Some(resource_type), Some((resource_id, resource_name))) = (
                    app.get_current_resource_type(),
                    app.get_current_resource_info(),
                )
            {
                let region = app.get_current_region();
                let resource = BlueprintResource {
                    resource_type,
                    region,
                    resource_id,
                    resource_name,
                };
                app.add_resource_to_current_blueprint(resource);

                // 리소스 정리 후 블루프린트 상세로 돌아가기
                app.ec2_detail = None;
                app.network_detail = None;
                app.sg_detail = None;
                app.lb_detail = None;
                app.ecr_detail = None;
                app.asg_detail = None;
                app.preview_scroll = 0;
                app.screen = Screen::BlueprintDetail;
            }
        }
        KeyCode::Esc => {
            app.preview_scroll = 0;
            if app.blueprint_mode {
                // 블루프린트 모드: 블루프린트 상세로 돌아가기
                app.ec2_detail = None;
                app.network_detail = None;
                app.sg_detail = None;
                app.lb_detail = None;
                app.ecr_detail = None;
                app.asg_detail = None;
                app.screen = Screen::BlueprintDetail;
            } else if app.ec2_detail.is_some() {
                app.ec2_detail = None;
                app.screen = Screen::Ec2Select;
            } else if app.network_detail.is_some() {
                app.network_detail = None;
                app.screen = Screen::VpcSelect;
            } else if app.sg_detail.is_some() {
                app.sg_detail = None;
                app.screen = Screen::SecurityGroupSelect;
            } else if app.lb_detail.is_some() {
                app.lb_detail = None;
                app.screen = Screen::LoadBalancerSelect;
            } else if app.ecr_detail.is_some() {
                app.ecr_detail = None;
                app.screen = Screen::EcrSelect;
            } else if app.asg_detail.is_some() {
                app.asg_detail = None;
                app.screen = Screen::AsgSelect;
            } else {
                app.screen = Screen::ServiceSelect;
            }
        }
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn handle_settings(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_setting > 0 {
                app.selected_setting -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            // Currently only 1 setting (language), so max index is 0
            // Will be expanded when more settings are added
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            // Toggle current setting
            if app.selected_setting == 0 {
                app.toggle_language();
            }
        }
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Tab | KeyCode::Esc => {
            // Switch back to Main tab
            app.selected_tab = 0;
            app.screen = Screen::BlueprintSelect;
        }
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn handle_asg_select(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_index < app.auto_scaling_groups.len().saturating_sub(1) {
                app.selected_index += 1;
            }
        }
        KeyCode::Enter => {
            if app.selected_index < app.auto_scaling_groups.len() {
                let asg = &app.auto_scaling_groups[app.selected_index];
                if app.blueprint_mode {
                    add_resource_to_blueprint(
                        app,
                        ResourceType::Asg,
                        asg.id.clone(),
                        asg.name.clone(),
                    );
                } else {
                    start_loading(app, LoadingTask::LoadAsgDetail(asg.name.clone()));
                }
            }
        }
        KeyCode::Char('r') => {
            start_loading(app, LoadingTask::RefreshAsg);
        }
        KeyCode::Esc => app.screen = Screen::ServiceSelect,
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn add_resource_to_blueprint(
    app: &mut App,
    resource_type: ResourceType,
    resource_id: String,
    resource_name: String,
) {
    if app.current_blueprint.is_some() {
        let region = app.get_current_region();
        let resource = BlueprintResource {
            resource_type,
            region,
            resource_id,
            resource_name,
        };
        app.add_resource_to_current_blueprint(resource);
    }
}

#[cfg(test)]
mod tests {
    use super::{handle_key, handle_mouse, process_loading};
    use crate::app::{App, LoadingTask, Screen};
    use crate::aws_cli::{
        AsgDetail, AwsResource, Ec2Detail, EcrDetail, LoadBalancerDetail, NetworkDetail,
        SecurityGroupDetail,
    };
    use crossterm::event::{
        KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn key_with_mod(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    fn sample_resource(id: &str, name: &str) -> AwsResource {
        AwsResource {
            name: name.to_string(),
            id: id.to_string(),
            state: "running".to_string(),
            az: "ap-northeast-2a".to_string(),
            cidr: "10.0.0.0/24".to_string(),
        }
    }

    #[test]
    fn service_select_enter_sets_loading_tasks() {
        let mut app = App::new();
        app.screen = Screen::ServiceSelect;

        app.selected_service = 0;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.loading_task, LoadingTask::LoadEc2);

        app.selected_service = 1;
        app.loading = false;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.loading_task, LoadingTask::LoadVpc);

        app.selected_service = 5;
        app.loading = false;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.loading_task, LoadingTask::LoadAsg);
    }

    #[test]
    fn region_select_navigation_and_escape_work() {
        let mut app = App::new();
        app.screen = Screen::RegionSelect;
        app.selected_region = 0;

        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected_region, 1);

        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected_region, 0);

        app.blueprint_mode = false;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::BlueprintSelect);
    }

    #[test]
    fn resource_select_enter_starts_detail_loading() {
        let mut app = App::new();

        app.screen = Screen::Ec2Select;
        app.instances = vec![sample_resource("i-1111", "web-1")];
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(
            app.loading_task,
            LoadingTask::LoadEc2Detail("i-1111".to_string())
        );

        app.screen = Screen::EcrSelect;
        app.loading = false;
        app.ecr_repositories = vec![sample_resource("repo-a", "repo-a")];
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(
            app.loading_task,
            LoadingTask::LoadEcrDetail("repo-a".to_string())
        );

        app.screen = Screen::AsgSelect;
        app.loading = false;
        app.auto_scaling_groups = vec![sample_resource("asg-a", "asg-a")];
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(
            app.loading_task,
            LoadingTask::LoadAsgDetail("asg-a".to_string())
        );
    }

    #[test]
    fn preview_keys_update_scroll_and_escape_route() {
        let mut app = App::new();
        app.screen = Screen::Preview;
        app.preview_content = (0..100)
            .map(|n| format!("line {}", n))
            .collect::<Vec<_>>()
            .join("\n");

        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.preview_scroll, 1);
        handle_key(&mut app, key(KeyCode::PageDown));
        assert!(app.preview_scroll >= 20);
        handle_key(&mut app, key(KeyCode::Home));
        assert_eq!(app.preview_scroll, 0);
        handle_key(&mut app, key(KeyCode::End));
        assert_eq!(app.preview_scroll, 99);
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::ServiceSelect);
    }

    #[test]
    fn preview_mouse_scroll_and_drag_work() {
        let mut app = App::new();
        app.screen = Screen::Preview;
        app.preview_content = (0..30)
            .map(|n| format!("line {}", n))
            .collect::<Vec<_>>()
            .join("\n");

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 0,
                row: 0,
                modifiers: KeyModifiers::NONE,
            },
        );
        assert!(app.preview_scroll > 0);

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 10,
                row: 10,
                modifiers: KeyModifiers::NONE,
            },
        );
        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                column: 10,
                row: 8,
                modifiers: KeyModifiers::NONE,
            },
        );
        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column: 10,
                row: 8,
                modifiers: KeyModifiers::NONE,
            },
        );
        assert!(app.preview_drag_start.is_none());
    }

    #[test]
    fn blueprint_name_input_and_settings_paths_work() {
        let mut app = App::new();
        app.screen = Screen::BlueprintNameInput;
        handle_key(&mut app, key(KeyCode::Char('a')));
        handle_key(&mut app, key(KeyCode::Char('b')));
        assert_eq!(app.input_buffer, "ab");
        handle_key(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.input_buffer, "a");
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::BlueprintSelect);

        app.screen = Screen::ServiceSelect;
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.screen, Screen::Settings);

        app.screen = Screen::Settings;
        let before = app.settings.language;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_ne!(app.settings.language, before);
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::BlueprintSelect);
    }

    #[test]
    fn blueprint_detail_shift_reorder_shortcuts_are_handled() {
        let mut app = App::new();
        app.screen = Screen::BlueprintDetail;
        app.current_blueprint = Some(crate::blueprint::Blueprint {
            id: "bp".to_string(),
            name: "bp".to_string(),
            resources: vec![
                crate::blueprint::BlueprintResource {
                    resource_type: crate::blueprint::ResourceType::Ec2,
                    region: "ap-northeast-2".to_string(),
                    resource_id: "i-1".to_string(),
                    resource_name: "one".to_string(),
                },
                crate::blueprint::BlueprintResource {
                    resource_type: crate::blueprint::ResourceType::Ec2,
                    region: "ap-northeast-2".to_string(),
                    resource_id: "i-2".to_string(),
                    resource_name: "two".to_string(),
                },
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
        app.blueprint_resource_index = 1;

        handle_key(&mut app, key_with_mod(KeyCode::Up, KeyModifiers::SHIFT));
        assert_eq!(app.blueprint_resource_index, 0);
    }

    #[test]
    fn process_loading_none_is_noop() {
        let mut app = App::new();
        app.loading = true;
        app.loading_task = LoadingTask::None;
        process_loading(&mut app);
        assert!(app.loading);
        assert_eq!(app.loading_task, LoadingTask::None);
    }

    #[test]
    fn preview_add_path_in_blueprint_mode_returns_to_blueprint_detail() {
        let mut app = App::new();
        app.screen = Screen::Preview;
        app.blueprint_mode = true;
        app.current_blueprint = Some(crate::blueprint::Blueprint {
            id: "bp".to_string(),
            name: "bp".to_string(),
            resources: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
        app.blueprint_store.blueprints = vec![app.current_blueprint.clone().expect("bp")];
        app.selected_blueprint_index = 0;
        app.ec2_detail = Some(Ec2Detail {
            name: "web".to_string(),
            instance_id: "i-1234".to_string(),
            instance_type: "t3.micro".to_string(),
            ami: "ami".to_string(),
            platform: "Linux".to_string(),
            architecture: "x86_64".to_string(),
            key_pair: "key".to_string(),
            vpc: "vpc".to_string(),
            subnet: "subnet".to_string(),
            az: "az".to_string(),
            public_ip: "-".to_string(),
            private_ip: "10.0.0.1".to_string(),
            security_groups: vec!["sg".to_string()],
            state: "running".to_string(),
            ebs_optimized: false,
            monitoring: "Disabled".to_string(),
            iam_role: None,
            iam_role_detail: None,
            launch_time: String::new(),
            tags: vec![("Name".to_string(), "web".to_string())],
            volumes: Vec::new(),
            user_data: None,
        });

        handle_key(&mut app, key(KeyCode::Char('a')));
        assert_eq!(app.screen, Screen::BlueprintDetail);
    }

    #[test]
    fn preview_escape_selects_resource_screen_for_each_detail_type() {
        let mut app = App::new();
        app.screen = Screen::Preview;

        app.ec2_detail = Some(Ec2Detail {
            name: "web".to_string(),
            instance_id: "i-1234".to_string(),
            instance_type: "t3.micro".to_string(),
            ami: "ami".to_string(),
            platform: "Linux".to_string(),
            architecture: "x86_64".to_string(),
            key_pair: "key".to_string(),
            vpc: "vpc".to_string(),
            subnet: "subnet".to_string(),
            az: "az".to_string(),
            public_ip: "-".to_string(),
            private_ip: "10.0.0.1".to_string(),
            security_groups: vec!["sg".to_string()],
            state: "running".to_string(),
            ebs_optimized: false,
            monitoring: "Disabled".to_string(),
            iam_role: None,
            iam_role_detail: None,
            launch_time: String::new(),
            tags: vec![("Name".to_string(), "web".to_string())],
            volumes: Vec::new(),
            user_data: None,
        });
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Ec2Select);

        app.screen = Screen::Preview;
        app.ecr_detail = Some(EcrDetail {
            name: "repo".to_string(),
            uri: "uri".to_string(),
            tag_mutability: "MUTABLE".to_string(),
            encryption_type: "AES256".to_string(),
            kms_key: None,
            created_at: "2026-01-01".to_string(),
            image_count: 0,
        });
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::EcrSelect);

        app.screen = Screen::Preview;
        app.asg_detail = Some(AsgDetail {
            name: "asg".to_string(),
            arn: "arn".to_string(),
            launch_template_name: None,
            launch_template_id: None,
            launch_config_name: None,
            min_size: 1,
            max_size: 1,
            desired_capacity: 1,
            default_cooldown: 0,
            availability_zones: Vec::new(),
            target_group_arns: Vec::new(),
            health_check_type: "EC2".to_string(),
            health_check_grace_period: 0,
            instances: Vec::new(),
            created_time: "2026-01-01".to_string(),
            scaling_policies: Vec::new(),
            tags: Vec::new(),
        });
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::AsgSelect);
    }

    #[test]
    fn blueprint_select_shortcuts_change_mode_and_tab() {
        let mut app = App::new();
        app.screen = Screen::BlueprintSelect;
        app.blueprint_store.blueprints = vec![crate::blueprint::Blueprint {
            id: "bp-1".to_string(),
            name: "bp-1".to_string(),
            resources: vec![crate::blueprint::BlueprintResource {
                resource_type: crate::blueprint::ResourceType::Ec2,
                region: "ap-northeast-2".to_string(),
                resource_id: "i-1234".to_string(),
                resource_name: "web".to_string(),
            }],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }];

        app.selected_blueprint_index = 1;
        handle_key(&mut app, key(KeyCode::Char('g')));
        assert_eq!(app.loading_task, LoadingTask::LoadBlueprintResources(0));

        app.loading = false;
        app.selected_blueprint_index = 1;
        handle_key(&mut app, key(KeyCode::Char('d')));
        assert!(app.selected_blueprint_index <= 1);

        handle_key(&mut app, key(KeyCode::Char('s')));
        assert_eq!(app.screen, Screen::RegionSelect);

        app.screen = Screen::BlueprintSelect;
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.screen, Screen::Settings);
    }

    #[test]
    fn blueprint_detail_actions_cover_add_delete_generate_and_escape() {
        let mut app = App::new();
        app.screen = Screen::BlueprintDetail;
        app.current_blueprint = Some(crate::blueprint::Blueprint {
            id: "bp".to_string(),
            name: "bp".to_string(),
            resources: vec![crate::blueprint::BlueprintResource {
                resource_type: crate::blueprint::ResourceType::Ec2,
                region: "ap-northeast-2".to_string(),
                resource_id: "i-1234".to_string(),
                resource_name: "web".to_string(),
            }],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });

        handle_key(&mut app, key(KeyCode::Char('a')));
        assert_eq!(app.screen, Screen::RegionSelect);
        assert!(app.blueprint_mode);

        app.screen = Screen::BlueprintDetail;
        app.blueprint_mode = false;
        app.blueprint_resource_index = 0;
        handle_key(&mut app, key(KeyCode::Char('d')));
        assert_eq!(
            app.current_blueprint
                .as_ref()
                .map(|bp| bp.resources.len())
                .unwrap_or_default(),
            0
        );

        app.current_blueprint = Some(crate::blueprint::Blueprint {
            id: "bp".to_string(),
            name: "bp".to_string(),
            resources: vec![crate::blueprint::BlueprintResource {
                resource_type: crate::blueprint::ResourceType::Ec2,
                region: "ap-northeast-2".to_string(),
                resource_id: "i-1234".to_string(),
                resource_name: "web".to_string(),
            }],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.loading_task, LoadingTask::LoadBlueprintResources(0));

        app.current_blueprint = Some(crate::blueprint::Blueprint {
            id: "bp".to_string(),
            name: "bp".to_string(),
            resources: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
        app.loading_task = LoadingTask::None;
        handle_key(&mut app, key(KeyCode::Char('g')));
        assert_eq!(app.message, app.i18n.no_resources());
        assert_eq!(app.loading_task, LoadingTask::None);

        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::BlueprintSelect);
    }

    #[test]
    fn service_select_exit_paths_cover_blueprint_and_running_flags() {
        let mut app = App::new();
        app.screen = Screen::ServiceSelect;
        app.selected_service = crate::app::SERVICE_KEYS.len();
        app.blueprint_mode = true;

        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::BlueprintDetail);

        app.screen = Screen::ServiceSelect;
        app.selected_service = crate::app::SERVICE_KEYS.len();
        app.blueprint_mode = false;
        app.running = true;
        handle_key(&mut app, key(KeyCode::Enter));
        assert!(!app.running);
    }

    fn sample_blueprint(name: &str) -> crate::blueprint::Blueprint {
        crate::blueprint::Blueprint {
            id: format!("id-{}", name),
            name: name.to_string(),
            resources: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn blueprint_name_input_enter_creates_blueprint_and_opens_detail() {
        let mut app = App::new();
        app.screen = Screen::BlueprintNameInput;
        app.input_buffer = "new-bp".to_string();

        handle_key(&mut app, key(KeyCode::Enter));

        assert_eq!(app.screen, Screen::BlueprintDetail);
        assert_eq!(
            app.current_blueprint
                .as_ref()
                .map(|bp| bp.name.clone())
                .unwrap_or_default(),
            "new-bp"
        );
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn blueprint_preview_keys_cover_navigation_save_and_escape_paths() {
        let mut app = App::new();
        app.screen = Screen::BlueprintPreview;
        app.current_blueprint = Some(sample_blueprint("bp-a"));
        app.preview_content = (0..50)
            .map(|n| format!("line {}", n))
            .collect::<Vec<_>>()
            .join("\n");

        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.preview_scroll, 1);
        handle_key(&mut app, key(KeyCode::PageDown));
        assert!(app.preview_scroll >= 20);
        handle_key(&mut app, key(KeyCode::Home));
        assert_eq!(app.preview_scroll, 0);
        handle_key(&mut app, key(KeyCode::End));
        assert_eq!(app.preview_scroll, 49);
        handle_key(&mut app, key(KeyCode::Char('s')));

        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::BlueprintDetail);

        app.screen = Screen::BlueprintPreview;
        app.current_blueprint = None;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::BlueprintSelect);
    }

    #[test]
    fn service_select_navigation_shortcuts_and_escape_work() {
        let mut app = App::new();
        app.screen = Screen::ServiceSelect;
        app.selected_service = 1;

        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected_service, 2);
        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected_service, 1);
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::RegionSelect);
    }

    #[test]
    fn resource_select_refresh_shortcuts_set_loading_tasks() {
        let mut app = App::new();

        app.screen = Screen::Ec2Select;
        handle_key(&mut app, key(KeyCode::Char('r')));
        assert_eq!(app.loading_task, LoadingTask::RefreshEc2);

        app.screen = Screen::VpcSelect;
        handle_key(&mut app, key(KeyCode::Char('r')));
        assert_eq!(app.loading_task, LoadingTask::RefreshVpc);

        app.screen = Screen::SecurityGroupSelect;
        handle_key(&mut app, key(KeyCode::Char('r')));
        assert_eq!(app.loading_task, LoadingTask::RefreshSecurityGroup);

        app.screen = Screen::LoadBalancerSelect;
        handle_key(&mut app, key(KeyCode::Char('r')));
        assert_eq!(app.loading_task, LoadingTask::RefreshLoadBalancer);

        app.screen = Screen::EcrSelect;
        handle_key(&mut app, key(KeyCode::Char('r')));
        assert_eq!(app.loading_task, LoadingTask::RefreshEcr);

        app.screen = Screen::AsgSelect;
        handle_key(&mut app, key(KeyCode::Char('r')));
        assert_eq!(app.loading_task, LoadingTask::RefreshAsg);
    }

    #[test]
    fn blueprint_mode_enter_adds_resource_from_each_select_screen() {
        let mut app = App::new();
        app.blueprint_mode = true;
        app.current_blueprint = Some(sample_blueprint("bp-r"));
        app.selected_index = 0;

        app.screen = Screen::Ec2Select;
        app.instances = vec![sample_resource("i-1", "ec2-a")];
        handle_key(&mut app, key(KeyCode::Enter));

        app.screen = Screen::VpcSelect;
        app.vpcs = vec![sample_resource("vpc-1", "vpc-a")];
        handle_key(&mut app, key(KeyCode::Enter));

        app.screen = Screen::SecurityGroupSelect;
        app.security_groups = vec![sample_resource("sg-1", "sg-a")];
        handle_key(&mut app, key(KeyCode::Enter));

        app.screen = Screen::LoadBalancerSelect;
        app.load_balancers = vec![sample_resource("lb-1", "lb-a")];
        handle_key(&mut app, key(KeyCode::Enter));

        app.screen = Screen::EcrSelect;
        app.ecr_repositories = vec![sample_resource("repo-a", "repo-a")];
        handle_key(&mut app, key(KeyCode::Enter));

        app.screen = Screen::AsgSelect;
        app.auto_scaling_groups = vec![sample_resource("asg-a", "asg-a")];
        handle_key(&mut app, key(KeyCode::Enter));

        let resources = app
            .current_blueprint
            .as_ref()
            .map(|bp| bp.resources.clone())
            .unwrap_or_default();
        assert_eq!(resources.len(), 6);
        assert_eq!(
            resources[0].resource_type,
            crate::blueprint::ResourceType::Ec2
        );
        assert_eq!(
            resources[1].resource_type,
            crate::blueprint::ResourceType::Network
        );
        assert_eq!(
            resources[2].resource_type,
            crate::blueprint::ResourceType::SecurityGroup
        );
        assert_eq!(
            resources[3].resource_type,
            crate::blueprint::ResourceType::LoadBalancer
        );
        assert_eq!(
            resources[4].resource_type,
            crate::blueprint::ResourceType::Ecr
        );
        assert_eq!(
            resources[5].resource_type,
            crate::blueprint::ResourceType::Asg
        );
    }

    #[test]
    fn settings_screen_left_returns_to_blueprint_select() {
        let mut app = App::new();
        app.screen = Screen::Settings;

        handle_key(&mut app, key(KeyCode::Left));
        assert_eq!(app.screen, Screen::BlueprintSelect);
        assert_eq!(app.selected_tab, 0);
    }

    #[test]
    fn preview_blueprint_mode_escape_clears_details_and_returns_to_detail_screen() {
        let mut app = App::new();
        app.screen = Screen::Preview;
        app.blueprint_mode = true;
        app.ec2_detail = Some(Ec2Detail {
            name: "web".to_string(),
            instance_id: "i-1".to_string(),
            instance_type: "t3.micro".to_string(),
            ami: "ami".to_string(),
            platform: "Linux".to_string(),
            architecture: "x86_64".to_string(),
            key_pair: "kp".to_string(),
            vpc: "vpc".to_string(),
            subnet: "subnet".to_string(),
            az: "az".to_string(),
            public_ip: "-".to_string(),
            private_ip: "10.0.0.1".to_string(),
            security_groups: vec![],
            state: "running".to_string(),
            ebs_optimized: false,
            monitoring: "disabled".to_string(),
            iam_role: None,
            iam_role_detail: None,
            launch_time: String::new(),
            tags: vec![],
            volumes: vec![],
            user_data: None,
        });

        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::BlueprintDetail);
        assert!(app.ec2_detail.is_none());
    }

    #[test]
    fn handle_mouse_outside_preview_is_noop() {
        let mut app = App::new();
        app.screen = Screen::ServiceSelect;
        app.preview_scroll = 7;

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 0,
                row: 0,
                modifiers: KeyModifiers::NONE,
            },
        );

        assert_eq!(app.preview_scroll, 7);
    }

    #[test]
    fn blueprint_select_enter_covers_new_and_existing_paths() {
        let mut app = App::new();
        app.screen = Screen::BlueprintSelect;
        app.input_buffer = "temp".to_string();

        app.selected_blueprint_index = 0;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::BlueprintNameInput);
        assert!(app.input_buffer.is_empty());

        app.blueprint_store.blueprints = vec![sample_blueprint("bp-existing")];
        app.screen = Screen::BlueprintSelect;
        app.selected_blueprint_index = 1;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::BlueprintDetail);
        assert_eq!(
            app.current_blueprint
                .as_ref()
                .map(|bp| bp.name.clone())
                .unwrap_or_default(),
            "bp-existing"
        );
    }

    #[test]
    fn process_loading_blueprint_resources_handles_none_and_empty_blueprint_paths() {
        let mut app = App::new();
        app.loading = true;
        app.loading_task = LoadingTask::LoadBlueprintResources(0);
        app.current_blueprint = None;

        process_loading(&mut app);
        assert!(!app.loading);
        assert_eq!(app.loading_task, LoadingTask::None);

        app.loading = true;
        app.loading_task = LoadingTask::LoadBlueprintResources(0);
        app.current_blueprint = Some(crate::blueprint::Blueprint {
            id: "bp-empty".to_string(),
            name: "bp-empty".to_string(),
            resources: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
        app.blueprint_markdown_parts = Vec::new();

        process_loading(&mut app);
        assert_eq!(app.screen, Screen::BlueprintPreview);
        assert!(app.preview_content.contains("# Blueprint: bp-empty"));
        assert!(!app.loading);
        assert_eq!(app.loading_task, LoadingTask::None);
    }

    #[test]
    fn process_loading_vpc_detail_steps_require_step_zero_init() {
        let mut app = App::new();
        app.loading = true;
        app.loading_task = LoadingTask::LoadVpcDetail("vpc-1".to_string(), 1);

        process_loading(&mut app);
        assert!(!app.loading);
        assert_eq!(app.network_detail, None);
        assert_eq!(
            app.loading_task,
            LoadingTask::None,
            "step 1 should not proceed without step 0 initialization"
        );
        assert_eq!(app.message, app.i18n.network_detail_unavailable("vpc-1"));
    }

    #[test]
    fn process_loading_refresh_preview_without_detail_finishes_cleanly() {
        let mut app = App::new();
        app.loading = true;
        app.loading_task = LoadingTask::RefreshPreview;

        process_loading(&mut app);
        assert!(!app.loading);
        assert_eq!(app.loading_task, LoadingTask::None);
        assert_eq!(app.message, app.i18n.refresh_complete());
    }

    #[test]
    fn process_loading_refresh_preview_updates_all_detail_types() {
        let mut app = App::new();
        app.selected_index = 0;

        let reset_refresh = |app: &mut App| {
            app.loading = true;
            app.loading_task = LoadingTask::RefreshPreview;
            app.message.clear();
        };
        let clear_details = |app: &mut App| {
            app.ec2_detail = None;
            app.network_detail = None;
            app.sg_detail = None;
            app.lb_detail = None;
            app.ecr_detail = None;
            app.asg_detail = None;
        };

        clear_details(&mut app);
        app.instances = vec![sample_resource("i-test", "ec2-test")];
        app.ec2_detail = Some(Ec2Detail {
            name: "seed-ec2".to_string(),
            instance_id: "i-seed".to_string(),
            instance_type: "t3.micro".to_string(),
            ami: "ami-seed".to_string(),
            platform: "Linux".to_string(),
            architecture: "x86_64".to_string(),
            key_pair: "kp".to_string(),
            vpc: "vpc-seed".to_string(),
            subnet: "subnet-seed".to_string(),
            az: "ap-northeast-2a".to_string(),
            public_ip: "-".to_string(),
            private_ip: "10.0.0.10".to_string(),
            security_groups: vec!["sg-seed".to_string()],
            state: "running".to_string(),
            ebs_optimized: false,
            monitoring: "Disabled".to_string(),
            iam_role: None,
            iam_role_detail: None,
            launch_time: String::new(),
            tags: vec![],
            volumes: vec![],
            user_data: None,
        });
        reset_refresh(&mut app);
        process_loading(&mut app);
        assert_eq!(app.preview_filename, "ec2-i-test.md");
        assert_eq!(app.message, app.i18n.refresh_complete());

        clear_details(&mut app);
        app.vpcs = vec![sample_resource("vpc-test", "vpc-test")];
        app.network_detail = Some(NetworkDetail {
            name: "seed-network".to_string(),
            id: "vpc-seed".to_string(),
            cidr: "10.0.0.0/16".to_string(),
            state: "available".to_string(),
            subnets: vec![],
            igws: vec![],
            nats: vec![],
            route_tables: vec![],
            eips: vec![],
            dns_support: true,
            dns_hostnames: true,
            tags: vec![],
        });
        reset_refresh(&mut app);
        process_loading(&mut app);
        assert_eq!(app.preview_filename, "network-vpc-test.md");
        assert_eq!(app.message, app.i18n.refresh_complete());

        clear_details(&mut app);
        app.security_groups = vec![sample_resource("sg-test", "sg-test")];
        app.sg_detail = Some(SecurityGroupDetail {
            name: "seed-sg".to_string(),
            id: "sg-seed".to_string(),
            description: "seed".to_string(),
            vpc_id: "vpc-seed".to_string(),
            inbound_rules: vec![],
            outbound_rules: vec![],
        });
        reset_refresh(&mut app);
        process_loading(&mut app);
        assert_eq!(app.preview_filename, "sg-sg-test.md");
        assert_eq!(app.message, app.i18n.refresh_complete());

        clear_details(&mut app);
        app.load_balancers = vec![sample_resource("lb-test", "lb-test")];
        app.lb_detail = Some(LoadBalancerDetail {
            name: "seed-lb".to_string(),
            arn: "arn:seed".to_string(),
            dns_name: "seed.example.com".to_string(),
            lb_type: "application".to_string(),
            scheme: "internal".to_string(),
            vpc_id: "vpc-seed".to_string(),
            ip_address_type: "ipv4".to_string(),
            state: "active".to_string(),
            availability_zones: vec![],
            security_groups: vec![],
            listeners: vec![],
            target_groups: vec![],
        });
        reset_refresh(&mut app);
        process_loading(&mut app);
        assert_eq!(app.preview_filename, "lb-lb-test.md");
        assert_eq!(app.message, app.i18n.refresh_complete());

        clear_details(&mut app);
        app.ecr_repositories = vec![sample_resource("repo-test", "repo-test")];
        app.ecr_detail = Some(EcrDetail {
            name: "seed-repo".to_string(),
            uri: "seed-uri".to_string(),
            tag_mutability: "MUTABLE".to_string(),
            encryption_type: "AES256".to_string(),
            kms_key: None,
            created_at: "2026-01-01".to_string(),
            image_count: 0,
        });
        reset_refresh(&mut app);
        process_loading(&mut app);
        assert_eq!(app.preview_filename, "repo-test.md");
        assert_eq!(app.message, app.i18n.refresh_complete());

        clear_details(&mut app);
        app.auto_scaling_groups = vec![sample_resource("asg-test", "asg-test")];
        app.asg_detail = Some(AsgDetail {
            name: "seed-asg".to_string(),
            arn: "arn:seed-asg".to_string(),
            launch_template_name: None,
            launch_template_id: None,
            launch_config_name: None,
            min_size: 1,
            max_size: 1,
            desired_capacity: 1,
            default_cooldown: 0,
            availability_zones: vec![],
            target_group_arns: vec![],
            health_check_type: "EC2".to_string(),
            health_check_grace_period: 0,
            instances: vec![],
            created_time: "2026-01-01".to_string(),
            scaling_policies: vec![],
            tags: vec![],
        });
        reset_refresh(&mut app);
        process_loading(&mut app);
        assert_eq!(app.preview_filename, "asg-test.md");
        assert_eq!(app.message, app.i18n.refresh_complete());
        assert!(!app.loading);
        assert_eq!(app.loading_task, LoadingTask::None);
    }

    #[test]
    fn login_screen_quit_shortcut_stops_app() {
        let mut app = App::new();
        app.screen = Screen::Login;
        app.running = true;
        handle_key(&mut app, key(KeyCode::Char('q')));
        assert!(!app.running);
    }

    #[test]
    fn process_loading_load_list_tasks_open_each_select_screen() {
        let mut app = App::new();

        app.loading = true;
        app.loading_task = LoadingTask::LoadEc2;
        process_loading(&mut app);
        assert_eq!(app.screen, Screen::Ec2Select);
        assert!(!app.instances.is_empty());

        app.loading = true;
        app.loading_task = LoadingTask::LoadVpc;
        process_loading(&mut app);
        assert_eq!(app.screen, Screen::VpcSelect);
        assert!(!app.vpcs.is_empty());

        app.loading = true;
        app.loading_task = LoadingTask::LoadSecurityGroup;
        process_loading(&mut app);
        assert_eq!(app.screen, Screen::SecurityGroupSelect);
        assert!(!app.security_groups.is_empty());

        app.loading = true;
        app.loading_task = LoadingTask::LoadLoadBalancer;
        process_loading(&mut app);
        assert_eq!(app.screen, Screen::LoadBalancerSelect);
        assert!(!app.load_balancers.is_empty());

        app.loading = true;
        app.loading_task = LoadingTask::LoadEcr;
        process_loading(&mut app);
        assert_eq!(app.screen, Screen::EcrSelect);
        assert!(!app.ecr_repositories.is_empty());

        app.loading = true;
        app.loading_task = LoadingTask::LoadAsg;
        process_loading(&mut app);
        assert_eq!(app.screen, Screen::AsgSelect);
        assert!(!app.auto_scaling_groups.is_empty());
    }

    #[test]
    fn process_loading_detail_tasks_open_preview_screen() {
        let mut app = App::new();
        app.settings.language = crate::i18n::Language::English;

        app.loading = true;
        app.loading_task = LoadingTask::LoadEc2Detail("i-1234".to_string());
        process_loading(&mut app);
        assert_eq!(app.screen, Screen::Preview);
        assert!(app.ec2_detail.is_some());

        app.loading = true;
        app.loading_task = LoadingTask::LoadSecurityGroupDetail("sg-1234".to_string());
        process_loading(&mut app);
        assert_eq!(app.screen, Screen::Preview);
        assert!(app.sg_detail.is_some());

        app.loading = true;
        app.loading_task = LoadingTask::LoadLoadBalancerDetail("lb-1234".to_string());
        process_loading(&mut app);
        assert_eq!(app.screen, Screen::Preview);
        assert!(app.lb_detail.is_some());

        app.loading = true;
        app.loading_task = LoadingTask::LoadEcrDetail("repo-a".to_string());
        process_loading(&mut app);
        assert_eq!(app.screen, Screen::Preview);
        assert!(app.ecr_detail.is_some());

        app.loading = true;
        app.loading_task = LoadingTask::LoadAsgDetail("asg-a".to_string());
        process_loading(&mut app);
        assert_eq!(app.screen, Screen::Preview);
        assert!(app.asg_detail.is_some());
    }

    #[test]
    fn process_loading_refresh_tasks_update_message_and_finish() {
        let mut app = App::new();

        app.loading = true;
        app.loading_task = LoadingTask::RefreshEc2;
        process_loading(&mut app);
        assert_eq!(app.message, app.i18n.refresh_complete());
        assert!(!app.loading);

        app.loading = true;
        app.loading_task = LoadingTask::RefreshVpc;
        process_loading(&mut app);
        assert_eq!(app.message, app.i18n.refresh_complete());
        assert!(!app.loading);

        app.loading = true;
        app.loading_task = LoadingTask::RefreshSecurityGroup;
        process_loading(&mut app);
        assert_eq!(app.message, app.i18n.refresh_complete());
        assert!(!app.loading);

        app.loading = true;
        app.loading_task = LoadingTask::RefreshLoadBalancer;
        process_loading(&mut app);
        assert_eq!(app.message, app.i18n.refresh_complete());
        assert!(!app.loading);

        app.loading = true;
        app.loading_task = LoadingTask::RefreshEcr;
        process_loading(&mut app);
        assert_eq!(app.message, app.i18n.refresh_complete());
        assert!(!app.loading);

        app.loading = true;
        app.loading_task = LoadingTask::RefreshAsg;
        process_loading(&mut app);
        assert_eq!(app.message, app.i18n.refresh_complete());
        assert!(!app.loading);
    }

    #[test]
    fn process_loading_vpc_detail_step_zero_initializes_network_detail() {
        let mut app = App::new();
        app.loading = true;
        app.loading_task = LoadingTask::LoadVpcDetail("vpc-1".to_string(), 0);

        process_loading(&mut app);

        assert!(app.loading_progress.vpc_info);
        assert!(app.network_detail.is_some());
        assert_eq!(
            app.loading_task,
            LoadingTask::LoadVpcDetail("vpc-1".to_string(), 1)
        );
    }

    #[test]
    fn process_loading_blueprint_resources_generates_preview_for_non_empty_blueprint() {
        let mut app = App::new();
        app.settings.language = crate::i18n::Language::English;
        app.loading = true;
        app.current_blueprint = Some(crate::blueprint::Blueprint {
            id: "bp-full".to_string(),
            name: "bp-full".to_string(),
            resources: vec![
                crate::blueprint::BlueprintResource {
                    resource_type: crate::blueprint::ResourceType::Ec2,
                    region: "ap-northeast-2".to_string(),
                    resource_id: "i-1".to_string(),
                    resource_name: "ec2-a".to_string(),
                },
                crate::blueprint::BlueprintResource {
                    resource_type: crate::blueprint::ResourceType::SecurityGroup,
                    region: "ap-northeast-2".to_string(),
                    resource_id: "sg-1".to_string(),
                    resource_name: "sg-a".to_string(),
                },
                crate::blueprint::BlueprintResource {
                    resource_type: crate::blueprint::ResourceType::LoadBalancer,
                    region: "ap-northeast-2".to_string(),
                    resource_id: "lb-1".to_string(),
                    resource_name: "lb-a".to_string(),
                },
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
        app.loading_task = LoadingTask::LoadBlueprintResources(0);

        for _ in 0..10 {
            if !app.loading {
                break;
            }
            process_loading(&mut app);
        }

        assert_eq!(app.screen, Screen::BlueprintPreview);
        assert!(app.preview_content.contains("# Blueprint: bp-full"));
        assert!(app.preview_content.contains("## EC2"));
        assert!(app.preview_content.contains("## Security Group"));
        assert!(app.preview_content.contains("## Load Balancer"));
        assert!(!app.loading);
        assert_eq!(app.loading_task, LoadingTask::None);
    }

    #[test]
    fn blueprint_select_empty_generate_sets_message_and_quit_works() {
        let mut app = App::new();
        app.screen = Screen::BlueprintSelect;
        app.blueprint_store.blueprints = vec![sample_blueprint("empty")];
        app.selected_blueprint_index = 1;

        handle_key(&mut app, key(KeyCode::Char('g')));
        assert_eq!(app.message, app.i18n.no_resources());

        handle_key(&mut app, key(KeyCode::Char('l')));
        assert_eq!(app.screen, Screen::Settings);

        app.screen = Screen::BlueprintSelect;
        app.running = true;
        handle_key(&mut app, key(KeyCode::Char('q')));
        assert!(!app.running);
    }

    #[test]
    fn blueprint_name_input_enter_with_blank_keeps_screen_and_clears_buffer() {
        let mut app = App::new();
        app.screen = Screen::BlueprintNameInput;
        app.input_buffer = "   ".to_string();

        handle_key(&mut app, key(KeyCode::Enter));

        assert_eq!(app.screen, Screen::BlueprintNameInput);
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn blueprint_detail_navigation_and_quit_shortcuts_work() {
        let mut app = App::new();
        app.screen = Screen::BlueprintDetail;
        app.current_blueprint = Some(crate::blueprint::Blueprint {
            id: "bp-nav".to_string(),
            name: "bp-nav".to_string(),
            resources: vec![
                crate::blueprint::BlueprintResource {
                    resource_type: crate::blueprint::ResourceType::Ec2,
                    region: "ap-northeast-2".to_string(),
                    resource_id: "i-1".to_string(),
                    resource_name: "one".to_string(),
                },
                crate::blueprint::BlueprintResource {
                    resource_type: crate::blueprint::ResourceType::Ec2,
                    region: "ap-northeast-2".to_string(),
                    resource_id: "i-2".to_string(),
                    resource_name: "two".to_string(),
                },
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });

        app.blueprint_resource_index = 0;
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.blueprint_resource_index, 1);
        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.blueprint_resource_index, 0);

        app.running = true;
        handle_key(&mut app, key(KeyCode::Char('q')));
        assert!(!app.running);
    }

    #[test]
    fn region_select_enter_blueprint_escape_and_quit_paths_work() {
        let mut app = App::new();
        app.screen = Screen::RegionSelect;
        app.selected_region = 1;
        app.blueprint_mode = false;

        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::ServiceSelect);

        app.screen = Screen::RegionSelect;
        app.blueprint_mode = true;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::BlueprintDetail);

        app.screen = Screen::RegionSelect;
        app.running = true;
        handle_key(&mut app, key(KeyCode::Char('q')));
        assert!(!app.running);
    }

    #[test]
    fn service_select_remaining_enter_branches_and_quit_paths_work() {
        let mut app = App::new();
        app.screen = Screen::ServiceSelect;

        app.selected_service = 2;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.loading_task, LoadingTask::LoadSecurityGroup);

        app.loading = false;
        app.selected_service = 3;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.loading_task, LoadingTask::LoadLoadBalancer);

        app.loading = false;
        app.selected_service = 4;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.loading_task, LoadingTask::LoadEcr);

        app.screen = Screen::ServiceSelect;
        handle_key(&mut app, key(KeyCode::Char('l')));
        assert_eq!(app.screen, Screen::Settings);

        app.screen = Screen::ServiceSelect;
        app.running = true;
        handle_key(&mut app, key(KeyCode::Char('q')));
        assert!(!app.running);
    }

    #[test]
    fn select_screens_bounds_escape_and_quit_paths_are_handled() {
        let mut app = App::new();

        app.screen = Screen::Ec2Select;
        app.instances = vec![sample_resource("i-1", "ec2-a")];
        app.selected_index = 0;
        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected_index, 0);
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected_index, 0);
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::ServiceSelect);

        app.screen = Screen::VpcSelect;
        app.vpcs = vec![sample_resource("vpc-1", "vpc-a")];
        app.selected_index = 0;
        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected_index, 0);
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected_index, 0);
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::ServiceSelect);

        app.screen = Screen::SecurityGroupSelect;
        app.security_groups = vec![sample_resource("sg-1", "sg-a")];
        app.selected_index = 0;
        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected_index, 0);
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected_index, 0);
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::ServiceSelect);

        app.screen = Screen::LoadBalancerSelect;
        app.load_balancers = vec![sample_resource("lb-1", "lb-a")];
        app.selected_index = 0;
        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected_index, 0);
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected_index, 0);
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::ServiceSelect);

        app.screen = Screen::EcrSelect;
        app.ecr_repositories = vec![sample_resource("repo-a", "repo-a")];
        app.selected_index = 0;
        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected_index, 0);
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected_index, 0);
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::ServiceSelect);

        app.screen = Screen::AsgSelect;
        app.auto_scaling_groups = vec![sample_resource("asg-a", "asg-a")];
        app.selected_index = 0;
        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected_index, 0);
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected_index, 0);
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::ServiceSelect);

        app.screen = Screen::AsgSelect;
        app.running = true;
        handle_key(&mut app, key(KeyCode::Char('q')));
        assert!(!app.running);
    }

    #[test]
    fn preview_shortcuts_cover_pageup_refresh_quit_and_non_blueprint_add() {
        let mut app = App::new();
        app.screen = Screen::Preview;
        app.preview_content = (0..120)
            .map(|n| format!("line {}", n))
            .collect::<Vec<_>>()
            .join("\n");
        app.preview_scroll = 30;
        app.blueprint_mode = false;

        handle_key(&mut app, key(KeyCode::PageUp));
        assert_eq!(app.preview_scroll, 10);

        handle_key(&mut app, key(KeyCode::Char('a')));
        assert_eq!(app.screen, Screen::Preview);

        handle_key(&mut app, key(KeyCode::Char('r')));
        assert_eq!(app.loading_task, LoadingTask::RefreshPreview);

        app.running = true;
        handle_key(&mut app, key(KeyCode::Char('q')));
        assert!(!app.running);
    }

    #[test]
    fn preview_escape_routes_for_network_security_group_and_load_balancer() {
        let mut app = App::new();
        app.screen = Screen::Preview;
        app.blueprint_mode = false;

        app.network_detail = Some(NetworkDetail {
            name: "vpc-main".to_string(),
            id: "vpc-1".to_string(),
            cidr: "10.0.0.0/16".to_string(),
            state: "available".to_string(),
            subnets: vec![],
            igws: vec![],
            nats: vec![],
            route_tables: vec![],
            eips: vec![],
            dns_support: true,
            dns_hostnames: true,
            tags: vec![],
        });
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::VpcSelect);

        app.screen = Screen::Preview;
        app.sg_detail = Some(SecurityGroupDetail {
            name: "sg-main".to_string(),
            id: "sg-1".to_string(),
            description: "sg".to_string(),
            vpc_id: "vpc-1".to_string(),
            inbound_rules: vec![],
            outbound_rules: vec![],
        });
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::SecurityGroupSelect);

        app.screen = Screen::Preview;
        app.lb_detail = Some(LoadBalancerDetail {
            name: "lb-main".to_string(),
            arn: "arn:...:loadbalancer/app/lb-main/1".to_string(),
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
        });
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::LoadBalancerSelect);
    }

    #[test]
    fn settings_down_h_tab_and_quit_paths_work() {
        let mut app = App::new();
        app.screen = Screen::Settings;
        app.selected_setting = 0;
        app.selected_tab = 1;

        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected_setting, 0);

        app.screen = Screen::Settings;
        app.selected_tab = 1;
        handle_key(&mut app, key(KeyCode::Char('h')));
        assert_eq!(app.screen, Screen::BlueprintSelect);
        assert_eq!(app.selected_tab, 0);

        app.screen = Screen::Settings;
        app.selected_tab = 1;
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.screen, Screen::BlueprintSelect);
        assert_eq!(app.selected_tab, 0);

        app.screen = Screen::Settings;
        app.running = true;
        handle_key(&mut app, key(KeyCode::Char('q')));
        assert!(!app.running);
    }
}
