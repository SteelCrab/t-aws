use crate::app::{App, LoadingTask, REGIONS, SERVICE_KEYS, Screen};
use crate::aws_cli::{self, NetworkDetail};
use crate::blueprint::{BlueprintResource, ResourceType};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

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
            app.instances = aws_cli::list_instances();
            app.message = app.i18n.refresh_complete().to_string();
            finish_loading(app);
        }
        LoadingTask::RefreshVpc => {
            app.vpcs = aws_cli::list_vpcs();
            app.message = app.i18n.refresh_complete().to_string();
            finish_loading(app);
        }
        LoadingTask::RefreshPreview => {
            if app.ec2_detail.is_some() {
                if let Some(new_detail) = aws_cli::get_instance_detail(
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
                if let Some(new_detail) = aws_cli::get_network_detail(
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
                && let Some(new_detail) = aws_cli::get_security_group_detail(
                    app.security_groups
                        .get(app.selected_index)
                        .map(|sg| sg.id.as_str())
                        .unwrap_or(""),
                )
            {
                app.preview_content = new_detail.to_markdown(app.settings.language);
                app.preview_filename = format!("{}.md", new_detail.name);
                app.sg_detail = Some(new_detail);
            }
            app.message = app.i18n.refresh_complete().to_string();
            finish_loading(app);
        }
        LoadingTask::LoadEc2 => {
            app.instances = aws_cli::list_instances();
            app.selected_index = 0;
            app.screen = Screen::Ec2Select;
            finish_loading(app);
        }
        LoadingTask::LoadVpc => {
            app.vpcs = aws_cli::list_vpcs();
            app.selected_index = 0;
            app.screen = Screen::VpcSelect;
            finish_loading(app);
        }
        LoadingTask::LoadEc2Detail(id) => {
            if let Some(detail) = aws_cli::get_instance_detail(&id) {
                app.preview_content = detail.to_markdown(app.settings.language);
                app.preview_filename = format!("{}.md", detail.name);
                app.ec2_detail = Some(detail);
                app.screen = Screen::Preview;
            }
            finish_loading(app);
        }
        LoadingTask::LoadVpcDetail(id, step) => {
            process_vpc_detail_step(app, &id, step);
        }
        LoadingTask::RefreshSecurityGroup => {
            app.security_groups = aws_cli::list_security_groups();
            app.message = app.i18n.refresh_complete().to_string();
            finish_loading(app);
        }
        LoadingTask::LoadSecurityGroup => {
            app.security_groups = aws_cli::list_security_groups();
            app.selected_index = 0;
            app.screen = Screen::SecurityGroupSelect;
            finish_loading(app);
        }
        LoadingTask::LoadSecurityGroupDetail(id) => {
            if let Some(detail) = aws_cli::get_security_group_detail(&id) {
                app.preview_content = detail.to_markdown(app.settings.language);
                app.preview_filename = format!("{}.md", detail.name);
                app.sg_detail = Some(detail);
                app.screen = Screen::Preview;
            }
            finish_loading(app);
        }
        LoadingTask::RefreshLoadBalancer => {
            app.load_balancers = aws_cli::list_load_balancers();
            app.message = app.i18n.refresh_complete().to_string();
            finish_loading(app);
        }
        LoadingTask::LoadLoadBalancer => {
            app.load_balancers = aws_cli::list_load_balancers();
            app.selected_index = 0;
            app.screen = Screen::LoadBalancerSelect;
            finish_loading(app);
        }
        LoadingTask::LoadLoadBalancerDetail(arn) => {
            if let Some(detail) = aws_cli::get_load_balancer_detail(&arn) {
                app.preview_content = detail.to_markdown(app.settings.language);
                app.preview_filename = format!("{}.md", detail.name);
                app.lb_detail = Some(detail);
                app.screen = Screen::Preview;
            }
            finish_loading(app);
        }
        LoadingTask::RefreshEcr => {
            app.ecr_repositories = aws_cli::list_ecr_repositories();
            app.message = app.i18n.refresh_complete().to_string();
            finish_loading(app);
        }
        LoadingTask::LoadEcr => {
            app.ecr_repositories = aws_cli::list_ecr_repositories();
            app.selected_index = 0;
            app.screen = Screen::EcrSelect;
            finish_loading(app);
        }
        LoadingTask::LoadEcrDetail(name) => {
            if let Some(detail) = aws_cli::get_ecr_detail(&name) {
                app.preview_content = detail.to_markdown(app.settings.language);
                app.preview_filename = format!("{}.md", detail.name);
                app.ecr_detail = Some(detail);
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
    aws_cli::set_region(&resource.region);

    // Fetch resource detail and generate markdown
    let failed = app.i18n.query_failed();
    let markdown = match resource.resource_type {
        ResourceType::Ec2 => aws_cli::get_instance_detail(&resource.resource_id)
            .map(|d| d.to_markdown(app.settings.language))
            .unwrap_or_else(|| format!("## EC2: {} ({})\n", resource.resource_name, failed)),
        ResourceType::Network => aws_cli::get_network_detail(&resource.resource_id)
            .map(|d| d.to_markdown(app.settings.language))
            .unwrap_or_else(|| format!("## Network: {} ({})\n", resource.resource_name, failed)),
        ResourceType::SecurityGroup => aws_cli::get_security_group_detail(&resource.resource_id)
            .map(|d| d.to_markdown(app.settings.language))
            .unwrap_or_else(|| {
                format!(
                    "## Security Group: {} ({})\n",
                    resource.resource_name, failed
                )
            }),
        ResourceType::LoadBalancer => aws_cli::get_load_balancer_detail(&resource.resource_id)
            .map(|d| d.to_markdown(app.settings.language))
            .unwrap_or_else(|| {
                format!(
                    "## Load Balancer: {} ({})\n",
                    resource.resource_name, failed
                )
            }),
        ResourceType::Ecr => aws_cli::get_ecr_detail(&resource.resource_id)
            .map(|d| d.to_markdown(app.settings.language))
            .unwrap_or_else(|| format!("## ECR: {} ({})\n", resource.resource_name, failed)),
    };

    app.blueprint_markdown_parts.push(markdown);

    // Move to next resource
    app.loading_task = LoadingTask::LoadBlueprintResources(current_index + 1);
}

fn process_vpc_detail_step(app: &mut App, vpc_id: &str, step: u8) {
    match step {
        0 => {
            // Step 0: VPC 기본 정보
            if let Some(info) = aws_cli::get_vpc_info(vpc_id) {
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
            }
            app.loading_task = LoadingTask::LoadVpcDetail(vpc_id.to_string(), 1);
        }
        1 => {
            // Step 1: Subnets
            if let Some(ref mut detail) = app.network_detail {
                detail.subnets = aws_cli::list_subnets(vpc_id);
            }
            app.loading_progress.subnets = true;
            app.loading_task = LoadingTask::LoadVpcDetail(vpc_id.to_string(), 2);
        }
        2 => {
            // Step 2: Internet Gateways
            if let Some(ref mut detail) = app.network_detail {
                detail.igws = aws_cli::list_internet_gateways(vpc_id);
            }
            app.loading_progress.igws = true;
            app.loading_task = LoadingTask::LoadVpcDetail(vpc_id.to_string(), 3);
        }
        3 => {
            // Step 3: NAT Gateways
            if let Some(ref mut detail) = app.network_detail {
                detail.nats = aws_cli::list_nat_gateways(vpc_id);
            }
            app.loading_progress.nats = true;
            app.loading_task = LoadingTask::LoadVpcDetail(vpc_id.to_string(), 4);
        }
        4 => {
            // Step 4: Route Tables
            if let Some(ref mut detail) = app.network_detail {
                detail.route_tables = aws_cli::list_route_tables(vpc_id);
            }
            app.loading_progress.route_tables = true;
            app.loading_task = LoadingTask::LoadVpcDetail(vpc_id.to_string(), 5);
        }
        5 => {
            // Step 5: Elastic IPs
            if let Some(ref mut detail) = app.network_detail {
                detail.eips = aws_cli::list_eips();
            }
            app.loading_progress.eips = true;
            app.loading_task = LoadingTask::LoadVpcDetail(vpc_id.to_string(), 6);
        }
        6 => {
            // Step 6: DNS Attributes
            if let Some(ref mut detail) = app.network_detail {
                detail.dns_support = aws_cli::get_vpc_dns_support(vpc_id);
                detail.dns_hostnames = aws_cli::get_vpc_dns_hostnames(vpc_id);
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
        KeyCode::Enter => app.check_login(),
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
            5 => {
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
                let id = app.instances[app.selected_index].id.clone();
                start_loading(app, LoadingTask::LoadEc2Detail(id));
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
                let id = app.vpcs[app.selected_index].id.clone();
                start_loading(app, LoadingTask::LoadVpcDetail(id, 0));
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
                let id = app.security_groups[app.selected_index].id.clone();
                start_loading(app, LoadingTask::LoadSecurityGroupDetail(id));
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
                let arn = app.load_balancers[app.selected_index].id.clone();
                start_loading(app, LoadingTask::LoadLoadBalancerDetail(arn));
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
                let name = app.ecr_repositories[app.selected_index].id.clone();
                start_loading(app, LoadingTask::LoadEcrDetail(name));
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
