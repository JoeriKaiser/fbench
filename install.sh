#!/bin/sh
# FBench Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/JoeriKaiser/fbench/main/install.sh | sh

set -e

REPO="JoeriKaiser/fbench"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
detect_target() {
    local os arch
    
    case "$(uname -s)" in
        Linux*)     os=linux;;
        Darwin*)    os=apple;;
        CYGWIN*|MINGW*|MSYS*) os=windows;;
        *)          echo "Unsupported OS: $(uname -s)" >&2; exit 1;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)   arch=x86_64;;
        aarch64|arm64)  arch=aarch64;;
        *)              echo "Unsupported architecture: $(uname -m)" >&2; exit 1;;
    esac
    
    if [ "$os" = "windows" ]; then
        echo "x86_64-pc-windows-msvc"
    elif [ "$os" = "apple" ]; then
        echo "${arch}-apple-darwin"
    else
        # Linux: x86_64 only for now
        if [ "$arch" = "x86_64" ]; then
            echo "x86_64-unknown-linux-gnu"
        else
            echo "Error: ARM Linux builds not yet available" >&2
            echo "Please build from source: cargo build --release" >&2
            exit 1
        fi
    fi
}

# Get latest release version
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"([^"]+)".*/\1/'
}

# Download and install
download_and_install() {
    local target version tmpdir archive binary_name
    
    target=$(detect_target)
    version=$(get_latest_version)
    
    if [ -z "$version" ]; then
        echo "Error: Could not determine latest version" >&2
        exit 1
    fi
    
    echo "Installing fbench ${version} for ${target}..."
    
    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT
    
    if echo "$target" | grep -q "windows"; then
        archive="fbench-${target}.zip"
        binary_name="fbench.exe"
    else
        archive="fbench-${target}.tar.gz"
        binary_name="fbench"
    fi
    
    url="https://github.com/${REPO}/releases/download/${version}/${archive}"
    
    echo "Downloading from ${url}..."
    curl -fsSL "$url" -o "${tmpdir}/${archive}"
    
    echo "Extracting..."
    cd "$tmpdir"
    if echo "$archive" | grep -q "\.zip$"; then
        unzip -q "$archive"
    else
        tar -xzf "$archive"
    fi
    
    echo "Installing to ${INSTALL_DIR}..."
    if [ -w "$INSTALL_DIR" ] || [ "$INSTALL_DIR" != "/usr/local/bin" ]; then
        mv "$binary_name" "$INSTALL_DIR/"
        chmod +x "${INSTALL_DIR}/${binary_name}"
    else
        sudo mv "$binary_name" "$INSTALL_DIR/"
        sudo chmod +x "${INSTALL_DIR}/${binary_name}"
    fi
    
    echo ""
    echo "✓ fbench ${version} installed successfully!"
    echo ""
    
    if command -v fbench >/dev/null 2>&1; then
        echo "Run 'fbench --help' to get started."
    else
        echo "Note: ${INSTALL_DIR} may not be in your PATH."
        echo "Add it to your PATH or run with the full path: ${INSTALL_DIR}/fbench"
    fi
}

# Main
main() {
    # Handle flags
    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                echo "FBench Installer"
                echo ""
                echo "Usage: curl -fsSL https://raw.githubusercontent.com/JoeriKaiser/fbench/main/install.sh | sh"
                echo ""
                echo "Environment variables:"
                echo "  INSTALL_DIR    Installation directory (default: /usr/local/bin)"
                echo ""
                exit 0
                ;;
            *)
                echo "Unknown option: $1" >&2
                echo "Run with --help for usage" >&2
                exit 1
                ;;
        esac
        shift
    done
    
    # Check for required commands
    for cmd in curl tar; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            echo "Error: Required command '$cmd' not found" >&2
            exit 1
        fi
    done
    
    download_and_install
}

main "$@"
