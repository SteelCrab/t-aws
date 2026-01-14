mod app;
mod cli;
mod download;
mod error;
mod install;
mod platform;
mod resources;
mod ui;

use std::io;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::{Action, App, Screen};
use cli::{Cli, Command};
use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands first
    if let Some(command) = cli.command {
        match command {
            Command::S3 => {
                cli::print_s3_cheatsheet();
                return Ok(());
            }
            Command::Ec2 => {
                cli::print_ec2_cheatsheet();
                return Ok(());
            }
            Command::Iam => {
                cli::print_iam_cheatsheet();
                return Ok(());
            }
            Command::Resources { region } => {
                if let Err(e) = resources::show_resources(region).await {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
                return Ok(());
            }
        }
    }

    // Handle --install flag
    if cli.install {
        let platform = platform::Platform::detect()?;
        println!("🔍 Platform: {}", platform);
        println!("📥 Downloading AWS CLI...");

        let path = download::download_installer(&platform, |p| {
            if let Some(pct) = p.percentage() {
                print!("\r📥 Downloading... {:.0}%", pct);
                let _ = io::Write::flush(&mut io::stdout());
            }
        })
        .await?;
        println!("\n✅ Download complete!");

        println!("⚙️  Installing AWS CLI...");
        install::install_aws_cli(&platform, &path)?;

        match install::verify_installation() {
            Ok(version) => println!("🎉 Installed: {}", version),
            Err(e) => println!("⚠️  Install succeeded but verification failed: {}", e),
        }
        return Ok(());
    }

    // Handle --uninstall flag
    if cli.uninstall {
        let platform = platform::Platform::detect()?;
        println!("🗑️  Uninstalling AWS CLI...");
        install::uninstall_aws_cli(&platform)?;
        println!("✅ Uninstall complete!");
        return Ok(());
    }

    // Default: Run TUI
    run_tui().await
}

async fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let app = Arc::new(App::new());
    let result = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: Arc<App>,
) -> Result<()> {
    loop {
        let state = app.get_state();

        if state.should_quit {
            break;
        }

        terminal.draw(|f| ui::render(f, &state))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            app.quit();
                        }
                        KeyCode::Enter => {
                            handle_enter(&app).await;
                        }
                        KeyCode::Char('1') => {
                            handle_menu_select(&app, Action::Install).await;
                        }
                        KeyCode::Char('2') => {
                            handle_menu_select(&app, Action::Uninstall).await;
                        }
                        KeyCode::Esc => {
                            app.quit();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_enter(app: &Arc<App>) {
    let state = app.get_state();

    match state.screen {
        Screen::Welcome => {
            app.set_screen(Screen::Menu);
        }
        Screen::Confirm => {
            start_download(app.clone()).await;
        }
        _ => {}
    }
}

async fn handle_menu_select(app: &Arc<App>, action: Action) {
    let state = app.get_state();
    if state.screen != Screen::Menu {
        return;
    }

    app.set_action(action);

    if let Err(e) = app.detect_platform() {
        app.set_error(e.to_string());
        return;
    }

    match action {
        Action::Install => {
            // detect_platform already sets screen to Confirm
        }
        Action::Uninstall => {
            start_uninstall(app.clone()).await;
        }
    }
}

async fn start_uninstall(app: Arc<App>) {
    app.set_screen(Screen::Uninstalling);

    let platform = match app.get_state().platform {
        Some(p) => p,
        None => {
            app.set_error("Platform not detected".to_string());
            return;
        }
    };

    let uninstall_result =
        tokio::task::spawn_blocking(move || install::uninstall_aws_cli(&platform)).await;

    match uninstall_result {
        Ok(Ok(())) => {
            app.set_screen(Screen::UninstallComplete);
        }
        Ok(Err(e)) => {
            app.set_error(e.to_string());
        }
        Err(e) => {
            app.set_error(format!("Uninstall task panicked: {}", e));
        }
    }
}

async fn start_download(app: Arc<App>) {
    app.set_screen(Screen::Downloading);

    let platform = match app.get_state().platform {
        Some(p) => p,
        None => {
            app.set_error("Platform not detected".to_string());
            return;
        }
    };

    let app_clone = app.clone();
    let download_result = download::download_installer(&platform, move |progress| {
        app_clone.set_download_progress(progress);
    })
    .await;

    match download_result {
        Ok(path) => {
            app.set_installer_path(path.clone());
            app.set_screen(Screen::Installing);

            // Run installation in blocking task
            let platform_clone = platform.clone();
            let install_result = tokio::task::spawn_blocking(move || {
                install::install_aws_cli(&platform_clone, &path)
            })
            .await;

            match install_result {
                Ok(Ok(())) => {
                    // Verify installation
                    match install::verify_installation() {
                        Ok(version) => {
                            app.set_aws_version(version);
                            app.set_screen(Screen::Complete);
                        }
                        Err(e) => {
                            app.set_error(format!(
                                "Installation succeeded but verification failed: {}",
                                e
                            ));
                        }
                    }
                }
                Ok(Err(e)) => {
                    app.set_error(e.to_string());
                }
                Err(e) => {
                    app.set_error(format!("Installation task panicked: {}", e));
                }
            }
        }
        Err(e) => {
            app.set_error(e.to_string());
        }
    }
}
