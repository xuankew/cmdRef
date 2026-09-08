pub mod config;
pub mod dav;
pub mod github;
pub mod merge;
pub mod snapshot;

use clap::Subcommand;

use config::{BackendKind, SyncConfig};
use dav::{DavClient, DavOutcome};
use github::GithubClient;
use merge::{merge_sync_data, Prefer};
use snapshot::SyncSnapshot;

#[derive(Debug, Subcommand)]
pub enum SyncCommand {
    /// 配置同步账号（WebDAV 或 GitHub）
    #[command(
        long_about = "配置同步账号。\n\n\
        WebDAV 模式（坚果云 / Nextcloud 等）:\n  \
        cmdref sync login <url> <username> <password>\n\n\
        GitHub 模式:\n  \
        cmdref sync login --backend github --token <PAT> --repo <owner/repo>\n\n\
        --repo 支持以下格式:\n  \
        owner/repo\n  \
        git@github.com:owner/repo.git\n  \
        https://github.com/owner/repo"
    )]
    Login {
        /// WebDAV 目录地址（WebDAV 模式）
        url: Option<String>,
        /// WebDAV 用户名（WebDAV 模式）
        username: Option<String>,
        /// WebDAV 密码/应用密码（WebDAV 模式）
        password: Option<String>,
        /// 后端类型: webdav（默认）或 github
        #[arg(long, default_value = "webdav")]
        backend: String,
        /// GitHub Personal Access Token（GitHub 模式）
        #[arg(long)]
        token: Option<String>,
        /// GitHub 仓库（GitHub 模式），支持 owner/repo、git@github.com:owner/repo.git、https://github.com/owner/repo
        #[arg(long)]
        repo: Option<String>,
    },
    /// 清除已保存的同步配置（不影响本地数据和云端数据）
    Logout,
    /// 查看本地与云端的同步状态及待同步变更
    Status,
    /// 上传本地数据到云端
    Push {
        /// 跳过版本检查与空数据守卫，强制覆盖云端
        #[arg(long)]
        force: bool,
    },
    /// 拉取云端数据并与本地合并（3-way merge）
    Pull {
        /// 冲突时保留本地版本（默认保留云端）
        #[arg(long)]
        prefer_local: bool,
        /// 跳过合并，直接用云端覆盖本地（仍会先备份）
        #[arg(long)]
        force: bool,
    },
}

pub fn run(command: SyncCommand) -> Result<(), String> {
    match command {
        SyncCommand::Login {
            url,
            username,
            password,
            backend,
            token,
            repo,
        } => cmd_login(url, username, password, &backend, token, repo),
        SyncCommand::Logout => cmd_logout(),
        SyncCommand::Status => cmd_status(),
        SyncCommand::Push { force } => cmd_push(force),
        SyncCommand::Pull { force, prefer_local } => cmd_pull(force, prefer_local),
    }
}

fn load_config() -> Result<SyncConfig, String> {
    SyncConfig::load().ok_or_else(|| {
        "尚未配置同步账号，请先执行:\n  WebDAV:  cmdref sync login <url> <username> <password>\n  GitHub:  cmdref sync login --backend github --token <PAT> --repo <owner/repo>".to_string()
    })
}

fn fetch_remote_snapshot(cfg: &SyncConfig) -> Result<Option<SyncSnapshot>, String> {
    match cfg.backend {
        BackendKind::Webdav => {
            let client = DavClient::new(cfg.clone());
            let url = cfg.snapshot_url();
            match client.get(&url)? {
                DavOutcome::Success(body) => {
                    let snap: SyncSnapshot = serde_json::from_slice(&body)
                        .map_err(|e| format!("云端快照解析失败: {}", e))?;
                    Ok(Some(snap))
                }
                DavOutcome::NotFound | DavOutcome::Conflict => Ok(None),
                DavOutcome::Unauthorized => Err(dav::unauthorized_msg()),
                DavOutcome::MethodNotAllowed | DavOutcome::Redirect => {
                    Err("云端地址不可访问，请检查登录的 WebDAV 目录地址".to_string())
                }
            }
        }
        BackendKind::Github => {
            let client = GithubClient::new(cfg.clone());
            match client.get_snapshot()? {
                Some(body) => {
                    let snap: SyncSnapshot = serde_json::from_slice(&body)
                        .map_err(|e| format!("云端快照解析失败: {}", e))?;
                    Ok(Some(snap))
                }
                None => Ok(None),
            }
        }
    }
}

