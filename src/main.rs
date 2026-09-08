#[macro_use]
mod debug;
mod app;
mod bookmarks;
mod clipboard;
mod crypto;
mod data;
mod history;
mod paths;
mod passwords;
mod prompts;
mod search;
mod sync;
mod ui;
mod update;

use std::io;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use ratatui::backend::CrosstermBackend;

use app::{App, AppMode, Focus, SidebarItemKind};

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
    /// 显示当前数据目录路径
    Path,
    /// 云端同步（GitHub / WebDAV）
    Sync {
        #[command(subcommand)]
        command: sync::SyncCommand,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // 初始化调试日志（设置 CMDREF_DEBUG=1 启用）
    debug::init();
    debug_log!("CmdRef starting, args: {:?}", cli);

    // 处理子命令
    if let Some(command) = cli.command {
        match command {
            Commands::Update => {
                update::run_update();
                return Ok(());
            }
            Commands::Path => {
                println!("{}", crate::paths::data_dir().display());
                return Ok(());
            }
            Commands::Sync { command } => {
                if let Err(e) = sync::run(command) {
                    eprintln!("✗ {}", e);
                    std::process::exit(1);
                }
                return Ok(());
            }
        }
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
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 运行应用
    let result = run_app(&mut terminal, &mut app);

    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
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

                match &app.mode {
                    AppMode::Search => handle_search_input(app, key.code, key.modifiers),
                    AppMode::AddCommand(_) => handle_editor_input(app, key.code, key.modifiers),
                    AppMode::PromptEditor(_) => handle_prompt_editor_input(app, key.code, key.modifiers),
                    AppMode::PasswordEditor(_) => handle_password_editor_input(app, key.code, key.modifiers),
                    AppMode::MasterPassword(_) => handle_master_password_input(app, key.code, key.modifiers),
                    AppMode::Normal => handle_normal_input(app, key.code, key.modifiers),
                }

                if app.should_quit {
                    return Ok(());
                }
            }
        }

        // 每帧更新（状态消息衰减等）
        app.tick();
    }
}

