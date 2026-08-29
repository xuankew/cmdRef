use crate::data::{Command, Category, Platform};
use crate::search::SearchEngine;
use crate::bookmarks::BookmarkManager;
use crate::history::HistoryManager;

/// 焦点所在的面板
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Sidebar,
    Content,
    Search,
}

/// 自定义命令编辑器中的字段
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorField {
    Name,
    Command,
    Tags,
}

impl EditorField {
    pub fn next(self) -> Self {
        match self {
            EditorField::Name => EditorField::Command,
            EditorField::Command => EditorField::Tags,
            EditorField::Tags => EditorField::Name,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            EditorField::Name => EditorField::Tags,
            EditorField::Command => EditorField::Name,
            EditorField::Tags => EditorField::Command,
        }
    }
}

/// 自定义命令编辑器状态
#[derive(Debug, Clone, PartialEq)]
pub struct EditorState {
    pub active_field: EditorField,
    pub name: String,
    pub command: String,
    pub tags: String,
    pub error: String,
}

impl EditorState {
    pub fn new() -> Self {
        EditorState {
            active_field: EditorField::Name,
            name: String::new(),
            command: String::new(),
            tags: String::new(),
            error: String::new(),
        }
    }

    pub fn active_field_mut(&mut self) -> &mut String {
        match self.active_field {
            EditorField::Name => &mut self.name,
            EditorField::Command => &mut self.command,
            EditorField::Tags => &mut self.tags,
        }
    }
}

/// 应用模式
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Search,
    AddCommand(EditorState),
}

/// 侧边栏中的项目类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarItemKind {
    Bookmarks,
    History,
    Platform,
    Category,
}

/// 侧边栏项目
#[derive(Debug, Clone)]
pub struct SidebarItem {
    pub kind: SidebarItemKind,
    pub platform_index: usize,
    pub category_index: Option<usize>,
    pub expanded: bool,
}

/// 应用状态
pub struct App {
    pub platforms: Vec<Platform>,
    pub search_engine: SearchEngine,
    pub bookmarks: BookmarkManager,
    pub history: HistoryManager,

    // 侧边栏状态
    pub sidebar_items: Vec<SidebarItem>,
    pub sidebar_cursor: usize,

    // 内容区域状态
    pub content_cursor: usize,
    pub selected_command: Option<usize>,

    // 焦点和模式
    pub focus: Focus,
    pub mode: AppMode,

    // 搜索
    pub search_query: String,
    pub search_results: Vec<SearchResultIndex>,
    pub search_cursor: usize,

    // 状态消息（复制反馈等）
    pub status_message: String,
    pub status_clear_counter: u8,

    // 退出标志
    pub should_quit: bool,
}

/// 搜索结果索引（避免引用问题）
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SearchResultIndex {
    pub platform_index: usize,
    pub category_index: usize,
    pub command_index: usize,
    pub score: i64,
}

