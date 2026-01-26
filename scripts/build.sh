#!/usr/bin/env bash
set -euo pipefail

# OpenKeyring CLI Cross-Platform Build Script
# Supports: macOS (Intel/ARM), Windows, Linux

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="ok"
VERSION="${VERSION:-$(grep '^version' keyring-cli/Cargo.toml | head -1 | cut -d'"' -f2)}"
OUTPUT_DIR="$PROJECT_ROOT/target/release"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Print banner
print_banner() {
    cat << "EOF"
 _               _              ___ ___
| |_____   _____| |__          / __/ _ \
| '_ \ \ / / _ \ '_ \ _____  | (_| (_) |
|_.__/\ V /  __/ |_) |_____|  \___\___/
|_|    \_/ \___|_.__/
EOF
    log_info "OpenKeyring CLI v${VERSION} - Cross-Platform Build Script"
}

# Check if command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Detect host platform
detect_host() {
    local os=$(uname -s)
    local arch=$(uname -m)

    case "$os" in
        Darwin)
            HOST_OS="macos"
            if [[ "$arch" == "arm64" ]]; then
                HOST_ARCH="arm64"
            else
                HOST_ARCH="x86_64"
            fi
            ;;
        Linux)
            HOST_OS="linux"
            HOST_ARCH="$arch"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            HOST_OS="windows"
            HOST_ARCH="$arch"
            ;;
        *)
            log_error "Unsupported host OS: $os"
            exit 1
            ;;
    esac

    log_info "Detected host: ${HOST_OS}-${HOST_ARCH}"
}

# Install Rust target
install_target() {
    local target="$1"

    if ! rustup target list --installed | grep -q "$target"; then
        log_info "Installing Rust target: $target"
        rustup target add "$target"
    else
        log_info "Target already installed: $target"
    fi
}

# Build for macOS Intel
build_macos_intel() {
    log_info "Building for macOS Intel (x86_64)..."

    install_target "x86_64-apple-darwin"

    cd keyring-cli
    cargo build --target x86_64-apple-darwin --release
    cd ..

    local bin_dir="$OUTPUT_DIR/x86_64-apple-darwin/release"
    mkdir -p "$bin_dir"

    # Strip binary
    if command_exists strip; then
        strip -x "$bin_dir/$PROJECT_NAME" 2>/dev/null || true
    fi

    log_success "macOS Intel binary: $bin_dir/$PROJECT_NAME"
}

# Build for macOS ARM (M-series)
build_macos_arm() {
    log_info "Building for macOS ARM (aarch64)..."

    install_target "aarch64-apple-darwin"

    cd keyring-cli
    cargo build --target aarch64-apple-darwin --release
    cd ..

    local bin_dir="$OUTPUT_DIR/aarch64-apple-darwin/release"
    mkdir -p "$bin_dir"

    # Strip binary
    if command_exists strip; then
        strip -x "$bin_dir/$PROJECT_NAME" 2>/dev/null || true
    fi

    log_success "macOS ARM binary: $bin_dir/$PROJECT_NAME"
}

# Build universal macOS binary
build_macos_universal() {
    log_info "Creating universal macOS binary..."

    local intel_bin="$OUTPUT_DIR/x86_64-apple-darwin/release/$PROJECT_NAME"
    local arm_bin="$OUTPUT_DIR/aarch64-apple-darwin/release/$PROJECT_NAME"
    local universal_dir="$OUTPUT_DIR/universal-apple-darwin/release"
    local universal_bin="$universal_dir/$PROJECT_NAME"

    if [[ ! -f "$intel_bin" ]]; then
        log_error "Intel binary not found. Run build_macos_intel first."
        exit 1
    fi

    if [[ ! -f "$arm_bin" ]]; then
        log_error "ARM binary not found. Run build_macos_arm first."
        exit 1
    fi

    mkdir -p "$universal_dir"

    if ! command_exists lipo; then
        log_error "lipo command not found. This script must run on macOS."
        exit 1
    fi

    lipo -create "$intel_bin" "$arm_bin" -output "$universal_bin"
    strip -x "$universal_bin" 2>/dev/null || true

    log_success "Universal macOS binary: $universal_bin"
}

