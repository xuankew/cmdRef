use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, Focus, SidebarItemKind};

/// 渲染侧边栏
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Sidebar;

    let block = Block::default()
        .title(" Categories ")
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.sidebar_items.is_empty() {
        return;
    }

    // 构建列表项
    let items: Vec<ListItem> = app
        .sidebar_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == app.sidebar_cursor;
            let style = if is_selected && is_focused {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            match item.kind {
                SidebarItemKind::Platform => {
                    if let Some(platform) = app.platforms.get(item.platform_index) {
                        let icon = if item.expanded { "▼" } else { "▶" };
                        let count = platform.command_count();
                        let text = format!("{} {} ({})", icon, platform.display_name, count);
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                text,
                                style.add_modifier(Modifier::BOLD),
                            ),
                        ]))
                    } else {
                        ListItem::new("")
                    }
                }
                SidebarItemKind::Category => {
                    if let Some(ci) = item.category_index {
                        if let Some(platform) = app.platforms.get(item.platform_index) {
                            if let Some(cat) = platform.categories.get(ci) {
                                let count = cat.command_count();
                                let text = format!("   {} ({})", cat.name, count);
                                ListItem::new(Line::from(vec![
                                    Span::styled(text, style),
                                ]))
                            } else {
                                ListItem::new("")
                            }
                        } else {
                            ListItem::new("")
                        }
                    } else {
                        ListItem::new("")
                    }
                }
            }
        })
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default());

    frame.render_widget(list, inner);
}
