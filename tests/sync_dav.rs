//! WebDAV 同步集成测试：驱动编译出的 cmdref 二进制，
//! 对 tests/webdav_server.py（Python 标准库迷你服务器）跑 login/push/pull 全链路。
//! python3 不可用时自动跳过。

use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

const USERNAME: &str = "testuser";
const PASSWORD: &str = "testpass";

const CMD_YAML: &str = "category: 我的命令\ndescription: 自定义的常用命令\nplatform: custom\ncommands:\n  - name: deploy-check\n    summary: 检查部署\n    examples:\n      - description: 运行检查\n        code: make check\n    tags:\n      - work\n";

// ---------- 迷你服务器管理 ----------

struct ServerGuard {
    child: Child,
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Server {
    _guard: ServerGuard,
    port: u16,
}

impl Server {
    /// 带 trailing slash 以顺带覆盖 URL 规范化逻辑
    fn url(&self) -> String {
        format!("http://127.0.0.1:{}/dav/cmdref/", self.port)
    }
}

/// 启动迷你 WebDAV 服务器；python3 缺失时返回 None（测试跳过）
fn start_server() -> Option<Server> {
    match Command::new("python3").arg("-c").arg("print(1)").output() {
        Ok(out) if out.status.success() => {}
        _ => return None,
    }

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/webdav_server.py");
    let mut child = Command::new("python3")
        .arg(&script)
        .arg("0")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("启动 WebDAV 测试服务器失败");

    let stdout = child.stdout.take().expect("服务器 stdout 未管道化");
    let mut line = String::new();
    {
        let mut reader = std::io::BufReader::new(stdout);
        reader
            .read_line(&mut line)
            .expect("读取服务器端口输出失败");
    }

    let port: u16 = line
        .trim()
        .strip_prefix("PORT=")
        .and_then(|p| p.parse().ok())
        .unwrap_or_else(|| panic!("服务器输出异常: {:?}", line));

    Some(Server {
        _guard: ServerGuard { child },
        port,
    })
}

// ---------- cmdref 沙盒 ----------

/// 每个测试独立的 cmdref 数据目录 / 本地状态目录
struct Sandbox {
    data_dir: PathBuf,
    local_dir: PathBuf,
}

impl Sandbox {
    fn new(tag: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!(
            "cmdref-sync-it-{}-{}-{}",
            std::process::id(),
            tag,
            nanos
        ));
        let data_dir = base.join("data");
        let local_dir = base.join("local");
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::create_dir_all(&local_dir).unwrap();
        Self { data_dir, local_dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_cmdref"))
            .args(args)
            .env("CMDREF_DATA_DIR", &self.data_dir)
            .env("CMDREF_LOCAL_DIR", &self.local_dir)
            .output()
            .expect("运行 cmdref 失败")
    }

    fn login(&self, server: &Server) -> Output {
        self.run(&["sync", "login", &server.url(), USERNAME, PASSWORD])
    }

    fn push(&self, extra: &[&str]) -> Output {
        let mut args = vec!["sync", "push"];
        args.extend_from_slice(extra);
        self.run(&args)
    }

    fn pull(&self, extra: &[&str]) -> Output {
        let mut args = vec!["sync", "pull"];
        args.extend_from_slice(extra);
        self.run(&args)
    }

    fn cleanup(&self) {
        if let Some(parent) = self.data_dir.parent() {
            let _ = std::fs::remove_dir_all(parent);
        }
    }

    // ---------- 数据种子 ----------

    fn write_bookmarks(&self, entries: &[(&str, &str, &str)]) {
        let body = entries
            .iter()
            .map(|(p, c, cmd)| {
                format!("  {{\"platform\": \"{p}\", \"category\": \"{c}\", \"command\": \"{cmd}\"}}")
            })
            .collect::<Vec<_>>()
            .join(",\n");
        std::fs::write(
            self.data_dir.join("bookmarks.json"),
            format!("[\n{body}\n]\n"),
        )
        .unwrap();
    }

    fn write_custom_commands(&self, yaml: &str) {
        let dir = self.data_dir.join("custom");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("my_commands.yaml"), yaml).unwrap();
    }

