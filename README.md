# CmdRef

> Interactive command reference tool — quickly look up Linux, macOS, Windows, and testing commands in your terminal.

CmdRef is a terminal-based cheatsheet that helps developers and testers quickly find and review commonly used commands. Browse by category, search with fuzzy matching, and view detailed examples — all without leaving the terminal.

## Features

- **100+ commands** across Linux, macOS, Windows, and testing tools
- **Interactive TUI** with sidebar navigation and detailed command views
- **Fuzzy search** — type any keyword to instantly find matching commands (searches names, examples, and tips)
- **Copy to clipboard** — press `y` to copy any command's first example code
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

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Expand platform / Select command |
| `Tab` | Switch between sidebar and content |
| `/` | Enter search mode |
| `y` | Copy command example to clipboard |
| `b` | Toggle bookmark on current command |
| `B` | Jump to bookmarks |
| `H` | Jump to history |
| `Esc` | Go back / Exit search |
| `1` - `4` | Jump to platform |
| `q` | Quit |

## Command Categories

| Platform | Categories |
|----------|-----------|
| **Linux** | File ops, Text processing, Editors, Archives, Network, Process management, System info, Log viewing, User management |
| **macOS** | Homebrew, System tools, Xcode |
| **Windows** | PowerShell, CMD, Package managers (winget/scoop) |
| **Testing** | ADB (Android), iOS, Network testing, Performance testing |

## Project Structure

```
command-tool/
├── data/commands/          # YAML command data files
│   ├── linux/              # 9 category files
│   ├── mac/                # 3 category files
│   ├── windows/            # 3 category files
│   └── testing/            # 4 category files
├── src/                    # Rust source code
│   ├── main.rs             # Entry point + event handling
│   ├── app.rs              # App state machine
│   ├── data.rs             # Data structures + YAML loading
│   ├── search.rs           # Fuzzy search engine
│   ├── bookmarks.rs        # Bookmark persistence
│   ├── clipboard.rs        # Cross-platform clipboard
│   ├── history.rs          # View history tracking
│   ├── update.rs           # Self-update via GitHub API
│   └── ui/                 # TUI rendering
├── brew/                   # Homebrew Formula + Scoop manifest
├── scripts/                # Release helper scripts
├── install.sh              # macOS/Linux install script
└── install.ps1             # Windows install script
```

## Custom Commands

You can add your own commands without modifying the source code. Create YAML files in `~/.config/cmdref/custom/`:

```yaml
# ~/.config/cmdref/custom/docker.yaml
category: Docker
description: Docker container management commands
platform: linux    # or mac, windows, or a custom name
commands:
  - name: docker ps
    summary: "List running containers"
    examples:
      - description: "Show all containers (including stopped)"
        code: "docker ps -a"
    tips:
      - "Use --format to customize output"
    related: ["docker images", "docker logs"]
```

Custom commands are merged into the built-in data at startup. Files with `platform: mac` will appear under macOS, `platform: linux` under Linux, etc. You can also use custom platform names to create entirely new sections.

## Contributing

Contributions are welcome! The easiest way to contribute is to add new commands.

### Adding a New Command

1. Open the appropriate YAML file in `data/commands/<platform>/`
2. Add your command entry:

```yaml
- name: your-command
  summary: "Brief description of what the command does"
  examples:
    - description: "Most common usage"
      code: "your-command --flag arg"
    - description: "Another example"
      code: "your-command -r dir/"
  tips:
    - "A useful tip about this command"
  related: ["similar-command-1", "similar-command-2"]
```

3. If adding a new category, create a new YAML file in the platform directory and add one line to `src/data.rs` in the `load_all_data()` function:

```rust
("linux", "Linux", include_str!("../data/commands/linux/your_new_file.yaml")),
```

4. Run `cargo run` to verify your changes look correct
5. Submit a pull request

### Adding a New Platform

1. Create a new directory: `data/commands/<platform>/`
2. Add YAML category files
3. Add `include_str!` entries in `src/data.rs`
4. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## Building from Source

Requires [Rust](https://rustup.rs/) 1.70+.

```bash
git clone https://github.com/xuankew/cmdRef.git
cd command-tool
cargo build --release
./target/release/cmdref
```

## License

[MIT](LICENSE)
