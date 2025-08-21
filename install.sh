#!/bin/bash

# FuzzyTail Installation Script
# Usage: curl -sSL https://raw.githubusercontent.com/your-username/fuzzytail/main/install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print functions
print_header() {
    echo -e "${CYAN}"
    echo "🚀 FuzzyTail Installation Script"
    echo "================================="
    echo -e "${NC}"
}

print_step() {
    echo -e "${BLUE}📋 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${PURPLE}💡 $1${NC}"
}

# Configuration
REPO_URL="https://github.com/yodabytz/fuzzytail"
INSTALL_DIR="/usr/local/bin"
THEMES_DIR="/etc/fuzzytail/themes"
CONFIG_DIR="/etc/fuzzytail"
USER_CONFIG_DIR="$HOME/.config/fuzzytail"

# Detect OS
OS=""
ARCH=""
case "$(uname -s)" in
    Linux*)     OS=linux;;
    Darwin*)    OS=macos;;
    CYGWIN*)    OS=windows;;
    MINGW*)     OS=windows;;
    *)          OS="unknown";;
esac

case "$(uname -m)" in
    x86_64*)    ARCH=x86_64;;
    arm64*)     ARCH=arm64;;
    aarch64*)   ARCH=arm64;;
    armv7*)     ARCH=armv7;;
    *)          ARCH="unknown";;
esac

print_header

print_step "Detecting system information..."
echo "   OS: $OS"
echo "   Architecture: $ARCH"
echo "   Install directory: $INSTALL_DIR"
echo "   Themes directory: $THEMES_DIR"
echo ""

# Check if running as root for system-wide installation
if [[ $EUID -eq 0 ]]; then
    print_warning "Running as root. Installing system-wide."
    INSTALL_SYSTEM=true
else
    print_info "Running as user. You may need sudo privileges for system installation."
    INSTALL_SYSTEM=false
fi

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check for dependencies
print_step "Checking dependencies..."

# Check for curl or wget
if ! command_exists curl && ! command_exists wget; then
    print_error "Neither curl nor wget found. Please install one of them."
    exit 1
fi

# Check for git
if ! command_exists git; then
    print_error "Git not found. Please install git."
    exit 1
fi

print_success "Dependencies check passed"

# Install Rust if not present
print_step "Checking for Rust installation..."
if ! command_exists cargo; then
    print_warning "Rust not found. Installing Rust..."
    
    if command_exists curl; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    else
        wget -qO- https://sh.rustup.rs | sh -s -- -y
    fi
    
    # Source the cargo environment
    source "$HOME/.cargo/env"
    
    if command_exists cargo; then
        print_success "Rust installed successfully"
    else
        print_error "Failed to install Rust"
        exit 1
    fi
else
    print_success "Rust found: $(rustc --version)"
fi

# Create temporary directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

print_step "Downloading FuzzyTail source code..."
if command_exists curl; then
    curl -sSL "$REPO_URL/archive/refs/heads/main.tar.gz" | tar -xz
else
    wget -qO- "$REPO_URL/archive/refs/heads/main.tar.gz" | tar -xz
fi

cd fuzzytail-main
print_success "Source code downloaded"

# Build FuzzyTail
print_step "Building FuzzyTail (this may take a few minutes)..."
if source "$HOME/.cargo/env" && cargo build --release; then
    print_success "FuzzyTail built successfully"
else
    print_error "Failed to build FuzzyTail"
    exit 1
fi

# Install binary
print_step "Installing FuzzyTail binary..."
if [[ $INSTALL_SYSTEM == true ]] || sudo -n true 2>/dev/null; then
    # System-wide installation
    if sudo cp target/release/ft "$INSTALL_DIR/ft"; then
        print_success "Binary installed to $INSTALL_DIR/ft"
    else
        print_error "Failed to install binary to $INSTALL_DIR"
        exit 1
    fi
