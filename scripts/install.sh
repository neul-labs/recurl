#!/usr/bin/env bash
set -euo pipefail

# recurl installer for Linux and macOS
# Usage: curl -fsSL https://recurl.dev/install.sh | bash

VERSION="${RECURL_VERSION:-latest}"
INSTALL_DIR="${RECURL_INSTALL_DIR:-}"
GITHUB_REPO="neul-labs/recurl"
BASE_URL="https://github.com/${GITHUB_REPO}/releases"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}[info]${NC} $1"; }
success() { echo -e "${GREEN}[success]${NC} $1"; }
warn() { echo -e "${YELLOW}[warn]${NC} $1"; }
error() { echo -e "${RED}[error]${NC} $1"; exit 1; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "darwin" ;;
        *)       error "Unsupported OS: $(uname -s). Use Windows installer for Windows." ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        arm64|aarch64) echo "aarch64" ;;
        *)             error "Unsupported architecture: $(uname -m)" ;;
    esac
}

# Detect shell config file
detect_shell_config() {
    local shell_name
    shell_name=$(basename "$SHELL")

    case "$shell_name" in
        bash)
            if [[ -f "$HOME/.bashrc" ]]; then
                echo "$HOME/.bashrc"
            elif [[ -f "$HOME/.bash_profile" ]]; then
                echo "$HOME/.bash_profile"
            else
                echo "$HOME/.bashrc"
            fi
            ;;
        zsh)
            echo "$HOME/.zshrc"
            ;;
        fish)
            echo "$HOME/.config/fish/config.fish"
            ;;
        *)
            echo "$HOME/.profile"
            ;;
    esac
}

# Check if command exists
has_command() {
    command -v "$1" &> /dev/null
}

# Download file
download() {
    local url="$1"
    local output="$2"

    if has_command curl; then
        curl -fsSL "$url" -o "$output"
    elif has_command wget; then
        wget -q "$url" -O "$output"
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

# Get latest version from GitHub
get_latest_version() {
    local url="${BASE_URL}/latest"
    if has_command curl; then
        curl -fsSL -o /dev/null -w '%{url_effective}' "$url" | rev | cut -d'/' -f1 | rev
    elif has_command wget; then
        wget -q -O /dev/null --server-response "$url" 2>&1 | grep -oP 'Location: .*/tag/\K[^/\s]+'
    fi
}

# Main installation
main() {
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║         recurl installer               ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
    echo ""

    # Detect platform
    local os arch
    os=$(detect_os)
    arch=$(detect_arch)
    info "Detected platform: ${os}-${arch}"

    # Determine version
    if [[ "$VERSION" == "latest" ]]; then
        info "Fetching latest version..."
        VERSION=$(get_latest_version)
        if [[ -z "$VERSION" ]]; then
            error "Failed to fetch latest version. Set RECURL_VERSION explicitly."
        fi
    fi
    info "Installing version: ${VERSION}"

    # Determine install directory
    if [[ -z "$INSTALL_DIR" ]]; then
        if [[ -w "/usr/local" ]]; then
            INSTALL_DIR="/usr/local/recurl"
        else
            INSTALL_DIR="$HOME/.local/recurl"
        fi
    fi
    info "Install directory: ${INSTALL_DIR}"

    # Create temp directory
    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    # Download archive
    local archive_name="recurl-${os}-${arch}.tar.gz"
    local download_url="${BASE_URL}/download/${VERSION}/${archive_name}"
    info "Downloading ${archive_name}..."
    download "$download_url" "${tmp_dir}/${archive_name}"

    # Extract
    info "Extracting..."
    mkdir -p "$INSTALL_DIR"
    tar -xzf "${tmp_dir}/${archive_name}" -C "$INSTALL_DIR" --strip-components=1

    # Make binaries executable
    chmod +x "$INSTALL_DIR/recurl"
    chmod +x "$INSTALL_DIR/recurld"
    chmod +x "$INSTALL_DIR/bin/"*

    success "recurl installed to ${INSTALL_DIR}"
    echo ""

    # Verify installation
    info "Verifying installation..."
    if "$INSTALL_DIR/recurl" --recurl-debug --version &> /dev/null; then
        success "recurl binary works correctly"
    else
        warn "recurl binary may have issues. Check ${INSTALL_DIR}/recurl"
    fi
    echo ""

    # Ask about shell alias
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}Shell configuration${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "To use recurl as a drop-in curl replacement, you can:"
    echo ""
    echo "  1. Use recurl directly:  recurl https://example.com"
    echo "  2. Create a shell alias: alias curl='${INSTALL_DIR}/recurl'"
    echo ""

    local shell_config
    shell_config=$(detect_shell_config)

    read -p "Add curl alias to ${shell_config}? [y/N] " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Check if alias already exists
        if grep -q "alias curl=.*recurl" "$shell_config" 2>/dev/null; then
            warn "Alias already exists in ${shell_config}"
        else
            echo "" >> "$shell_config"
            echo "# recurl - drop-in curl replacement with anti-bot bypass" >> "$shell_config"
            echo "alias curl='${INSTALL_DIR}/recurl'" >> "$shell_config"
            success "Alias added to ${shell_config}"
        fi

        echo ""
        info "Run this to apply changes now:"
        echo ""
        echo "    source ${shell_config}"
        echo ""
    else
        info "Skipping alias configuration."
        echo ""
        echo "To use recurl, either:"
        echo ""
        echo "  1. Call recurl directly:"
        echo "     ${INSTALL_DIR}/recurl https://example.com"
        echo ""
        echo "  2. Add to your shell config manually:"
        echo "     echo \"alias curl='${INSTALL_DIR}/recurl'\" >> ${shell_config}"
        echo ""
        echo "  3. Add to PATH:"
        echo "     export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo ""
    fi

    # Final summary
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}Installation complete!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "Installed files:"
    echo "  ${INSTALL_DIR}/recurl        - main binary"
    echo "  ${INSTALL_DIR}/recurld       - daemon"
    echo "  ${INSTALL_DIR}/bin/         - curl engines"
    echo ""
    echo "Documentation: https://github.com/${GITHUB_REPO}#readme"
    echo ""
}

# Run main
main "$@"
