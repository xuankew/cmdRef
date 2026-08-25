use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::App;

/// 渲染底部帮助栏
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let _ = app; // 未来可根据焦点动态调整帮助信息

    let help = Paragraph::new(Line::from(vec![
        Span::styled(" ↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(":导航  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(":展开/选中  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(":切换面板  "),
        Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(":搜索  "),
        Span::styled("1-4", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(":跳转平台  "),
        Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(":退出  "),
    ]));

    frame.render_widget(help, area);
}
