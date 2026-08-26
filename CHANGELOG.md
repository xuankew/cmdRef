# Changelog

All notable changes to CmdRef are documented in this file.

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