fn cmd_login(
    url: Option<String>,
    username: Option<String>,
    password: Option<String>,
    backend_str: &str,
    token: Option<String>,
    repo: Option<String>,
) -> Result<(), String> {
    match backend_str {
        "github" => {
            let token = token.ok_or(
                "GitHub 登录需要提供 --token 参数\n用法: cmdref sync login --backend github --token <PAT> --repo <owner/repo>\n\n生成 PAT: https://github.com/settings/tokens → Generate new token (classic) → 勾选 repo 权限".to_string()
            )?;
            let repo = repo.ok_or(
                "GitHub 登录需要提供 --repo 参数\n用法: cmdref sync login --backend github --token <PAT> --repo <owner/repo>\n支持格式: owner/repo、git@github.com:owner/repo.git、https://github.com/owner/repo\n\n请先在 GitHub 上创建一个私有仓库（如 cmdref-sync），然后指定该仓库".to_string()
            )?;

            let repo = parse_github_repo(&repo)?;

            let mut cfg = SyncConfig {
                backend: BackendKind::Github,
                github_token: token,
                github_repo: repo.clone(),
                url: String::new(),
                username: String::new(),
                password: String::new(),
                device_id: config::generate_device_id(),
            };
            if let Some(old) = SyncConfig::load() {
                if !old.device_id.is_empty() {
                    cfg.device_id = old.device_id;
                }
            }

            let client = GithubClient::new(cfg.clone());
            let login = client.verify_token()?;
            client.verify_repo_access()?;

            cfg.save()?;
            println!("✓ GitHub 登录成功");
            println!("  账号: {}", login);
            println!("  仓库: {}", cfg.github_repo);
            println!("  设备标识: {}", cfg.device_id);
            println!();
            println!("下一步: cmdref sync push 上传本地数据");
            Ok(())
        }
        "webdav" | "" => {
            let (url, username, password) = match (url, username, password) {
                (Some(u), Some(user), Some(pass)) => (u, user, pass),
                _ => return Err("WebDAV 登录需要提供 url、username、password 三个参数\n用法: cmdref sync login <url> <username> <password>".to_string()),
            };

            let url = config::normalize_url(&url);
            let mut cfg = SyncConfig {
                backend: BackendKind::Webdav,
                url: url.clone(),
                username,
                password,
                github_token: String::new(),
                github_repo: String::new(),
                device_id: config::generate_device_id(),
            };
            if let Some(old) = SyncConfig::load() {
                if !old.device_id.is_empty() {
                    cfg.device_id = old.device_id;
                }
            }

            let client = DavClient::new(cfg.clone());
            match client.propfind(&url)? {
                DavOutcome::Success(_) | DavOutcome::NotFound | DavOutcome::MethodNotAllowed | DavOutcome::Redirect => {}
                DavOutcome::Unauthorized => return Err(dav::unauthorized_msg()),
                DavOutcome::Conflict => return Err("云端地址不可访问，请检查 URL".to_string()),
            }

            cfg.save()?;
            println!("✓ 登录成功");
            println!("  服务器: {}", cfg.url);
            println!("  账号: {}", cfg.username);
            println!("  设备标识: {}", cfg.device_id);
            println!();
            println!("下一步: cmdref sync push 上传本地数据");
            Ok(())
        }
        other => Err(format!("不支持的后端类型: {}。支持: webdav, github", other)),
    }
}