impl App {
    pub fn new(platforms: Vec<Platform>) -> Self {
        let mut sidebar_items = Vec::new();

        // 第一项：Bookmarks
        sidebar_items.push(SidebarItem {
            kind: SidebarItemKind::Bookmarks,
            platform_index: 0,
            category_index: None,
            expanded: false,
        });

        // 第二项：History
        sidebar_items.push(SidebarItem {
            kind: SidebarItemKind::History,
            platform_index: 0,
            category_index: None,
            expanded: false,
        });

        // 检测当前平台
        let detected_platform = Self::detect_platform_index(&platforms);

        for (pi, platform) in platforms.iter().enumerate() {
            let is_current = pi == detected_platform;
            sidebar_items.push(SidebarItem {
                kind: SidebarItemKind::Platform,
                platform_index: pi,
                category_index: None,
                expanded: is_current,
            });

            if is_current {
                for (ci, _) in platform.categories.iter().enumerate() {
                    sidebar_items.push(SidebarItem {
                        kind: SidebarItemKind::Category,
                        platform_index: pi,
                        category_index: Some(ci),
                        expanded: false,
                    });
                }
            }
        }

        let search_engine = SearchEngine::new();
        let bookmarks = BookmarkManager::new();
        let history = HistoryManager::new();

        // 光标定位到当前检测到的平台
        let initial_cursor = sidebar_items
            .iter()
            .position(|item| item.kind == SidebarItemKind::Platform && item.platform_index == detected_platform)
            .unwrap_or(1);

        let mut app = App {
            platforms,
            search_engine,
            bookmarks,
            history,
            sidebar_items,
            sidebar_cursor: initial_cursor,
            content_cursor: 0,
            selected_command: Some(0),
            focus: Focus::Sidebar,
            mode: AppMode::Normal,
            search_query: String::new(),
            search_results: Vec::new(),
            search_cursor: 0,
            status_message: String::new(),
            status_clear_counter: 0,
            should_quit: false,
        };

        app.update_selection();
        debug_log!("App initialized: {} platforms, {} bookmarks, {} history",
            app.platforms.len(), app.bookmarks.count(), app.history.count());
        app
    }

    /// 检测当前操作系统对应的平台索引
    fn detect_platform_index(platforms: &[Platform]) -> usize {
        let os_name = if cfg!(target_os = "macos") {
            "mac"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            "linux"
        };

        platforms
            .iter()
            .position(|p| p.name == os_name)
            .unwrap_or(0)
    }

    /// 设置状态消息（自动在几次绘制后清除）
    fn set_status(&mut self, msg: String) {
        self.status_message = msg;
        self.status_clear_counter = 0;
    }

    /// 每帧调用，递减状态消息计数器
    pub fn tick(&mut self) {
        if !self.status_message.is_empty() {
            self.status_clear_counter += 1;
            if self.status_clear_counter > 40 {
                self.status_message.clear();
            }
        }
    }

    /// 更新当前选中项
    pub fn update_selection(&mut self) {
        if let Some(item) = self.current_sidebar_item() {
            match item.kind {
                SidebarItemKind::Bookmarks | SidebarItemKind::History => {
                    // 保留已选中的命令索引，仅在越界时重置
                    let len = if item.kind == SidebarItemKind::Bookmarks {
                        self.bookmarks.count()
                    } else {
                        self.history.count()
                    };
                    if let Some(idx) = self.selected_command {
                        if idx >= len {
                            self.selected_command = if len > 0 { Some(0) } else { None };
                        }
                    } else {
                        self.selected_command = if len > 0 { Some(0) } else { None };
                    }
                    self.content_cursor = self.selected_command.unwrap_or(0);
                }
                SidebarItemKind::Platform => {
                    let pi = item.platform_index;
                    if let Some(platform) = self.platforms.get(pi) {
                        if let Some(cat) = platform.categories.first() {
                            // 保留已选中的命令索引，仅在越界时重置
                            let len = cat.commands.len();
                            if let Some(idx) = self.selected_command {
                                if idx >= len {
                                    self.selected_command = Some(0);
                                }
                            } else {
                                self.selected_command = Some(0);
                            }
                            self.content_cursor = self.selected_command.unwrap_or(0);
                        }
                    }
                }
                SidebarItemKind::Category => {
                    // 保留已选中的命令索引，仅在越界时重置
                    let len = self.current_category_commands().len();
                    if let Some(idx) = self.selected_command {
                        if idx >= len {
                            self.selected_command = Some(0);
                        }
                    } else {
                        self.selected_command = Some(0);
                    }
                    self.content_cursor = self.selected_command.unwrap_or(0);
                }
            }
        }

        // 记录到历史
        self.record_current_to_history();
    }

