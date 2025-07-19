#!/bin/bash

# Clean Language Manager Installer
# This script downloads and installs the latest version of cleanmanager

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="Ivan-Pasco/clean-language-manager"  # Update this to actual repo
BINARY_NAME="cleanmanager"
INSTALL_DIR="$HOME/.local/bin"

# Detect platform
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    
    case "$os" in
        linux*)
            case "$arch" in
                x86_64|amd64) echo "linux-x86_64" ;;
                *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
            esac
            ;;
        darwin*)
            case "$arch" in
                x86_64) echo "macos-x86_64" ;;
                arm64) echo "macos-aarch64" ;;
                *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
            esac
            ;;
        mingw*|cygwin*|msys*)
            case "$arch" in
                x86_64|amd64) echo "windows-x86_64" ;;
                *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
            esac
            ;;
        *)
            echo "Unsupported operating system: $os" >&2
            exit 1
            ;;
    esac
}

# Get latest release version
get_latest_version() {
    curl -s "https://api.github.com/repos/$REPO/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"([^"]+)".*/\1/'
}

# Download and extract binary
install_cleanmanager() {
    local platform=$(detect_platform)
    local version=$(get_latest_version)
    
    if [ -z "$version" ]; then
        echo -e "${RED}Error: Could not fetch latest version${NC}" >&2
        exit 1
    fi
    
    echo -e "${BLUE}Clean Language Manager Installer${NC}"
    echo -e "${BLUE}=================================${NC}"
    echo
    echo -e "Platform: ${GREEN}$platform${NC}"
    echo -e "Version:  ${GREEN}$version${NC}"
    echo -e "Install:  ${GREEN}$INSTALL_DIR${NC}"
    echo
    
    # Determine archive format and binary extension
    local archive_ext="tar.gz"
    local binary_ext=""
    if [[ "$platform" == *"windows"* ]]; then
        archive_ext="zip"
        binary_ext=".exe"
    fi
    
    local archive_name="${BINARY_NAME}-${platform}.${archive_ext}"
    local download_url="https://github.com/$REPO/releases/download/$version/$archive_name"
    
    echo -e "${YELLOW}Downloading cleanmanager...${NC}"
    echo "URL: $download_url"
    
    # Create temporary directory
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # Download the archive
    if ! curl -L -o "$archive_name" "$download_url"; then
        echo -e "${RED}Error: Failed to download cleanmanager${NC}" >&2
        rm -rf "$temp_dir"
        exit 1
    fi
    
    echo -e "${YELLOW}Extracting archive...${NC}"
    
    # Extract based on archive type
    if [[ "$archive_ext" == "tar.gz" ]]; then
        tar -xzf "$archive_name"
    else
        unzip -q "$archive_name"
    fi
    
    # Create install directory
    mkdir -p "$INSTALL_DIR"
    
    # Move binary to install directory
    local binary_path="${BINARY_NAME}${binary_ext}"
    if [ ! -f "$binary_path" ]; then
        echo -e "${RED}Error: Binary not found in archive${NC}" >&2
        rm -rf "$temp_dir"
        exit 1
    fi
    
    echo -e "${YELLOW}Installing to $INSTALL_DIR...${NC}"
    cp "$binary_path" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    
    # Cleanup
    rm -rf "$temp_dir"
    
    echo -e "${GREEN}✅ Clean Language Manager installed successfully!${NC}"
    echo
    
    # Check if install directory is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo -e "${YELLOW}⚠️  Installation directory is not in your PATH${NC}"
        echo
        echo "Add the following line to your shell configuration file:"
        echo -e "  ${BLUE}export PATH=\"$INSTALL_DIR:\$PATH\"${NC}"
        echo
        echo "Shell configuration files:"
        echo "  - Bash: ~/.bashrc or ~/.bash_profile"
        echo "  - Zsh: ~/.zshrc"
        echo "  - Fish: ~/.config/fish/config.fish"
        echo
        echo "Then restart your terminal or run:"
        echo -e "  ${BLUE}source ~/.bashrc${NC}  # or your shell config file"
        echo
        echo "Alternatively, run cleanmanager directly:"
        echo -e "  ${BLUE}$INSTALL_DIR/cleanmanager --help${NC}"
    else
        echo -e "${GREEN}✅ Installation directory is already in your PATH${NC}"
        echo
        echo "You can now run:"
        echo -e "  ${BLUE}cleanmanager --help${NC}"
    fi
    
    echo
    echo -e "${BLUE}Next steps:${NC}"
    echo "1. Run: cleanmanager init"
    echo "2. Run: cleanmanager doctor"
    echo "3. Install a Clean Language version: cleanmanager install <version>"
    echo
    echo "For more information: https://github.com/$REPO"
}

# Main execution
main() {
    # Check dependencies
    for cmd in curl tar; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            echo -e "${RED}Error: $cmd is required but not installed${NC}" >&2
            exit 1
        fi
    done
    
    # Check for unzip on systems that might need it
    if [[ "$(uname -s)" =~ ^(MINGW|CYGWIN|MSYS) ]] && ! command -v unzip >/dev/null 2>&1; then
        echo -e "${RED}Error: unzip is required but not installed${NC}" >&2
        exit 1
    fi
    
    install_cleanmanager
}

# Run the installer
main "$@"