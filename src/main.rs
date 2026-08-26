#[macro_use]
mod debug;
mod app;
mod bookmarks;
mod clipboard;
mod data;
mod history;
mod search;
mod ui;
mod update;

use std::io;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use ratatui::backend::CrosstermBackend;

use app::{App, AppMode, Focus};

/// CmdRef - 交互式命令速查工具
#[derive(Debug, Parser)]
#[command(name = "cmdref", version, about = "Interactive command reference tool")]
struct Cli {
    /// 直接搜索指定关键字
    #[arg(short, long)]
    search: Option<String>,

    /// 子命令
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    /// 检查并更新到最新版本
    Update,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // 初始化调试日志（设置 CMDREF_DEBUG=1 启用）
    debug::init();
    debug_log!("CmdRef starting, args: {:?}", cli);

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
            match event::read()? {
                Event::Key(key) => {
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
                Event::Mouse(mouse) => {
                    if mouse.kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                        let sz = terminal.size()?;
                        let area = ratatui::prelude::Rect::new(0, 0, sz.width, sz.height);
                        handle_mouse_click(app, mouse.column, mouse.row, area);
                    }
                }
                _ => {}
            }
        }

        // 每帧更新（状态消息衰减等）
        app.tick();
    }
}

fn handle_normal_input(app: &mut App, key: KeyCode, _modifiers: KeyModifiers) {
    match app.focus {
        Focus::Sidebar => {
            match key {
                KeyCode::Char('j') | KeyCode::Down => app.move_sidebar_down(),
                KeyCode::Char('k') | KeyCode::Up => app.move_sidebar_up(),
                KeyCode::Enter | KeyCode::Right => app.toggle_sidebar_item(),
                KeyCode::Tab => app.switch_focus(),
                KeyCode::Char('/') => app.enter_search_mode(),
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char('B') => app.jump_to_bookmarks(),
                KeyCode::Char('H') => app.jump_to_history(),
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
                KeyCode::Left | KeyCode::Tab => app.switch_focus(),
                KeyCode::Char('/') => app.enter_search_mode(),
                KeyCode::Char('y') => app.copy_current_command(),
                KeyCode::Char('b') => app.toggle_bookmark(),
                KeyCode::Char('B') => app.jump_to_bookmarks(),
                KeyCode::Char('H') => app.jump_to_history(),
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

/// 处理鼠标点击事件
fn handle_mouse_click(app: &mut App, col: u16, row: u16, size: ratatui::prelude::Rect) {
    if app.mode == AppMode::Search {
        return;
    }

    let height = size.height;
    let width = size.width;

    // 布局计算（与 ui/layout.rs 一致）
    let title_row = 0u16;
    let help_row = height.saturating_sub(1);
    let main_top = title_row + 1;
    let main_bottom = help_row;

    // 不在主区域内则忽略
    if row < main_top || row >= main_bottom {
        return;
    }

    let sidebar_width = width * 25 / 100;

    // 点击侧边栏
    if col < sidebar_width {
        app.focus = Focus::Sidebar;
        // 侧边栏有 1 行 border，内部从 main_top+1 开始
        let inner_row = row.saturating_sub(main_top + 1);
        let cursor = inner_row as usize;
        if cursor < app.sidebar_items.len() {
            app.sidebar_cursor = cursor;
            app.update_selection();
        }
        return;
    }

    // 点击内容区域
    let content_left = sidebar_width;
    let content_width = width - sidebar_width;
    let cmd_list_width = content_width * 30 / 100;

    // 点击命令列表区域（左侧 30%）
    if col < content_left + cmd_list_width {
        app.focus = Focus::Content;
        // 命令列表有 border (1行) + 列表项从第 2 行开始
        let inner_row = row.saturating_sub(main_top + 1);
        let idx = inner_row as usize;
        let len = app.current_category_commands().len();
        if len > 0 && idx < len {
            app.selected_command = Some(idx);
            app.content_cursor = idx;
            // 点击即复制
            app.copy_current_command();
        }
    } else {
        // 点击详情区域，切换焦点到内容
        app.focus = Focus::Content;
    }
}
