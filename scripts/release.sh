#!/bin/bash
# CmdRef 发布辅助脚本
# 用法: ./scripts/release.sh <version>
# 示例: ./scripts/release.sh 0.1.0
#
# 前置条件:
#   1. GitHub Release 已创建并且 CI 已完成编译
#   2. 已安装 cargo (用于 crates.io 发布)
#
# 本脚本会:
#   - 从 GitHub Release 下载各平台二进制文件
#   - 计算 SHA256 校验值
#   - 更新 brew Formula 和 scoop manifest
#   - 提示后续手动步骤

set -euo pipefail

VERSION="${1:-}"
REPO="xuankew/cmdRef"

if [ -z "$VERSION" ]; then
    echo "Usage: ./scripts/release.sh <version>"
    echo "Example: ./scripts/release.sh 0.1.0"
    exit 1
fi

VERSION="${VERSION#v}"
BASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}"
TEMP_DIR=$(mktemp -d)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# 用于存储 SHA256 的变量（兼容 bash 3.x，不用 declare -A）
HASH_MACOS_AARCH64=""
HASH_MACOS_X86_64=""
HASH_LINUX_AARCH64=""
HASH_LINUX_X86_64=""
HASH_WINDOWS=""

echo "=========================================="
echo "  CmdRef Release Helper v${VERSION}"
echo "=========================================="
echo ""

download_and_hash() {
    local name="$1"
    local var_name="$2"
    local url="${BASE_URL}/${name}"
    local file="${TEMP_DIR}/${name}"

    echo -n "  Downloading ${name}... "
    if curl -fsSL -o "$file" "$url" 2>/dev/null; then
        local hash
        hash=$(shasum -a 256 "$file" | awk '{print $1}')
        eval "${var_name}=\"${hash}\""
        echo "OK (SHA256: ${hash:0:16}...)"
    else
        echo "FAILED"
        eval "${var_name}=\"SHA256_NOT_AVAILABLE\""
    fi
}

echo "[1/3] Downloading release binaries and computing SHA256..."
echo ""
download_and_hash "cmdref-macos-aarch64" "HASH_MACOS_AARCH64"
download_and_hash "cmdref-macos-x86_64"  "HASH_MACOS_X86_64"
download_and_hash "cmdref-linux-aarch64" "HASH_LINUX_AARCH64"
download_and_hash "cmdref-linux-x86_64"  "HASH_LINUX_X86_64"
download_and_hash "cmdref.exe"           "HASH_WINDOWS"
echo ""

# 更新 Homebrew Formula
echo "[2/3] Updating Homebrew Formula..."
FORMULA_FILE="${PROJECT_DIR}/brew/cmdref.rb"

sed -i.bak \
    -e "s|version \".*\"|version \"${VERSION}\"|" \
    -e "s|DOWNLOAD_URL_MACOS_AARCH64|${BASE_URL}/cmdref-macos-aarch64|" \
    -e "s|DOWNLOAD_URL_MACOS_X86_64|${BASE_URL}/cmdref-macos-x86_64|" \
    -e "s|DOWNLOAD_URL_LINUX_AARCH64|${BASE_URL}/cmdref-linux-aarch64|" \
    -e "s|DOWNLOAD_URL_LINUX_X86_64|${BASE_URL}/cmdref-linux-x86_64|" \
    -e "s|SHA256_MACOS_AARCH64|${HASH_MACOS_AARCH64}|" \
    -e "s|SHA256_MACOS_X86_64|${HASH_MACOS_X86_64}|" \
    -e "s|SHA256_LINUX_AARCH64|${HASH_LINUX_AARCH64}|" \
    -e "s|SHA256_LINUX_X86_64|${HASH_LINUX_X86_64}|" \
    "$FORMULA_FILE"
rm -f "${FORMULA_FILE}.bak"
echo "  Updated: ${FORMULA_FILE}"
echo ""

# 更新 Scoop manifest
echo "[3/3] Updating Scoop manifest..."
MANIFEST_FILE="${PROJECT_DIR}/brew/cmdref.json"

python3 -c "
import json
with open('${MANIFEST_FILE}', 'r') as f:
    data = json.load(f)
data['version'] = '${VERSION}'
data['architecture']['64bit']['url'] = '${BASE_URL}/cmdref.exe'
data['architecture']['64bit']['hash'] = '${HASH_WINDOWS}'
data['autoupdate']['architecture']['64bit']['url'] = 'https://github.com/${REPO}/releases/download/v\$version/cmdref.exe'
with open('${MANIFEST_FILE}', 'w') as f:
    json.dump(data, f, indent=4)
    f.write('\n')
"
echo "  Updated: ${MANIFEST_FILE}"
echo ""

# 清理
rm -rf "$TEMP_DIR"

# 输出后续步骤
echo "=========================================="
echo "  后续步骤"
echo "=========================================="
echo ""
echo "1. 更新 Cargo.toml 中的版本号:"
echo "   version = \"${VERSION}\""
echo ""
echo "2. 提交并推送更改:"
echo "   git add -A && git commit -m 'release v${VERSION}'"
echo "   git push origin main"
echo ""
echo "3. Homebrew (同步 Formula 到 homebrew-cmdref tap 仓库):"
echo "   cp brew/cmdref.rb ../homebrew-cmdref/cmdref.rb"
echo "   cd ../homebrew-cmdref && git add . && git commit -m 'v${VERSION}' && git push"
echo ""
echo "   用户安装: brew tap xuankew/cmdref && brew install cmdref"
echo ""
echo "4. 发布到 crates.io:"
echo "   cargo publish"
echo ""
echo "Done!"