# Build for Linux
build_linux() {
    local arch="${1:-x86_64}"

    log_info "Building for Linux (${arch})..."

    install_target "${arch}-unknown-linux-gnu"

    cd keyring-cli
    cargo build --target "${arch}-unknown-linux-gnu" --release
    cd ..

    local bin_dir="$OUTPUT_DIR/${arch}-unknown-linux-gnu/release"
    mkdir -p "$bin_dir"

    # Strip binary
    if command_exists strip; then
        strip "$bin_dir/$PROJECT_NAME" 2>/dev/null || true
    fi

    log_success "Linux (${arch}) binary: $bin_dir/$PROJECT_NAME"
}

# Build for Windows
build_windows() {
    local arch="${1:-x86_64}"

    log_info "Building for Windows (${arch})..."

    install_target "${arch}-pc-windows-msvc"

    cd keyring-cli
    cargo build --target "${arch}-pc-windows-msvc" --release
    cd ..

    local bin_dir="$OUTPUT_DIR/${arch}-pc-windows-msvc/release"
    local exe_name="${PROJECT_NAME}.exe"
    mkdir -p "$bin_dir"

    log_success "Windows (${arch}) binary: $bin_dir/$exe_name"
}

# Package release archives
package_release() {
    log_info "Creating release archives..."

    local pkg_dir="$PROJECT_ROOT/target/packages"
    rm -rf "$pkg_dir"
    mkdir -p "$pkg_dir"

    # macOS Universal
    if [[ -f "$OUTPUT_DIR/universal-apple-darwin/release/$PROJECT_NAME" ]]; then
        local archive="$pkg_dir/${PROJECT_NAME}-${VERSION}-macos-universal.tar.gz"
        cd "$OUTPUT_DIR/universal-apple-darwin/release"
        tar czf "$archive" "$PROJECT_NAME"
        log_success "Created: $archive"
    fi

    # macOS Intel
    if [[ -f "$OUTPUT_DIR/x86_64-apple-darwin/release/$PROJECT_NAME" ]]; then
        local archive="$pkg_dir/${PROJECT_NAME}-${VERSION}-macos-x86_64.tar.gz"
        cd "$OUTPUT_DIR/x86_64-apple-darwin/release"
        tar czf "$archive" "$PROJECT_NAME"
        log_success "Created: $archive"
    fi

    # macOS ARM
    if [[ -f "$OUTPUT_DIR/aarch64-apple-darwin/release/$PROJECT_NAME" ]]; then
        local archive="$pkg_dir/${PROJECT_NAME}-${VERSION}-macos-aarch64.tar.gz"
        cd "$OUTPUT_DIR/aarch64-apple-darwin/release"
        tar czf "$archive" "$PROJECT_NAME"
        log_success "Created: $archive"
    fi

    # Linux x86_64
    if [[ -f "$OUTPUT_DIR/x86_64-unknown-linux-gnu/release/$PROJECT_NAME" ]]; then
        local archive="$pkg_dir/${PROJECT_NAME}-${VERSION}-linux-x86_64.tar.gz"
        cd "$OUTPUT_DIR/x86_64-unknown-linux-gnu/release"
        tar czf "$archive" "$PROJECT_NAME"
        log_success "Created: $archive"
    fi

    # Linux ARM64
    if [[ -f "$OUTPUT_DIR/aarch64-unknown-linux-gnu/release/$PROJECT_NAME" ]]; then
        local archive="$pkg_dir/${PROJECT_NAME}-${VERSION}-linux-aarch64.tar.gz"
        cd "$OUTPUT_DIR/aarch64-unknown-linux-gnu/release"
        tar czf "$archive" "$PROJECT_NAME"
        log_success "Created: $archive"
    fi

    # Windows x86_64
    if [[ -f "$OUTPUT_DIR/x86_64-pc-windows-msvc/release/$PROJECT_NAME.exe" ]]; then
        local archive="$pkg_dir/${PROJECT_NAME}-${VERSION}-windows-x86_64.zip"
        cd "$OUTPUT_DIR/x86_64-pc-windows-msvc/release"
        if command_exists zip; then
            zip "$archive" "$PROJECT_NAME.exe" -q
            log_success "Created: $archive"
        else
            log_warn "zip not found, skipping Windows archive"
        fi
    fi

    # Windows ARM64
    if [[ -f "$OUTPUT_DIR/aarch64-pc-windows-msvc/release/$PROJECT_NAME.exe" ]]; then
        local archive="$pkg_dir/${PROJECT_NAME}-${VERSION}-windows-aarch64.zip"
        cd "$OUTPUT_DIR/aarch64-pc-windows-msvc/release"
        if command_exists zip; then
            zip "$archive" "$PROJECT_NAME.exe" -q
            log_success "Created: $archive"
        else
            log_warn "zip not found, skipping Windows ARM64 archive"
        fi
    fi

    cd "$PROJECT_ROOT"
}

