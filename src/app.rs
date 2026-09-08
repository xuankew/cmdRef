use crate::data::{Category, Command, Platform};
use crate::search::{CommandSearchResult, PromptSearchResult, PasswordSearchResult, SearchEngine};
use crate::bookmarks::BookmarkManager;
use crate::history::HistoryManager;
use crate::prompts::{Prompt, PromptStore};
use crate::passwords::{Password, PasswordStore};
use crate::crypto::CryptoContext;

/// 检查 my_commands.yaml 中是否已存在指定 name 的自定义命令。
/// 文件不存在或解析失败时返回 false。
pub fn is_existing_custom_command(name: &str) -> bool {
    let name = name.trim();
    if name.is_empty() {
        return false;
    }
    let path = crate::paths::custom_dir().join("my_commands.yaml");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let cmd_file: crate::data::CommandFile = match serde_yaml::from_str(&content) {
        Ok(f) => f,
        Err(_) => return false,
    };
    cmd_file.commands.iter().any(|c| c.name == name)
}

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

/// Prompt 编辑器中的字段
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PromptField {
    Name,
    Description,
    Content,
    Tags,
}

impl PromptField {
    pub fn next(self) -> Self {
        match self {
            PromptField::Name => PromptField::Description,
            PromptField::Description => PromptField::Content,
            PromptField::Content => PromptField::Tags,
            PromptField::Tags => PromptField::Name,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            PromptField::Name => PromptField::Tags,
            PromptField::Description => PromptField::Name,
            PromptField::Content => PromptField::Description,
            PromptField::Tags => PromptField::Content,
        }
    }
}

/// Password 编辑器中的字段
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PasswordField {
    Name,
    Value,
    Description,
}

impl PasswordField {
    pub fn next(self) -> Self {
        match self {
            PasswordField::Name => PasswordField::Value,
            PasswordField::Value => PasswordField::Description,
            PasswordField::Description => PasswordField::Name,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            PasswordField::Name => PasswordField::Description,
            PasswordField::Value => PasswordField::Name,
            PasswordField::Description => PasswordField::Value,
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

/// Prompt 编辑器状态
#[derive(Debug, Clone, PartialEq)]
pub struct PromptEditorState {
    pub active_field: PromptField,
    pub name: String,
    pub description: String,
    pub content: String,
    pub tags: String,
    pub error: String,
    /// Some(原名) 表示编辑模式
    pub editing: Option<String>,
}

impl PromptEditorState {
    pub fn new() -> Self {
        Self {
            active_field: PromptField::Name,
            name: String::new(),
            description: String::new(),
            content: String::new(),
            tags: String::new(),
            error: String::new(),
            editing: None,
        }
    }

    pub fn from_prompt(prompt: &Prompt) -> Self {
        Self {
            active_field: PromptField::Name,
            name: prompt.name.clone(),
            description: prompt.description.clone(),
            content: prompt.content.clone(),
            tags: prompt.tags.join(", "),
            error: String::new(),
            editing: Some(prompt.name.clone()),
        }
    }

    pub fn active_field_mut(&mut self) -> &mut String {
        match self.active_field {
            PromptField::Name => &mut self.name,
            PromptField::Description => &mut self.description,
            PromptField::Content => &mut self.content,
            PromptField::Tags => &mut self.tags,
        }
    }
}

/// Password 编辑器状态
#[derive(Debug, Clone, PartialEq)]
pub struct PasswordEditorState {
    pub active_field: PasswordField,
    pub name: String,
    pub value: String,
    pub description: String,
    pub error: String,
    /// Some(原名) 表示编辑模式
    pub editing: Option<String>,
}

impl PasswordEditorState {
    pub fn new() -> Self {
        Self {
            active_field: PasswordField::Name,
            name: String::new(),
            value: String::new(),
            description: String::new(),
            error: String::new(),
            editing: None,
        }
    }

    pub fn from_password(pw: &Password, decrypted_value: String) -> Self {
        Self {
            active_field: PasswordField::Name,
            name: pw.name.clone(),
            value: decrypted_value,
            description: pw.description.clone(),
            error: String::new(),
            editing: Some(pw.name.clone()),
        }
    }

    pub fn active_field_mut(&mut self) -> &mut String {
        match self.active_field {
            PasswordField::Name => &mut self.name,
            PasswordField::Value => &mut self.value,
            PasswordField::Description => &mut self.description,
        }
    }
}

/// 主密码输入状态
#[derive(Debug, Clone, PartialEq)]
pub struct MasterPasswordState {
    pub input: String,
    pub confirm: String,
    pub active_field: u8,       // 0 = password, 1 = confirm (create/change mode)
    pub is_create_mode: bool,
    pub error: String,
    pub pending_action: Option<PendingAction>,
}

impl MasterPasswordState {
    pub fn is_change_mode(&self) -> bool {
        self.pending_action == Some(PendingAction::ChangeMasterPassword)
    }

    fn for_unlock(action: PendingAction, is_create: bool) -> Self {
        Self {
            input: String::new(),
            confirm: String::new(),
            active_field: 0,
            is_create_mode: is_create,
            error: String::new(),
            pending_action: Some(action),
        }
    }
}

/// 等待主密码解锁后执行的操作
#[derive(Debug, Clone, PartialEq)]
pub enum PendingAction {
    AddPassword,
    EditPassword,
    CopyPassword,
    ChangeMasterPassword,
}

/// 应用模式
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Search,
    AddCommand(EditorState),
    PromptEditor(PromptEditorState),
    PasswordEditor(PasswordEditorState),
    MasterPassword(MasterPasswordState),
}

/// 侧边栏中的项目类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarItemKind {
    Bookmarks,
    History,
    Prompts,
    Passwords,
    Platform,
    Category,
}

/// 侧边栏固定索引常量
pub const SIDEBAR_BOOKMARKS: usize = 0;
pub const SIDEBAR_HISTORY: usize = 1;
pub const SIDEBAR_PROMPTS: usize = 2;
pub const SIDEBAR_PASSWORDS: usize = 3;

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
    pub prompts: PromptStore,
    pub passwords: PasswordStore,
    pub crypto: Option<CryptoContext>,

