pub mod widgets;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::{AppState, Screen};

pub fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, chunks[0]);
    render_content(frame, chunks[1], state);
    render_footer(frame, chunks[2], state);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("☁️  AWS CLI Installer")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
    frame.render_widget(title, area);
}

fn render_content(frame: &mut Frame, area: Rect, state: &AppState) {
    let content = match &state.screen {
        Screen::Welcome => render_welcome(),
        Screen::Menu => render_menu(),
        Screen::Detecting => render_detecting(),
        Screen::Confirm => render_confirm(state),
        Screen::Downloading => render_downloading(frame, area, state),
        Screen::Installing => render_installing(),
        Screen::Complete => render_complete(state),
        Screen::Uninstalling => render_uninstalling(),
        Screen::UninstallComplete => render_uninstall_complete(),
        Screen::Error => render_error(state),
    };

    if state.screen != Screen::Downloading {
        frame.render_widget(content, area);
    }
}

fn render_welcome() -> Paragraph<'static> {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Welcome to AWS CLI v2 Manager",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Install or Uninstall AWS CLI v2"),
        Line::from("for your current platform."),
        Line::from(""),
        Line::from(Span::styled(
            "Press ENTER to continue...",
            Style::default().fg(Color::Yellow),
        )),
    ];
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
}

fn render_menu() -> Paragraph<'static> {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Select Action",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [1] ", Style::default().fg(Color::Cyan)),
            Span::raw("Install AWS CLI"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [2] ", Style::default().fg(Color::Red)),
            Span::raw("Uninstall AWS CLI"),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Press 1 or 2 to select, Q to quit",
            Style::default().fg(Color::Yellow),
        )),
    ];
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
}

fn render_detecting() -> Paragraph<'static> {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "🔍 Detecting platform...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
}

fn render_confirm(state: &AppState) -> Paragraph<'static> {
    let platform_name = state
        .platform
        .as_ref()
        .map(|p| p.display_name())
        .unwrap_or_else(|| "Unknown".to_string());

    let download_url = state
        .platform
        .as_ref()
        .map(|p| p.download_url().to_string())
        .unwrap_or_default();

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "✅ Platform Detected",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  OS/Arch: "),
            Span::styled(platform_name, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Source: "),
            Span::styled(download_url, Style::default().fg(Color::Blue)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Press ENTER to start download, or Q to quit",
            Style::default().fg(Color::Yellow),
        )),
    ];
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
}

fn render_downloading(frame: &mut Frame, area: Rect, state: &AppState) -> Paragraph<'static> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new(Span::styled(
        "📥 Downloading AWS CLI...",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let (percentage, label) = if let Some(progress) = &state.download_progress {
        if let Some(pct) = progress.percentage() {
            let downloaded_mb = progress.downloaded as f64 / 1_048_576.0;
            let total_mb = progress
                .total
                .map(|t| t as f64 / 1_048_576.0)
                .unwrap_or(0.0);
            (
                pct / 100.0,
                format!("{:.1} MB / {:.1} MB ({:.0}%)", downloaded_mb, total_mb, pct),
            )
        } else {
            let downloaded_mb = progress.downloaded as f64 / 1_048_576.0;
            (0.0, format!("{:.1} MB downloaded", downloaded_mb))
        }
    } else {
        (0.0, "Starting...".to_string())
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .ratio(percentage.min(1.0))
        .label(label);
    frame.render_widget(gauge, chunks[1]);

    Paragraph::new("")
}

fn render_installing() -> Paragraph<'static> {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "⚙️  Installing AWS CLI...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("This may require administrator privileges."),
        Line::from("Please follow any prompts that appear."),
    ];
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
}

fn render_complete(state: &AppState) -> Paragraph<'static> {
    let version = state
        .aws_version
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "🎉 Installation Complete!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Version: "),
            Span::styled(version, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Press Q to exit",
            Style::default().fg(Color::Yellow),
        )),
    ];
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
}

fn render_error(state: &AppState) -> Paragraph<'static> {
    let error_msg = state
        .error_message
        .clone()
        .unwrap_or_else(|| "Unknown error".to_string());

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "❌ Error",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(error_msg, Style::default().fg(Color::Red))),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Press Q to exit",
            Style::default().fg(Color::Yellow),
        )),
    ];
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
}

fn render_uninstalling() -> Paragraph<'static> {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "🗑️  Uninstalling AWS CLI...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("This may require administrator privileges."),
        Line::from("Please follow any prompts that appear."),
    ];
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
}

fn render_uninstall_complete() -> Paragraph<'static> {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "✅ Uninstall Complete!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("AWS CLI has been removed from your system."),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Press Q to exit",
            Style::default().fg(Color::Yellow),
        )),
    ];
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let platform_info = state
        .platform
        .as_ref()
        .map(|p| p.display_name())
        .unwrap_or_else(|| "Detecting...".to_string());

    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled(" Platform: ", Style::default().fg(Color::DarkGray)),
        Span::styled(platform_info, Style::default().fg(Color::Gray)),
        Span::raw("  |  "),
        Span::styled("Q: Quit", Style::default().fg(Color::DarkGray)),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(footer, area);
}
