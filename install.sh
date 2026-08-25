#!/bin/bash
# CmdRef - Interactive Command Reference Tool Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/xuanke/command-tool/main/install.sh | bash

set -euo pipefail

REPO="xuanke/command-tool"
BINARY_NAME="cmdref"
VERSION="${1:-latest}"

# Detect OS and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="linux" ;;
        Darwin*) os="macos" ;;
        *)       echo "Unsupported OS: $(uname -s)"; exit 1 ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)             echo "Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac

    echo "${os}-${arch}"
}

# Determine install directory
get_install_dir() {
    if [ -n "${PREFIX:-}" ]; then
        echo "$PREFIX"
    elif [ -w "/usr/local/bin" ]; then
        echo "/usr/local/bin"
    else
        echo "$HOME/.local/bin"
    fi
}

# Download and install
install() {
    local platform install_dir url

    platform=$(detect_platform)
    install_dir=$(get_install_dir)

    echo "CmdRef Installer"
    echo "================"
    echo "Platform:  $platform"
    echo "Install to: $install_dir"
    echo ""

    # Determine download URL
    if [ "$VERSION" = "latest" ]; then
        url="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}-${platform}"
    else
        # Remove leading 'v' if present
        VERSION="${VERSION#v}"
        url="https://github.com/${REPO}/releases/download/v${VERSION}/${BINARY_NAME}-${platform}"
    fi

    # Create install directory
    mkdir -p "$install_dir"

    # Download binary
    echo "Downloading ${BINARY_NAME}..."
    if command -v curl &>/dev/null; then
        curl -fsSL -o "${install_dir}/${BINARY_NAME}" "$url"
    elif command -v wget &>/dev/null; then
        wget -q -O "${install_dir}/${BINARY_NAME}" "$url"
    else
        echo "Error: curl or wget is required"
        exit 1
    fi

    # Make executable
    chmod +x "${install_dir}/${BINARY_NAME}"

    echo ""
    echo "Installation complete!"
    echo ""

    # Check if install dir is in PATH
    if ! echo "$PATH" | tr ':' '\n' | grep -q "^${install_dir}$"; then
        echo "NOTE: ${install_dir} is not in your PATH."
        echo "Add it by running:"
        echo ""
        echo "  export PATH=\"${install_dir}:\$PATH\""
        echo ""
        echo "Add this line to your ~/.zshrc or ~/.bashrc for persistence."
        echo ""
    fi

    # Verify installation
    if "${install_dir}/${BINARY_NAME}" --version &>/dev/null; then
        echo "Run 'cmdref' to start!"
    else
        echo "Warning: binary installed but may not work on this platform."
    fi
}

install