    // 侧边栏状态
    pub sidebar_items: Vec<SidebarItem>,
    pub sidebar_cursor: usize,

    // 内容区域状态
    pub content_cursor: usize,
    pub selected_command: Option<usize>,
    pub detail_scroll: usize,

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
pub enum SearchResultIndex {
    Command {
        platform_index: usize,
        category_index: usize,
        command_index: usize,
        score: i64,
    },
    Prompt {
        prompt_index: usize,
        score: i64,
    },
    Password {
        password_index: usize,
        score: i64,
    },
}

impl SearchResultIndex {
    pub fn score(&self) -> i64 {
        match self {
            SearchResultIndex::Command { score, .. } => *score,
            SearchResultIndex::Prompt { score, .. } => *score,
            SearchResultIndex::Password { score, .. } => *score,
        }
    }
}

impl App {
    pub fn new(platforms: Vec<Platform>) -> Self {
        let detected_platform = Self::detect_platform_index(&platforms);
        let sidebar_items = Self::build_sidebar_items(&platforms, detected_platform);

        let search_engine = SearchEngine::new();
        let bookmarks = BookmarkManager::new();
        let history = HistoryManager::new();
        let prompts = PromptStore::load();
        let passwords = PasswordStore::load();

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
            prompts,
            passwords,
            crypto: None,
            sidebar_items,
            sidebar_cursor: initial_cursor,
            content_cursor: 0,
            selected_command: Some(0),
            detail_scroll: 0,
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
        debug_log!("App initialized: {} platforms, {} bookmarks, {} history, {} passwords",
            app.platforms.len(), app.bookmarks.count(), app.history.count(), app.passwords.count());
        app
    }

