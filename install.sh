#!/bin/bash
# Scratchpad v2 Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/Krakaw/scratchpad/main/install.sh | bash
#
# Environment variables:
#   SCRATCHPAD_VERSION - specific version to install (default: latest release)
#   SCRATCHPAD_INSTALL_DIR - installation directory (default: /usr/local/bin or ~/.local/bin)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}ℹ${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1" >&2; }

# Detect OS and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux)  os="linux" ;;
        Darwin) os="macos" ;;
        *)
            error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *)
            error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}

# Determine installation directory
get_install_dir() {
    if [ -n "${SCRATCHPAD_INSTALL_DIR:-}" ]; then
        echo "$SCRATCHPAD_INSTALL_DIR"
        return
    fi

    # Prefer /usr/local/bin if writable, otherwise ~/.local/bin
    if [ -w "/usr/local/bin" ]; then
        echo "/usr/local/bin"
    else
        local local_bin="$HOME/.local/bin"
        mkdir -p "$local_bin"
        echo "$local_bin"
    fi
}

# Get latest release version from GitHub
get_latest_version() {
    local version
    version=$(curl -fsSL "https://api.github.com/repos/Krakaw/scratchpad/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    echo "$version"
}

# Download and install binary
install_binary() {
    local version="$1"
    local platform="$2"
    local install_dir="$3"

    local binary_name="scratchpad"
    local download_url="https://github.com/Krakaw/scratchpad/releases/download/${version}/scratchpad-${platform}"

    info "Downloading scratchpad ${version} for ${platform}..."

    local tmp_file
    tmp_file=$(mktemp)
    trap "rm -f $tmp_file" EXIT

    if ! curl -fsSL "$download_url" -o "$tmp_file"; then
        error "Failed to download binary from $download_url"
        error "This version may not have pre-built binaries for your platform."
        echo ""
        warn "To build from source instead:"
        echo "  cargo install --git https://github.com/Krakaw/scratchpad"
        exit 1
    fi

    chmod +x "$tmp_file"
    mv "$tmp_file" "${install_dir}/${binary_name}"
    trap - EXIT

    success "Installed to ${install_dir}/${binary_name}"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Verify installation
verify_installation() {
    local install_dir="$1"
    local binary="${install_dir}/scratchpad"

    if [ ! -x "$binary" ]; then
        error "Installation failed: binary not found or not executable"
        exit 1
    fi

    local version
    version=$("$binary" --version 2>/dev/null || echo "unknown")
    success "Verified: $version"
}

# Check PATH
check_path() {
    local install_dir="$1"

    if [[ ":$PATH:" != *":${install_dir}:"* ]]; then
        warn "Installation directory is not in your PATH"
        echo ""
        echo "Add to your shell config (~/.bashrc, ~/.zshrc, etc.):"
        echo -e "  ${BOLD}export PATH=\"${install_dir}:\$PATH\"${NC}"
        echo ""
    fi
}

# Check dependencies
check_dependencies() {
    local missing=()

    if ! command_exists docker; then
        missing+=("docker")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        warn "Optional dependencies not found: ${missing[*]}"
        echo "  Scratchpad works best with Docker installed."
        echo ""
    fi
}

# Main
main() {
    echo ""
    echo -e "${BOLD}Scratchpad Installer${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Check for curl
    if ! command_exists curl; then
        error "curl is required but not installed"
        exit 1
    fi

    local platform install_dir version

    platform=$(detect_platform)
    install_dir=$(get_install_dir)
    version="${SCRATCHPAD_VERSION:-$(get_latest_version)}"

    if [ -z "$version" ]; then
        warn "Could not determine latest version, trying 'v2.0.0'"
        version="v2.0.0"
    fi

    info "Platform: $platform"
    info "Install directory: $install_dir"
    info "Version: $version"
    echo ""

    install_binary "$version" "$platform" "$install_dir"
    verify_installation "$install_dir"
    echo ""

    check_path "$install_dir"
    check_dependencies

    echo -e "${GREEN}${BOLD}Installation complete!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Run ${BOLD}scratchpad setup${NC} to configure your first project"
    echo "  2. Run ${BOLD}scratchpad help${NC} to see all commands"
    echo ""
    echo "Documentation: https://github.com/Krakaw/scratchpad"
    echo ""
}

main "$@"