fn cmd_logout() -> Result<(), String> {
    if SyncConfig::load().is_none() {
        println!("当前没有已保存的同步配置");
        return Ok(());
    }
    SyncConfig::remove();
    println!("✓ 已清除同步配置（本地数据与云端数据均未改动）");
    Ok(())
}

fn cmd_status() -> Result<(), String> {
    let cfg = load_config()?;

    let backend_label = match cfg.backend {
        BackendKind::Github => "GitHub",
        BackendKind::Webdav => "WebDAV",
    };
    println!("后端: {}", backend_label);
    match cfg.backend {
        BackendKind::Webdav => {
            println!("服务器: {}", cfg.url);
            println!("账号: {}", cfg.username);
        }
        BackendKind::Github => {
            println!("仓库: {}", cfg.github_repo);
        }
    }
    println!("本机设备: {}", cfg.device_id);
    println!();

    let local = snapshot::build_local_data(&crate::paths::data_dir());
    println!(
        "本地数据: {} 条书签 / {} 条自定义命令 / {} 条 Prompts / {} 条密码",
        local.bookmarks.len(),
        local.custom_commands.len(),
        local.prompts.len(),
        local.passwords.len()
    );

    match fetch_remote_snapshot(&cfg)? {
        Some(remote) => {
            println!(
                "云端数据: v{} (最后由设备 {} 推送，{} 条书签 / {} 条命令 / {} 条 Prompts / {} 条密码)",
                remote.version,
                remote.device_id,
                remote.data.bookmarks.len(),
                remote.data.custom_commands.len(),
                remote.data.prompts.len(),
                remote.data.passwords.len()
            );
            let base = snapshot::load_base();
            let base_ver = base.as_ref().map(|b| b.version).unwrap_or(0);
            println!("本地基线: v{}", base_ver);

            let local_changed = base
                .as_ref()
                .map(|b| b.data != local)
                .unwrap_or(!local.is_empty());
            if remote.version > base_ver {
                println!("→ 云端有未拉取的更新 (v{} > v{})，建议: cmdref sync pull", remote.version, base_ver);
            } else if local_changed {
                println!("→ 本地有未推送的变更，建议: cmdref sync push");
            } else {
                println!("→ 已同步");
            }
        }
        None => {
            println!("云端数据: 无（尚未推送过）");
            if !local.is_empty() {
                println!("→ 建议: cmdref sync push");
            }
        }
    }
    Ok(())
}

