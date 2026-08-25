use crate::data::{Command, Category, Platform};
use crate::search::SearchEngine;

/// 焦点所在的面板
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Sidebar,
    Content,
    Search,
}

/// 应用模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    Search,
}

/// 侧边栏中的项目类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarItemKind {
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

    // 侧边栏状态
    pub sidebar_items: Vec<SidebarItem>,
    pub sidebar_cursor: usize,

    // 内容区域状态
    pub content_cursor: usize,
    pub selected_command: Option<usize>, // 当前在 category commands 中的索引

    // 焦点和模式
    pub focus: Focus,
    pub mode: AppMode,

    // 搜索
    pub search_query: String,
    pub search_results: Vec<SearchResultIndex>,
    pub search_cursor: usize,

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

        for (pi, platform) in platforms.iter().enumerate() {
            // 平台项
            sidebar_items.push(SidebarItem {
                kind: SidebarItemKind::Platform,
                platform_index: pi,
                category_index: None,
                expanded: pi == 0, // 默认展开第一个
            });

            // 默认展开第一个平台的分类
            if pi == 0 {
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

        let mut app = App {
            platforms,
            search_engine,
            sidebar_items,
            sidebar_cursor: 0,
            content_cursor: 0,
            selected_command: Some(0),
            focus: Focus::Sidebar,
            mode: AppMode::Normal,
            search_query: String::new(),
            search_results: Vec::new(),
            search_cursor: 0,
            should_quit: false,
        };

        // 初始化选中第一个命令
        app.update_selection();
        app
    }

    /// 更新当前选中项
    fn update_selection(&mut self) {
        if let Some(item) = self.current_sidebar_item() {
            match item.kind {
                SidebarItemKind::Platform => {
                    let pi = item.platform_index;
                    if let Some(platform) = self.platforms.get(pi) {
                        if let Some(cat) = platform.categories.first() {
                            self.selected_command = Some(0);
                            self.content_cursor = 0;
                            let _ = cat; // suppress unused warning
                        }
                    }
                }
                SidebarItemKind::Category => {
                    self.selected_command = Some(0);
                    self.content_cursor = 0;
                }
            }
        }
    }

    /// 获取当前侧边栏项的克隆
    fn current_sidebar_item(&self) -> Option<SidebarItem> {
        self.sidebar_items.get(self.sidebar_cursor).cloned()
    }

    /// 获取当前应该显示的命令
    pub fn current_command(&self) -> Option<&Command> {
        if self.mode == AppMode::Search {
            // 搜索模式下，返回搜索结果中的命令
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
            }
        }
        &[]
    }

    /// 获取当前分类信息
    pub fn current_category(&self) -> Option<&Category> {
        if let Some(item) = self.current_sidebar_item() {
            match item.kind {
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
    #[allow(dead_code)]
    pub fn current_platform(&self) -> Option<&Platform> {
        if let Some(item) = self.current_sidebar_item() {
            self.platforms.get(item.platform_index)
        } else {
            None
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
            }
        }
    }

    pub fn move_content_down(&mut self) {
        let commands = self.current_category_commands();
        if let Some(idx) = self.selected_command {
            if idx < commands.len().saturating_sub(1) {
                self.selected_command = Some(idx + 1);
                self.content_cursor = idx + 1;
            }
        }
    }

    pub fn toggle_sidebar_item(&mut self) {
        let cursor = self.sidebar_cursor;
        if let Some(item) = self.sidebar_items.get(cursor) {
            if item.kind == SidebarItemKind::Platform {
                let pi = item.platform_index;
                let was_expanded = item.expanded;

                // 重建侧边栏
                let mut new_items = Vec::new();
                for (i, si) in self.sidebar_items.iter().enumerate() {
                    if si.kind == SidebarItemKind::Platform && si.platform_index == pi {
                        new_items.push(SidebarItem {
                            expanded: !was_expanded,
                            ..si.clone()
                        });
                        if !was_expanded {
                            // 展开：插入子分类
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
                            // 折叠：跳过此项
                            continue;
                        } else {
                            new_items.push(si.clone());
                        }
                    } else {
                        new_items.push(si.clone());
                    }
                    // 保留非当前平台的项
                    let _ = i;
                }
                self.sidebar_items = new_items;

                // 确保光标有效
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
                // 找到对应的索引
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
            // 找到对应的侧边栏项并跳转
            self.mode = AppMode::Normal;
            self.focus = Focus::Content;

            // 确保目标平台已展开
            let target_pi = result.platform_index;
            let target_ci = result.category_index;

            // 展开平台
            let mut found_platform = false;
            for (_i, item) in self.sidebar_items.iter().enumerate() {
                if item.kind == SidebarItemKind::Platform && item.platform_index == target_pi {
                    if !item.expanded {
                        self.toggle_sidebar_item();
                    }
                    found_platform = true;
                    break;
                }
            }

            if !found_platform {
                return;
            }

            // 找到对应的分类项
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
