use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, AppMode};
use super::{sidebar, content, search, help};

/// 渲染整个 UI
pub fn draw(frame: &mut Frame, app: &App) {
    if app.mode == AppMode::Search {
        draw_search_layout(frame, app);
    } else {
        draw_main_layout(frame, app);
    }
}

/// 主界面布局
fn draw_main_layout(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // 整体布局: title bar + main area + help bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // 标题栏
            Constraint::Min(10),    // 主内容区
            Constraint::Length(1),  // 底部帮助栏
        ])
        .split(size);

    // 标题栏
    let detected = if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else {
        "Linux"
    };
    let bm_count = app.bookmarks.count();
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" CmdRef ", Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::raw(" - "),
        Span::styled(
            format!("{} commands", app.total_commands()),
            Style::default().fg(Color::Gray),
        ),
        Span::raw("  "),
        Span::styled(
            format!("[{}]", detected),
            Style::default().fg(Color::Cyan),
        ),
        if bm_count > 0 {
            Span::styled(
                format!("  ★{}", bm_count),
                Style::default().fg(Color::Yellow),
            )
        } else {
            Span::raw("")
        },
    ]));
    frame.render_widget(title, main_chunks[0]);

    // 主内容区: sidebar + content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),  // 侧边栏
            Constraint::Percentage(75),  // 内容区
        ])
        .split(main_chunks[1]);

    // 渲染侧边栏
    sidebar::draw(frame, app, content_chunks[0]);

    // 渲染内容区
    content::draw(frame, app, content_chunks[1]);

    // 底部帮助栏
    help::draw(frame, app, main_chunks[2]);
}

/// 搜索界面布局
fn draw_search_layout(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // 标题栏
            Constraint::Length(3),  // 搜索输入框
            Constraint::Min(10),    // 搜索结果
            Constraint::Length(1),  // 底部帮助
        ])
        .split(size);

    // 标题栏
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" CmdRef ", Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::raw(" - "),
        Span::styled("搜索模式", Style::default().fg(Color::Yellow)),
    ]));
    frame.render_widget(title, chunks[0]);

    // 搜索输入框
    search::draw_input(frame, app, chunks[1]);

    // 搜索结果
    search::draw_results(frame, app, chunks[2]);

    // 底部帮助
    let help_text = Paragraph::new(Line::from(vec![
        Span::styled(" Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(":查看详情  "),
        Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(":退出搜索  "),
        Span::styled("↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(":选择结果  "),
    ]));
    frame.render_widget(help_text, chunks[3]);
}
