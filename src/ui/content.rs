use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, Focus};

/// 渲染内容面板
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Content;

    // 获取当前分类信息
    let category_name = app.current_category()
        .map(|c| format!(" {} ", c.name))
        .unwrap_or_else(|| " Command Details ".to_string());

    let block = Block::default()
        .title(category_name)
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
    let commands = app.current_category_commands();
    let selected = app.selected_command.unwrap_or(0);

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
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {} ", cmd.name),
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

    // 命令名和摘要
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {} ", cmd.name),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("- {}", cmd.summary),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(""));

    // 示例
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

        for example in &cmd.examples {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {}:", example.description),
                    Style::default().fg(Color::Gray),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("      $ {}", example.code),
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
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
