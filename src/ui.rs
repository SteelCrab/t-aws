use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
};

use crate::app::{App, LoadingTask, REGIONS, SERVICE_KEYS, Screen};

const EMD_LOGO: &str = r#"
  ______ __  __ _____  
 |  ____|  \/  |  __ \ 
 | |__  | \  / | |  | |
 |  __| | |\/| | |  | |
 | |____| |  | | |__| |
 |______|_|  |_|_____/  AWS Markdown Template Generator 
"#;
use crate::blueprint::ResourceType;

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(1), // Tab bar
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_header(frame, chunks[0]);
    draw_tabs(frame, app, chunks[1]);
    draw_main(frame, app, chunks[2]);
    draw_footer(frame, app, chunks[3]);
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let i = &app.i18n;

    let tab_titles = vec![
        Line::from(format!("  {}  ", i.main_tab())).style(if app.selected_tab == 0 {
            Style::default().fg(Color::White).bg(Color::Blue)
        } else {
            Style::default().fg(Color::DarkGray)
        }),
        Line::from(format!("  {}  ", i.settings())).style(if app.selected_tab == 1 {
            Style::default().fg(Color::White).bg(Color::Magenta)
        } else {
            Style::default().fg(Color::DarkGray)
        }),
    ];

    let tabs = Tabs::new(tab_titles)
        .select(app.selected_tab)
        .padding("", "")
        .divider(" ");

    frame.render_widget(tabs, area);
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let mut header_content = Vec::new();
    for line in EMD_LOGO.lines() {
        if !line.trim().is_empty() {
            header_content.push(Line::from(Span::styled(
                line,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
        }
    }

    let title = Paragraph::new(header_content).block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let i = &app.i18n;
    let help = match &app.screen {
        Screen::Login => format!("Enter: {} | q: {}", i.retry(), i.exit()),
        Screen::BlueprintSelect => format!(
            "↑↓/jk: {} | Enter: {} | g: {} | d: {} | s: {} | ►: {} | q: {}",
            i.move_cursor(),
            i.select(),
            i.markdown_generate(),
            i.delete(),
            i.single_mode(),
            i.settings(),
            i.exit()
        ),
        Screen::BlueprintDetail => format!(
            "↑↓/jk: {} | Shift+↑↓/JK: {} | a: {} | d: {} | g/Enter: {} | Esc: {} | q: {}",
            i.move_cursor(),
            i.reorder(),
            i.add(),
            i.delete(),
            i.generate(),
            i.back(),
            i.exit()
        ),
        Screen::BlueprintNameInput => format!("Enter: {} | Esc: {}", i.confirm(), i.cancel()),
        Screen::BlueprintPreview => format!(
            "↑↓/jk: {} | PgUp/PgDn: {} | Home/End | Enter/s: {} | Esc: {} | q: {}",
            i.scroll(),
            i.page(),
            i.save(),
            i.back(),
            i.exit()
        ),
        Screen::RegionSelect => format!(
            "↑↓/jk: {} | Enter: {} | Esc: {} | q: {}",
            i.move_cursor(),
            i.select(),
            i.back(),
            i.exit()
        ),
        Screen::ServiceSelect => format!(
            "↑↓/jk: {} | Enter: {} | ►: {} | Esc: {} | q: {}",
            i.move_cursor(),
            i.select(),
            i.settings(),
            i.back(),
            i.exit()
        ),
        Screen::Ec2Select
        | Screen::VpcSelect
        | Screen::SecurityGroupSelect
        | Screen::LoadBalancerSelect
        | Screen::EcrSelect
        | Screen::AsgSelect => format!(
            "↑↓/jk: {} | Enter: {} | r: {} | Esc: {} | q: {}",
            i.move_cursor(),
            i.select(),
            i.refresh(),
            i.back(),
            i.exit()
        ),
        Screen::Preview => {
            if app.blueprint_mode {
                format!(
                    "↑↓/jk: {} | Enter/s: {} | a: {} | r: {} | Esc: {} | q: {}",
                    i.scroll(),
                    i.save(),
                    i.add_to_blueprint(),
                    i.refresh(),
                    i.back(),
                    i.exit()
                )
            } else {
                format!(
                    "↑↓/jk: {} | Enter/s: {} | r: {} | Esc: {} | q: {}",
                    i.scroll(),
                    i.save(),
                    i.refresh(),
                    i.back(),
                    i.exit()
                )
            }
        }
        Screen::Settings => format!(
            "↑↓/jk: {} | Enter/Space: {} | ◄: {} | q: {}",
            i.move_cursor(),
            i.change(),
            i.back(),
            i.exit()
        ),
    };

    let msg = if app.message.is_empty() {
        help
    } else {
        format!("{} | {}", app.message, help)
    };

    let footer = Paragraph::new(msg)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, area);
}

fn draw_main(frame: &mut Frame, app: &App, area: Rect) {
    // 로딩 중이면 로딩 화면 표시
    if app.loading {
        draw_loading(frame, app, area);
        return;
    }

    match &app.screen {
        Screen::Login => draw_login(frame, app, area),
        Screen::BlueprintSelect => draw_blueprint_select(frame, app, area),
        Screen::BlueprintDetail => draw_blueprint_detail(frame, app, area),
        Screen::BlueprintNameInput => draw_blueprint_name_input(frame, app, area),
        Screen::BlueprintPreview => draw_blueprint_preview(frame, app, area),
        Screen::RegionSelect => draw_region_select(frame, app, area),
        Screen::ServiceSelect => draw_service_select(frame, app, area),
        Screen::Ec2Select => draw_ec2_select(frame, app, area),
        Screen::VpcSelect => draw_vpc_select(frame, app, area),
        Screen::SecurityGroupSelect => draw_security_group_select(frame, app, area),
        Screen::Preview => draw_preview(frame, app, area),
        Screen::LoadBalancerSelect => draw_load_balancer_select(frame, app, area),
        Screen::EcrSelect => draw_ecr_select(frame, app, area),
        Screen::AsgSelect => draw_asg_select(frame, app, area),
        Screen::Settings => draw_settings(frame, app, area),
    }
}

fn draw_loading(frame: &mut Frame, app: &App, area: Rect) {
    let i = &app.i18n;

    // VPC 상세 로딩일 경우 체크리스트 표시
    if let LoadingTask::LoadVpcDetail(_, _) = &app.loading_task {
        draw_vpc_loading_checklist(frame, app, area);
        return;
    }

    // Blueprint 리소스 로딩일 경우 진행 상황 표시
    if let LoadingTask::LoadBlueprintResources(current_index) = &app.loading_task {
        draw_blueprint_loading(frame, app, area, *current_index);
        return;
    }

    let task_name = match &app.loading_task {
        LoadingTask::None => i.processing(),
        LoadingTask::RefreshEc2 => i.refreshing_ec2_list(),
        LoadingTask::RefreshVpc => i.refreshing_vpc_list(),
        LoadingTask::RefreshSecurityGroup => i.refreshing_sg_list(),
        LoadingTask::RefreshPreview => i.refreshing_preview(),
        LoadingTask::LoadEc2 => i.loading_ec2_list(),
        LoadingTask::LoadVpc => i.loading_vpc_list(),
        LoadingTask::LoadSecurityGroup => i.loading_sg_list(),

        LoadingTask::LoadVpcDetail(_, _) => i.loading_vpc_detail(),

        LoadingTask::RefreshLoadBalancer => i.refreshing_lb_list(),
        LoadingTask::LoadLoadBalancer => i.loading_lb_list(),

        LoadingTask::RefreshEcr => i.refreshing_ecr_list(),
        LoadingTask::LoadEcr => i.loading_ecr_list(),

        LoadingTask::RefreshAsg => i.loading_asg_list(),
        LoadingTask::LoadAsg => i.loading_asg_list(),
        LoadingTask::LoadAsgDetail(_) => i.loading_asg_detail(),
        LoadingTask::LoadBlueprintResources(_) => i.loading_blueprint_resources(),
    };

    let content = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            format!("    ⏳ {}", i.loading_msg()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("    {}", task_name),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("    {}", i.aws_cli_waiting()),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let title = format!(" {} ", i.loading());
    let para = Paragraph::new(content).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(para, area);
}

fn draw_vpc_loading_checklist(frame: &mut Frame, app: &App, area: Rect) {
    let p = &app.loading_progress;
    let i = &app.i18n;

    // 현재 로딩 중인 단계 결정
    let current_step = if !p.vpc_info {
        0
    } else if !p.subnets {
        1
    } else if !p.igws {
        2
    } else if !p.nats {
        3
    } else if !p.route_tables {
        4
    } else if !p.eips {
        5
    } else if !p.dns_attrs {
        6
    } else {
        7
    };

    fn item(done: bool, loading: bool, text: &str) -> Line<'static> {
        let (check, color) = if done {
            ("  ✓ ", Color::Green)
        } else if loading {
            ("  ▸ ", Color::Cyan)
        } else {
            ("  ○ ", Color::DarkGray)
        };
        let text_color = if done {
            Color::Green
        } else if loading {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        Line::from(vec![
            Span::styled(check, Style::default().fg(color)),
            Span::styled(text.to_string(), Style::default().fg(text_color)),
        ])
    }

    let steps = [
        i.vpc_basic_info(),
        i.subnets(),
        i.internet_gateway(),
        i.nat_gateway(),
        i.route_tables(),
        i.elastic_ip(),
        i.dns_settings(),
    ];

    let current_task = if current_step < steps.len() {
        steps[current_step]
    } else {
        i.completing()
    };

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  ⏳ {}...", i.loading_vpc_detail()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        item(p.vpc_info, current_step == 0, steps[0]),
        item(p.subnets, current_step == 1, steps[1]),
        item(p.igws, current_step == 2, steps[2]),
        item(p.nats, current_step == 3, steps[3]),
        item(p.route_tables, current_step == 4, steps[4]),
        item(p.eips, current_step == 5, steps[5]),
        item(p.dns_attrs, current_step == 6, steps[6]),
        Line::from(""),
        Line::from(Span::styled(
            i.current_loading(current_task),
            Style::default().fg(Color::Cyan),
        )),
    ];

    let title = format!(" Network {} ", i.loading());
    let para = Paragraph::new(content).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(para, area);
}

fn draw_login(frame: &mut Frame, app: &App, area: Rect) {
    let i = &app.i18n;
    let content = if let Some(ref info) = app.login_info {
        vec![
            Line::from(Span::styled(
                i.aws_login_verified(),
                Style::default().fg(Color::Green),
            )),
            Line::from(""),
            Line::from(info.as_str()),
        ]
    } else if let Some(ref err) = app.login_error {
        vec![
            Line::from(Span::styled(
                i.aws_login_required(),
                Style::default().fg(Color::Red),
            )),
            Line::from(""),
            Line::from(err.as_str()),
            Line::from(""),
            Line::from(i.aws_configure_hint()),
        ]
    } else {
        vec![Line::from(i.aws_login_checking())]
    };

    let mut login_content = vec![Line::from("")];

    for line in EMD_LOGO.lines() {
        login_content.push(Line::from(Span::styled(
            line,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
    }

    login_content.push(Line::from(""));
    login_content.extend(content);

    let title = format!(" {} ", i.login());
    let para = Paragraph::new(login_content)
        .block(Block::default().title(title).borders(Borders::ALL))
        .alignment(Alignment::Center);
    frame.render_widget(para, area);
}

fn draw_region_select(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.settings.language;
    let items: Vec<ListItem> = REGIONS
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let style = if i == app.selected_region {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if i == app.selected_region {
                "▶ "
            } else {
                "  "
            };
            ListItem::new(format!("{}{} ({})", prefix, r.code, r.name(lang))).style(style)
        })
        .collect();

    let title = format!(" {} ", app.i18n.region());
    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn draw_service_select(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.settings.language;
    let i = &app.i18n;
    let region = &REGIONS[app.selected_region];
    let title = format!(
        " {} [{} - {}] ",
        i.service(),
        region.code,
        region.name(lang)
    );

    // Build service list: services + exit
    let services: Vec<&str> = SERVICE_KEYS.to_vec();
    let exit_label = i.exit();

    let items: Vec<ListItem> = services
        .iter()
        .chain(std::iter::once(&exit_label))
        .enumerate()
        .map(|(idx, s)| {
            let style = if idx == app.selected_service {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if idx == app.selected_service {
                "▶ "
            } else {
                "  "
            };
            ListItem::new(format!("{}{}", prefix, s)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn draw_ec2_select(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.settings.language;
    let region = &REGIONS[app.selected_region];
    let title = format!(" EC2 [{} - {}] ", region.code, region.name(lang));

    if app.instances.is_empty() {
        let para = Paragraph::new(app.i18n.no_instances())
            .block(Block::default().title(title).borders(Borders::ALL));
        frame.render_widget(para, area);
        return;
    }

    let items: Vec<ListItem> = app
        .instances
        .iter()
        .enumerate()
        .map(|(i, inst)| {
            let is_in_blueprint = app.current_blueprint.as_ref().is_some_and(|bp| {
                bp.resources
                    .iter()
                    .any(|r| r.resource_id == inst.id && r.resource_type == ResourceType::Ec2)
            });

            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let mut prefix = if i == app.selected_index {
                "▶ ".to_string()
            } else {
                "  ".to_string()
            };

            if is_in_blueprint {
                prefix = format!("{}✓ ", prefix);
            }

            ListItem::new(format!("{}{}", prefix, inst.name)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn draw_preview(frame: &mut Frame, app: &App, area: Rect) {
    let title = format!(" {} - {} ", app.i18n.preview(), app.preview_filename);
    let para = Paragraph::new(app.preview_content.as_str())
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0))
        .block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(para, area);
}

fn draw_vpc_select(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.settings.language;
    let region = &REGIONS[app.selected_region];
    let title = format!(" Network [{} - {}] ", region.code, region.name(lang));

    if app.vpcs.is_empty() {
        let para = Paragraph::new(app.i18n.no_vpcs())
            .block(Block::default().title(title).borders(Borders::ALL));
        frame.render_widget(para, area);
        return;
    }

    let items: Vec<ListItem> = app
        .vpcs
        .iter()
        .enumerate()
        .map(|(i, vpc)| {
            let is_in_blueprint = app.current_blueprint.as_ref().is_some_and(|bp| {
                bp.resources
                    .iter()
                    .any(|r| r.resource_id == vpc.id && r.resource_type == ResourceType::Network)
            });

            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let mut prefix = if i == app.selected_index {
                "▶ ".to_string()
            } else {
                "  ".to_string()
            };

            if is_in_blueprint {
                prefix = format!("{}✓ ", prefix);
            }

            ListItem::new(format!("{}{}", prefix, vpc.display())).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn draw_security_group_select(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.settings.language;
    let region = &REGIONS[app.selected_region];
    let title = format!(" Security Group [{} - {}] ", region.code, region.name(lang));

    if app.security_groups.is_empty() {
        let para = Paragraph::new(app.i18n.no_security_groups())
            .block(Block::default().title(title).borders(Borders::ALL));
        frame.render_widget(para, area);
        return;
    }

    let items: Vec<ListItem> = app
        .security_groups
        .iter()
        .enumerate()
        .map(|(i, sg)| {
            let is_in_blueprint = app.current_blueprint.as_ref().is_some_and(|bp| {
                bp.resources.iter().any(|r| {
                    r.resource_id == sg.id && r.resource_type == ResourceType::SecurityGroup
                })
            });

            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let mut prefix = if i == app.selected_index {
                "▶ ".to_string()
            } else {
                "  ".to_string()
            };

            if is_in_blueprint {
                prefix = format!("{}✓ ", prefix);
            }

            ListItem::new(format!("{}{}", prefix, sg.name)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn draw_load_balancer_select(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.settings.language;
    let region = &REGIONS[app.selected_region];
    let title = format!(" Load Balancer [{} - {}] ", region.code, region.name(lang));

    if app.load_balancers.is_empty() {
        let para = Paragraph::new(app.i18n.no_load_balancers())
            .block(Block::default().title(title).borders(Borders::ALL));
        frame.render_widget(para, area);
        return;
    }

    let items: Vec<ListItem> = app
        .load_balancers
        .iter()
        .enumerate()
        .map(|(i, lb)| {
            let is_in_blueprint = app.current_blueprint.as_ref().is_some_and(|bp| {
                bp.resources.iter().any(|r| {
                    r.resource_id == lb.id && r.resource_type == ResourceType::LoadBalancer
                })
            });

            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let mut prefix = if i == app.selected_index {
                "▶ ".to_string()
            } else {
                "  ".to_string()
            };

            if is_in_blueprint {
                prefix = format!("{}✓ ", prefix);
            }

            ListItem::new(format!("{}{}", prefix, lb.display())).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn draw_ecr_select(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.settings.language;
    let region = &REGIONS[app.selected_region];
    let title = format!(" ECR [{} - {}] ", region.code, region.name(lang));

    if app.ecr_repositories.is_empty() {
        let para = Paragraph::new(app.i18n.no_ecr_repos())
            .block(Block::default().title(title).borders(Borders::ALL));
        frame.render_widget(para, area);
        return;
    }

    let items: Vec<ListItem> = app
        .ecr_repositories
        .iter()
        .enumerate()
        .map(|(i, repo)| {
            let is_in_blueprint = app.current_blueprint.as_ref().is_some_and(|bp| {
                bp.resources
                    .iter()
                    .any(|r| r.resource_id == repo.id && r.resource_type == ResourceType::Ecr)
            });

            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let mut prefix = if i == app.selected_index {
                "▶ ".to_string()
            } else {
                "  ".to_string()
            };

            if is_in_blueprint {
                prefix = format!("{}✓ ", prefix);
            }

            ListItem::new(format!("{}{}", prefix, repo.name)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn draw_blueprint_select(frame: &mut Frame, app: &App, area: Rect) {
    let i = &app.i18n;
    let title = format!(" {} ", i.blueprint());

    let mut items: Vec<ListItem> = Vec::new();

    // "새 블루프린터" 항목
    let new_bp_style = if app.selected_blueprint_index == 0 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };
    let new_bp_prefix = if app.selected_blueprint_index == 0 {
        "▶ "
    } else {
        "  "
    };
    items
        .push(ListItem::new(format!("{}{}", new_bp_prefix, i.new_blueprint())).style(new_bp_style));

    // 기존 블루프린터 목록
    for (idx, bp) in app.blueprint_store.blueprints.iter().enumerate() {
        let style = if idx + 1 == app.selected_blueprint_index {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if idx + 1 == app.selected_blueprint_index {
            "▶ "
        } else {
            "  "
        };
        let resource_count = bp.resources.len();
        items.push(
            ListItem::new(format!(
                "{}{} ({} {})",
                prefix,
                bp.name,
                resource_count,
                i.resources()
            ))
            .style(style),
        );
    }

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn draw_blueprint_detail(frame: &mut Frame, app: &App, area: Rect) {
    let i = &app.i18n;
    let bp = match &app.current_blueprint {
        Some(bp) => bp,
        None => {
            let title = format!(" {} ", i.blueprint());
            let para = Paragraph::new(i.blueprint_load_failed())
                .block(Block::default().title(title).borders(Borders::ALL));
            frame.render_widget(para, area);
            return;
        }
    };

    let title = format!(" {}: {} ", i.blueprint(), bp.name);

    if bp.resources.is_empty() {
        let content = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}.", i.no_resources()),
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", i.press_a_to_add()),
                Style::default().fg(Color::Cyan),
            )),
        ];
        let para =
            Paragraph::new(content).block(Block::default().title(title).borders(Borders::ALL));
        frame.render_widget(para, area);
        return;
    }

    let items: Vec<ListItem> = bp
        .resources
        .iter()
        .enumerate()
        .map(|(i, res)| {
            let style = if i == app.blueprint_resource_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if i == app.blueprint_resource_index {
                "▶ "
            } else {
                "  "
            };

            let type_color = match res.resource_type {
                ResourceType::Ec2 => Color::Cyan,
                ResourceType::Network => Color::Green,
                ResourceType::SecurityGroup => Color::Magenta,
                ResourceType::LoadBalancer => Color::Blue,
                ResourceType::Ecr => Color::LightRed,
                ResourceType::Asg => Color::LightCyan,
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(
                    format!("[{}] ", res.resource_type.display()),
                    Style::default().fg(type_color),
                ),
                Span::styled(format!("{} ({})", res.resource_name, res.region), style),
            ]))
        })
        .collect();

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn draw_blueprint_name_input(frame: &mut Frame, app: &App, area: Rect) {
    let i = &app.i18n;
    let title = format!(" {} ", i.new_blueprint());

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", i.enter_blueprint_name()),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(format!("  > {}_", app.input_buffer)),
    ];

    let para = Paragraph::new(content).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(para, area);
}

fn draw_blueprint_preview(frame: &mut Frame, app: &App, area: Rect) {
    let i = &app.i18n;
    let title = format!(
        " {} {} - {} ",
        i.blueprint(),
        i.preview(),
        app.preview_filename
    );
    let para = Paragraph::new(app.preview_content.as_str())
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0))
        .block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(para, area);
}

fn draw_blueprint_loading(frame: &mut Frame, app: &App, area: Rect, current_index: usize) {
    let i = &app.i18n;
    let bp = match &app.current_blueprint {
        Some(bp) => bp,
        None => {
            let title = format!(" {} ", i.loading());
            let para = Paragraph::new(i.loading_msg())
                .block(Block::default().title(title).borders(Borders::ALL));
            frame.render_widget(para, area);
            return;
        }
    };

    let total = bp.resources.len();

    fn item(done: bool, loading: bool, text: String) -> Line<'static> {
        let (check, color) = if done {
            ("  ✓ ", Color::Green)
        } else if loading {
            ("  ▸ ", Color::Cyan)
        } else {
            ("  ○ ", Color::DarkGray)
        };
        let text_color = if done {
            Color::Green
        } else if loading {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        Line::from(vec![
            Span::styled(check, Style::default().fg(color)),
            Span::styled(text, Style::default().fg(text_color)),
        ])
    }

    let mut content = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "  {}... ({}/{})",
                i.loading_blueprint_resources(),
                current_index,
                total
            ),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (idx, res) in bp.resources.iter().enumerate() {
        let done = idx < current_index;
        let loading = idx == current_index;
        content.push(item(done, loading, res.display()));
    }

    let title = format!(" {} {} ", i.blueprint(), i.loading());
    let para = Paragraph::new(content).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(para, area);
}

fn draw_settings(frame: &mut Frame, app: &App, area: Rect) {
    let i = &app.i18n;
    let title = format!(" {} ", i.settings());

    let lang_current = app.settings.language.display();
    let lang_next = app.settings.language.toggle().display();

    let items: Vec<ListItem> = vec![ListItem::new(Line::from(vec![
        Span::styled(
            if app.selected_setting == 0 {
                "▶ "
            } else {
                "  "
            },
            if app.selected_setting == 0 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            },
        ),
        Span::styled(
            format!("{}: ", i.language()),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            lang_current,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" → {}", lang_next),
            Style::default().fg(Color::DarkGray),
        ),
    ]))];

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn draw_asg_select(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.settings.language;
    let region = &REGIONS[app.selected_region];
    let title = format!(
        " {} [{} - {}] ",
        app.i18n.auto_scaling_group(),
        region.code,
        region.name(lang)
    );

    if app.auto_scaling_groups.is_empty() {
        let para = Paragraph::new(app.i18n.no_asgs())
            .block(Block::default().title(title).borders(Borders::ALL));
        frame.render_widget(para, area);
        return;
    }

    let items: Vec<ListItem> = app
        .auto_scaling_groups
        .iter()
        .enumerate()
        .map(|(i, asg)| {
            let is_in_blueprint = app.current_blueprint.as_ref().is_some_and(|bp| {
                bp.resources
                    .iter()
                    .any(|r| r.resource_type == ResourceType::Asg && r.resource_id == asg.id)
            });

            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_in_blueprint {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };
            let prefix = if i == app.selected_index {
                "▶ "
            } else if is_in_blueprint {
                "✓ "
            } else {
                "  "
            };

            let _state_style = if is_in_blueprint {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::LightCyan)
            };

            let content = format!("{} [{}]", asg.display(), asg.state);

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(content, style),
            ]))
        })
        .collect();

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}