fn cmd_push(force: bool) -> Result<(), String> {
    let cfg = load_config()?;

    let remote = fetch_remote_snapshot(&cfg)?;
    let base = snapshot::load_base();
    let base_ver = base.as_ref().map(|b| b.version).unwrap_or(0);
    let remote_ver = remote.as_ref().map(|r| r.version).unwrap_or(0);

    if !force {
        if let Some(r) = &remote {
            if r.version > base_ver {
                return Err(format!(
                    "云端有其他设备推送的新版本 (v{} > 本地基线 v{})，请先执行: cmdref sync pull",
                    r.version, base_ver
                ));
            }
            if r.version < base_ver {
                return Err(format!(
                    "云端版本异常 (v{} < 本地基线 v{})，可能被回滚；确认覆盖请加 --force",
                    r.version, base_ver
                ));
            }
        }
    }

    let local = snapshot::build_local_data(&crate::paths::data_dir());

    if local.is_empty() && remote.is_some() && !force {
        return Err(
            "本地数据为空而云端非空，为防止误清空云端已中止；确认清空请加 --force".to_string(),
        );
    }

    let snap = SyncSnapshot {
        version: base_ver.max(remote_ver) + 1,
        updated_at: snapshot::now_millis(),
        device_id: cfg.device_id.clone(),
        data: local,
    };
    let body = serde_json::to_vec(&snap).map_err(|e| format!("序列化失败: {}", e))?;

    match cfg.backend {
        BackendKind::Webdav => {
            let client = DavClient::new(cfg.clone());
            let url = cfg.snapshot_url();

            if remote.is_none() {
                if let Some(parent) = parent_url(&url) {
                    client.ensure_remote_dir(&parent)?;
                }
            }

            match client.put(&url, &body)? {
                DavOutcome::Success(_) => {}
                DavOutcome::NotFound | DavOutcome::Conflict => {
                    if let Some(parent) = parent_url(&url) {
                        client.ensure_remote_dir(&parent)?;
                    }
                    match client.put(&url, &body)? {
                        DavOutcome::Success(_) => {}
                        DavOutcome::Unauthorized => return Err(dav::unauthorized_msg()),
                        _ => return Err("上传失败：远端目录不可写".to_string()),
                    }
                }
                DavOutcome::Unauthorized => return Err(dav::unauthorized_msg()),
                DavOutcome::MethodNotAllowed | DavOutcome::Redirect => return Err("上传失败：远端地址不可写".to_string()),
            }
        }
        BackendKind::Github => {
            let client = GithubClient::new(cfg.clone());

            let sha = client.get_file_sha()?;
            client.put_snapshot(&body, sha.as_deref())?;
        }
    }

    snapshot::save_base(&snap)?;
    println!("✓ 已推送到云端 (v{})", snap.version);
    println!(
        "  书签 {} / 自定义命令 {} / Prompts {} / 密码 {}",
        snap.data.bookmarks.len(),
        snap.data.custom_commands.len(),
        snap.data.prompts.len(),
        snap.data.passwords.len()
    );
    Ok(())
}

fn cmd_pull(force: bool, prefer_local: bool) -> Result<(), String> {
    let cfg = load_config()?;
    let remote = fetch_remote_snapshot(&cfg)?.ok_or(
        "云端暂无同步数据，请先在本机执行: cmdref sync push".to_string(),
    )?;

    if force {
        let backup = snapshot::backup_local_data()?;
        snapshot::apply_local_data(&crate::paths::data_dir(), &remote.data)?;
        snapshot::save_base(&remote)?;
        println!("✓ 已用云端数据覆盖本地 (v{})", remote.version);
        println!(
            "  书签 {} / 自定义命令 {} / Prompts {} / 密码 {}",
            remote.data.bookmarks.len(),
            remote.data.custom_commands.len(),
            remote.data.prompts.len(),
            remote.data.passwords.len()
        );
        println!("  备份位置: {}", backup.display());
        return Ok(());
    }

    let base = snapshot::load_base();
    if let Some(b) = &base {
        if b.version == remote.version && b.data == remote.data {
            let local = snapshot::build_local_data(&crate::paths::data_dir());
            if b.data != local {
                println!("✓ 云端无新内容 (v{})；本地有未推送的变更", remote.version);
            } else {
                println!("✓ 已是最新 (v{})", remote.version);
            }
            return Ok(());
        }
    }

    let local = snapshot::build_local_data(&crate::paths::data_dir());
    let base_data = base.as_ref().map(|b| b.data.clone()).unwrap_or_default();
    let prefer = if prefer_local {
        Prefer::Local
    } else {
        Prefer::Remote
    };

    let backup = snapshot::backup_local_data()?;
    let (merged, report) = merge_sync_data(&base_data, &local, &remote.data, prefer);
    snapshot::apply_local_data(&crate::paths::data_dir(), &merged)?;
    snapshot::save_base(&remote)?;

    println!(
        "✓ 已同步云端数据 (v{}, 设备 {})",
        remote.version, remote.device_id
    );
    println!(
        "  当前: 书签 {} / 自定义命令 {} / Prompts {} / 密码 {}",
        merged.bookmarks.len(),
        merged.custom_commands.len(),
        merged.prompts.len(),
        merged.passwords.len()
    );
    println!("  备份位置: {}", backup.display());

    if report.total() > 0 {
        println!();
        println!(
            "⚠ 检测到 {} 处冲突，已按【{}】解决：",
            report.total(),
            if prefer_local { "本地优先" } else { "云端优先" }
        );
        for name in &report.bookmark_conflicts {
            println!("  - 书签: {}", name);
        }
        for name in &report.command_conflicts {
            println!("  - 自定义命令: {}", name);
        }
        for name in &report.prompt_conflicts {
            println!("  - Prompt: {}", name);
        }
        for name in &report.password_conflicts {
            println!("  - 密码: {}", name);
        }
        println!("  如需保留本地版本，可执行: cmdref sync pull --force（用备份恢复后重试）");
    }
    Ok(())
}