    fn write_prompts(&self, yaml: &str) {
        let dir = self.data_dir.join("prompts");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("my_prompts.yaml"), yaml).unwrap();
    }

    // ---------- 数据读取 ----------

    fn read_bookmarks(&self) -> String {
        std::fs::read_to_string(self.data_dir.join("bookmarks.json")).unwrap_or_default()
    }

    fn read_prompts(&self) -> String {
        read_any_yaml(&self.data_dir.join("prompts"))
    }

    fn read_custom(&self) -> String {
        read_any_yaml(&self.data_dir.join("custom"))
    }
}

fn read_any_yaml(dir: &Path) -> String {
    let mut out = String::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            if let Ok(c) = std::fs::read_to_string(&p) {
                out.push_str(&c);
            }
        }
    }
    out
}

/// 翻译助手 Prompt（content 可变）+ 可选代码审查 Prompt
fn prompt_yaml(content: &str, with_review: bool) -> String {
    let mut yaml = format!(
        "title: 我的 Prompts\nprompts:\n  - name: 翻译助手\n    description: 翻译\n    content: {content}\n"
    );
    if with_review {
        yaml.push_str("  - name: 代码审查\n    description: 审查\n    content: 请审查以下代码\n");
    }
    yaml
}

// ---------- 断言辅助 ----------

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn ok(output: &Output, expect: &str) {
    assert!(
        output.status.success(),
        "期望成功但失败\n输出: {}",
        combined(output)
    );
    assert!(
        combined(output).contains(expect),
        "输出中未找到 '{expect}'\n输出: {}",
        combined(output)
    );
}

fn fail(output: &Output, expect: &str) {
    assert!(
        !output.status.success(),
        "期望失败但成功了\n输出: {}",
        combined(output)
    );
    assert!(
        combined(output).contains(expect),
        "输出中未找到 '{expect}'\n输出: {}",
        combined(output)
    );
}

fn skip_if_no_python() -> Option<Server> {
    let server = start_server();
    if server.is_none() {
        eprintln!("跳过：python3 不可用");
    }
    server
}

// ---------- 测试 ----------

/// 全链路：首次推送 → 新设备恢复 → 双端并发变更合并 → 冲突解决 → 状态一致
#[test]
fn test_full_chain() {
    let Some(server) = skip_if_no_python() else {
        return;
    };
    let a = Sandbox::new("chain-a");
    let b = Sandbox::new("chain-b");

    // 1. 设备 A 首次推送（自动创建远端目录）
    a.write_bookmarks(&[("mac", "Homebrew", "brew install ripgrep")]);
    a.write_custom_commands(CMD_YAML);
    a.write_prompts(&prompt_yaml("翻译成英文", false));
    ok(&a.login(&server), "登录成功");
    ok(&a.push(&[]), "已推送到云端 (v1)");

    // 2. 设备 B 全新环境拉取恢复
    ok(&b.login(&server), "登录成功");
    ok(&b.pull(&[]), "已同步云端数据 (v1");
    assert!(b.read_bookmarks().contains("brew install ripgrep"));
    assert!(b.read_custom().contains("deploy-check"));
    assert!(b.read_prompts().contains("翻译助手"));

    // 3. A 加书签、B 加 Prompt，B 拉取后双方变更都保留
    a.write_bookmarks(&[
        ("mac", "Homebrew", "brew install ripgrep"),
        ("linux", "Systemd", "journalctl -u nginx"),
    ]);
    b.write_prompts(&prompt_yaml("翻译成英文", true));
    ok(&a.push(&[]), "已推送到云端 (v2)");
    ok(&b.pull(&[]), "已同步云端数据 (v2");
    assert!(b.read_bookmarks().contains("journalctl -u nginx"));
    assert!(b.read_prompts().contains("代码审查"));
    // B 推送，A 拉取后也拿到 B 的 Prompt
    ok(&b.push(&[]), "已推送到云端 (v3)");
    ok(&a.pull(&[]), "已同步云端数据 (v3");
    assert!(a.read_prompts().contains("代码审查"));

    // 4. 冲突：双方改同一条 Prompt，默认云端优先
    a.write_prompts(&prompt_yaml("请翻译成英文（A 版）", true));
    ok(&a.push(&[]), "已推送到云端 (v4)");
    b.write_prompts(&prompt_yaml("请翻译成英文（B 版）", true));
    ok(&b.pull(&[]), "冲突");
    let b_prompts = b.read_prompts();
    assert!(b_prompts.contains("A 版"), "云端优先应保留 A 版:\n{b_prompts}");
    assert!(!b_prompts.contains("B 版"), "B 版应被云端覆盖:\n{b_prompts}");

    // 5. 再次冲突时 --prefer-local 保留本地版本
    a.write_prompts(&prompt_yaml("请翻译成英文（A2 版）", true));
    ok(&a.push(&[]), "已推送到云端 (v5)");
    b.write_prompts(&prompt_yaml("请翻译成英文（B2 版）", true));
    ok(&b.pull(&["--prefer-local"]), "本地优先");
    assert!(b.read_prompts().contains("B2 版"));

    // 6. 推送后状态应显示已同步
    ok(&b.push(&[]), "已推送到云端 (v6)");
    ok(&b.run(&["sync", "status"]), "已同步");

    a.cleanup();
    b.cleanup();
}

