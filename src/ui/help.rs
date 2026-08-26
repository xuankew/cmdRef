use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, Focus, SidebarItemKind};

/// 渲染底部帮助栏
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let help = match app.focus {
        Focus::Content => Paragraph::new(Line::from(vec![
            Span::styled(" ↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":导航  "),
            Span::styled("b", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":收藏  "),
            Span::styled("←", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":返回  "),
            Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":搜索  "),
            Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(":退出  "),
        ])),
        _ => {
            // 根据当前侧边栏选中项动态显示
            let action_text = if let Some(item) = app.sidebar_items.get(app.sidebar_cursor) {
                if item.kind == SidebarItemKind::Platform {
                    "展开/折叠"
                } else {
                    "进入"
                }
            } else {
                "进入"
            };
            Paragraph::new(Line::from(vec![
                Span::styled(" ↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(":导航  "),
                Span::styled("→/Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(format!(":{}  ", action_text)),
                Span::styled("B", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(":书签  "),
                Span::styled("H", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(":历史  "),
                Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(":搜索  "),
                Span::styled("1-5", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(":跳转  "),
                Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(":退出  "),
            ]))
        }
    };

    frame.render_widget(help, area);
}
