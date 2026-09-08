use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, Focus, SidebarItemKind};

/// 渲染底部帮助栏
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let help = match app.focus {
        Focus::Content => {
            let is_prompts = app.sidebar_items.get(app.sidebar_cursor)
                .map(|i| i.kind == SidebarItemKind::Prompts)
                .unwrap_or(false);

            let is_passwords = app.sidebar_items.get(app.sidebar_cursor)
                .map(|i| i.kind == SidebarItemKind::Passwords)
                .unwrap_or(false);

            if is_prompts {
                Paragraph::new(Line::from(vec![
                    Span::styled(" ↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":切换 Prompt  "),
                    Span::styled("^D/^U", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(":滚动详情  "),
                    Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(":复制全文  "),
                    Span::styled("n", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(":新建  "),
                    Span::styled("e", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":编辑  "),
                    Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(":删除  "),
                    Span::styled("←", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":返回  "),
                    Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":搜索  "),
                    Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":退出  "),
                ]))
            } else if is_passwords {
                Paragraph::new(Line::from(vec![
                    Span::styled(" ↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":切换  "),
                    Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(":复制密码  "),
                    Span::styled("n", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(":新建  "),
                    Span::styled("e", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":编辑  "),
                    Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(":删除  "),
                    Span::styled("C", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    Span::raw(":修改主密码  "),
                    Span::styled("←", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":返回  "),
                    Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":搜索  "),
                    Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":退出  "),
                ]))
            } else {
                Paragraph::new(Line::from(vec![
                    Span::styled(" ↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":切换命令  "),
                    Span::styled("^D/^U", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(":滚动详情  "),
                    Span::styled("1-9", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(":复制命令  "),
                    Span::styled("b", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":收藏  "),
                    Span::styled("n", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(":新增  "),
                    Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(":删除  "),
                    Span::styled("←", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":返回  "),
                    Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":搜索  "),
                    Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(":退出  "),
                ]))
            }
        }
        _ => {
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
                Span::styled("n", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(":新增命令  "),
                Span::styled("B", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(":书签  "),
                Span::styled("H", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(":历史  "),
                Span::styled("P", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw(":Prompts  "),
                Span::styled("S", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(":Passwords  "),
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
