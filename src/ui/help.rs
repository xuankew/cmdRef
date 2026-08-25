use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, Focus};

/// 渲染底部帮助栏
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let help = match app.focus {
        Focus::Content => Paragraph::new(Line::from(vec![
            Span::styled(" ↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":导航  "),
            Span::styled("y", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":复制命令  "),
            Span::styled("b", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":收藏  "),
            Span::styled("B", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":书签列表  "),
            Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":搜索  "),
            Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":切换  "),
            Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":退出  "),
        ])),
        _ => Paragraph::new(Line::from(vec![
            Span::styled(" ↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":导航  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":展开  "),
            Span::styled("B", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":书签  "),
            Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":切换面板  "),
            Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":搜索  "),
            Span::styled("1-4", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":跳转  "),
            Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":退出  "),
        ])),
    };

    frame.render_widget(help, area);
}