# Show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS] [TARGETS]

OPTIONS:
    -h, --help          Show this help message
    -p, --package       Create release archives after building
    -v, --version       Show version information

TARGETS:
    macos               Build all macOS variants (Intel + ARM + Universal)
    macos-intel         Build for macOS Intel only
    macos-arm           Build for macOS ARM (M-series) only
    macos-universal     Build universal macOS binary (requires both variants)
    linux               Build for Linux x86_64
    linux-arm64         Build for Linux ARM64
    windows             Build for Windows x86_64
    windows-arm64       Build for Windows ARM64 (Windows on ARM)
    all                 Build for all platforms (default)

EXAMPLES:
    $0 macos                    # Build all macOS variants
    $0 linux windows            # Build Linux and Windows
    $0 -p all                   # Build all platforms and create archives
    VERSION=1.0.0 $0 -p macos   # Build with custom version

CROSS-COMPILATION NOTES:
    macOS builds: Require Xcode toolchain
    Linux builds: Can be cross-compiled from macOS
    Windows builds: Require MSVC toolchain on Windows

For cross-compilation, you may need additional tools:
    - macOS -> Linux: rustup target add x86_64-unknown-linux-gnu
    - macOS -> Windows: Not supported, build on Windows directly

EOF
}

# Main build function
main() {
    local do_package=false
    local targets=()

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -p|--package)
                do_package=true
                shift
                ;;
            -v|--version)
                echo "OpenKeyring CLI v${VERSION}"
                exit 0
                ;;
            macos|macos-intel|macos-arm|macos-universal|linux|linux-arm64|windows|windows-arm64|all)
                targets+=("$1")
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    # Default to all targets if none specified
    if [[ ${#targets[@]} -eq 0 ]]; then
        targets=("all")
    fi

    print_banner
    detect_host

    # Build for each target
    for target in "${targets[@]}"; do
        case $target in
            all)
                log_info "Building for all platforms..."
                build_macos_intel
                build_macos_arm
                build_macos_universal
                build_linux
                build_linux "aarch64"
                build_windows
                build_windows "aarch64"
                ;;
            macos)
                build_macos_intel
                build_macos_arm
                build_macos_universal
                ;;
            macos-intel)
                build_macos_intel
                ;;
            macos-arm)
                build_macos_arm
                ;;
            macos-universal)
                build_macos_universal
                ;;
            linux)
                build_linux
                ;;
            linux-arm64)
                build_linux "aarch64"
                ;;
            windows)
                build_windows
                ;;
            windows-arm64)
                build_windows "aarch64"
                ;;
        esac
    done

    # Package if requested
    if [[ "$do_package" == true ]]; then
        package_release
    fi

    log_success "Build complete!"
    log_info "Binaries location: $OUTPUT_DIR"
}

# Run main
main "$@"
