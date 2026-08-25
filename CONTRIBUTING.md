# Contributing to CmdRef

Thank you for your interest in contributing to CmdRef! The easiest way to contribute is by adding new commands to the database.

## How to Add Commands

### Step 1: Choose the Right File

Commands are organized by platform and category:

```
data/commands/
├── linux/      # Linux commands
├── mac/        # macOS-specific commands
├── windows/    # Windows commands (PowerShell, CMD, winget)
└── testing/    # Testing tools (ADB, iOS, network, performance)
```

Pick the YAML file that matches your command's category, or create a new one.

### Step 2: Add Your Command

Each command follows this format:

```yaml
- name: command-name
  summary: "One-line description"
  examples:
    - description: "What this example does"
      code: "command-name -flag arg"
    - description: "Another usage"
      code: "command-name --other-flag"
  tips:
    - "A helpful tip"
    - "Another tip"
  related: ["similar-command-1", "similar-command-2"]
```

**Guidelines:**
- `name`: Use the exact command name as typed in the terminal
- `summary`: Keep it concise (under 80 characters)
- `examples`: Include 3-6 examples covering the most common use cases
- `tips`: Add gotchas, platform differences, or memory aids
- `related`: List 2-4 commands that serve similar purposes

### Step 3: Adding a New Category File

If your command doesn't fit any existing category:

1. Create a new YAML file: `data/commands/<platform>/your_category.yaml`
2. Add the file header:

```yaml
category: "Category Name"
description: "Description of this category"
platform: linux  # or mac, windows, testing
commands:
  - name: your-command
    # ...
```

3. Register it in `src/data.rs` by adding one line in `load_all_data()`:

```rust
("linux", "Linux", include_str!("../data/commands/linux/your_category.yaml")),
```

### Step 4: Test Locally

```bash
cargo run
# Navigate to your new command and verify it displays correctly
```

### Step 5: Submit a PR

1. Fork the repository
2. Create a feature branch: `git checkout -b add-docker-commands`
3. Commit your changes: `git commit -m "Add Docker commands"`
4. Push and open a Pull Request

## Code Contributions

If you'd like to contribute to the Rust codebase:

1. Ensure `cargo build` compiles without warnings
2. Test with `cargo run`
3. Follow the existing code style

## Reporting Issues

- **Bug**: Use the Bug Report template
- **Feature**: Use the Feature Request template
- **Missing commands**: Use the Add Commands template

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
