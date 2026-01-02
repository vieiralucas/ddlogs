#!/bin/sh
# Install script for ddlogs
set -e

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64)
                TARGET="x86_64-unknown-linux-gnu"
                ;;
            *)
                echo "Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64)
                TARGET="x86_64-apple-darwin"
                ;;
            arm64)
                TARGET="aarch64-apple-darwin"
                ;;
            *)
                echo "Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    *)
        echo "Unsupported operating system: $OS"
        exit 1
        ;;
esac

# Get latest version from GitHub
REPO="faiscadev/ddlogs"
LATEST_URL="https://api.github.com/repos/$REPO/releases/latest"

echo "Fetching latest release..."
VERSION=$(curl -s "$LATEST_URL" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
    echo "Failed to fetch latest version"
    exit 1
fi

echo "Installing ddlogs $VERSION for $TARGET..."

# Download URL
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/ddlogs-$TARGET.tar.gz"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# Download and extract
echo "Downloading from $DOWNLOAD_URL..."
curl -L -o "$TMPDIR/ddlogs.tar.gz" "$DOWNLOAD_URL"

echo "Extracting..."
tar -xzf "$TMPDIR/ddlogs.tar.gz" -C "$TMPDIR"

# Determine install location
if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
elif [ -d "$HOME/.local/bin" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    echo ""
    echo "Note: $INSTALL_DIR is not in your PATH."
    echo "Add it to your PATH by adding this line to your shell profile:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

# Install binary
echo "Installing to $INSTALL_DIR..."
mv "$TMPDIR/ddlogs" "$INSTALL_DIR/ddlogs"
chmod +x "$INSTALL_DIR/ddlogs"

echo ""
echo "ddlogs installed successfully!"
echo "Run 'ddlogs configure' to set up your Datadog credentials."
