use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{AppMode, EditorField};

/// 渲染自定义命令添加表单（全屏弹窗覆盖）
pub fn draw(frame: &mut Frame, app: &crate::app::App) {
    let state = match &app.mode {
        AppMode::AddCommand(s) => s,
        _ => return,
    };

    let area = frame.area();

    // 用 Clear 彻底清除整个屏幕，防止底层内容透出
    frame.render_widget(Clear, area);

    // 整屏填充深色背景
    let bg = Block::default().style(Style::default().bg(Color::Rgb(10, 20, 30)));
    frame.render_widget(bg, area);

    // 对话框居中
    let width = area.width.min(68);
    let height = 16u16;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let dialog = Rect { x, y, width, height };

    // 先 Clear 对话框区域，再画边框
    frame.render_widget(Clear, dialog);

    let block = Block::default()
        .title(" 新建自定义命令 ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Rgb(15, 25, 40)));
    frame.render_widget(block, dialog);

    let inner = dialog.inner(Margin { vertical: 1, horizontal: 2 });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 名称
            Constraint::Length(3), // 命令
            Constraint::Length(3), // 标签
            Constraint::Length(3), // 提示行
        ])
        .split(inner);

    draw_field(frame, chunks[0], "名称 * (如: brew upgrade)", &state.name, state.active_field == EditorField::Name);
    draw_field(frame, chunks[1], "命令 * (如: brew upgrade --greedy)", &state.command, state.active_field == EditorField::Command);
    draw_field(frame, chunks[2], "标签  (逗号分隔，可选)", &state.tags, state.active_field == EditorField::Tags);

    // 提示/错误行
    let msg = if !state.error.is_empty() {
        Paragraph::new(state.error.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
    } else {
        Paragraph::new("Tab:下一项  Shift+Tab:上一项  Enter:保存  Esc:取消")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
    };
    frame.render_widget(msg, chunks[3]);

    // 光标定位
    let (cursor_chunk, value) = match state.active_field {
        EditorField::Name => (chunks[0], &state.name),
        EditorField::Command => (chunks[1], &state.command),
        EditorField::Tags => (chunks[2], &state.tags),
    };
    let cursor_x = cursor_chunk.x + 1 + value.len().min((cursor_chunk.width as usize).saturating_sub(3)) as u16;
    let cursor_y = cursor_chunk.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn draw_field(frame: &mut Frame, area: Rect, label: &str, value: &str, active: bool) {
    let border_style = if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let bg_color = if active {
        Color::Rgb(20, 35, 55)
    } else {
        Color::Rgb(15, 25, 40)
    };

    let block = Block::default()
        .title(format!(" {} ", label))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(bg_color));

    let max_width = area.width.saturating_sub(4) as usize;
    let display = if value.len() > max_width {
        &value[value.len() - max_width..]
    } else {
        value
    };

    let text_style = if active {
        Style::default().fg(Color::White).bg(bg_color)
    } else {
        Style::default().fg(Color::Gray).bg(bg_color)
    };

    let paragraph = Paragraph::new(display).block(block).style(text_style);
    frame.render_widget(paragraph, area);
}

