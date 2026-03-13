#!/usr/bin/env bash
set -euo pipefail

REPO="rockia/claude-status"
BINARY_NAME="claude-status"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin) OS_TARGET="apple-darwin" ;;
  Linux)  OS_TARGET="unknown-linux-gnu" ;;
  *)
    echo "Error: Unsupported OS: $OS"
    echo "Supported: macOS (Darwin), Linux"
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64)  ARCH_TARGET="x86_64" ;;
  arm64)   ARCH_TARGET="aarch64" ;;
  aarch64) ARCH_TARGET="aarch64" ;;
  *)
    echo "Error: Unsupported architecture: $ARCH"
    echo "Supported: x86_64, arm64/aarch64"
    exit 1
    ;;
esac

TARGET="${ARCH_TARGET}-${OS_TARGET}"
ARCHIVE="${BINARY_NAME}-${TARGET}.tar.gz"

echo "Detecting platform: ${TARGET}"

# Get latest release download URL
RELEASE_URL="https://github.com/${REPO}/releases/latest/download/${ARCHIVE}"

# Download
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

echo "Downloading ${ARCHIVE}..."
if command -v curl &>/dev/null; then
  curl -fsSL "$RELEASE_URL" -o "${TMPDIR}/${ARCHIVE}"
elif command -v wget &>/dev/null; then
  wget -q "$RELEASE_URL" -O "${TMPDIR}/${ARCHIVE}"
else
  echo "Error: curl or wget required"
  exit 1
fi

# Extract
echo "Extracting..."
tar xzf "${TMPDIR}/${ARCHIVE}" -C "$TMPDIR"
chmod +x "${TMPDIR}/${BINARY_NAME}"

# Run install
echo "Installing..."
"${TMPDIR}/${BINARY_NAME}" install

echo ""
echo "Done! Restart Claude Code to see the new status line."
