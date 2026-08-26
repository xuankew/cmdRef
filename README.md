# CmdRef

> Interactive command reference tool — 189 commands across Linux, macOS, Windows, Dev Tools, and Testing, right in your terminal.

CmdRef is a terminal-based cheatsheet that helps developers, testers, and ops engineers quickly find and review commonly used commands. Browse by category, search with fuzzy matching, view detailed examples with danger warnings and frequency indicators — all without leaving the terminal.

## Features

- **189 commands** across 5 platforms: Linux, macOS, Windows, Dev Tools, and Testing
- **Interactive TUI** with sidebar navigation and detailed command views
- **Fuzzy search** — type any keyword to instantly find matching commands (searches names, examples, tips, and scenario tags)
- **Scenario tags** — commands are tagged with use-case labels (e.g. #回退, #调试, #部署) for context-based discovery
- **Quick Reference** — daily-use examples shown at the top of each command's detail page
- **Danger warnings** — high-risk commands (e.g. `rm -rf`, `git push --force`) are marked with ⚠ alerts
- **Frequency indicators** — examples tagged as ⚡ daily / ○ weekly / · rarely
- **Bookmarks** — press `b` to save favorite commands, `B` to jump to your bookmark list
- **History** — press `H` to quickly revisit recently viewed commands
- **Auto-detect platform** — automatically highlights your OS on startup
- **Custom commands** — add your own YAML files to `~/.config/cmdref/custom/` without rebuilding
- **Self-update** — `cmdref update` checks for new versions and upgrades in-place
- **Cross-platform** — single binary for macOS, Linux, and Windows
- **Zero dependencies** — all command data is embedded in the binary
- **Easy to extend** — add commands by editing YAML files, no code changes needed

## Quick Start

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/xuankew/cmdRef/main/install.sh | bash
```

### macOS (Homebrew)

```bash
brew tap xuankew/cmdref
brew install cmdref
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/xuankew/cmdRef/main/install.ps1 | iex
```

### From Source

```bash
cargo install --git https://github.com/xuankew/cmdRef
```

## Updating

```bash
cmdref update       # Self-update to latest version
```

Or reinstall via your package manager:

```bash
brew upgrade cmdref                                    # Homebrew
curl -fsSL https://raw.githubusercontent.com/xuankew/cmdRef/main/install.sh | bash  # macOS/Linux
irm https://raw.githubusercontent.com/xuankew/cmdRef/main/install.ps1 | iex         # Windows
```

## Usage

```bash
cmdref                # Launch interactive TUI
cmdref --search tail  # Launch with search pre-filled
cmdref update         # Check and install updates
cmdref --help         # Show help
cmdref --version      # Show version
```

### Keyboard Shortcuts

**Sidebar:**

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` / `→` | Expand/collapse platform, or enter category |
| `Tab` | Switch to content area |
| `/` | Enter search mode |
| `B` | Jump to bookmarks |
| `H` | Jump to history |
| `1` - `5` | Jump to platform (Linux / macOS / Windows / Dev Tools / Testing) |
| `q` | Quit |

**Content:**

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down in command list |
| `k` / `↑` | Move up in command list |
| `←` / `Tab` | Back to sidebar |
| `b` | Toggle bookmark on current command |
| `B` | Jump to bookmarks |
| `H` | Jump to history |
| `/` | Enter search mode |
| `Esc` | Back to sidebar |
| `q` | Quit |

**Search:**

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate search results |
| `Enter` | Jump to selected result |
| `Esc` | Exit search |

## Command Categories

| Platform | Categories | Commands |
|----------|-----------|----------|
| **Linux** | File ops, Text processing, Editors, Archives, Network, Process management, System info, Log viewing, User management, Service management | 60 |
| **macOS** | Homebrew, System tools, Xcode | 17 |
| **Windows** | PowerShell, CMD, Package managers (winget) | 22 |
| **Dev Tools** | Git (with SSH setup), Docker, Database (MySQL/PostgreSQL/Redis/MongoDB), Kubernetes, Data processing (jq/yq) | 63 |
| **Testing** | ADB (Android), iOS, Network testing, Performance testing | 27 |

## Project Structure

```
command-tool/
├── data/commands/              # YAML command data files (189 commands)
│   ├── dev/                    # Dev Tools: git, docker, database, k8s, json_yaml
│   ├── linux/                  # Linux: file_ops, text_proc, editors, archive,
│   │                           #   network, process, system, log_view,
│   │                           #   user_mgmt, systemd
│   ├── mac/                    # macOS: brew, system, xcode
│   ├── windows/                # Windows: powershell, cmd, winget
│   └── testing/                # Testing: adb, ios, network, perf
├── src/                        # Rust source code
│   ├── main.rs                 # Entry point + event handling
│   ├── app.rs                  # App state machine + navigation
│   ├── data.rs                 # Data structures + YAML loading
│   ├── search.rs               # Fuzzy search engine (names, examples, tips, tags)
│   ├── bookmarks.rs            # Bookmark persistence
│   ├── clipboard.rs            # Cross-platform clipboard
│   ├── history.rs              # View history tracking
│   ├── debug.rs                # Debug logging (CMDREF_DEBUG=1)
│   ├── update.rs               # Self-update via GitHub API
│   └── ui/                     # TUI rendering
│       ├── layout.rs           # Main layout (title + main + help)
│       ├── sidebar.rs          # Sidebar (platforms, categories, bookmarks, history)
│       ├── content.rs          # Content panel (command list + detail view)
│       ├── help.rs             # Bottom help bar (context-sensitive)
│       └── search.rs           # Search input + results
├── brew/                       # Homebrew Formula + Scoop manifest
├── scripts/                    # Release helper scripts
├── .github/                    # Issue/PR templates + CI workflow
├── install.sh                  # macOS/Linux install script
├── install.ps1                 # Windows install script
├── CONTRIBUTING.md             # Contribution guidelines
└── CHANGELOG.md                # Version history
```

## Custom Commands

You can add your own commands without modifying the source code. Create YAML files in `~/.config/cmdref/custom/`:

```yaml
# ~/.config/cmdref/custom/my-tools.yaml
category: "My Tools"
description: "My team's custom commands"
platform: dev    # or linux, mac, windows, testing, or a new name
commands:
  - name: deploy
    summary: "Deploy application to staging"
    tags: ["部署", "staging"]
    examples:
      - description: "Deploy to staging"
        code: "deploy --env staging --version latest"
        frequency: daily
      - description: "Deploy to production"
        code: "deploy --env production --version v1.2.3"
        frequency: weekly
        danger: high
    tips:
      - "Always deploy to staging first"
    related: ["rollback", "health-check"]
```

Custom commands are merged into the built-in data at startup. Files with `platform: dev` will appear under Dev Tools, `platform: linux` under Linux, etc. You can also use custom platform names to create entirely new sections.

### YAML Fields Reference

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Command name |
| `summary` | Yes | One-line description |
| `tags` | No | Scenario tags for search (e.g. `["回退", "调试"]`) |
| `examples` | No | List of usage examples |
| `examples[].description` | Yes | What this example does |
| `examples[].code` | Yes | The actual command |
| `examples[].frequency` | No | `daily` / `weekly` / `rarely` |
| `examples[].danger` | No | `high` / `medium` / `none` |
| `tips` | No | Helpful notes about the command |
| `related` | No | Related commands |

## Contributing

Contributions are welcome! The easiest way to contribute is to add new commands.

### Adding a New Command

1. Open the appropriate YAML file in `data/commands/<platform>/`
2. Add your command entry:

```yaml
- name: your-command
  summary: "Brief description of what the command does"
  tags: ["场景1", "场景2"]
  examples:
    - description: "Most common usage"
      code: "your-command --flag arg"
      frequency: daily
    - description: "Dangerous variant"
      code: "your-command --force"
      frequency: rarely
      danger: high
  tips:
    - "A useful tip about this command"
  related: ["similar-command-1", "similar-command-2"]
```

3. If adding a new category, create a new YAML file in the platform directory and register it in `src/data.rs`:

```rust
("linux", "Linux", include_str!("../data/commands/linux/your_new_file.yaml")),
```

4. Run `cargo run` to verify your changes look correct
5. Submit a pull request

### Adding a New Platform

1. Create a new directory: `data/commands/<platform>/`
2. Add YAML category files
3. Register `include_str!` entries in `src/data.rs`
4. Add the platform to the `display_name_map` in `merge_custom_commands()`
5. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## Building from Source

Requires [Rust](https://rustup.rs/) 1.70+.

```bash
git clone https://github.com/xuankew/cmdRef.git
cd command-tool
cargo build --release
./target/release/cmdref
```

### Debug Mode

```bash
CMDREF_DEBUG=1 cargo run
cat ~/Library/Application\ Support/cmdref/debug.log   # macOS
cat ~/.config/cmdref/debug.log                         # Linux
```

## Roadmap

- [ ] Copy-to-clipboard with keyboard shortcut (pending stable cross-terminal solution)
- [ ] More macOS commands (defaults, launchctl, plutil)
- [ ] CI/CD commands (GitHub Actions, GitLab CI)
- [ ] Build tools (make, cmake, gradle, npm, cargo)
- [ ] Test frameworks (pytest, jest, go test)
- [ ] Scenario-based search mode
- [ ] Screenshots and demo GIF

## License

[MIT](LICENSE)
