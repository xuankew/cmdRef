# Changelog

All notable changes to CmdRef are documented in this file.

## [0.6.0] - 2026-09-08

### New Features

- **Prompts manager** — record AI prompts with name, description, and multi-line content (`P` to jump, `n` to create, `e` to edit, `d` to delete, `y` to copy)
- **WebDAV cloud sync** — `cmdref sync login/push/pull/status/logout` keeps bookmarks, custom commands, and prompts in sync across devices (works with 坚果云, Nextcloud, and any standard WebDAV server)
- **GitHub cloud sync** — `cmdref sync login --backend github --token <PAT> --repo <owner/repo>` syncs data to a user-created private GitHub repository via Contents API; supports fine-grained PAT scoped to a single repo
- **3-way merge on pull** — non-conflicting changes from multiple devices are combined automatically; conflicts default to the cloud version (`--prefer-local` to keep local), with pre-sync backups (last 10 kept)
- **Cross-device data folder** — set `CMDREF_DATA_DIR` to a cloud-synced folder so bookmarks, custom commands, and prompts follow you across devices
- **`cmdref path`** — print the resolved data directory

### UI Improvements

- Sidebar now shows `🤖 Prompts` alongside Bookmarks and History
- Content area supports prompt-specific rendering with scrollable detail view
- New prompt editor with multi-line content input and `Ctrl+S` save
- Help bar updated with Prompts shortcuts and `P` navigation

### Data Structure

- Added `src/prompts.rs` with dedicated `Prompt` / `PromptFile` / `PromptStore` models
- Added `src/paths.rs` for centralized path resolution (`data_dir`, `prompts_dir`, `custom_dir`, `bookmarks_file`, `history_file`, `log_path`)
- Added `src/sync/` module (`config.rs`, `dav.rs`, `github.rs`, `snapshot.rs`, `merge.rs`) — WebDAV and GitHub via curl shell-out, credentials passed securely, zero HTTP library dependencies
- Added WebDAV integration tests (`tests/webdav_server.py` + `tests/sync_dav.rs`, auto-skip without python3)
- Custom commands and prompts share the configurable `CMDREF_DATA_DIR`; history and debug logs remain local

### Bug Fixes

- Fixed subtract-with-overflow panic in prompt editor cursor positioning on small terminals
- Fixed CJK input byte-slice panic in editor rendering

## [0.3.0] - 2026-08-26

### New Content — Dev Tools Platform (63 commands)

- **Git** (22 commands): clone, init, add, commit, status, push, pull, fetch, branch, checkout, switch, merge, rebase, log, diff, stash, reset, revert, cherry-pick, show, blame, tag, remote, reflog, config, clean, SSH setup
- **Docker** (16 commands): ps, images, run, build, exec, logs, compose up/down/logs, cp, inspect, volume, network, system prune, tag, push
- **Database** (6 tools): mysql, psql, redis-cli, mongosh, sqlite3, createdb/dropdb
- **Kubernetes** (13 commands): get, describe, logs, exec, apply, delete, port-forward, rollout, scale, top, config, debug
- **Data Processing** (2 tools): jq (10 examples), yq (6 examples)

### New Content — Linux Extensions

- **Service Management** (new category): systemctl, journalctl
- **Network** (extended): tcpdump, iptables

### UI Improvements

- **Quick Reference card**: daily-use examples displayed at top of detail view
- **Danger warnings**: `⚠ DANGEROUS` (red) and `⚠ use with caution` (yellow) for risky commands
- **Frequency indicators**: ⚡ daily, ○ weekly, · rarely on each example
- **Scenario tags**: displayed as `#tags` below command summary
- **Tags in search**: scenario tags indexed with 2x search weight
- **Platform jump**: `1-5` keys (added Dev Tools)

### Data Structure

- Added `tags` field to Command (scenario-based search)
- Added `frequency` field to Example (`daily` / `weekly` / `rarely`)
- Added `danger` field to Example (`high` / `medium` / `none`)

### Bug Fixes

- Fixed `select_search_result()`: platform expansion now correctly sets cursor before calling `toggle_sidebar_item()`
- Fixed sidebar Enter/Right: Platform items only expand/collapse (no longer auto-enters content area)
- Fixed `update_selection()`: preserves `selected_command` index across sidebar navigation

## [0.2.0] - 2026-08-25

### Features

- **Bookmarks**: press `b` to bookmark commands, `B` to view bookmark list
- **History**: press `H` to view recently viewed commands (max 10)
- **Copy to clipboard**: press `y` to copy command example (currently disabled, pending stable cross-terminal solution)
- **Enhanced search**: fuzzy matching on names, examples, tips, and scenario tags
- **Custom commands**: load user YAML files from `~/.config/cmdref/custom/`
- **Platform auto-detection**: highlights current OS on startup
- **Debug logging**: `CMDREF_DEBUG=1` enables file-based debug log

### Content (v0.2.0)

- Linux: 9 categories (58 commands)
- macOS: 3 categories (17 commands)
- Windows: 3 categories (22 commands)
- Testing: 4 categories (27 commands)

## [0.1.0] - 2026-08-24

### Initial Release

- Interactive TUI with sidebar navigation and content view
- Fuzzy search across command names and descriptions
- Cross-platform: macOS, Linux, Windows
- Self-update via GitHub Releases API
- Homebrew and install script distribution
- 120+ commands across 19 categories