fn handle_normal_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    match app.focus {
        Focus::Sidebar => {
            match key {
                KeyCode::Char('j') | KeyCode::Down => app.move_sidebar_down(),
                KeyCode::Char('k') | KeyCode::Up => app.move_sidebar_up(),
                KeyCode::Enter | KeyCode::Right => app.toggle_sidebar_item(),
                KeyCode::Tab => app.switch_focus(),
                KeyCode::Char('/') => app.enter_search_mode(),
                KeyCode::Char('n') => {
                    let kind = app.sidebar_items.get(app.sidebar_cursor)
                        .map(|i| &i.kind);
                    match kind {
                        Some(SidebarItemKind::Prompts) => app.open_add_prompt(),
                        Some(SidebarItemKind::Passwords) => app.open_add_password(),
                        _ => app.open_add_command(),
                    }
                }
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char('B') => app.jump_to_bookmarks(),
                KeyCode::Char('H') => app.jump_to_history(),
                KeyCode::Char('P') => app.jump_to_prompts(),
                KeyCode::Char('S') => app.jump_to_passwords(),
                KeyCode::Char('1') => app.jump_to_platform(0),
                KeyCode::Char('2') => app.jump_to_platform(1),
                KeyCode::Char('3') => app.jump_to_platform(2),
                KeyCode::Char('4') => app.jump_to_platform(3),
                KeyCode::Char('5') => app.jump_to_platform(4),
                KeyCode::Esc => app.should_quit = true,
                _ => {}
            }
        }
        Focus::Content => {
            // Ctrl+D / Ctrl+U 滚动右侧详情区
            if modifiers.contains(KeyModifiers::CONTROL) {
                match key {
                    KeyCode::Char('d') => { app.scroll_detail_down(5); return; }
                    KeyCode::Char('u') => { app.scroll_detail_up(5); return; }
                    _ => {}
                }
            }

            let is_prompts = app.sidebar_items.get(app.sidebar_cursor)
                .map(|i| i.kind == SidebarItemKind::Prompts)
                .unwrap_or(false);
            let is_passwords = app.sidebar_items.get(app.sidebar_cursor)
                .map(|i| i.kind == SidebarItemKind::Passwords)
                .unwrap_or(false);

            if is_prompts {
                match key {
                    KeyCode::Char('j') | KeyCode::Down => app.move_content_down(),
                    KeyCode::Char('k') | KeyCode::Up => app.move_content_up(),
                    KeyCode::Left | KeyCode::Tab => app.switch_focus(),
                    KeyCode::Char('/') => app.enter_search_mode(),
                    KeyCode::Char('y') | KeyCode::Enter => app.copy_current_prompt(),
                    KeyCode::Char('n') => app.open_add_prompt(),
                    KeyCode::Char('e') => app.open_edit_prompt(),
                    KeyCode::Char('d') => app.delete_current_prompt(),
                    KeyCode::Char('B') => app.jump_to_bookmarks(),
                    KeyCode::Char('H') => app.jump_to_history(),
                    KeyCode::Char('P') => app.jump_to_prompts(),
                    KeyCode::Char('S') => app.jump_to_passwords(),
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Esc => { app.focus = Focus::Sidebar; }
                    _ => {}
                }
            } else if is_passwords {
                match key {
                    KeyCode::Char('j') | KeyCode::Down => app.move_content_down(),
                    KeyCode::Char('k') | KeyCode::Up => app.move_content_up(),
                    KeyCode::Left | KeyCode::Tab => app.switch_focus(),
                    KeyCode::Char('/') => app.enter_search_mode(),
                    KeyCode::Char('y') | KeyCode::Enter => app.copy_current_password(),
                    KeyCode::Char('n') => app.open_add_password(),
                    KeyCode::Char('e') => app.open_edit_password(),
                    KeyCode::Char('d') => app.delete_current_password(),
                    KeyCode::Char('C') => app.change_master_password(),
                    KeyCode::Char('B') => app.jump_to_bookmarks(),
                    KeyCode::Char('H') => app.jump_to_history(),
                    KeyCode::Char('P') => app.jump_to_prompts(),
                    KeyCode::Char('S') => app.jump_to_passwords(),
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Esc => { app.focus = Focus::Sidebar; }
                    _ => {}
                }
            } else {
                match key {
                    KeyCode::Char('j') | KeyCode::Down => app.move_content_down(),
                    KeyCode::Char('k') | KeyCode::Up => app.move_content_up(),
                    KeyCode::Left | KeyCode::Tab => app.switch_focus(),
                    KeyCode::Char('/') => app.enter_search_mode(),
                    KeyCode::Char('b') => app.toggle_bookmark(),
                    KeyCode::Char('n') => app.open_add_command(),
                    KeyCode::Char('d') => app.delete_custom_command(),
                    KeyCode::Char('B') => app.jump_to_bookmarks(),
                    KeyCode::Char('H') => app.jump_to_history(),
                    KeyCode::Char('P') => app.jump_to_prompts(),
                    KeyCode::Char('S') => app.jump_to_passwords(),
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Esc => { app.focus = Focus::Sidebar; }
                    KeyCode::Char(c @ '1'..='9') => {
                        let idx = (c as usize) - ('0' as usize);
                        app.copy_example_by_index(idx);
                    }
                    _ => {}
                }
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

fn handle_editor_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    match key {
        KeyCode::Esc => app.editor_cancel(),
        KeyCode::Enter => {
            // Command 字段支持多行换行，其他字段触发保存
            let is_command = if let AppMode::AddCommand(ref state) = app.mode {
                state.active_field == app::EditorField::Command
            } else {
                false
            };
            if is_command {
                app.editor_input('\n');
            } else {
                app.editor_save();
            }
        }
        KeyCode::Tab => {
            if modifiers.contains(KeyModifiers::SHIFT) {
                app.editor_prev_field();
            } else {
                app.editor_next_field();
            }
        }
        KeyCode::BackTab => app.editor_prev_field(),
        KeyCode::Backspace => app.editor_backspace(),
        KeyCode::Char(c) => app.editor_input(c),
        _ => {}
    }
}

fn handle_prompt_editor_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    match key {
        KeyCode::Esc => app.prompt_editor_cancel(),
        KeyCode::Enter => {
            // Content 字段支持多行换行，其他字段跳到下一项
            let is_content = if let AppMode::PromptEditor(ref state) = app.mode {
                state.active_field == app::PromptField::Content
            } else {
                false
            };
            if is_content {
                app.prompt_editor_input('\n');
            } else {
                app.prompt_editor_next_field();
            }
        }
        KeyCode::Char('s') => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                app.prompt_editor_save();
            } else {
                app.prompt_editor_input('s');
            }
        }
        KeyCode::Tab => {
            if modifiers.contains(KeyModifiers::SHIFT) {
                app.prompt_editor_prev_field();
            } else {
                app.prompt_editor_next_field();
            }
        }
        KeyCode::BackTab => app.prompt_editor_prev_field(),
        KeyCode::Backspace => app.prompt_editor_backspace(),
        KeyCode::Char(c) => app.prompt_editor_input(c),
        _ => {}
    }
}

fn handle_password_editor_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    match key {
        KeyCode::Esc => app.password_editor_cancel(),
        KeyCode::Enter => {
            let is_last = if let AppMode::PasswordEditor(ref state) = app.mode {
                state.active_field == app::PasswordField::Description
            } else {
                false
            };
            if is_last {
                app.password_editor_save();
            } else {
                app.password_editor_next_field();
            }
        }
        KeyCode::Char('s') => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                app.password_editor_save();
            } else {
                app.password_editor_input('s');
            }
        }
        KeyCode::Tab => {
            if modifiers.contains(KeyModifiers::SHIFT) {
                app.password_editor_prev_field();
            } else {
                app.password_editor_next_field();
            }
        }
        KeyCode::BackTab => app.password_editor_prev_field(),
        KeyCode::Backspace => app.password_editor_backspace(),
        KeyCode::Char(c) => app.password_editor_input(c),
        _ => {}
    }
}

fn handle_master_password_input(app: &mut App, key: KeyCode, _modifiers: KeyModifiers) {
    match key {
        KeyCode::Esc => app.master_password_cancel(),
        KeyCode::Enter => app.master_password_submit(),
        KeyCode::Tab | KeyCode::BackTab => app.master_password_next_field(),
        KeyCode::Backspace => app.master_password_backspace(),
        KeyCode::Char(c) => app.master_password_input(c),
        _ => {}
    }
}
