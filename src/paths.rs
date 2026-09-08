use std::path::PathBuf;

/// 用户数据根目录。
/// 优先读取 CMDREF_DATA_DIR 环境变量（支持 ~/ 展开），否则使用 dirs::config_dir()/cmdref。
/// 返回前会尝试创建目录。
pub fn data_dir() -> PathBuf {
    if let Ok(raw) = std::env::var("CMDREF_DATA_DIR") {
        let t = raw.trim();
        if !t.is_empty() {
            let p = match t.strip_prefix("~/") {
                Some(rest) => dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(rest),
                None => PathBuf::from(t),
            };
            std::fs::create_dir_all(&p).ok();
            return p;
        }
    }

    let p = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cmdref");
    std::fs::create_dir_all(&p).ok();
    p
}

/// 自定义命令目录（可同步）
pub fn custom_dir() -> PathBuf {
    data_dir().join("custom")
}

/// Prompts 数据目录（可同步）
pub fn prompts_dir() -> PathBuf {
    data_dir().join("prompts")
}

/// 书签文件路径（可同步）
pub fn bookmarks_file() -> PathBuf {
    data_dir().join("bookmarks.json")
}

/// 指定根目录下的书签文件（同步快照按根目录读写用）
pub fn bookmarks_file_in(root: &std::path::Path) -> PathBuf {
    root.join("bookmarks.json")
}

/// 指定根目录下的自定义命令目录
pub fn custom_dir_in(root: &std::path::Path) -> PathBuf {
    root.join("custom")
}

/// 指定根目录下的 Prompts 目录
pub fn prompts_dir_in(root: &std::path::Path) -> PathBuf {
    root.join("prompts")
}

/// Passwords 数据目录（可同步，值已加密）
pub fn passwords_dir() -> PathBuf {
    data_dir().join("passwords")
}

/// 指定根目录下的 Passwords 目录
pub fn passwords_dir_in(root: &std::path::Path) -> PathBuf {
    root.join("passwords")
}

/// 本地状态目录，不参与用户数据同步（history.json、debug.log、sync 状态）。
/// 默认在 config_dir 下；可用 CMDREF_LOCAL_DIR 覆盖（测试或高级用途）。
pub fn local_dir() -> PathBuf {
    if let Ok(raw) = std::env::var("CMDREF_LOCAL_DIR") {
        let t = raw.trim();
        if !t.is_empty() {
            let p = match t.strip_prefix("~/") {
                Some(rest) => dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(rest),
                None => PathBuf::from(t),
            };
            std::fs::create_dir_all(&p).ok();
            return p;
        }
    }

    let p = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cmdref");
    std::fs::create_dir_all(&p).ok();
    p
}

/// 历史记录文件路径（本地，不跟随 CMDREF_DATA_DIR 同步）
pub fn history_file() -> PathBuf {
    local_dir().join("history.json")
}

/// 调试日志文件路径（本地，不跟随 CMDREF_DATA_DIR 同步）
pub fn log_path() -> PathBuf {
    local_dir().join("debug.log")
}
