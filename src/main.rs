mod app;
mod aws_cli;
mod blueprint;
mod cli;
mod handler;
mod i18n;
mod output;
mod settings;
mod ui;
mod update;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io;
use std::time::Duration;
use tracing_appender;
use tracing_subscriber;

use app::App;

fn main() -> io::Result<()> {
    if cli::run().is_some() {
        return Ok(());
    }

    // Setup logging
    let file_appender = tracing_appender::rolling::daily(".", "emd.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    tracing::info!("Application started");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.check_login();

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        tracing::error!("Application error: {}", err);
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    while app.running {
        terminal.draw(|f| ui::draw(f, app))?;

        // 로딩 중이면 실제 작업 수행
        if app.loading {
            handler::process_loading(app);
            continue;
        }

        // 100ms 타임아웃으로 이벤트 폴링
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        handler::handle_key(app, key);
                    }
                }
                Event::Mouse(mouse) => {
                    handler::handle_mouse(app, mouse);
                }
                _ => {}
            }
        }
    }
    Ok(())
}