    /// 构建侧边栏项目列表（Bookmarks、History、各平台）
    /// custom 平台始终展开；detected_platform 对应的 OS 平台也展开
    fn build_sidebar_items(platforms: &[Platform], detected_platform: usize) -> Vec<SidebarItem> {
        let mut items = vec![
            SidebarItem { kind: SidebarItemKind::Bookmarks, platform_index: 0, category_index: None, expanded: false },
            SidebarItem { kind: SidebarItemKind::History,   platform_index: 0, category_index: None, expanded: false },
            SidebarItem { kind: SidebarItemKind::Prompts,   platform_index: 0, category_index: None, expanded: false },
            SidebarItem { kind: SidebarItemKind::Passwords, platform_index: 0, category_index: None, expanded: false },
        ];

        for (pi, platform) in platforms.iter().enumerate() {
            let is_custom  = platform.name == "custom";
            let is_current = pi == detected_platform;
            let expanded   = is_custom || is_current;

            items.push(SidebarItem {
                kind: SidebarItemKind::Platform,
                platform_index: pi,
                category_index: None,
                expanded,
            });

            if expanded {
                for (ci, _) in platform.categories.iter().enumerate() {
                    items.push(SidebarItem {
                        kind: SidebarItemKind::Category,
                        platform_index: pi,
                        category_index: Some(ci),
                        expanded: false,
                    });
                }
            }
        }

        items
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
                SidebarItemKind::Bookmarks | SidebarItemKind::History | SidebarItemKind::Prompts | SidebarItemKind::Passwords => {
                    // 保留已选中的命令索引，仅在越界时重置
                    let len = match item.kind {
                        SidebarItemKind::Bookmarks => self.bookmarks.count(),
                        SidebarItemKind::History => self.history.count(),
                        SidebarItemKind::Prompts => self.prompts.count(),
                        SidebarItemKind::Passwords => self.passwords.count(),
                        _ => 0,
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
        // Prompts 和 Passwords 不属于命令，不记录到历史
        if let Some(item) = self.current_sidebar_item() {
            if item.kind == SidebarItemKind::Prompts || item.kind == SidebarItemKind::Passwords {
                return;
            }
        }

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
            if let Some(SearchResultIndex::Command {
                platform_index,
                category_index,
                command_index,
                ..
            }) = self.search_results.get(self.search_cursor)
            {
                return self.platforms
                    .get(*platform_index)
                    .and_then(|p| p.categories.get(*category_index))
                    .and_then(|c| c.commands.get(*command_index));
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
            SidebarItemKind::Prompts | SidebarItemKind::Passwords => None,
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
                SidebarItemKind::Bookmarks | SidebarItemKind::History | SidebarItemKind::Prompts | SidebarItemKind::Passwords => {}
            }
        }
        &[]
    }

    /// 获取当前选中的 Prompt
    pub fn current_prompt(&self) -> Option<&Prompt> {
        if matches!(self.mode, AppMode::Search) {
            if let Some(SearchResultIndex::Prompt { prompt_index, .. }) = self.search_results.get(self.search_cursor) {
                return self.prompts.get(*prompt_index);
            }
            return None;
        }

        if let Some(item) = self.current_sidebar_item() {
            if item.kind == SidebarItemKind::Prompts {
                return self.prompts.get(self.selected_command.unwrap_or(0));
            }
        }
        None
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
                SidebarItemKind::Bookmarks | SidebarItemKind::History | SidebarItemKind::Prompts | SidebarItemKind::Passwords => None,
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
                SidebarItemKind::Bookmarks | SidebarItemKind::History | SidebarItemKind::Prompts | SidebarItemKind::Passwords => None,
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

    /// 按 1-based 编号复制对应 example code 到剪贴板
    pub fn copy_example_by_index(&mut self, index: usize) {
        let text = if let Some(cmd) = self.current_command() {
            cmd.examples.get(index.saturating_sub(1)).map(|e| e.code.clone())
        } else {
            None
        };

        match text {
            Some(code) => {
                match crate::clipboard::copy_to_clipboard(&code) {
                    Ok(()) => {
                        let preview = if code.len() > 50 { format!("{}…", &code[..50]) } else { code.clone() };
                        self.set_status(format!("✓ [{}] {}", index, preview));
                    }
                    Err(e) => {
                        self.set_status(format!("✗ 复制失败: {}", e));
                    }
                }
            }
            None => {
                self.set_status(format!("✗ 没有第 {} 条命令", index));
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
        self.sidebar_cursor = SIDEBAR_BOOKMARKS;
        self.selected_command = Some(0);
        self.content_cursor = 0;
    }

    /// 跳转到历史记录
    pub fn jump_to_history(&mut self) {
        self.mode = AppMode::Normal;
        self.focus = Focus::Sidebar;
        self.sidebar_cursor = SIDEBAR_HISTORY;
        self.selected_command = Some(0);
        self.content_cursor = 0;
    }

    /// 跳转到 Prompts
    pub fn jump_to_prompts(&mut self) {
        self.mode = AppMode::Normal;
        self.focus = Focus::Sidebar;
        self.sidebar_cursor = SIDEBAR_PROMPTS;
        self.selected_command = Some(0);
        self.content_cursor = 0;
    }

    /// 删除当前选中的自定义命令（仅限 custom 平台）
    pub fn delete_custom_command(&mut self) {
        // 确认当前在 custom 平台
        let is_custom = self.current_platform().map(|p| p.name == "custom").unwrap_or(false);
        if !is_custom {
            return;
        }

        let cmd_name = match self.current_command() {
            Some(c) => c.name.clone(),
            None => return,
        };

        let file_path = crate::paths::custom_dir().join("my_commands.yaml");

        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        // 重新解析，删除目标命令，重新序列化写回
        let mut cmd_file: crate::data::CommandFile = match serde_yaml::from_str(&content) {
            Ok(f) => f,
            Err(_) => return,
        };

        let before = cmd_file.commands.len();
        cmd_file.commands.retain(|c| c.name != cmd_name);
        if cmd_file.commands.len() == before {
            return;
        }

        // 全部删完就直接删文件
        if cmd_file.commands.is_empty() {
            let _ = std::fs::remove_file(&file_path);
        } else {
            // 序列化回文件
            let new_content = match serde_yaml::to_string(&cmd_file) {
                Ok(s) => s,
                Err(_) => return,
            };
            if std::fs::write(&file_path, new_content).is_err() {
                self.set_status("✗ 删除失败：写文件错误".to_string());
                return;
            }
        }

        // 热重载
        self.platforms = crate::data::load_all_data();
        let detected = Self::detect_platform_index(&self.platforms);
        self.sidebar_items = Self::build_sidebar_items(&self.platforms, detected);

        // 定位光标：留在 custom 平台，索引往前挪一步
        let new_idx = self.selected_command.unwrap_or(0).saturating_sub(1);
        let custom_pi = self.platforms.iter().position(|p| p.name == "custom");
        if let Some(pi) = custom_pi {
            if let Some(si) = self.sidebar_items.iter().position(|i| {
                i.kind == SidebarItemKind::Category && i.platform_index == pi
            }) {
                self.sidebar_cursor = si;
                self.selected_command = Some(new_idx);
                self.content_cursor = new_idx;
                self.focus = Focus::Content;
            }
        }

        self.set_status(format!("✓ 已删除: {}", cmd_name));
    }

    /// 打开 Prompt 添加表单
    pub fn open_add_prompt(&mut self) {
        self.mode = AppMode::PromptEditor(PromptEditorState::new());
    }

    /// 打开 Prompt 编辑表单
    pub fn open_edit_prompt(&mut self) {
        if let Some(prompt) = self.current_prompt().cloned() {
            self.mode = AppMode::PromptEditor(PromptEditorState::from_prompt(&prompt));
        }
    }

    /// 取消 Prompt 编辑
    pub fn prompt_editor_cancel(&mut self) {
        self.mode = AppMode::Normal;
    }

    /// 向当前 Prompt 编辑字段输入字符
    pub fn prompt_editor_input(&mut self, c: char) {
        if let AppMode::PromptEditor(ref mut state) = self.mode {
            state.active_field_mut().push(c);
        }
    }

    /// 删除当前 Prompt 编辑字段末尾字符
    pub fn prompt_editor_backspace(&mut self) {
        if let AppMode::PromptEditor(ref mut state) = self.mode {
            state.active_field_mut().pop();
        }
    }

    /// 切换到 Prompt 编辑器的下一个字段
    pub fn prompt_editor_next_field(&mut self) {
        if let AppMode::PromptEditor(ref mut state) = self.mode {
            state.active_field = state.active_field.next();
        }
    }

    /// 切换到 Prompt 编辑器的上一个字段
    pub fn prompt_editor_prev_field(&mut self) {
        if let AppMode::PromptEditor(ref mut state) = self.mode {
            state.active_field = state.active_field.prev();
        }
    }

    /// 保存 Prompt 并热重载
    pub fn prompt_editor_save(&mut self) {
        let state = if let AppMode::PromptEditor(ref s) = self.mode {
            s.clone()
        } else {
            return;
        };

        let name = state.name.trim().to_string();
        let description = state.description.trim().to_string();
        let content = state.content.to_string();
        let tags: Vec<String> = state
            .tags
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        let prompt = Prompt {
            name,
            description,
            content,
            tags,
            source: std::path::PathBuf::new(),
        };

        match self.prompts.upsert(prompt, state.editing.as_deref()) {
            Ok(idx) => {
                self.mode = AppMode::Normal;
                // 跳转到 Prompts 内容区并选中刚保存的项
                self.sidebar_cursor = SIDEBAR_PROMPTS;
                self.selected_command = Some(idx);
                self.content_cursor = idx;
                self.focus = Focus::Content;
                self.detail_scroll = 0;
                self.set_status(format!("✓ 已保存: {}", self.prompts.get(idx).map(|p| p.name.clone()).unwrap_or_default()));
            }
            Err(e) => {
                if let AppMode::PromptEditor(ref mut s) = self.mode {
                    s.error = e;
                }
            }
        }
    }

    /// 删除当前选中的 Prompt
    pub fn delete_current_prompt(&mut self) {
        if let Some(name) = self.current_prompt().map(|p| p.name.clone()) {
            match self.prompts.delete(&name) {
                Ok(()) => {
                    let idx = self.selected_command.unwrap_or(0).saturating_sub(1);
                    self.selected_command = if self.prompts.count() > 0 { Some(idx) } else { None };
                    self.content_cursor = idx;
                    self.detail_scroll = 0;
                    self.set_status(format!("✓ 已删除: {}", name));
                }
                Err(e) => {
                    self.set_status(format!("✗ 删除失败: {}", e));
                }
            }
        }
    }

    /// 复制当前 Prompt 内容到剪贴板
    pub fn copy_current_prompt(&mut self) {
        if let Some(prompt) = self.current_prompt() {
            match crate::clipboard::copy_to_clipboard(&prompt.content) {
                Ok(()) => {
                    let preview = if prompt.content.len() > 50 {
                        format!("{}…", &prompt.content[..50])
                    } else {
                        prompt.content.clone()
                    };
                    self.set_status(format!("✓ 已复制: {}", preview));
                }
                Err(e) => {
                    self.set_status(format!("✗ 复制失败: {}", e));
                }
            }
        }
    }

    // ======== Password 操作 ========

    /// 获取当前选中的 Password
    pub fn current_password(&self) -> Option<&Password> {
        if matches!(self.mode, AppMode::Search) {
            if let Some(SearchResultIndex::Password { password_index, .. }) = self.search_results.get(self.search_cursor) {
                return self.passwords.get(*password_index);
            }
            return None;
        }

        if let Some(item) = self.current_sidebar_item() {
            if item.kind == SidebarItemKind::Passwords {
                return self.passwords.get(self.selected_command.unwrap_or(0));
            }
        }
        None
    }

    /// 跳转到 Passwords
    pub fn jump_to_passwords(&mut self) {
        self.mode = AppMode::Normal;
        self.focus = Focus::Sidebar;
        self.sidebar_cursor = SIDEBAR_PASSWORDS;
        self.selected_command = Some(0);
        self.content_cursor = 0;
    }

    /// 检查加密上下文是否就绪，若未就绪则弹出主密码输入
    fn ensure_crypto_ready(&mut self, action: PendingAction) -> bool {
        if self.crypto.is_some() {
            return true;
        }
        let has_salt = crate::crypto::has_salt();
        self.mode = AppMode::MasterPassword(MasterPasswordState::for_unlock(action, !has_salt));
        false
    }

    /// 打开 Password 添加表单
    pub fn open_add_password(&mut self) {
        if !self.ensure_crypto_ready(PendingAction::AddPassword) {
            return;
        }
        self.mode = AppMode::PasswordEditor(PasswordEditorState::new());
    }

    /// 打开 Password 编辑表单
    pub fn open_edit_password(&mut self) {
        let pw = match self.current_password() {
            Some(p) => p.clone(),
            None => return,
        };

        if !self.ensure_crypto_ready(PendingAction::EditPassword) {
            return;
        }

        // 解密 value
        let decrypted = match &self.crypto {
            Some(ctx) => match ctx.decrypt(&pw.value) {
                Ok(v) => v,
                Err(e) => {
                    self.set_status(format!("✗ 解密失败: {}", e));
                    return;
                }
            },
            None => return,
        };

        self.mode = AppMode::PasswordEditor(PasswordEditorState::from_password(&pw, decrypted));
    }

    /// 向当前 Password 编辑字段输入字符
    pub fn password_editor_input(&mut self, c: char) {
        if let AppMode::PasswordEditor(ref mut state) = self.mode {
            state.active_field_mut().push(c);
        }
    }

    /// 删除当前 Password 编辑字段末尾字符
    pub fn password_editor_backspace(&mut self) {
        if let AppMode::PasswordEditor(ref mut state) = self.mode {
            state.active_field_mut().pop();
        }
    }

    /// 切换到 Password 编辑器的下一个字段
    pub fn password_editor_next_field(&mut self) {
        if let AppMode::PasswordEditor(ref mut state) = self.mode {
            state.active_field = state.active_field.next();
        }
    }

    /// 切换到 Password 编辑器的上一个字段
    pub fn password_editor_prev_field(&mut self) {
        if let AppMode::PasswordEditor(ref mut state) = self.mode {
            state.active_field = state.active_field.prev();
        }
    }

    /// 取消 Password 编辑
    pub fn password_editor_cancel(&mut self) {
        self.mode = AppMode::Normal;
    }

    /// 保存 Password
    pub fn password_editor_save(&mut self) {
        let state = if let AppMode::PasswordEditor(ref s) = self.mode {
            s.clone()
        } else {
            return;
        };

        let name = state.name.trim().to_string();
        let description = state.description.trim().to_string();
        let plaintext_value = state.value.clone();

        if name.is_empty() {
            if let AppMode::PasswordEditor(ref mut s) = self.mode {
                s.error = "名称不能为空".to_string();
            }
            return;
        }

        if plaintext_value.is_empty() {
            if let AppMode::PasswordEditor(ref mut s) = self.mode {
                s.error = "密码值不能为空".to_string();
            }
            return;
        }

        // 加密 value
        let encrypted = match &self.crypto {
            Some(ctx) => match ctx.encrypt(&plaintext_value) {
                Ok(v) => v,
                Err(e) => {
                    if let AppMode::PasswordEditor(ref mut s) = self.mode {
                        s.error = format!("加密失败: {}", e);
                    }
                    return;
                }
            },
            None => {
                if let AppMode::PasswordEditor(ref mut s) = self.mode {
                    s.error = "加密上下文未就绪".to_string();
                }
                return;
            }
        };

        let password = Password {
            name,
            description,
            value: encrypted,
            source: std::path::PathBuf::new(),
        };

        match self.passwords.upsert(password, state.editing.as_deref()) {
            Ok(idx) => {
                self.mode = AppMode::Normal;
                self.sidebar_cursor = SIDEBAR_PASSWORDS;
                self.selected_command = Some(idx);
                self.content_cursor = idx;
                self.focus = Focus::Content;
                self.detail_scroll = 0;
                self.set_status(format!("✓ 已保存: {}",
                    self.passwords.get(idx).map(|p| p.name.clone()).unwrap_or_default()));
            }
            Err(e) => {
                if let AppMode::PasswordEditor(ref mut s) = self.mode {
                    s.error = e;
                }
            }
        }
    }

    /// 删除当前选中的 Password
    pub fn delete_current_password(&mut self) {
        if let Some(name) = self.current_password().map(|p| p.name.clone()) {
            match self.passwords.delete(&name) {
                Ok(()) => {
                    let idx = self.selected_command.unwrap_or(0).saturating_sub(1);
                    self.selected_command = if self.passwords.count() > 0 { Some(idx) } else { None };
                    self.content_cursor = idx;
                    self.detail_scroll = 0;
                    self.set_status(format!("✓ 已删除: {}", name));
                }
                Err(e) => {
                    self.set_status(format!("✗ 删除失败: {}", e));
                }
            }
        }
    }

    /// 复制当前 Password 的解密值到剪贴板
    pub fn copy_current_password(&mut self) {
        let pw = match self.current_password() {
            Some(p) => p.clone(),
            None => return,
        };

        if !self.ensure_crypto_ready(PendingAction::CopyPassword) {
            return;
        }

        let decrypted = match &self.crypto {
            Some(ctx) => match ctx.decrypt(&pw.value) {
                Ok(v) => v,
                Err(e) => {
                    self.set_status(format!("✗ 解密失败: {}", e));
                    return;
                }
            },
            None => return,
        };

        match crate::clipboard::copy_to_clipboard(&decrypted) {
            Ok(()) => {
                self.set_status("✓ 密码已复制到剪贴板".to_string());
            }
            Err(e) => {
                self.set_status(format!("✗ 复制失败: {}", e));
            }
        }
    }

    /// 主密码输入字符
    pub fn master_password_input(&mut self, c: char) {
        if let AppMode::MasterPassword(ref mut state) = self.mode {
            if state.is_create_mode && state.active_field == 1 {
                state.confirm.push(c);
            } else {
                state.input.push(c);
            }
        }
    }

    /// 主密码退格
    pub fn master_password_backspace(&mut self) {
        if let AppMode::MasterPassword(ref mut state) = self.mode {
            if state.is_create_mode && state.active_field == 1 {
                state.confirm.pop();
            } else {
                state.input.pop();
            }
        }
    }

    /// 提交主密码
    pub fn master_password_submit(&mut self) {
        let state = if let AppMode::MasterPassword(ref s) = self.mode {
            s.clone()
        } else {
            return;
        };

        if state.is_create_mode {
            if state.active_field == 0 {
                // 第一个字段（主密码）输入完毕，切换到确认字段
                if state.input.is_empty() {
                    if let AppMode::MasterPassword(ref mut s) = self.mode {
                        s.error = "请输入主密码".to_string();
                    }
                    return;
                }
                if let AppMode::MasterPassword(ref mut s) = self.mode {
                    s.active_field = 1;
                    s.error = String::new();
                }
                return;
            }

            // active_field == 1: 确认密码
            if state.input != state.confirm {
                if let AppMode::MasterPassword(ref mut s) = self.mode {
                    s.error = "两次输入的密码不一致".to_string();
                    s.confirm.clear();
                }
                return;
            }

            // 生成新 salt
            let new_salt = crate::crypto::generate_salt();
            if let Err(e) = crate::crypto::save_salt(&new_salt) {
                if let AppMode::MasterPassword(ref mut s) = self.mode {
                    s.error = e;
                }
                return;
            }

            let new_ctx = match CryptoContext::new(&state.input, new_salt) {
                Ok(ctx) => ctx,
                Err(e) => {
                    if let AppMode::MasterPassword(ref mut s) = self.mode {
                        s.error = e;
                    }
                    return;
                }
            };

            // 如果是修改主密码流程，需要用新密钥重新加密所有条目
            if state.is_change_mode() {
                if let Some(ref old_ctx) = self.crypto {
                    let mut passwords_to_update: Vec<(String, String, String)> = Vec::new();
                    for pw in self.passwords.all() {
                        match old_ctx.decrypt(&pw.value) {
                            Ok(plaintext) => {
                                match new_ctx.encrypt(&plaintext) {
                                    Ok(new_encrypted) => {
                                        passwords_to_update.push((
                                            pw.name.clone(),
                                            new_encrypted,
                                            pw.description.clone(),
                                        ));
                                    }
                                    Err(e) => {
                                        if let AppMode::MasterPassword(ref mut s) = self.mode {
                                            s.error = format!("重新加密失败: {}", e);
                                        }
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                if let AppMode::MasterPassword(ref mut s) = self.mode {
                                    s.error = format!("解密旧密码失败: {}", e);
                                }
                                return;
                            }
                        }
                    }

                    // 用新密文更新每个条目
                    for (name, new_value, description) in passwords_to_update {
                        let updated = Password {
                            name: name.clone(),
                            description,
                            value: new_value,
                            source: std::path::PathBuf::new(),
                        };
                        if let Err(e) = self.passwords.upsert(updated, Some(&name)) {
                            if let AppMode::MasterPassword(ref mut s) = self.mode {
                                s.error = format!("保存失败: {}", e);
                            }
                            return;
                        }
                    }
                }
            }

            self.crypto = Some(new_ctx);
        } else {
            // 解锁/验证模式：加载 salt 并验证
            if state.input.is_empty() {
                if let AppMode::MasterPassword(ref mut s) = self.mode {
                    s.error = "请输入主密码".to_string();
                }
                return;
            }

            let salt = match crate::crypto::load_salt() {
                Some(s) => s,
                None => {
                    if let AppMode::MasterPassword(ref mut s) = self.mode {
                        s.error = "Salt 文件不存在，请重新创建主密码".to_string();
                        s.is_create_mode = true;
                        s.active_field = 0;
                    }
                    return;
                }
            };

            let ctx = match CryptoContext::new(&state.input, salt) {
                Ok(ctx) => ctx,
                Err(e) => {
                    if let AppMode::MasterPassword(ref mut s) = self.mode {
                        s.error = e;
                    }
                    return;
                }
            };

            // 如果有密码条目，尝试解密第一个来验证主密码
            if let Some(first_pw) = self.passwords.get(0) {
                if ctx.decrypt(&first_pw.value).is_err() {
                    if let AppMode::MasterPassword(ref mut s) = self.mode {
                        s.error = "主密码错误".to_string();
                        s.input.clear();
                    }
                    return;
                }
            }

            // 如果是修改主密码流程，验证通过后切换到创建模式输入新密码
            if state.is_change_mode() {
                self.crypto = Some(ctx);
                if let AppMode::MasterPassword(ref mut s) = self.mode {
                    s.is_create_mode = true;
                    s.active_field = 0;
                    s.input.clear();
                    s.confirm.clear();
                    s.error = String::new();
                }
                return;
            }

            self.crypto = Some(ctx);
        }

        // 执行挂起的操作
        let action = state.pending_action.clone();
        self.mode = AppMode::Normal;

        if let Some(action) = action {
            self.execute_pending_action(action);
        }
    }

    /// 切换到确认密码字段（创建模式）
    pub fn master_password_next_field(&mut self) {
        if let AppMode::MasterPassword(ref mut state) = self.mode {
            if state.is_create_mode && state.active_field == 0 && !state.input.is_empty() {
                state.active_field = 1;
                state.error = String::new();
            }
        }
    }

    /// 取消主密码输入
    pub fn master_password_cancel(&mut self) {
        self.mode = AppMode::Normal;
    }

    /// 发起修改主密码流程
    pub fn change_master_password(&mut self) {
        if self.crypto.is_none() {
            self.set_status("请先解锁主密码".to_string());
            return;
        }
        if self.passwords.count() == 0 {
            self.set_status("暂无密码条目，无需修改主密码".to_string());
            return;
        }
        self.mode = AppMode::MasterPassword(
            MasterPasswordState::for_unlock(PendingAction::ChangeMasterPassword, false)
        );
    }

    /// 执行挂起的操作
    fn execute_pending_action(&mut self, action: PendingAction) {
        match action {
            PendingAction::AddPassword => {
                self.mode = AppMode::PasswordEditor(PasswordEditorState::new());
            }
            PendingAction::EditPassword => {
                self.open_edit_password();
            }
            PendingAction::CopyPassword => {
                self.copy_current_password();
            }
            PendingAction::ChangeMasterPassword => {
                self.set_status("✓ 主密码已修改".to_string());
            }
        }
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
                self.detail_scroll = 0;
            }
        }
    }

    pub fn move_content_down(&mut self) {
        let len = if let Some(item) = self.current_sidebar_item() {
            match item.kind {
                SidebarItemKind::Bookmarks => self.bookmarks.count(),
                SidebarItemKind::History => self.history.count(),
                SidebarItemKind::Prompts => self.prompts.count(),
                SidebarItemKind::Passwords => self.passwords.count(),
                _ => self.current_category_commands().len(),
            }
        } else {
            0
        };

        if let Some(idx) = self.selected_command {
            if idx < len.saturating_sub(1) {
                self.selected_command = Some(idx + 1);
                self.content_cursor = idx + 1;
                self.detail_scroll = 0;
            }
        }
    }

    pub fn scroll_detail_down(&mut self, step: usize) {
        self.detail_scroll = self.detail_scroll.saturating_add(step);
    }

    pub fn scroll_detail_up(&mut self, step: usize) {
        self.detail_scroll = self.detail_scroll.saturating_sub(step);
    }

    pub fn toggle_sidebar_item(&mut self) {
        let cursor = self.sidebar_cursor;
        if let Some(item) = self.sidebar_items.get(cursor) {
            // Bookmarks / History / Prompts / Passwords / Category → 直接进入内容区
            if item.kind == SidebarItemKind::Bookmarks
                || item.kind == SidebarItemKind::History
                || item.kind == SidebarItemKind::Prompts
                || item.kind == SidebarItemKind::Passwords
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

        // 按行拆分 command 字段，每行一条 example
        let lines: Vec<String> = command.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        if lines.is_empty() {
            if let AppMode::AddCommand(ref mut s) = self.mode {
                s.error = "命令内容不能为空".to_string();
            }
            return;
        }

        let new_examples: Vec<crate::data::Example> = lines.iter().map(|line| {
            crate::data::Example {
                description: "运行".to_string(),
                code: line.clone(),
                frequency: String::new(),
                danger: String::new(),
            }
        }).collect();

        let tags: Vec<String> = if state.tags.trim().is_empty() {
            Vec::new()
        } else {
            state.tags.split(',').map(|t| t.trim().to_string()).collect()
        };

        // 确定保存路径并确保目录存在
        let custom_dir = crate::paths::custom_dir();
        if let Err(e) = std::fs::create_dir_all(&custom_dir) {
            if let AppMode::AddCommand(ref mut s) = self.mode {
                s.error = format!("无法创建目录: {}", e);
            }
            return;
        }
        let file_path = custom_dir.join("my_commands.yaml");

        // 读取现有文件或构造默认文件头
        let mut cmd_file: crate::data::CommandFile = if file_path.exists() {
            match std::fs::read_to_string(&file_path) {
                Ok(content) => serde_yaml::from_str(&content).unwrap_or_else(|_| crate::data::CommandFile {
                    category: "我的命令".to_string(),
                    description: "自定义的常用命令".to_string(),
                    platform: "custom".to_string(),
                    commands: Vec::new(),
                }),
                Err(_) => crate::data::CommandFile {
                    category: "我的命令".to_string(),
                    description: "自定义的常用命令".to_string(),
                    platform: "custom".to_string(),
                    commands: Vec::new(),
                },
            }
        } else {
            crate::data::CommandFile {
                category: "我的命令".to_string(),
                description: "自定义的常用命令".to_string(),
                platform: "custom".to_string(),
                commands: Vec::new(),
            }
        };

        // 重名 → 追加 examples；否则新建命令
        let example_count = new_examples.len();
        let appended = if let Some(existing) = cmd_file.commands.iter_mut().find(|c| c.name == name) {
            existing.examples.extend(new_examples);
            true
        } else {
            cmd_file.commands.push(crate::data::Command {
                name: name.clone(),
                summary: lines[0].clone(),
                examples: new_examples,
                tips: Vec::new(),
                related: Vec::new(),
                tags,
            });
            false
        };

        let new_content = match serde_yaml::to_string(&cmd_file) {
            Ok(s) => s,
            Err(e) => {
                if let AppMode::AddCommand(ref mut s) = self.mode {
                    s.error = format!("序列化失败: {}", e);
                }
                return;
            }
        };
        if let Err(e) = std::fs::write(&file_path, new_content) {
            if let AppMode::AddCommand(ref mut s) = self.mode {
                s.error = format!("保存失败: {}", e);
            }
            return;
        }

        // 重新加载数据并重建侧边栏
        self.platforms = crate::data::load_all_data();
        let detected = Self::detect_platform_index(&self.platforms);
        self.sidebar_items = Self::build_sidebar_items(&self.platforms, detected);

        // 跳转到 我的命令 平台
        let cmd_name = name.clone();

        let target = self.platforms.iter().enumerate().find_map(|(pi, platform)| {
            if platform.name != "custom" { return None; }
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
        let msg = if appended {
            if example_count > 1 {
                format!("✓ 已追加 {} 条到: {}", example_count, cmd_name)
            } else {
                format!("✓ 已追加到: {}", cmd_name)
            }
        } else {
            if example_count > 1 {
                format!("✓ 已保存: {} ({} 条命令)", cmd_name, example_count)
            } else {
                format!("✓ 已保存: {}", cmd_name)
            }
        };
        self.set_status(msg);
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
        let command_results = self.search_engine.search_commands(&self.search_query, &self.platforms);
        let prompt_results = self.search_engine.search_prompts(&self.search_query, &self.prompts);
        let password_results = self.search_engine.search_passwords(&self.search_query, &self.passwords);

        let mut results: Vec<SearchResultIndex> = command_results
            .into_iter()
            .map(|r: CommandSearchResult| {
                let pi = self.platforms.iter().position(|p| std::ptr::eq(p, r.platform)).unwrap_or(0);
                let ci = self.platforms[pi].categories.iter().position(|c| std::ptr::eq(c, r.category)).unwrap_or(0);
                let cmi = self.platforms[pi].categories[ci].commands.iter().position(|cmd| std::ptr::eq(cmd, r.command)).unwrap_or(0);
                SearchResultIndex::Command {
                    platform_index: pi,
                    category_index: ci,
                    command_index: cmi,
                    score: r.score,
                }
            })
            .collect();

        results.extend(prompt_results.into_iter().map(|r: PromptSearchResult| {
            let pi = self.prompts.all().iter().position(|p| std::ptr::eq(p, r.prompt)).unwrap_or(0);
            SearchResultIndex::Prompt {
                prompt_index: pi,
                score: r.score,
            }
        }));

        results.extend(password_results.into_iter().map(|r: PasswordSearchResult| {
            let pi = self.passwords.all().iter().position(|p| std::ptr::eq(p, r.password)).unwrap_or(0);
            SearchResultIndex::Password {
                password_index: pi,
                score: r.score,
            }
        }));

        results.sort_by_key(|a| std::cmp::Reverse(a.score()));
        self.search_results = results;
        self.search_cursor = 0;
    }

    /// 从搜索结果跳转到对应详情（命令或 Prompt）
    pub fn select_search_result(&mut self) {
        if let Some(result) = self.search_results.get(self.search_cursor).cloned() {
            self.mode = AppMode::Normal;
            self.focus = Focus::Content;

            match result {
                SearchResultIndex::Prompt { prompt_index, .. } => {
                    self.sidebar_cursor = SIDEBAR_PROMPTS;
                    self.selected_command = Some(prompt_index);
                    self.content_cursor = prompt_index;
                    self.detail_scroll = 0;
                }
                SearchResultIndex::Password { password_index, .. } => {
                    self.sidebar_cursor = SIDEBAR_PASSWORDS;
                    self.selected_command = Some(password_index);
                    self.content_cursor = password_index;
                    self.detail_scroll = 0;
                }
                SearchResultIndex::Command {
                    platform_index: target_pi,
                    category_index: target_ci,
                    command_index: target_cmi,
                    ..
                } => {
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
                            self.selected_command = Some(target_cmi);
                            self.content_cursor = target_cmi;
                            self.detail_scroll = 0;
                            break;
                        }
                    }
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
