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
- **Prompts** — record and quickly reuse AI prompts with name, description, and multi-line content
- **Cloud sync** — `cmdref sync push/pull` syncs bookmarks, custom commands, and prompts across devices via GitHub (private repo) or WebDAV (坚果云, Nextcloud) with automatic 3-way merge
- **Cross-device sync** — set `CMDREF_DATA_DIR` to a cloud-synced folder (iCloud, Dropbox, OneDrive, etc.)
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
cmdref path           # Show the resolved data directory
cmdref sync login --backend github --token <PAT>  # Configure GitHub sync (private repo)
cmdref sync login <url> <user> <password>          # Configure WebDAV sync (e.g. Jianguoyun)
cmdref sync push      # Upload local data to the cloud
cmdref sync pull      # Download and merge cloud data
cmdref sync status    # Show local/remote sync status
cmdref sync logout    # Remove saved credentials
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
| `n` | Add a new command or prompt |
| `B` | Jump to bookmarks |
| `H` | Jump to history |
| `P` | Jump to prompts |
| `1` - `5` | Jump to platform (Linux / macOS / Windows / Dev Tools / Testing) |
| `q` | Quit |

**Content:**

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down in command/prompt list |
| `k` / `↑` | Move up in command/prompt list |
| `←` / `Tab` | Back to sidebar |
| `1` - `9` | Copy the corresponding example |
| `b` | Toggle bookmark on current command |
| `y` | Copy the full prompt content (when viewing prompts) |
| `n` | Add a new command or prompt |
| `e` | Edit the current prompt |
| `d` | Delete the current custom command or prompt |
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
│   ├── paths.rs                # Data/config path resolution
│   ├── prompts.rs              # Prompt data model and persistence
│   ├── sync/                   # Cloud sync (GitHub + WebDAV)
│   │   ├── mod.rs              # sync subcommands (login/push/pull/status/logout)
│   │   ├── config.rs           # Credentials, URL normalization, device id
│   │   ├── dav.rs              # WebDAV client (curl shell-out, no new deps)
│   │   ├── github.rs           # GitHub client (Contents API, PAT auth, curl shell-out)
│   │   ├── snapshot.rs         # Snapshot build/apply, merge baseline, backups
│   │   └── merge.rs            # 3-way merge with conflict report
│   ├── update.rs               # Self-update via GitHub API
│   └── ui/                     # TUI rendering
│       ├── layout.rs           # Main layout (title + main + help)
│       ├── sidebar.rs          # Sidebar (platforms, categories, bookmarks, history, prompts)
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

## Prompts

CmdRef now includes a dedicated **Prompts** manager for AI prompts. Each prompt has a name, description, and multi-line content, and is stored independently from commands so long text renders correctly.

**Keyboard shortcuts (Prompts):**

| Key | Action |
|-----|--------|
| `P` | Jump to prompts from sidebar |
| `n` | Create a new prompt |
| `e` | Edit the selected prompt |
| `d` | Delete the selected prompt |
| `y` / `Enter` / `1` | Copy the full prompt content to clipboard |

Prompts are saved to `<data_dir>/prompts/my_prompts.yaml` by default. You can also create additional `*.yaml` or `*.yml` files in that directory and CmdRef will load them on startup.

```yaml
# ~/.config/cmdref/prompts/my_prompts.yaml
title: "My Prompts"
prompts:
  - name: code-review
    description: "Review a code diff"
    content: |
      Please review the following code diff. Focus on:
      1. Correctness
      2. Performance
      3. Readability
      4. Security
    tags: ["coding", "review"]
```

## Cross-Device Sync

Sync bookmarks, custom commands, and prompts across devices in one of three ways. Only one backend is active at a time — running `sync login` switches the backend.

### Option 1: GitHub Sync

Uses a private GitHub repository to store sync data. You create the repository yourself, then grant CmdRef access via a Personal Access Token scoped to that single repo.

#### Setup

```bash
# Step 1: Create a new private repository on GitHub (e.g. "cmdref-sync")
#         → https://github.com/new → name it, set Private, click Create
#         (no need to add README or .gitignore)

# Step 2: Generate a fine-grained Personal Access Token
#         → https://github.com/settings/tokens?type=beta
#         → "Generate new token"
#         → Name: "cmdref-sync" (or any name you like)
#         → Repository access: "Only select repositories" → pick your sync repo
#         → Permissions → Repository permissions → Contents: Read and write
#         → Click "Generate token" and copy it

# Step 3: Login (--repo accepts multiple formats)
cmdref sync login --backend github --token github_pat_xxxx --repo yourname/cmdref-sync
# Or using SSH URL:
cmdref sync login --backend github --token github_pat_xxxx --repo git@github.com:yourname/cmdref-sync.git
# Or using HTTPS URL:
cmdref sync login --backend github --token github_pat_xxxx --repo https://github.com/yourname/cmdref-sync
```