/// 守卫行为：未配置账号 / 版本检查 / 空数据保护 / 回滚保护 / 覆盖前备份
#[test]
fn test_guards() {
    let Some(server) = skip_if_no_python() else {
        return;
    };
    let a = Sandbox::new("guard-a");
    let b = Sandbox::new("guard-b");
    let c = Sandbox::new("guard-c");

    // 未配置账号直接 push
    fail(&c.run(&["sync", "push"]), "尚未配置同步账号");

    // A 推送 v1
    a.write_bookmarks(&[("mac", "Homebrew", "brew install ripgrep")]);
    ok(&a.login(&server), "登录成功");
    ok(&a.push(&[]), "已推送到云端 (v1)");

    // B 本地非空但从未拉取 → 版本守卫要求先 pull
    b.write_bookmarks(&[("linux", "Text", "grep -r foo")]);
    ok(&b.login(&server), "登录成功");
    fail(&b.push(&[]), "请先执行: cmdref sync pull");

    // C 先拉取（新设备常规流程），再清空本地 → push 触发空数据守卫
    ok(&c.login(&server), "登录成功");
    ok(&c.pull(&[]), "已同步云端数据");
    std::fs::remove_file(c.data_dir.join("bookmarks.json")).unwrap();
    fail(&c.push(&[]), "本地数据为空");
    // --force 允许覆盖
    ok(&c.push(&["--force"]), "已推送到云端 (v2)");

    // A 未感知 C 的覆盖（远端 v2 > 本地基线 v1）
    fail(&a.push(&[]), "请先执行: cmdref sync pull");

    // pull --force 用云端（空）覆盖本地，覆盖前先备份
    ok(&a.pull(&["--force"]), "已用云端数据覆盖本地");
    assert!(!a.data_dir.join("bookmarks.json").exists());
    let backup_root = a.local_dir.join("sync-backup");
    assert!(
        backup_root.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false),
        "覆盖前应生成备份"
    );

    // 手工抬高基线版本，模拟云端被回滚
    std::fs::write(
        a.local_dir.join("sync-base.json"),
        r#"{"version": 99, "updated_at": 0, "device_id": "manual", "data": {"bookmarks": [], "custom_commands": [], "prompts": []}}"#,
    )
    .unwrap();
    fail(&a.push(&[]), "可能被回滚");

    a.cleanup();
    b.cleanup();
    c.cleanup();
}

/// 认证失败：错误密码登录报错且不保存配置
#[test]
fn test_auth_failure() {
    let Some(server) = skip_if_no_python() else {
        return;
    };
    let s = Sandbox::new("auth");

    let out = s.run(&["sync", "login", &server.url(), "testuser", "wrongpass"]);
    fail(&out, "认证失败");
    assert!(
        !s.local_dir.join("sync.json").exists(),
        "登录失败不应保存凭据"
    );

    s.cleanup();
}