    /// 记录当前命令到历史
    fn record_current_to_history(&mut self) {
        let info = {
            let platform = self.current_platform().map(|p| p.name.clone());
            let cat = self.current_category().map(|c| c.name.clone());
            let cmd = self.current_command().map(|c| c.name.clone());
            (platform, cat, cmd)
        };
        if let (Some(platform), Some(cat), Some(cmd)) = info {
            self.history.record(&platform, &cat, &cmd);
        }
    }

    /// 获取当前侧边栏项的克隆
    fn current_sidebar_item(&self) -> Option<SidebarItem> {
        self.sidebar_items.get(self.sidebar_cursor).cloned()
    }

    /// 获取当前应该显示的命令
    pub fn current_command(&self) -> Option<&Command> {
        if matches!(self.mode, AppMode::Search) {
            if let Some(result) = self.search_results.get(self.search_cursor) {
                return self.platforms
                    .get(result.platform_index)
                    .and_then(|p| p.categories.get(result.category_index))
                    .and_then(|c| c.commands.get(result.command_index));
            }
            return None;
        }

        let item = self.current_sidebar_item()?;
        match item.kind {
            SidebarItemKind::Bookmarks => {
                self.bookmark_commands().get(self.selected_command.unwrap_or(0)).copied()
            }
            SidebarItemKind::History => {
                self.history_commands().get(self.selected_command.unwrap_or(0)).copied()
            }
            SidebarItemKind::Platform => {
                let pi = item.platform_index;
                self.platforms
                    .get(pi)
                    .and_then(|p| p.categories.first())
                    .and_then(|c| {
                        let idx = self.selected_command.unwrap_or(0).min(c.commands.len().saturating_sub(1));
                        c.commands.get(idx)
                    })
            }
            SidebarItemKind::Category => {
                let pi = item.platform_index;
                let ci = item.category_index?;
                self.platforms
                    .get(pi)
                    .and_then(|p| p.categories.get(ci))
                    .and_then(|c| {
                        let idx = self.selected_command.unwrap_or(0).min(c.commands.len().saturating_sub(1));
                        c.commands.get(idx)
                    })
            }
        }
    }

    /// 获取当前分类的命令列表
    pub fn current_category_commands(&self) -> &[Command] {
        if let Some(item) = self.current_sidebar_item() {
            match item.kind {
                SidebarItemKind::Platform => {
                    if let Some(platform) = self.platforms.get(item.platform_index) {
                        if let Some(cat) = platform.categories.first() {
                            return &cat.commands;
                        }
                    }
                }
                SidebarItemKind::Category => {
                    if let Some(ci) = item.category_index {
                        if let Some(platform) = self.platforms.get(item.platform_index) {
                            if let Some(cat) = platform.categories.get(ci) {
                                return &cat.commands;
                            }
                        }
                    }
                }
                SidebarItemKind::Bookmarks | SidebarItemKind::History => {}
            }
        }
        &[]
    }

    /// 获取书签命令的引用列表
    fn bookmark_commands(&self) -> Vec<&Command> {
        let mut result = Vec::new();
        for bm in self.bookmarks.all() {
            for platform in &self.platforms {
                if platform.name == bm.platform {
                    for cat in &platform.categories {
                        if cat.name == bm.category {
                            if let Some(cmd) = cat.commands.iter().find(|c| c.name == bm.command) {
                                result.push(cmd);
                            }
                        }
                    }
                }
            }
        }
        result
    }

    /// 获取历史命令的引用列表
    fn history_commands(&self) -> Vec<&Command> {
        let mut result = Vec::new();
        for entry in self.history.all() {
            for platform in &self.platforms {
                if platform.name == entry.platform {
                    for cat in &platform.categories {
                        if cat.name == entry.category {
                            if let Some(cmd) = cat.commands.iter().find(|c| c.name == entry.command) {
                                result.push(cmd);
                            }
                        }
                    }
                }
            }
        }
        result
    }

