use ratatui::prelude::*;
use ratatui::widgets::*;
use unicode_width::UnicodeWidthStr;
use crate::app::{AppMode, EditorField, PromptField, PasswordField};

const BG_OVERLAY: Color = Color::Rgb(10, 20, 30);
const BG_DIALOG: Color = Color::Rgb(15, 25, 40);
const BG_ACTIVE: Color = Color::Rgb(20, 35, 55);

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
    let bg = Block::default().style(Style::default().bg(BG_OVERLAY));
    frame.render_widget(bg, area);

    // 对话框居中
    let width = area.width.min(68);
    let height = 20u16;
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
        .style(Style::default().bg(BG_DIALOG));
    frame.render_widget(block, dialog);

    let inner = dialog.inner(Margin { vertical: 1, horizontal: 2 });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 名称
            Constraint::Min(5),    // 命令（多行）
            Constraint::Length(3), // 标签
            Constraint::Length(3), // 提示行
        ])
        .split(inner);

    draw_field(frame, chunks[0], "名称 * (如: brew upgrade)", &state.name, state.active_field == EditorField::Name);
    draw_prompt_content_field(frame, chunks[1], "命令 * (Enter 换行，每行一条)", &state.command, state.active_field == EditorField::Command);
    draw_field(frame, chunks[2], "标签  (逗号分隔，可选)", &state.tags, state.active_field == EditorField::Tags);

    // 提示/错误行：error 优先；否则根据 name 是否已存在给出动态反馈
    let msg = if !state.error.is_empty() {
        Paragraph::new(state.error.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
    } else if crate::app::is_existing_custom_command(&state.name) {
        Paragraph::new(format!("⚡ 将追加到已有命令: {}", state.name.trim()))
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
    } else {
        Paragraph::new("Tab:下一项  Shift+Tab:上一项  Enter:保存  Esc:取消")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
    };
    frame.render_widget(msg, chunks[3]);

    // 光标定位（display width，支持 CJK）
    let (cursor_chunk, value) = match state.active_field {
        EditorField::Name => (chunks[0], state.name.as_str()),
        EditorField::Command => (chunks[1], state.command.as_str()),
        EditorField::Tags => (chunks[2], state.tags.as_str()),
    };
    let (cursor_x, cursor_y) = if state.active_field == EditorField::Command {
        // 多行：定位到最后一行末尾
        let lines: Vec<&str> = value.split('\n').collect();
        let line_idx = lines.len().saturating_sub(1);
        let last_line = lines.last().unwrap_or(&"");
        let dw = UnicodeWidthStr::width(*last_line).min(cursor_chunk.width.saturating_sub(3) as usize);
        (cursor_chunk.x + 1 + dw as u16, cursor_chunk.y + 1 + line_idx as u16)
    } else {
        let dw = UnicodeWidthStr::width(value).min(cursor_chunk.width.saturating_sub(3) as usize);
        (cursor_chunk.x + 1 + dw as u16, cursor_chunk.y + 1)
    };
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn draw_field(frame: &mut Frame, area: Rect, label: &str, value: &str, active: bool) {
    let border_style = if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let bg_color = if active {
        BG_ACTIVE
    } else {
        BG_DIALOG
    };

    let block = Block::default()
        .title(format!(" {} ", label))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(bg_color));

    let max_width = area.width.saturating_sub(4) as usize;
    let display = if UnicodeWidthStr::width(value) > max_width {
        // 从右侧按 display width 截断，保留完整字符
        let mut w = 0;
        let start = value.char_indices().rev().find_map(|(i, ch)| {
            let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if w + cw > max_width { Some(i + ch.len_utf8()) } else { w += cw; None }
        }).unwrap_or(0);
        value[start..].to_string()
    } else {
        value.to_string()
    };

    let text_style = if active {
        Style::default().fg(Color::White).bg(bg_color)
    } else {
        Style::default().fg(Color::Gray).bg(bg_color)
    };

    let paragraph = Paragraph::new(display).block(block).style(text_style);
    frame.render_widget(paragraph, area);
}

