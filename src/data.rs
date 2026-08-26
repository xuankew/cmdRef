use serde::Deserialize;
use std::collections::BTreeMap;

/// 单条命令示例
#[derive(Debug, Clone, Deserialize)]
pub struct Example {
    pub description: String,
    pub code: String,
    /// 使用频率: daily / weekly / rarely
    #[serde(default)]
    pub frequency: String,
    /// 危险等级: high / medium / none
    #[serde(default)]
    pub danger: String,
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
    /// 场景标签，用于按场景检索
    #[serde(default)]
    pub tags: Vec<String>,
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
        ("linux", "Linux", include_str!("../data/commands/linux/systemd.yaml")),
        // Mac
        ("mac", "macOS", include_str!("../data/commands/mac/brew.yaml")),
        ("mac", "macOS", include_str!("../data/commands/mac/system.yaml")),
        ("mac", "macOS", include_str!("../data/commands/mac/xcode.yaml")),
        // Windows
        ("windows", "Windows", include_str!("../data/commands/windows/powershell.yaml")),
        ("windows", "Windows", include_str!("../data/commands/windows/cmd.yaml")),
        ("windows", "Windows", include_str!("../data/commands/windows/winget.yaml")),
        // Dev Tools
        ("dev", "Dev Tools", include_str!("../data/commands/dev/git.yaml")),
        ("dev", "Dev Tools", include_str!("../data/commands/dev/docker.yaml")),
        ("dev", "Dev Tools", include_str!("../data/commands/dev/database.yaml")),
        ("dev", "Dev Tools", include_str!("../data/commands/dev/k8s.yaml")),
        ("dev", "Dev Tools", include_str!("../data/commands/dev/json_yaml.yaml")),
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

    // 加载用户自定义命令
    merge_custom_commands(&mut platforms);

    platforms
}

/// 从 ~/.config/cmdref/custom/ 加载用户自定义命令并合并到平台数据
fn merge_custom_commands(platforms: &mut Vec<Platform>) {
    let custom_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("cmdref")
        .join("custom");

    if !custom_dir.is_dir() {
        return;
    }

    let display_name_map = [
        ("linux", "Linux"),
        ("mac", "macOS"),
        ("windows", "Windows"),
        ("testing", "Testing"),
        ("dev", "Dev Tools"),
    ];

    let entries = match std::fs::read_dir(&custom_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml")
            && path.extension().and_then(|e| e.to_str()) != Some("yml")
        {
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let cmd_file: CommandFile = match serde_yaml::from_str(&content) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let category = Category {
            name: cmd_file.category.clone(),
            description: cmd_file.description,
            platform: cmd_file.platform.clone(),
            commands: cmd_file.commands,
        };

        let platform_key = cmd_file.platform.to_lowercase();
        let display = display_name_map
            .iter()
            .find(|(k, _)| *k == platform_key.as_str())
            .map(|(_, v)| *v)
            .unwrap_or_else(|| {
                // For unknown platforms, capitalize first letter
                Box::leak(cmd_file.platform.clone().into_boxed_str())
            });

        // Merge into existing platform or create new one
        if let Some(platform) = platforms.iter_mut().find(|p| p.name == platform_key) {
            platform.categories.push(category);
        } else {
            platforms.push(Platform {
                name: platform_key,
                display_name: display.to_string(),
                categories: vec![category],
            });
        }
    }
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