    /// 获取当前分类信息
    pub fn current_category(&self) -> Option<&Category> {
        if let Some(item) = self.current_sidebar_item() {
            match item.kind {
                SidebarItemKind::Bookmarks | SidebarItemKind::History => None,
                SidebarItemKind::Platform => {
                    self.platforms
                        .get(item.platform_index)
                        .and_then(|p| p.categories.first())
                }
                SidebarItemKind::Category => {
                    item.category_index.and_then(|ci| {
                        self.platforms
                            .get(item.platform_index)
                            .and_then(|p| p.categories.get(ci))
                    })
                }
            }
        } else {
            None
        }
    }

    /// 获取当前平台信息
    pub fn current_platform(&self) -> Option<&Platform> {
        if let Some(item) = self.current_sidebar_item() {
            match item.kind {
                SidebarItemKind::Bookmarks | SidebarItemKind::History => None,
                _ => self.platforms.get(item.platform_index),
            }
        } else {
            None
        }
    }

    /// 检查当前命令是否已收藏
    pub fn is_current_bookmarked(&self) -> bool {
        if let (Some(platform), Some(cat), Some(cmd)) =
            (self.current_platform(), self.current_category(), self.current_command())
        {
            self.bookmarks.is_bookmarked(&platform.name, &cat.name, &cmd.name)
        } else {
            false
        }
    }

    // ======== 功能操作 ========

    /// 复制当前命令的第一个示例代码到剪贴板
    #[allow(dead_code)]
    pub fn copy_current_command(&mut self) {
        if let Some(cmd) = self.current_command() {
            debug_log!("copy_current_command: {}", cmd.name);
            // 优先复制第一个 example code，否则复制命令名
            let text = cmd
                .examples
                .first()
                .map(|e| e.code.clone())
                .unwrap_or_else(|| cmd.name.clone());

            match crate::clipboard::copy_to_clipboard(&text) {
                Ok(()) => {
                    let preview = if text.len() > 40 { &text[..40] } else { &text };
                    self.set_status(format!("✓ Copied: {}", preview));
                }
                Err(e) => {
                    self.set_status(format!("✗ Copy failed: {}", e));
                }
            }
        }
    }

    /// 切换当前命令的书签状态
    pub fn toggle_bookmark(&mut self) {
        debug_log!("toggle_bookmark called");
        if let (Some(platform), Some(cat), Some(cmd)) = (
            self.current_platform().map(|p| p.name.clone()),
            self.current_category().map(|c| c.name.clone()),
            self.current_command().map(|c| c.name.clone()),
        ) {
            let added = self.bookmarks.toggle(&platform, &cat, &cmd);
            if added {
                self.set_status(format!("★ Bookmarked: {}", cmd));
            } else {
                self.set_status(format!("☆ Removed: {}", cmd));
            }
        }
    }

    /// 跳转到书签列表
    pub fn jump_to_bookmarks(&mut self) {
        self.mode = AppMode::Normal;
        self.focus = Focus::Sidebar;
        self.sidebar_cursor = 0;
        self.selected_command = Some(0);
        self.content_cursor = 0;
    }

    /// 跳转到历史记录
    pub fn jump_to_history(&mut self) {
        self.mode = AppMode::Normal;
        self.focus = Focus::Sidebar;
        // History is always at index 1
        self.sidebar_cursor = 1;
        self.selected_command = Some(0);
        self.content_cursor = 0;
    }

    // ======== 导航操作 ========

    pub fn move_sidebar_up(&mut self) {
        if self.sidebar_cursor > 0 {
            self.sidebar_cursor -= 1;
            self.update_selection();
        }
    }

    pub fn move_sidebar_down(&mut self) {
        if self.sidebar_cursor < self.sidebar_items.len().saturating_sub(1) {
            self.sidebar_cursor += 1;
            self.update_selection();
        }
    }

    pub fn move_content_up(&mut self) {
        if let Some(idx) = self.selected_command {
            if idx > 0 {
                self.selected_command = Some(idx - 1);
                self.content_cursor = idx - 1;
            }
        }
    }

