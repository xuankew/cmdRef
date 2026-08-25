mod app;
mod data;
mod search;
mod ui;
mod update;

use std::io;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use ratatui::backend::CrosstermBackend;

use app::{App, AppMode, Focus};

/// CmdRef - 交互式命令速查工具
#[derive(Parser)]
#[command(name = "cmdref", version, about = "Interactive command reference tool")]
struct Cli {
    /// 直接搜索指定关键字
    #[arg(short, long)]
    search: Option<String>,

    /// 子命令
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// 检查并更新到最新版本
    Update,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // 处理子命令
    if let Some(Commands::Update) = cli.command {
        update::run_update();
        return Ok(());
    }

    // 加载数据
    let platforms = data::load_all_data();

    // 初始化 App 状态
    let mut app = App::new(platforms);

    // 如果有搜索参数，直接进入搜索模式
    if let Some(query) = cli.search {
        app.enter_search_mode();
        for c in query.chars() {
            app.search_input(c);
        }
    }

    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 运行应用
    let result = run_app(&mut terminal, &mut app);

    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui::layout::draw(frame, app))?;

        // 事件处理
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Ctrl+C 始终退出
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(());
                }

                match app.mode {
                    AppMode::Search => handle_search_input(app, key.code, key.modifiers),
                    AppMode::Normal => handle_normal_input(app, key.code, key.modifiers),
                }

                if app.should_quit {
                    return Ok(());
                }
            }
        }
    }
}

fn handle_normal_input(app: &mut App, key: KeyCode, _modifiers: KeyModifiers) {
    match app.focus {
        Focus::Sidebar => {
            match key {
                KeyCode::Char('j') | KeyCode::Down => app.move_sidebar_down(),
                KeyCode::Char('k') | KeyCode::Up => app.move_sidebar_up(),
                KeyCode::Enter => app.toggle_sidebar_item(),
                KeyCode::Tab => app.switch_focus(),
                KeyCode::Char('/') => app.enter_search_mode(),
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char('1') => app.jump_to_platform(0),
                KeyCode::Char('2') => app.jump_to_platform(1),
                KeyCode::Char('3') => app.jump_to_platform(2),
                KeyCode::Char('4') => app.jump_to_platform(3),
                KeyCode::Esc => app.should_quit = true,
                _ => {}
            }
        }
        Focus::Content => {
            match key {
                KeyCode::Char('j') | KeyCode::Down => app.move_content_down(),
                KeyCode::Char('k') | KeyCode::Up => app.move_content_up(),
                KeyCode::Tab => app.switch_focus(),
                KeyCode::Char('/') => app.enter_search_mode(),
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Esc => { app.focus = Focus::Sidebar; }
                _ => {}
            }
        }
        Focus::Search => {
            // 不应该在 Normal 模式下到达这里
        }
    }
}

fn handle_search_input(app: &mut App, key: KeyCode, _modifiers: KeyModifiers) {
    match key {
        KeyCode::Esc => app.exit_search_mode(),
        KeyCode::Enter => {
            if !app.search_results.is_empty() {
                app.select_search_result();
            }
        }
        KeyCode::Up => app.move_search_up(),
        KeyCode::Down => app.move_search_down(),
        KeyCode::Backspace => app.search_backspace(),
        KeyCode::Char(c) => app.search_input(c),
        _ => {}
    }
}
