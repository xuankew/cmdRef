use serde::Deserialize;
use std::collections::BTreeMap;

/// 单条命令示例
#[derive(Debug, Clone, Deserialize)]
pub struct Example {
    pub description: String,
    pub code: String,
}

/// 命令定义
#[derive(Debug, Clone, Deserialize)]
pub struct Command {
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub examples: Vec<Example>,
    #[serde(default)]
    pub tips: Vec<String>,
    #[serde(default)]
    pub related: Vec<String>,
}

/// 单个 YAML 文件的数据结构
#[derive(Debug, Clone, Deserialize)]
pub struct CommandFile {
    pub category: String,
    pub description: String,
    pub platform: String,
    pub commands: Vec<Command>,
}

/// 分类（一个 YAML 文件对应一个分类）
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Category {
    pub name: String,
    pub description: String,
    pub platform: String,
    pub commands: Vec<Command>,
}

/// 平台
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Platform {
    pub name: String,
    pub display_name: String,
    pub categories: Vec<Category>,
}

impl Platform {
    pub fn command_count(&self) -> usize {
        self.categories.iter().map(|c| c.commands.len()).sum()
    }
}

impl Category {
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }
}

/// 加载所有嵌入的命令数据
pub fn load_all_data() -> Vec<Platform> {
    // 定义所有嵌入的 YAML 文件: (platform_name, platform_display, file_content)
    let yaml_entries: Vec<(&str, &str, &str)> = vec![
        // Linux
        ("linux", "Linux", include_str!("../data/commands/linux/file_ops.yaml")),
        ("linux", "Linux", include_str!("../data/commands/linux/text_proc.yaml")),
        ("linux", "Linux", include_str!("../data/commands/linux/editors.yaml")),
        ("linux", "Linux", include_str!("../data/commands/linux/archive.yaml")),
        ("linux", "Linux", include_str!("../data/commands/linux/network.yaml")),
        ("linux", "Linux", include_str!("../data/commands/linux/process.yaml")),
        ("linux", "Linux", include_str!("../data/commands/linux/system.yaml")),
        ("linux", "Linux", include_str!("../data/commands/linux/log_view.yaml")),
        ("linux", "Linux", include_str!("../data/commands/linux/user_mgmt.yaml")),
        // Mac
        ("mac", "macOS", include_str!("../data/commands/mac/brew.yaml")),
        ("mac", "macOS", include_str!("../data/commands/mac/system.yaml")),
        ("mac", "macOS", include_str!("../data/commands/mac/xcode.yaml")),
        // Windows
        ("windows", "Windows", include_str!("../data/commands/windows/powershell.yaml")),
        ("windows", "Windows", include_str!("../data/commands/windows/cmd.yaml")),
        ("windows", "Windows", include_str!("../data/commands/windows/winget.yaml")),
        // Testing
        ("testing", "Testing", include_str!("../data/commands/testing/adb.yaml")),
        ("testing", "Testing", include_str!("../data/commands/testing/ios.yaml")),
        ("testing", "Testing", include_str!("../data/commands/testing/network.yaml")),
        ("testing", "Testing", include_str!("../data/commands/testing/perf.yaml")),
    ];

    // 按平台分组
    let mut platform_map: BTreeMap<String, (String, Vec<Category>)> = BTreeMap::new();

    for (platform_name, display_name, yaml_content) in yaml_entries {
        match serde_yaml::from_str::<CommandFile>(yaml_content) {
            Ok(cmd_file) => {
                let category = Category {
                    name: cmd_file.category,
                    description: cmd_file.description,
                    platform: cmd_file.platform,
                    commands: cmd_file.commands,
                };
                let entry = platform_map
                    .entry(platform_name.to_string())
                    .or_insert_with(|| (display_name.to_string(), Vec::new()));
                entry.1.push(category);
            }
            Err(e) => {
                eprintln!("Warning: failed to parse YAML for {}: {}", platform_name, e);
            }
        }
    }

    // 转换为有序 Vec
    let mut platforms = Vec::new();
    for (name, (display_name, categories)) in platform_map {
        platforms.push(Platform {
            name,
            display_name,
            categories,
        });
    }

    platforms
}

/// 获取所有命令的扁平列表（用于搜索）
#[allow(dead_code)]
pub fn flatten_commands(platforms: &[Platform]) -> Vec<(&Platform, &Category, &Command)> {
    let mut result = Vec::new();
    for platform in platforms {
        for category in &platform.categories {
            for command in &category.commands {
                result.push((platform, category, command));
            }
        }
    }
    result
}