    pub fn move_content_down(&mut self) {
        let len = if let Some(item) = self.current_sidebar_item() {
            match item.kind {
                SidebarItemKind::Bookmarks => self.bookmarks.count(),
                SidebarItemKind::History => self.history.count(),
                _ => self.current_category_commands().len(),
            }
        } else {
            0
        };

        if let Some(idx) = self.selected_command {
            if idx < len.saturating_sub(1) {
                self.selected_command = Some(idx + 1);
                self.content_cursor = idx + 1;
            }
        }
    }

    pub fn toggle_sidebar_item(&mut self) {
        let cursor = self.sidebar_cursor;
        if let Some(item) = self.sidebar_items.get(cursor) {
            // Bookmarks / History / Category → 直接进入内容区
            if item.kind == SidebarItemKind::Bookmarks
                || item.kind == SidebarItemKind::History
                || item.kind == SidebarItemKind::Category
            {
                self.focus = Focus::Content;
                return;
            }
            if item.kind == SidebarItemKind::Platform {
                let pi = item.platform_index;
                let was_expanded = item.expanded;

                let mut new_items = Vec::new();
                for si in self.sidebar_items.iter() {
                    if si.kind == SidebarItemKind::Platform && si.platform_index == pi {
                        new_items.push(SidebarItem {
                            expanded: !was_expanded,
                            ..si.clone()
                        });
                        if !was_expanded {
                            if let Some(platform) = self.platforms.get(pi) {
                                for (ci, _) in platform.categories.iter().enumerate() {
                                    new_items.push(SidebarItem {
                                        kind: SidebarItemKind::Category,
                                        platform_index: pi,
                                        category_index: Some(ci),
                                        expanded: false,
                                    });
                                }
                            }
                        }
                    } else if si.kind == SidebarItemKind::Category && si.platform_index == pi {
                        if was_expanded {
                            continue;
                        } else {
                            new_items.push(si.clone());
                        }
                    } else {
                        new_items.push(si.clone());
                    }
                }
                self.sidebar_items = new_items;

                if self.sidebar_cursor >= self.sidebar_items.len() {
                    self.sidebar_cursor = self.sidebar_items.len().saturating_sub(1);
                }
                self.update_selection();
            }
        }
    }

