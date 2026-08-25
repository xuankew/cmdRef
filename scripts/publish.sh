#!/bin/bash
# CmdRef 一键发布脚本
# 用法: ./scripts/publish.sh [version]
# 示例: ./scripts/publish.sh 0.2.0
#       ./scripts/publish.sh          # 自动 bump patch 版本
#
# 完整流程:
#   1. 更新 Cargo.toml 版本号
#   2. 本地编译验证
#   3. 提交版本变更并推送
#   4. 创建 git tag 触发 CI
#   5. 等待 CI 编译完成
#   6. 下载产物并更新 SHA256
#   7. 提交 SHA256 更新
#   8. 同步 Homebrew Formula 到 tap 仓库
#   9. 本地验证 brew install

set -euo pipefail

# ==================== 配置 ====================
REPO="xuankew/cmdRef"
TAP_REPO_DIR="../homebrew-cmdref"
TAP_REPO_NAME="xuankew/homebrew-cmdref"
CI_POLL_INTERVAL=15
CI_TIMEOUT=600

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# ==================== 颜色输出 ====================
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${BLUE}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
fail()  { echo -e "${RED}[FAIL]${NC}  $*"; exit 1; }
step()  { echo -e "\n${CYAN}━━━ $* ━━━${NC}"; }

# ==================== 版本计算 ====================
get_current_version() {
    grep '^version' "${PROJECT_DIR}/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/'
}

bump_patch_version() {
    local ver="$1"
    local major minor patch
    IFS='.' read -r major minor patch <<< "$ver"
    echo "${major}.${minor}.$((patch + 1))"
}

# ==================== 前置检查 ====================
preflight() {
    step "0/8 前置检查"

    # cargo
    if ! command -v cargo &>/dev/null; then
        fail "cargo not found. Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    fi
    ok "cargo available"

    # git
    if ! command -v git &>/dev/null; then
        fail "git not found"
    fi
    ok "git available"

    # python3
    if ! command -v python3 &>/dev/null; then
        fail "python3 not found (needed for Scoop manifest)"
    fi
    ok "python3 available"

    # 项目目录
    cd "$PROJECT_DIR"
    if [ ! -f "Cargo.toml" ]; then
        fail "Cargo.toml not found in ${PROJECT_DIR}"
    fi
    ok "project directory: ${PROJECT_DIR}"

    # homebrew tap 仓库
    local tap_path
    tap_path="$(cd "$PROJECT_DIR" && cd "$TAP_REPO_DIR" 2>/dev/null && pwd)" || true
    if [ -z "$tap_path" ] || [ ! -d "$tap_path/.git" ]; then
        fail "Homebrew tap repo not found at ${TAP_REPO_DIR}\n  Expected: $(cd "$PROJECT_DIR" && pwd)/../homebrew-cmdref"
    fi
    TAP_REPO_DIR="$tap_path"
    ok "tap repo: ${TAP_REPO_DIR}"

    # git status
    if [ -n "$(git status --porcelain)" ]; then
        warn "working directory has uncommitted changes"
        echo ""
        echo "  Uncommitted files:"
        git status --short | sed 's/^/    /'
        echo ""
        echo -n "  Continue and auto-commit these changes? [y/N] "
        read -r answer
        if [ "$answer" != "y" ] && [ "$answer" != "Y" ]; then
            fail "Aborted. Please commit or stash your changes first."
        fi
        AUTO_COMMIT_CHANGES=true
    else
        AUTO_COMMIT_CHANGES=false
        ok "working directory clean"
    fi
}

# ==================== Step 1: 版本号 ====================
step_version() {
    step "1/8 更新版本号"

    local current_version
    current_version=$(get_current_version)
    info "current version: ${current_version}"

    # 确定目标版本
    if [ -n "${TARGET_VERSION:-}" ]; then
        VERSION="$TARGET_VERSION"
    else
        VERSION=$(bump_patch_version "$current_version")
        echo -n "  Auto-bump to ${VERSION}? [Y/n] "
        read -r answer
        if [ "$answer" = "n" ] || [ "$answer" = "N" ]; then
            echo -n "  Enter version: "
            read -r VERSION
        fi
    fi

    if [ "$VERSION" = "$current_version" ]; then
        fail "Version ${VERSION} is same as current. Nothing to release."
    fi

    # 更新 Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"${VERSION}\"/" "${PROJECT_DIR}/Cargo.toml"
    rm -f "${PROJECT_DIR}/Cargo.toml.bak"

    ok "version: ${current_version} -> ${VERSION}"
}

# ==================== Step 2: 本地编译 ====================
step_build() {
    step "2/8 本地编译验证"

    cd "$PROJECT_DIR"
    info "running cargo build --release ..."
    if cargo build --release 2>&1 | tail -3; then
        ok "local build passed"
    else
        fail "local build failed"
    fi

    # 验证二进制能运行
    local ver_output
    ver_output=$(./target/release/cmdref --version 2>&1)
    if echo "$ver_output" | grep -q "cmdref ${VERSION}"; then
        ok "binary check: ${ver_output}"
    else
        warn "version mismatch: binary reports '${ver_output}' but expected '${VERSION}'"
    fi
}

# ==================== Step 3: 提交代码 ====================
step_commit() {
    step "3/8 提交并推送代码"

    cd "$PROJECT_DIR"
    git add -A
    git commit -m "release v${VERSION}"
    info "commit created: release v${VERSION}"

    git push origin main
    ok "code pushed to origin/main"
}