#### Daily Usage

```bash
cmdref sync push               # Upload local data to GitHub
cmdref sync pull               # Download and merge cloud data into local
cmdref sync status             # Show local/remote versions and pending changes
cmdref sync logout             # Remove saved credentials (data untouched)
```

Data is stored as `cmdref-sync.json` in the root of the repository you specified.

#### On a New Device

```bash
# Install cmdref, then pull existing data first
cmdref sync login --backend github --token <same-PAT> --repo <same-repo>
cmdref sync pull               # Downloads cloud data to local
```

### Option 2: WebDAV Sync

Works with Jianguoyun (坚果云), Nextcloud, or any standard WebDAV server.

#### Setup

```bash
# Jianguoyun (坚果云): use an app password, NOT your account password
#   → Account Info → Security Options → App Passwords → create one
cmdref sync login https://dav.jianguoyun.com/dav/cmdref/ you@example.com <app-password>

# Nextcloud example
cmdref sync login https://cloud.example.com/remote.php/dav/files/username/cmdref/ username password
```

#### Daily Usage

```bash
cmdref sync push               # Upload local data to the WebDAV server
cmdref sync pull               # Download and merge cloud data into local
cmdref sync status             # Show local/remote versions and pending changes
cmdref sync logout             # Remove saved credentials (data untouched)
```

The sync file (`cmdref-sync.json`) is stored under the WebDAV directory you configured. The directory is created automatically on first push.

### Merging & Backups

Both backends share the same merge behavior:

- **`pull` performs a 3-way merge** (local / cloud / last-synced baseline). Non-conflicting changes from different devices are combined automatically — a bookmark added on your laptop and a prompt added on your desktop both survive.
- **Conflict resolution**: if the same item was changed differently on both devices, the cloud version wins by default. Pass `--prefer-local` to keep the local version instead.
- **Automatic backups**: before writing, `pull` backs up your current data to `sync-backup/` in the config directory (last 10 backups kept).
- **First device**: run `push` to upload. On subsequent devices, run `pull` first — `push` will refuse if the cloud has newer data you haven't pulled yet. Use `--force` to override.

### Security Notes

- Credentials are stored in plain text in `sync.json` under the config directory (permission `600`), similar to `~/.netrc`
- For WebDAV, prefer an application password over your main account password
- For GitHub, use a fine-grained PAT scoped to only the sync repository — avoid classic tokens with full `repo` scope
- Requires the system `curl` executable (same as `cmdref update`)

### Option 3: Cloud Folder (`CMDREF_DATA_DIR`)

Point `CMDREF_DATA_DIR` at a folder synced by iCloud, Dropbox, OneDrive, or any other cloud provider:

```bash
# macOS with iCloud Drive
export CMDREF_DATA_DIR="$HOME/Library/Mobile Documents/com~apple~CloudDocs/cmdref"

# Or anywhere else
cmdref path                      # Verify the resolved path
CMDREF_DATA_DIR=/tmp/cmdref-test cmdref
```

When `CMDREF_DATA_DIR` is set, CmdRef reads and writes:

- `<CMDREF_DATA_DIR>/custom/` — custom commands
- `<CMDREF_DATA_DIR>/prompts/` — prompts
- `<CMDREF_DATA_DIR>/bookmarks.json` — bookmarks

Local-only state remains in the default config directory:

- `~/.config/cmdref/history.json`
- `~/.config/cmdref/debug.log`
- Sync state: `sync.json` (credentials), `sync-base.json` (merge baseline), `sync-backup/` (pre-sync backups)

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

- [x] Copy-to-clipboard with keyboard shortcut
- [x] Custom commands
- [x] Prompts manager
- [x] WebDAV cloud sync (坚果云 / Nextcloud)
- [x] GitHub cloud sync (private repo, PAT auth)
- [ ] More macOS commands (defaults, launchctl, plutil)
- [ ] CI/CD commands (GitHub Actions, GitLab CI)
- [ ] Build tools (make, cmake, gradle, npm, cargo)
- [ ] Test frameworks (pytest, jest, go test)
- [ ] Scenario-based search mode
- [ ] Screenshots and demo GIF

## License

[MIT](LICENSE)
