use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, LoadingTask, REGIONS, SERVICES, Screen};

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
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_header(frame, chunks[0]);
    draw_main(frame, app, chunks[1]);
    draw_footer(frame, app, chunks[2]);
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
    let help = match &app.screen {
        Screen::Login => "Enter: 재시도 | q: 종료",
        Screen::BlueprintSelect => {
            "↑↓/jk: 이동 | Enter: 선택 | g: 마크다운 생성 | d: 삭제 | s: 단일 모드 | q: 종료"
        }
        Screen::BlueprintDetail => {
            "↑↓/jk: 이동 | Shift+↑↓/JK: 순서변경 | a: 추가 | d: 삭제 | g/Enter: 생성 | Esc: 뒤로 | q: 종료"
        }
        Screen::BlueprintNameInput => "Enter: 확인 | Esc: 취소",
        Screen::BlueprintPreview => {
            "↑↓/jk: 스크롤 | PgUp/PgDn: 페이지 | Home/End | Enter/s: 저장 | Esc: 뒤로 | q: 종료"
        }
        Screen::RegionSelect => "↑↓/jk: 이동 | Enter: 선택 | Esc: 뒤로 | q: 종료",
        Screen::ServiceSelect => "↑↓/jk: 이동 | Enter: 선택 | Esc: 뒤로 | q: 종료",
        Screen::Ec2Select => "↑↓/jk: 이동 | Enter: 선택 | r: 새로고침 | Esc: 뒤로 | q: 종료",
        Screen::VpcSelect => "↑↓/jk: 이동 | Enter: 선택 | r: 새로고침 | Esc: 뒤로 | q: 종료",
        Screen::SecurityGroupSelect => {
            "↑↓/jk: 이동 | Enter: 선택 | r: 새로고침 | Esc: 뒤로 | q: 종료"
        }
        Screen::Preview => {
            if app.blueprint_mode {
                "↑↓/jk: 스크롤 | Enter/s: 저장 | a: 블루프린터에 추가 | r: 새로고침 | Esc: 뒤로 | q: 종료"
            } else {
                "↑↓/jk: 스크롤 | Enter/s: 저장 | r: 새로고침 | Esc: 뒤로 | q: 종료"
            }
        }
        Screen::LoadBalancerSelect => {
            "↑↓/jk: 이동 | Enter: 선택 | r: 새로고침 | Esc: 뒤로 | q: 종료"
        }
    };

    let msg = if app.message.is_empty() {
        help.to_string()
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
    }
}

fn draw_loading(frame: &mut Frame, app: &App, area: Rect) {
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
        LoadingTask::None => "처리 중",
        LoadingTask::RefreshEc2 => "EC2 목록 새로고침 중",
        LoadingTask::RefreshVpc => "Network 목록 새로고침 중",
        LoadingTask::RefreshSecurityGroup => "Security Group 목록 새로고침 중",
        LoadingTask::RefreshPreview => "미리보기 새로고침 중",
        LoadingTask::LoadEc2 => "EC2 인스턴스 목록 조회 중",
        LoadingTask::LoadVpc => "Network(VPC) 목록 조회 중",
        LoadingTask::LoadSecurityGroup => "Security Group 목록 조회 중",
        LoadingTask::LoadEc2Detail(_) => "EC2 상세 정보 조회 중",
        LoadingTask::LoadVpcDetail(_, _) => "Network 상세 정보 조회 중",
        LoadingTask::LoadSecurityGroupDetail(_) => "Security Group 상세 정보 조회 중",
        LoadingTask::RefreshLoadBalancer => "Load Balancer 목록 새로고침 중",
        LoadingTask::LoadLoadBalancer => "Load Balancer 목록 조회 중",
        LoadingTask::LoadLoadBalancerDetail(_) => "Load Balancer 상세 정보 조회 중",
        LoadingTask::LoadBlueprintResources(_) => "블루프린트 리소스 조회 중",
    };

    let content = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "    ⏳ 로딩 중...",
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
            "    AWS CLI 응답 대기 중입니다.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let para =
        Paragraph::new(content).block(Block::default().title(" 로딩 ").borders(Borders::ALL));
    frame.render_widget(para, area);
}

