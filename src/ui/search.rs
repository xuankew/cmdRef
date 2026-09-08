use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, SearchResultIndex};

/// 渲染搜索输入框
pub fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(Line::from(vec![
        Span::styled("  搜索: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(
            &app.search_query,
            Style::default().fg(Color::White),
        ),
        Span::styled(
            "█",
            Style::default().fg(Color::White),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(input, area);
}

/// 渲染搜索结果（命令 + Prompts 混合）
pub fn draw_results(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!(" 搜索结果 ({}) ", app.search_results.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.search_results.is_empty() {
        let msg = if app.search_query.is_empty() {
            "  Type to search..."
        } else {
            "  No results found"
        };
        let empty = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let is_selected = i == app.search_cursor;
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let (breadcrumb, summary) = match result {
                SearchResultIndex::Command {
                    platform_index,
                    category_index,
                    command_index,
                    ..
                } => {
                    let platform = &app.platforms[*platform_index];
                    let category = &platform.categories[*category_index];
                    let command = &category.commands[*command_index];
                    (
                        format!("  {} > {} > {}", platform.display_name, category.name, command.name),
                        command.summary.clone(),
                    )
                }
                SearchResultIndex::Prompt { prompt_index, .. } => {
                    let prompt = app.prompts.get(*prompt_index).unwrap();
                    (
                        format!("  🤖 Prompts > {}", prompt.name),
                        if prompt.description.is_empty() {
                            prompt.content.lines().next().unwrap_or("").to_string()
                        } else {
                            prompt.description.clone()
                        },
                    )
                }
                SearchResultIndex::Password { password_index, .. } => {
                    let password = app.passwords.get(*password_index).unwrap();
                    (
                        format!("  🔑 Passwords > {}", password.name),
                        if password.description.is_empty() {
                            "(已加密)".to_string()
                        } else {
                            password.description.clone()
                        },
                    )
                }
            };

            let spans = vec![
                Span::styled(breadcrumb, style),
            ];

            // 第二行：摘要
            let summary_line = if is_selected {
                Line::from(vec![
                    Span::styled(
                        format!("    {}", summary),
                        Style::default().fg(Color::Black).bg(Color::Cyan),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("    {}", summary),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            };

            ListItem::new(vec![
                Line::from(spans),
                summary_line,
                Line::from(""),
            ])
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}