/// 渲染 Prompt 编辑表单（全屏弹窗覆盖）
pub fn draw_prompt(frame: &mut Frame, app: &crate::app::App) {
    let state = match &app.mode {
        AppMode::PromptEditor(s) => s,
        _ => return,
    };

    let area = frame.area();

    frame.render_widget(Clear, area);

    let bg = Block::default().style(Style::default().bg(BG_OVERLAY));
    frame.render_widget(bg, area);

    let width = area.width.min(76);
    let height = area.height.clamp(14, 22);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let dialog = Rect { x, y, width, height };

    frame.render_widget(Clear, dialog);

    let title = if state.editing.is_some() {
        " 编辑 Prompt "
    } else {
        " 新建 Prompt "
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .style(Style::default().bg(BG_DIALOG));
    frame.render_widget(block, dialog);

    let inner = dialog.inner(Margin { vertical: 1, horizontal: 2 });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),     // 名称
            Constraint::Length(3),     // 描述
            Constraint::Min(4),        // 内容（多行）
            Constraint::Length(3),     // 标签
            Constraint::Length(3),     // 提示行
        ])
        .split(inner);

    draw_field(frame, chunks[0], "名称 *", &state.name, state.active_field == PromptField::Name);
    draw_field(frame, chunks[1], "描述", &state.description, state.active_field == PromptField::Description);
    draw_prompt_content_field(frame, chunks[2], "内容 * (Enter 换行)", &state.content, state.active_field == PromptField::Content);
    draw_field(frame, chunks[3], "标签 (逗号分隔)", &state.tags, state.active_field == PromptField::Tags);

    let msg = if !state.error.is_empty() {
        Paragraph::new(state.error.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
    } else {
        Paragraph::new("Tab:下一项  Shift+Tab:上一项  ^S:保存  Esc:取消")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
    };
    frame.render_widget(msg, chunks[4]);

    // 光标定位（display width，支持 CJK）
    let (cursor_chunk, value) = match state.active_field {
        PromptField::Name => (chunks[0], state.name.as_str()),
        PromptField::Description => (chunks[1], state.description.as_str()),
        PromptField::Content => (chunks[2], state.content.as_str()),
        PromptField::Tags => (chunks[3], state.tags.as_str()),
    };
    let (cursor_x, cursor_y) = if state.active_field == PromptField::Content {
        let lines: Vec<&str> = value.split('\n').collect();
        let line_idx = lines.len().saturating_sub(1);
        let last_line = lines.last().unwrap_or(&"");
        let dw = UnicodeWidthStr::width(*last_line).min(cursor_chunk.width.saturating_sub(3) as usize);
        let cx = cursor_chunk.x + 1 + dw as u16;
        let cy = cursor_chunk.y + 1 + line_idx as u16;
        let max_cursor_x = cursor_chunk.x.saturating_add(cursor_chunk.width.saturating_sub(2));
        (cx.min(max_cursor_x), cy)
    } else {
        let dw = UnicodeWidthStr::width(value).min(cursor_chunk.width.saturating_sub(3) as usize);
        (cursor_chunk.x + 1 + dw as u16, cursor_chunk.y + 1)
    };
    frame.set_cursor_position((cursor_x, cursor_y));
}

/// 渲染多行 Content 输入框
fn draw_prompt_content_field(frame: &mut Frame, area: Rect, label: &str, value: &str, active: bool) {
    let border_style = if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let bg_color = if active {
        BG_ACTIVE
    } else {
        BG_DIALOG
    };

    let block = Block::default()
        .title(format!(" {} ", label))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(bg_color));

    let text_style = if active {
        Style::default().fg(Color::White).bg(bg_color)
    } else {
        Style::default().fg(Color::Gray).bg(bg_color)
    };

    let paragraph = Paragraph::new(value.to_string())
        .block(block)
        .style(text_style)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

