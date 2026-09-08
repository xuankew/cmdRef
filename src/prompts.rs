use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 单条 Prompt 记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prompt {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// 该 Prompt 所属的数据文件路径，不参与序列化
    #[serde(skip)]
    pub source: PathBuf,
}

/// Prompt 数据文件的 YAML 格式
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptFile {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,
    #[serde(default)]
    pub prompts: Vec<Prompt>,
}

/// Prompt 存储管理器
pub struct PromptStore {
    prompts: Vec<Prompt>,
}

impl PromptStore {
    /// 从 prompts_dir 加载所有 *.yaml / *.yml 文件
    pub fn load() -> Self {
        let dir = crate::paths::prompts_dir();
        let mut prompts: Vec<Prompt> = Vec::new();

        if !dir.is_dir() {
            return Self { prompts };
        }

        let mut entries: Vec<_> = match std::fs::read_dir(&dir) {
            Ok(e) => e.flatten().collect(),
            Err(_) => return Self { prompts },
        };

        // 按文件名排序，保证顺序稳定
        entries.sort_by_key(|a| a.file_name());

        for entry in entries {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "yaml" && ext != "yml" {
                continue;
            }

            // 跳过可能的同步冲突文件
            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if file_stem.starts_with('.')
                || file_stem.to_lowercase().contains("conflict")
                || file_stem.to_lowercase().contains("冲突")
            {
                continue;
            }

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut prompt_file: PromptFile = match serde_yaml::from_str(&content) {
                Ok(f) => f,
                Err(_) => continue,
            };

            for mut prompt in prompt_file.prompts.drain(..) {
                prompt.source = path.clone();
                prompts.push(prompt);
            }
        }

        Self { prompts }
    }

    /// 新建或更新 Prompt。
    ///
    /// - `replacing` 为 Some(原名) 时表示编辑模式，会删除旧条目后再插入新条目。
    ///   返回新 Prompt 在列表中的索引。
    pub fn upsert(&mut self, prompt: Prompt, replacing: Option<&str>) -> Result<usize, String> {
        // 校验
        let name = prompt.name.trim();
        if name.is_empty() {
            return Err("Prompt 名称不能为空".to_string());
        }
        if prompt.content.trim().is_empty() {
            return Err("Prompt 内容不能为空".to_string());
        }

        // 重名检查（编辑时排除自身）
        if replacing.map(|old| old != name).unwrap_or(true)
            && self.prompts.iter().any(|p| p.name == name)
        {
            return Err(format!("已存在名为 '{}' 的 Prompt", name));
        }

        // 确定目标文件
        let target_path = if let Some(old_name) = replacing {
            self.prompts
                .iter()
                .find(|p| p.name == old_name)
                .map(|p| p.source.clone())
                .unwrap_or_else(default_prompts_file)
        } else {
            default_prompts_file()
        };

        // 读取原文件
        let mut prompt_file: PromptFile = if target_path.exists() {
            match std::fs::read_to_string(&target_path) {
                Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
                Err(e) => return Err(format!("读取文件失败: {}", e)),
            }
        } else {
            PromptFile {
                title: "我的 Prompts".to_string(),
                prompts: Vec::new(),
            }
        };

        // 如果是编辑模式，先移除旧条目
        if let Some(old_name) = replacing {
            prompt_file.prompts.retain(|p| p.name != old_name);
        }

        // 插入新条目（不带 source，序列化前复制一份）
        let mut persisted = prompt.clone();
        persisted.source = PathBuf::new();
        prompt_file.prompts.push(persisted);

        // 写回文件
        let yaml = match serde_yaml::to_string(&prompt_file) {
            Ok(y) => y,
            Err(e) => return Err(format!("序列化失败: {}", e)),
        };

        if let Err(e) = std::fs::create_dir_all(target_path.parent().unwrap_or(&target_path)) {
            return Err(format!("创建目录失败: {}", e));
        }

        if let Err(e) = std::fs::write(&target_path, yaml) {
            return Err(format!("写入文件失败: {}", e));
        }

        // 热重载
        *self = Self::load();

        // 返回新索引
        let idx = self
            .prompts
            .iter()
            .position(|p| p.name == name)
            .unwrap_or(0);
        Ok(idx)
    }

    /// 删除指定名称的 Prompt
    pub fn delete(&mut self, name: &str) -> Result<(), String> {
        let source = match self.prompts.iter().find(|p| p.name == name) {
            Some(p) => p.source.clone(),
            None => return Err(format!("未找到 Prompt: {}", name)),
        };

        if !source.exists() {
            *self = Self::load();
            return Ok(());
        }

        let mut prompt_file: PromptFile = match std::fs::read_to_string(&source) {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
            Err(e) => return Err(format!("读取文件失败: {}", e)),
        };

        let before = prompt_file.prompts.len();
        prompt_file.prompts.retain(|p| p.name != name);

        if prompt_file.prompts.is_empty() {
            let _ = std::fs::remove_file(&source);
        } else if prompt_file.prompts.len() < before {
            let yaml = match serde_yaml::to_string(&prompt_file) {
                Ok(y) => y,
                Err(e) => return Err(format!("序列化失败: {}", e)),
            };
            if let Err(e) = std::fs::write(&source, yaml) {
                return Err(format!("写入文件失败: {}", e));
            }
        }

        *self = Self::load();
        Ok(())
    }

    pub fn all(&self) -> &[Prompt] {
        &self.prompts
    }

    pub fn count(&self) -> usize {
        self.prompts.len()
    }

    pub fn get(&self, index: usize) -> Option<&Prompt> {
        self.prompts.get(index)
    }
}

fn default_prompts_file() -> PathBuf {
    crate::paths::prompts_dir().join("my_prompts.yaml")
}