/// 从多种格式中解析出 owner/repo：
/// - owner/repo
/// - git@github.com:owner/repo.git
/// - https://github.com/owner/repo(.git)
fn parse_github_repo(input: &str) -> Result<String, String> {
    let s = input.trim();

    // git@github.com:owner/repo.git
    if let Some(rest) = s.strip_prefix("git@github.com:") {
        let repo = rest.strip_suffix(".git").unwrap_or(rest);
        if repo.contains('/') && !repo.is_empty() {
            return Ok(repo.to_string());
        }
    }

    // https://github.com/owner/repo(.git)
    if let Some(rest) = s.strip_prefix("https://github.com/")
        .or_else(|| s.strip_prefix("http://github.com/"))
    {
        let repo = rest.strip_suffix(".git").unwrap_or(rest).trim_end_matches('/');
        if repo.contains('/') && !repo.is_empty() {
            return Ok(repo.to_string());
        }
    }

    // owner/repo
    if s.contains('/') && !s.starts_with('/') && !s.ends_with('/') {
        return Ok(s.to_string());
    }

    Err(format!(
        "无法识别的仓库格式: {}\n支持: owner/repo、git@github.com:owner/repo.git、https://github.com/owner/repo",
        input
    ))
}

/// 取 URL 的父目录（去掉最后一段路径）；无路径部分时返回 None
fn parent_url(url: &str) -> Option<String> {
    let idx = url.rfind('/')?;
    let scheme_end = url.find("://").map(|i| i + 3).unwrap_or(0);
    if idx < scheme_end {
        return None;
    }
    Some(url[..idx].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_url() {
        assert_eq!(
            parent_url("https://dav.example.com/dav/cmdref/cmdref-sync.json"),
            Some("https://dav.example.com/dav/cmdref".to_string())
        );
        assert_eq!(
            parent_url("https://dav.example.com/cmdref-sync.json"),
            Some("https://dav.example.com".to_string())
        );
        assert_eq!(
            parent_url("http://127.0.0.1:8888/dav/cmdref-sync.json"),
            Some("http://127.0.0.1:8888/dav".to_string())
        );
    }

    #[test]
    fn test_parse_github_repo() {
        assert_eq!(parse_github_repo("owner/repo").unwrap(), "owner/repo");
        assert_eq!(parse_github_repo("git@github.com:owner/repo.git").unwrap(), "owner/repo");
        assert_eq!(parse_github_repo("git@github.com:owner/repo").unwrap(), "owner/repo");
        assert_eq!(parse_github_repo("https://github.com/owner/repo").unwrap(), "owner/repo");
        assert_eq!(parse_github_repo("https://github.com/owner/repo.git").unwrap(), "owner/repo");
        assert_eq!(parse_github_repo("https://github.com/owner/repo/").unwrap(), "owner/repo");
        assert_eq!(parse_github_repo("xuankew/cmdRef_saved").unwrap(), "xuankew/cmdRef_saved");
        assert_eq!(parse_github_repo("git@github.com:xuankew/cmdRef_saved.git").unwrap(), "xuankew/cmdRef_saved");
        assert!(parse_github_repo("just-a-name").is_err());
        assert!(parse_github_repo("").is_err());
    }
}