/// 渲染 Password 编辑表单（全屏弹窗覆盖）
pub fn draw_password(frame: &mut Frame, app: &crate::app::App) {
    let state = match &app.mode {
        AppMode::PasswordEditor(s) => s,
        _ => return,
    };

    let area = frame.area();

    frame.render_widget(Clear, area);

    let bg = Block::default().style(Style::default().bg(BG_OVERLAY));
    frame.render_widget(bg, area);

    let width = area.width.min(76);
    let height = area.height.clamp(12, 18);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let dialog = Rect { x, y, width, height };

    frame.render_widget(Clear, dialog);

    let title = if state.editing.is_some() {
        " 编辑密码 "
    } else {
        " 新建密码 "
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(BG_DIALOG));
    frame.render_widget(block, dialog);

    let inner = dialog.inner(Margin { vertical: 1, horizontal: 2 });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),     // 名称
            Constraint::Length(3),     // 密码值
            Constraint::Length(3),     // 描述
            Constraint::Length(3),     // 提示行
        ])
        .split(inner);

    draw_field(frame, chunks[0], "名称 *", &state.name, state.active_field == PasswordField::Name);
    draw_field(frame, chunks[1], "密码值 *", &state.value, state.active_field == PasswordField::Value);
    draw_field(frame, chunks[2], "描述", &state.description, state.active_field == PasswordField::Description);

    let msg = if !state.error.is_empty() {
        Paragraph::new(state.error.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
    } else {
        Paragraph::new("Tab:下一项  Shift+Tab:上一项  ^S:保存  Esc:取消")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
    };
    frame.render_widget(msg, chunks[3]);

    // 光标定位（display width，支持 CJK）
    let (cursor_chunk, value) = match state.active_field {
        PasswordField::Name => (chunks[0], state.name.as_str()),
        PasswordField::Value => (chunks[1], state.value.as_str()),
        PasswordField::Description => (chunks[2], state.description.as_str()),
    };
    let dw = UnicodeWidthStr::width(value).min(cursor_chunk.width.saturating_sub(3) as usize);
    let cursor_x = cursor_chunk.x + 1 + dw as u16;
    let cursor_y = cursor_chunk.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

/// 渲染主密码输入对话框（全屏弹窗覆盖）
pub fn draw_master_password(frame: &mut Frame, app: &crate::app::App) {
    let state = match &app.mode {
        AppMode::MasterPassword(s) => s,
        _ => return,
    };

    let area = frame.area();

    frame.render_widget(Clear, area);

    let bg = Block::default().style(Style::default().bg(BG_OVERLAY));
    frame.render_widget(bg, area);

    let width = area.width.min(60);
    let height = area.height.clamp(10, 14);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let dialog = Rect { x, y, width, height };

    frame.render_widget(Clear, dialog);

    let title = if state.is_change_mode() && state.is_create_mode {
        " 设置新主密码 "
    } else if state.is_change_mode() {
        " 验证当前主密码 "
    } else if state.is_create_mode {
        " 设置主密码 "
    } else {
        " 输入主密码 "
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(BG_DIALOG));
    frame.render_widget(block, dialog);

    let inner = dialog.inner(Margin { vertical: 1, horizontal: 2 });

    if state.is_create_mode {
        // 创建模式：两个字段
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),     // 主密码
                Constraint::Length(3),     // 确认密码
                Constraint::Length(3),     // 提示/错误
            ])
            .split(inner);

        // 主密码字段
        let first_active = state.active_field == 0;
        let first_value: String = "•".repeat(state.input.chars().count());
        let first_label = if state.is_change_mode() { "新主密码" } else { "主密码" };
        draw_masked_field(frame, chunks[0], first_label, &first_value, first_active);

        // 确认密码字段
        let second_active = state.active_field == 1;
        let second_value: String = "•".repeat(state.confirm.chars().count());
        let second_label = if state.is_change_mode() { "确认新密码" } else { "确认密码" };
        draw_masked_field(frame, chunks[1], second_label, &second_value, second_active);

        // 提示/错误行
        let msg = if !state.error.is_empty() {
            Paragraph::new(state.error.as_str())
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
        } else {
            Paragraph::new("Enter: 确认  Esc: 取消")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
        };
        frame.render_widget(msg, chunks[2]);

        // 光标定位
        let (cursor_chunk, char_count) = if first_active {
            (chunks[0], state.input.chars().count())
        } else {
            (chunks[1], state.confirm.chars().count())
        };
        let dw = char_count.min(cursor_chunk.width.saturating_sub(3) as usize);
        let cursor_x = cursor_chunk.x + 1 + dw as u16;
        let cursor_y = cursor_chunk.y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    } else {
        // 解锁模式：一个字段
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),     // 主密码
                Constraint::Length(3),     // 提示/错误
            ])
            .split(inner);

        let masked: String = "•".repeat(state.input.chars().count());
        let label = if state.is_change_mode() { "当前主密码" } else { "主密码" };
        draw_masked_field(frame, chunks[0], label, &masked, true);

        let msg = if !state.error.is_empty() {
            Paragraph::new(state.error.as_str())
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
        } else {
            Paragraph::new("Enter: 确认  Esc: 取消")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
        };
        frame.render_widget(msg, chunks[1]);

        // 光标定位
        let dw = state.input.chars().count().min(chunks[0].width.saturating_sub(3) as usize);
        let cursor_x = chunks[0].x + 1 + dw as u16;
        let cursor_y = chunks[0].y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

/// 渲染主密码掩码输入框（所有字符显示为 •）
fn draw_masked_field(frame: &mut Frame, area: Rect, label: &str, masked_value: &str, active: bool) {
    let border_style = if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let bg_color = if active {
        BG_ACTIVE
    } else {
        BG_DIALOG
    };

    let block = Block::default()
        .title(format!(" {} ", label))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(bg_color));

    let text_style = if active {
        Style::default().fg(Color::White).bg(bg_color)
    } else {
        Style::default().fg(Color::Gray).bg(bg_color)
    };

    let paragraph = Paragraph::new(masked_value.to_string())
        .block(block)
        .style(text_style);
    frame.render_widget(paragraph, area);
}