    pub fn switch_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Sidebar => Focus::Content,
            Focus::Content => Focus::Sidebar,
            Focus::Search => Focus::Sidebar,
        };
    }

    pub fn enter_search_mode(&mut self) {
        self.mode = AppMode::Search;
        self.focus = Focus::Search;
        self.search_query.clear();
        self.search_results.clear();
        self.search_cursor = 0;
    }

    pub fn exit_search_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.focus = Focus::Sidebar;
        self.search_query.clear();
        self.search_results.clear();
    }

    /// 打开自定义命令添加表单
    pub fn open_add_command(&mut self) {
        self.mode = AppMode::AddCommand(EditorState::new());
    }

    /// 向当前编辑字段输入字符
    pub fn editor_input(&mut self, c: char) {
        if let AppMode::AddCommand(ref mut state) = self.mode {
            state.active_field_mut().push(c);
        }
    }

    /// 删除当前编辑字段末尾字符
    pub fn editor_backspace(&mut self) {
        if let AppMode::AddCommand(ref mut state) = self.mode {
            state.active_field_mut().pop();
        }
    }

    /// 切换到下一个字段
    pub fn editor_next_field(&mut self) {
        if let AppMode::AddCommand(ref mut state) = self.mode {
            state.active_field = state.active_field.next();
        }
    }

    /// 切换到上一个字段
    pub fn editor_prev_field(&mut self) {
        if let AppMode::AddCommand(ref mut state) = self.mode {
            state.active_field = state.active_field.prev();
        }
    }

    /// 取消添加命令
    pub fn editor_cancel(&mut self) {
        self.mode = AppMode::Normal;
    }

    /// 保存自定义命令并热重载
    pub fn editor_save(&mut self) {
        let state = if let AppMode::AddCommand(ref mut s) = self.mode {
            s.clone()
        } else {
            return;
        };

        let name = state.name.trim().to_string();
        let command = state.command.trim().to_string();

        if name.is_empty() {
            if let AppMode::AddCommand(ref mut s) = self.mode {
                s.error = "命令名称不能为空".to_string();
            }
            return;
        }
        if command.is_empty() {
            if let AppMode::AddCommand(ref mut s) = self.mode {
                s.error = "命令内容不能为空".to_string();
            }
            return;
        }

        // 构建 YAML 内容：name 作标题，command 同时作 summary 和 example code
        let tags_line = if !state.tags.trim().is_empty() {
            let tags: Vec<String> = state.tags.split(',')
                .map(|t| format!("      - \"{}\"", t.trim()))
                .collect();
            format!("    tags:\n{}", tags.join("\n"))
        } else {
            String::new()
        };

        let entry = format!(
            "  - name: \"{}\"\n    summary: \"{}\"\n    examples:\n      - description: \"运行\"\n        code: \"{}\"\n{}\n",
            name.replace('"', "\\\""),
            command.replace('"', "\\\""),
            command.replace('"', "\\\""),
            tags_line,
        );

        // 确定保存路径
        let custom_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("cmdref")
            .join("custom");

        if let Err(e) = std::fs::create_dir_all(&custom_dir) {
            if let AppMode::AddCommand(ref mut s) = self.mode {
                s.error = format!("无法创建目录: {}", e);
            }
            return;
        }

        let file_path = custom_dir.join("my_commands.yaml");

        // 如果文件不存在，写入文件头
        let needs_header = !file_path.exists();
        let mut content = String::new();
        if needs_header {
            content.push_str("category: \"我的命令\"\ndescription: \"自定义的常用命令\"\nplatform: dev\ncommands:\n");
        }
        content.push_str(&entry);

        let write_result = if needs_header {
            std::fs::write(&file_path, &content)
        } else {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new().append(true).open(&file_path);
            match f {
                Ok(ref mut file) => file.write_all(entry.as_bytes()),
                Err(e) => Err(e),
            }
        };

        if let Err(e) = write_result {
            if let AppMode::AddCommand(ref mut s) = self.mode {
                s.error = format!("保存失败: {}", e);
            }
            return;
        }

        // 重新加载数据
        self.platforms = crate::data::load_all_data();

        // 重建侧边栏
        let detected = Self::detect_platform_index(&self.platforms);
        let mut sidebar_items = Vec::new();
        sidebar_items.push(SidebarItem { kind: SidebarItemKind::Bookmarks, platform_index: 0, category_index: None, expanded: false });
        sidebar_items.push(SidebarItem { kind: SidebarItemKind::History, platform_index: 0, category_index: None, expanded: false });
        for (pi, platform) in self.platforms.iter().enumerate() {
            let is_current = pi == detected;
            sidebar_items.push(SidebarItem { kind: SidebarItemKind::Platform, platform_index: pi, category_index: None, expanded: is_current });
            if is_current {
                for (ci, _) in platform.categories.iter().enumerate() {
                    sidebar_items.push(SidebarItem { kind: SidebarItemKind::Category, platform_index: pi, category_index: Some(ci), expanded: false });
                }
            }
        }
        self.sidebar_items = sidebar_items;

        // 跳转到 dev 平台 / 我的命令 分类
        let cmd_name = name.clone();

        // 先收集目标索引，避免同时借用 self.platforms 和 self.sidebar_items
        let target = self.platforms.iter().enumerate().find_map(|(pi, platform)| {
            if platform.name != "dev" { return None; }
            platform.categories.iter().enumerate().find_map(|(ci, cat)| {
                if cat.name != "我的命令" { return None; }
                cat.commands.iter().position(|c| c.name == cmd_name)
                    .map(|cmd_idx| (pi, ci, cmd_idx))
            })
        });

        if let Some((pi, ci, cmd_idx)) = target {
            self.sidebar_cursor = self.sidebar_items.iter()
                .position(|i| i.kind == SidebarItemKind::Platform && i.platform_index == pi)
                .unwrap_or(0);
            self.toggle_sidebar_item();

            if let Some(si) = self.sidebar_items.iter().position(|item| {
                item.kind == SidebarItemKind::Category
                    && item.platform_index == pi
                    && item.category_index == Some(ci)
            }) {
                self.sidebar_cursor = si;
                self.selected_command = Some(cmd_idx);
                self.content_cursor = cmd_idx;
                self.focus = Focus::Content;
            }
        }


        self.mode = AppMode::Normal;
        self.set_status(format!("✓ 已保存: {}", cmd_name));
    }


    pub fn search_input(&mut self, c: char) {
        self.search_query.push(c);
        self.perform_search();
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.perform_search();
    }

    pub fn move_search_up(&mut self) {
        if self.search_cursor > 0 {
            self.search_cursor -= 1;
        }
    }

    pub fn move_search_down(&mut self) {
        if self.search_cursor < self.search_results.len().saturating_sub(1) {
            self.search_cursor += 1;
        }
    }

    fn perform_search(&mut self) {
        let results = self.search_engine.search(&self.search_query, &self.platforms);
        self.search_results = results
            .into_iter()
            .map(|r| {
                let pi = self.platforms.iter().position(|p| std::ptr::eq(p, r.platform)).unwrap_or(0);
                let ci = self.platforms[pi].categories.iter().position(|c| std::ptr::eq(c, r.category)).unwrap_or(0);
                let cmi = self.platforms[pi].categories[ci].commands.iter().position(|cmd| std::ptr::eq(cmd, r.command)).unwrap_or(0);
                SearchResultIndex {
                    platform_index: pi,
                    category_index: ci,
                    command_index: cmi,
                    score: r.score,
                }
            })
            .collect();
        self.search_cursor = 0;
    }

    /// 从搜索结果跳转到命令详情
    pub fn select_search_result(&mut self) {
        if let Some(result) = self.search_results.get(self.search_cursor).cloned() {
            self.mode = AppMode::Normal;
            self.focus = Focus::Content;

            let target_pi = result.platform_index;
            let target_ci = result.category_index;

            // 先将 cursor 定位到目标平台，再展开
            let mut found_platform = false;
            for (i, item) in self.sidebar_items.iter().enumerate() {
                if item.kind == SidebarItemKind::Platform && item.platform_index == target_pi {
                    self.sidebar_cursor = i;
                    if !item.expanded {
                        self.toggle_sidebar_item(); // 展开，cursor 已指向该平台
                    }
                    found_platform = true;
                    break;
                }
            }

            if !found_platform {
                return;
            }

            // 定位到目标分类
            for (i, item) in self.sidebar_items.iter().enumerate() {
                if item.kind == SidebarItemKind::Category
                    && item.platform_index == target_pi
                    && item.category_index == Some(target_ci)
                {
                    self.sidebar_cursor = i;
                    self.selected_command = Some(result.command_index);
                    self.content_cursor = result.command_index;
                    break;
                }
            }
        }
    }

    /// 快速跳转到指定平台
    pub fn jump_to_platform(&mut self, index: usize) {
        for (i, item) in self.sidebar_items.iter().enumerate() {
            if item.kind == SidebarItemKind::Platform && item.platform_index == index {
                self.sidebar_cursor = i;
                self.focus = Focus::Sidebar;
                self.update_selection();
                return;
            }
        }
    }

    /// 获取总命令数
    pub fn total_commands(&self) -> usize {
        self.platforms.iter().map(|p| p.command_count()).sum()
    }
}
