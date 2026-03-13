#!/bin/bash
set -euo pipefail

# GitID Installer
# Usage: curl -fsSL https://gitid.dev/install | sh

REPO="vipinkashyap/gitid"
INSTALL_DIR="${GITID_INSTALL_DIR:-$HOME/.local/bin}"

echo "╔═══════════════════════════════════════╗"
echo "║         GitID Installer               ║"
echo "║  Multi-profile Git identity manager   ║"
echo "╚═══════════════════════════════════════╝"
echo ""

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  OS_NAME="linux" ;;
    Darwin) OS_NAME="macos" ;;
    *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64)  ARCH_NAME="x86_64" ;;
    aarch64|arm64) ARCH_NAME="aarch64" ;;
    *)       echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

echo "Detected: $OS_NAME-$ARCH_NAME"

# Check for cargo as fallback
if command -v cargo &>/dev/null; then
    echo "Found cargo. Installing from source..."
    cargo install --git "https://github.com/$REPO" gitid-cli git-credential-gitid
else
    echo "Cargo not found. Downloading pre-built binaries..."

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download binaries (placeholder URL - replace with actual release URLs)
    RELEASE_URL="https://github.com/$REPO/releases/latest/download"
    for binary in gitid git-credential-gitid; do
        echo "  Downloading $binary..."
        curl -fsSL "$RELEASE_URL/${binary}-${OS_NAME}-${ARCH_NAME}" -o "$INSTALL_DIR/$binary"
        chmod +x "$INSTALL_DIR/$binary"
    done

    # Check PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo ""
        echo "⚠  $INSTALL_DIR is not in your PATH."
        echo "   Add this to your shell profile:"
        echo ""
        echo "   export PATH=\"\$PATH:$INSTALL_DIR\""
        echo ""
    fi
fi

echo ""
echo "✓ GitID installed successfully!"
echo ""
echo "Next steps:"
echo "  1. Run 'gitid init' to set up your first profile"
echo "  2. Run 'gitid doctor' to verify everything is working"
echo ""
