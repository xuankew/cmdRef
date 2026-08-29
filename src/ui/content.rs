use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, Focus, SidebarItemKind};

/// 渲染内容面板
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Content;

    // 获取当前分类信息
    let title = if let Some(item) = app.sidebar_items.get(app.sidebar_cursor) {
        if item.kind == SidebarItemKind::Bookmarks {
            " ★ Bookmarks ".to_string()
        } else if item.kind == SidebarItemKind::History {
            " 🕑 History ".to_string()
        } else {
            app.current_category()
                .map(|c| format!(" {} ", c.name))
                .unwrap_or_else(|| " Command Details ".to_string())
        }
    } else {
        " Command Details ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // 左侧命令列表 + 右侧命令详情
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(inner);

    // 命令列表
    draw_command_list(frame, app, chunks[0]);

    // 命令详情
    draw_command_detail(frame, app, chunks[1]);
}

/// 渲染命令列表
fn draw_command_list(frame: &mut Frame, app: &App, area: Rect) {
    let selected = app.selected_command.unwrap_or(0);

    // Bookmarks 模式：显示收藏的命令
    if let Some(item) = app.sidebar_items.get(app.sidebar_cursor) {
        if item.kind == SidebarItemKind::Bookmarks {
            let bm_commands: Vec<String> = app.bookmarks.all().iter().map(|b| {
                format!("{} > {}", b.category, b.command)
            }).collect();

            if bm_commands.is_empty() {
                let empty = Paragraph::new("  (no bookmarks yet)\n\n  Press 'b' on a command\n  to bookmark it")
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(empty, area);
                return;
            }

            let items: Vec<ListItem> = bm_commands
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let is_selected = i == selected;
                    let style = if is_selected {
                        Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Yellow)
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("  ★ {}", name), style),
                    ]))
                })
                .collect();

            let list = List::new(items);
            frame.render_widget(list, area);
            return;
        }

        // History 模式：显示浏览历史
        if item.kind == SidebarItemKind::History {
            let history_commands: Vec<String> = app.history.all().iter().map(|h| {
                format!("{} > {}", h.category, h.command)
            }).collect();

            if history_commands.is_empty() {
                let empty = Paragraph::new("  (no history yet)\n\n  Browse commands to\n  build your history")
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(empty, area);
                return;
            }

            let items: Vec<ListItem> = history_commands
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let is_selected = i == selected;
                    let style = if is_selected {
                        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("  🕑 {}", name), style),
                    ]))
                })
                .collect();

            let list = List::new(items);
            frame.render_widget(list, area);
            return;
        }
    }

    // 普通模式
    let commands = app.current_category_commands();

    if commands.is_empty() {
        let empty = Paragraph::new("  (no commands)")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let is_selected = i == selected;
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // 检查是否已收藏
            let bm_icon = if let (Some(platform), Some(cat)) = (app.current_platform(), app.current_category()) {
                if app.bookmarks.is_bookmarked(&platform.name, &cat.name, &cmd.name) {
                    "★ "
                } else {
                    "  "
                }
            } else {
                "  "
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {}{}", bm_icon, cmd.name),
                    style,
                ),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, area);
}

/// 渲染命令详情
fn draw_command_detail(frame: &mut Frame, app: &App, area: Rect) {
    let cmd = match app.current_command() {
        Some(cmd) => cmd.clone(),
        None => {
            let msg = Paragraph::new("  Select a command to view details")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(msg, area);
            return;
        }
    };

    let mut lines: Vec<Line> = Vec::new();

    // 状态消息（如果有）
    if !app.status_message.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {}", app.status_message),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
    }

    // 命令名 + 摘要 + 标签
    let bookmark_indicator = if app.is_current_bookmarked() { " ★" } else { "" };
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}{} ", cmd.name, bookmark_indicator),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("- {}", cmd.summary),
            Style::default().fg(Color::White),
        ),
    ]));

    // 标签
    if !cmd.tags.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} ", cmd.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ")),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }
    lines.push(Line::from(""));

    // Quick Reference: daily 示例
    let daily_examples: Vec<_> = cmd.examples.iter()
        .filter(|e| e.frequency == "daily")
        .collect();

    if !daily_examples.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                "  ⚡ QUICK REFERENCE",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled(
                "  ────────────────────────────────",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        for example in &daily_examples {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    $ {}", example.code),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!("  // {}", example.description),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
        lines.push(Line::from(""));
    }

    // 全部示例
    if !cmd.examples.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                "  EXAMPLES",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled(
                "  ────────────────────────────────",
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        for (idx, example) in cmd.examples.iter().enumerate() {
            let num = idx + 1;
            // 描述 + 频率标记
            let mut desc_spans = vec![
                Span::styled(
                    format!("    {}:", example.description),
                    Style::default().fg(Color::Gray),
                ),
            ];
            match example.frequency.as_str() {
                "daily" => desc_spans.push(Span::styled(" ⚡", Style::default().fg(Color::Green))),
                "weekly" => desc_spans.push(Span::styled(" ○", Style::default().fg(Color::Blue))),
                "rarely" => desc_spans.push(Span::styled(" ·", Style::default().fg(Color::DarkGray))),
                _ => {}
            }
            lines.push(Line::from(desc_spans));

            // 危险警告
            if example.danger == "high" {
                lines.push(Line::from(vec![
                    Span::styled(
                        "      ⚠ DANGEROUS - use with caution",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                ]));
            } else if example.danger == "medium" {
                lines.push(Line::from(vec![
                    Span::styled("      ⚠ use with caution", Style::default().fg(Color::Yellow)),
                ]));
            }

            lines.push(Line::from(vec![
                Span::styled(
                    format!("    [{}]", num),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" $ {}", example.code),
                    Style::default().fg(Color::Green),
                ),
            ]));
            lines.push(Line::from(""));
        }
    }

    // Tips
    if !cmd.tips.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                "  TIPS",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled(
                "  ────────────────────────────────",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        for tip in &cmd.tips {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    • {}", tip),
                    Style::default().fg(Color::Yellow),
                ),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Related
    if !cmd.related.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                "  RELATED",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled(
                "  ────────────────────────────────",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled(
                format!("    {}", cmd.related.join(", ")),
                Style::default().fg(Color::Magenta),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll as u16, 0));
    frame.render_widget(paragraph, area);
}
