#!/bin/bash
# Cross-build script for keyring-cli
# Builds release binaries for all supported platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Targets to build
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-pc-windows-msvc"
)

# Build type (debug or release)
BUILD_TYPE="${1:-release}"
OUTPUT_DIR="dist/$BUILD_TYPE"

# Validate build type
if [[ "$BUILD_TYPE" != "debug" && "$BUILD_TYPE" != "release" ]]; then
    echo -e "${RED}Error: BUILD_TYPE must be 'debug' or 'release'${NC}"
    echo "Usage: $0 [debug|release]"
    exit 1
fi

# Create output directory
echo -e "${YELLOW}Creating output directory: $OUTPUT_DIR${NC}"
mkdir -p "$OUTPUT_DIR"

# Check if cross is installed
if ! command -v cross &> /dev/null; then
    echo -e "${RED}Error: 'cross' command not found${NC}"
    echo "Install it with: cargo install cross --git https://github.com/cross-rs/cross"
    exit 1
fi

# Build for each target
for target in "${TARGETS[@]}"; do
    echo -e "${YELLOW}================================${NC}"
    echo -e "${YELLOW}Building for $target${NC}"
    echo -e "${YELLOW}================================${NC}"

    if cross build --target "$target" --"$BUILD_TYPE"; then
        echo -e "${GREEN}✓ Build successful for $target${NC}"

        # Copy binary to output directory with appropriate name
        case "$target" in
            *windows*)
                BINARY_NAME="ok-windows-x64.exe"
                SRC="target/$target/$BUILD_TYPE/ok.exe"
                ;;
            *linux*)
                if [[ "$target" == *"aarch64"* ]]; then
                    BINARY_NAME="ok-linux-arm64"
                else
                    BINARY_NAME="ok-linux-x64"
                fi
                SRC="target/$target/$BUILD_TYPE/ok"
                ;;
            *)
                BINARY_NAME="ok-$target"
                SRC="target/$target/$BUILD_TYPE/ok"
                ;;
        esac

        if [ -f "$SRC" ]; then
            cp "$SRC" "$OUTPUT_DIR/$BINARY_NAME"
            echo -e "${GREEN}  → Copied to $OUTPUT_DIR/$BINARY_NAME${NC}"
        else
            echo -e "${RED}  → Warning: Binary not found at $SRC${NC}"
        fi
    else
        echo -e "${RED}✗ Build failed for $target${NC}"
        exit 1
    fi
done

echo -e "${YELLOW}================================${NC}"
echo -e "${GREEN}All builds complete!${NC}"
echo -e "${YELLOW}================================${NC}"
echo ""
echo "Binaries are available in: $OUTPUT_DIR"
ls -lh "$OUTPUT_DIR"