fn draw_vpc_loading_checklist(frame: &mut Frame, app: &App, area: Rect) {
    let p = &app.loading_progress;

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

    fn item(done: bool, loading: bool, text: &'static str) -> Line<'static> {
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

    let current_task = match current_step {
        0 => "VPC 기본 정보",
        1 => "서브넷",
        2 => "인터넷 게이트웨이",
        3 => "NAT 게이트웨이",
        4 => "라우팅 테이블",
        5 => "Elastic IP",
        6 => "DNS 설정",
        _ => "완료 중",
    };

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  ⏳ Network 상세 정보 조회 중...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        item(p.vpc_info, current_step == 0, "VPC 기본 정보"),
        item(p.subnets, current_step == 1, "서브넷"),
        item(p.igws, current_step == 2, "인터넷 게이트웨이"),
        item(p.nats, current_step == 3, "NAT 게이트웨이"),
        item(p.route_tables, current_step == 4, "라우팅 테이블"),
        item(p.eips, current_step == 5, "Elastic IP"),
        item(p.dns_attrs, current_step == 6, "DNS 설정"),
        Line::from(""),
        Line::from(Span::styled(
            format!("  현재: {} 조회 중...", current_task),
            Style::default().fg(Color::Cyan),
        )),
    ];

    let para = Paragraph::new(content).block(
        Block::default()
            .title(" Network 로딩 ")
            .borders(Borders::ALL),
    );
    frame.render_widget(para, area);
}

fn draw_login(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(ref info) = app.login_info {
        vec![
            Line::from(Span::styled(
                "✓ AWS 로그인 확인됨",
                Style::default().fg(Color::Green),
            )),
            Line::from(""),
            Line::from(info.as_str()),
        ]
    } else if let Some(ref err) = app.login_error {
        vec![
            Line::from(Span::styled(
                "✗ AWS 로그인 필요",
                Style::default().fg(Color::Red),
            )),
            Line::from(""),
            Line::from(err.as_str()),
            Line::from(""),
            Line::from("aws configure 또는 aws sso login을 실행하세요."),
        ]
    } else {
        vec![Line::from("AWS CLI 로그인 확인 중...")]
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

    let para = Paragraph::new(login_content)
        .block(Block::default().title(" 로그인 ").borders(Borders::ALL))
        .alignment(Alignment::Center);
    frame.render_widget(para, area);
}

fn draw_region_select(frame: &mut Frame, app: &App, area: Rect) {
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
            ListItem::new(format!("{}{} ({})", prefix, r.code, r.name)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().title(" 리전 ").borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn draw_service_select(frame: &mut Frame, app: &App, area: Rect) {
    let region = &REGIONS[app.selected_region];
    let title = format!(" 서비스 [{} - {}] ", region.code, region.name);

    let items: Vec<ListItem> = SERVICES
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.selected_service {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if i == app.selected_service {
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
    let region = &REGIONS[app.selected_region];
    let title = format!(" EC2 인스턴스 [{} - {}] ", region.code, region.name);

    if app.instances.is_empty() {
        let para = Paragraph::new("인스턴스가 없습니다.")
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
    let title = format!(" 미리보기 - {} ", app.preview_filename);
    let para = Paragraph::new(app.preview_content.as_str())
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0))
        .block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(para, area);
}

fn draw_vpc_select(frame: &mut Frame, app: &App, area: Rect) {
    let region = &REGIONS[app.selected_region];
    let title = format!(" Network [{} - {}] ", region.code, region.name);

    if app.vpcs.is_empty() {
        let para = Paragraph::new("Network가 없습니다.")
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
    let region = &REGIONS[app.selected_region];
    let title = format!(" Security Group [{} - {}] ", region.code, region.name);

    if app.security_groups.is_empty() {
        let para = Paragraph::new("Security Group이 없습니다.")
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
    let region = &REGIONS[app.selected_region];
    let title = format!(" Load Balancer [{} - {}] ", region.code, region.name);

    if app.load_balancers.is_empty() {
        let para = Paragraph::new("Load Balancer가 없습니다.")
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

fn draw_blueprint_select(frame: &mut Frame, app: &App, area: Rect) {
    let title = " 블루프린터 ";

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
    items.push(ListItem::new(format!("{}+ 새 블루프린터", new_bp_prefix)).style(new_bp_style));

    // 기존 블루프린터 목록
    for (i, bp) in app.blueprint_store.blueprints.iter().enumerate() {
        let style = if i + 1 == app.selected_blueprint_index {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if i + 1 == app.selected_blueprint_index {
            "▶ "
        } else {
            "  "
        };
        let resource_count = bp.resources.len();
        items.push(
            ListItem::new(format!("{}{} ({} 리소스)", prefix, bp.name, resource_count))
                .style(style),
        );
    }

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn draw_blueprint_detail(frame: &mut Frame, app: &App, area: Rect) {
    let bp = match &app.current_blueprint {
        Some(bp) => bp,
        None => {
            let para = Paragraph::new("블루프린터 로드 실패")
                .block(Block::default().title(" 블루프린터 ").borders(Borders::ALL));
            frame.render_widget(para, area);
            return;
        }
    };

    let title = format!(" 블루프린터: {} ", bp.name);

    if bp.resources.is_empty() {
        let content = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  리소스가 없습니다.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  'a' 키를 눌러 리소스를 추가하세요.",
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
    let title = " 새 블루프린터 ";

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  블루프린터 이름을 입력하세요:",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(format!("  > {}_", app.input_buffer)),
    ];

    let para = Paragraph::new(content).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(para, area);
}

fn draw_blueprint_preview(frame: &mut Frame, app: &App, area: Rect) {
    let title = format!(" 블루프린터 미리보기 - {} ", app.preview_filename);
    let para = Paragraph::new(app.preview_content.as_str())
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0))
        .block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(para, area);
}

fn draw_blueprint_loading(frame: &mut Frame, app: &App, area: Rect, current_index: usize) {
    let bp = match &app.current_blueprint {
        Some(bp) => bp,
        None => {
            let para = Paragraph::new("로딩 중...")
                .block(Block::default().title(" 로딩 ").borders(Borders::ALL));
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
                "  블루프린터 리소스 조회 중... ({}/{})",
                current_index, total
            ),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (i, res) in bp.resources.iter().enumerate() {
        let done = i < current_index;
        let loading = i == current_index;
        content.push(item(done, loading, res.display()));
    }

    let para = Paragraph::new(content).block(
        Block::default()
            .title(" 블루프린터 로딩 ")
            .borders(Borders::ALL),
    );
    frame.render_widget(para, area);
}