# ==================== Step 4: 创建 Tag ====================
step_tag() {
    step "4/8 创建 Tag 触发 CI"

    cd "$PROJECT_DIR"

    # 如果 tag 已存在则删除
    if git rev-parse "v${VERSION}" &>/dev/null; then
        warn "tag v${VERSION} already exists, deleting"
        git tag -d "v${VERSION}" &>/dev/null || true
        git push origin ":refs/tags/v${VERSION}" &>/dev/null || true
    fi

    git tag "v${VERSION}"
    git push origin "v${VERSION}"
    ok "tag v${VERSION} pushed, CI triggered"
}

# ==================== Step 5: 等待 CI ====================
wait_for_ci() {
    step "5/8 等待 CI 编译完成"

    local api_url="https://api.github.com/repos/${REPO}/actions/runs?branch=v${VERSION}&event=push"
    local elapsed=0
    local run_id=""

    info "polling GitHub API for workflow run..."

    # 等待 run 出现
    while [ $elapsed -lt $CI_TIMEOUT ]; do
        local response
        response=$(curl -fsSL "$api_url" 2>/dev/null) || true

        run_id=$(echo "$response" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    runs = data.get('workflow_runs', [])
    for r in runs:
        if r['head_sha'] == '$(git rev-parse HEAD)':
            print(r['id'])
            break
except:
    pass
" 2>/dev/null) || true

        if [ -n "$run_id" ]; then
            break
        fi

        sleep "$CI_POLL_INTERVAL"
        elapsed=$((elapsed + CI_POLL_INTERVAL))
        echo -n "."
    done

    if [ -z "$run_id" ]; then
        fail "timeout waiting for CI to start (no workflow run found)"
    fi

    echo ""
    ok "workflow run detected: ${run_id}"
    info "details: https://github.com/${REPO}/actions/runs/${run_id}"
    echo ""

    # 轮询 run 状态
    local run_url="https://api.github.com/repos/${REPO}/actions/runs/${run_id}"
    local status=""
    local conclusion=""

    while [ $elapsed -lt $CI_TIMEOUT ]; do
        local response
        response=$(curl -fsSL "$run_url" 2>/dev/null) || true

        status=$(echo "$response" | python3 -c "
import json, sys
try:
    print(json.load(sys.stdin).get('status', ''))
except:
    pass
" 2>/dev/null) || true

        conclusion=$(echo "$response" | python3 -c "
import json, sys
try:
    print(json.load(sys.stdin).get('conclusion', '') or '')
except:
    pass
" 2>/dev/null) || true

        if [ "$status" = "completed" ]; then
            break
        fi

        printf "\r  CI status: %-12s (%ds elapsed)" "$status" "$elapsed"
        sleep "$CI_POLL_INTERVAL"
        elapsed=$((elapsed + CI_POLL_INTERVAL))
    done

    echo ""

    if [ "$status" != "completed" ]; then
        fail "CI did not complete within ${CI_TIMEOUT}s. Check: https://github.com/${REPO}/actions/runs/${run_id}"
    fi

    if [ "$conclusion" != "success" ]; then
        fail "CI failed with conclusion: ${conclusion}\n  Check: https://github.com/${REPO}/actions/runs/${run_id}"
    fi

    ok "CI completed successfully (${elapsed}s)"
}

# ==================== Step 6: 更新 SHA256 ====================
step_sha256() {
    step "6/8 下载产物并更新 SHA256"

    cd "$PROJECT_DIR"
    info "running release.sh ${VERSION}..."
    bash "${SCRIPT_DIR}/release.sh" "${VERSION}"
    ok "SHA256 and Formula updated"
}

# ==================== Step 7: 提交 SHA256 ====================
step_sha256_commit() {
    step "7/8 提交 SHA256 更新"

    cd "$PROJECT_DIR"
    git add -A
    git commit -m "release v${VERSION} - update SHA256"
    git push origin main
    ok "SHA256 updates committed and pushed"
}

# ==================== Step 8: 同步 Tap & 验证 ====================
step_tap() {
    step "8/8 同步 Homebrew Tap 并验证"

    # 复制 Formula
    cp "${PROJECT_DIR}/brew/cmdref.rb" "${TAP_REPO_DIR}/cmdref.rb"
    info "Formula copied to tap repo"

    # 提交 tap 仓库
    cd "$TAP_REPO_DIR"
    git add -A
    if git diff --cached --quiet; then
        warn "tap repo: no changes to commit"
    else
        git commit -m "cmdref v${VERSION}"
        git push origin main
        ok "tap repo updated and pushed"
    fi

    # 验证 brew install
    cd "$PROJECT_DIR"
    info "testing brew reinstall..."
    if brew reinstall cmdref 2>&1 | tail -3; then
        local installed_ver
        installed_ver=$(cmdref --version 2>&1)
        ok "brew install verified: ${installed_ver}"
    else
        warn "brew reinstall had issues, please check manually"
    fi
}

# ==================== 完成 ====================
done_message() {
    echo ""
    echo -e "${GREEN}==========================================${NC}"
    echo -e "${GREEN}  CmdRef v${VERSION} 发布成功!${NC}"
    echo -e "${GREEN}==========================================${NC}"
    echo ""
    echo "  Release:  https://github.com/${REPO}/releases/tag/v${VERSION}"
    echo "  Brew:     brew tap ${TAP_REPO_NAME} && brew install cmdref"
    echo ""
    echo "  Optional: cargo publish  (publish to crates.io)"
    echo ""
}

# ==================== 主流程 ====================
main() {
    TARGET_VERSION="${1:-}"
    AUTO_COMMIT_CHANGES=false

    echo ""
    echo -e "${CYAN}==========================================${NC}"
    echo -e "${CYAN}  CmdRef One-Click Release${NC}"
    echo -e "${CYAN}==========================================${NC}"
    echo ""

    preflight
    step_version
    step_build
    step_commit
    step_tag
    wait_for_ci
    step_sha256
    step_sha256_commit
    step_tap
    done_message
}

main "$@"