else
    # User installation
    LOCAL_BIN="$HOME/.local/bin"
    mkdir -p "$LOCAL_BIN"
    if cp target/release/ft "$LOCAL_BIN/ft"; then
        print_success "Binary installed to $LOCAL_BIN/ft"
        print_info "Make sure $LOCAL_BIN is in your PATH"
        
        # Check if it's in PATH
        if ! echo "$PATH" | grep -q "$LOCAL_BIN"; then
            print_warning "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
            echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
        fi
    else
        print_error "Failed to install binary to $LOCAL_BIN"
        exit 1
    fi
fi

# Install themes
print_step "Installing themes..."
if [[ $INSTALL_SYSTEM == true ]] || sudo -n true 2>/dev/null; then
    # System-wide themes installation
    if sudo mkdir -p "$THEMES_DIR" && sudo cp themes/ft.conf.* "$THEMES_DIR/"; then
        print_success "Themes installed to $THEMES_DIR"
    else
        print_warning "Failed to install themes system-wide, trying user installation..."
        mkdir -p "$USER_CONFIG_DIR/themes"
        cp themes/ft.conf.* "$USER_CONFIG_DIR/themes/"
        print_success "Themes installed to $USER_CONFIG_DIR/themes"
    fi
else
    # User themes installation
    mkdir -p "$USER_CONFIG_DIR/themes"
    cp themes/ft.conf.* "$USER_CONFIG_DIR/themes/"
    print_success "Themes installed to $USER_CONFIG_DIR/themes"
fi

# Create default configuration
print_step "Creating default configuration..."
mkdir -p "$USER_CONFIG_DIR"

if [[ ! -f "$USER_CONFIG_DIR/config.toml" ]]; then
    cat > "$USER_CONFIG_DIR/config.toml" << 'EOF'
[general]
theme = "catppuccin"
buffer_size = 65536
follow_retry_interval = 1000

[themes]
builtin_path = "/etc/fuzzytail/themes"
user_path = "~/.config/fuzzytail/themes"
EOF
    print_success "Default configuration created at $USER_CONFIG_DIR/config.toml"
else
    print_info "Configuration file already exists at $USER_CONFIG_DIR/config.toml"
fi

# Cleanup
cd /
rm -rf "$TEMP_DIR"

# Verify installation
print_step "Verifying installation..."
if command_exists ft; then
    print_success "FuzzyTail installed successfully!"
    echo ""
    print_info "Version: $(ft --help | head -1)"
    echo ""
    print_info "Try these commands:"
    echo "   ft --help                    # Show help"
    echo "   echo 'Hello World' | ft      # Test with sample input"
    echo "   ft /var/log/syslog          # View system logs (if accessible)"
    echo "   ft -f /var/log/auth.log     # Follow authentication logs"
    echo ""
    print_info "Available themes:"
    for theme in catppuccin dracula tokyo-night rose-pine lackluster miasma; do
        echo "   • $theme"
    done
    echo ""
    print_info "Change theme by editing: $USER_CONFIG_DIR/config.toml"
    echo ""
    print_success "🎉 Installation complete! Enjoy your beautiful logs!"
else
    print_error "Installation verification failed. ft command not found."
    print_info "You may need to:"
    echo "   • Restart your terminal"
    echo "   • Check your PATH environment variable"
    echo "   • Run: source ~/.bashrc (or ~/.zshrc)"
    exit 1
fi

# Check for common log files and suggest usage
print_step "Checking for common log files..."
common_logs=(
    "/var/log/syslog"
    "/var/log/messages"
    "/var/log/auth.log"
    "/var/log/kern.log"
)

accessible_logs=()
for log in "${common_logs[@]}"; do
    if [[ -r "$log" ]]; then
        accessible_logs+=("$log")
    fi
done

if [[ ${#accessible_logs[@]} -gt 0 ]]; then
    echo ""
    print_info "Accessible log files found:"
    for log in "${accessible_logs[@]}"; do
        echo "   ft $log"
    done
fi

echo ""
echo -e "${CYAN}🚀 Happy log viewing with FuzzyTail! 🚀${NC}"