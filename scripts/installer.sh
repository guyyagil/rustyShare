#!/bin/bash
# RustyShare Simple Installer - Improved Version
# Downloads, builds, and installs RustyShare from GitHub

set -e

# Configuration
REPO_URL="https://github.com/GuyYagil/rustyShare"
INSTALL_DIR="/opt/rustyshare"
BIN_NAME="rustyShare"
ENV_FILE="/etc/rustyshare.env"
SERVICE_FILE="/etc/systemd/system/rustyshare.service"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    log_error "This script should not be run as root for the build process."
    log_error "Please run as a regular user. The script will prompt for sudo when needed."
    exit 1
fi

echo "==============================================="
echo "          RustyShare Simple Installer"
echo "==============================================="
echo ""

# Get the original user
ORIGINAL_USER="${SUDO_USER:-$USER}"
ORIGINAL_HOME=$(eval echo ~$ORIGINAL_USER)

log_info "Installing for user: $ORIGINAL_USER"
log_info "Home directory: $ORIGINAL_HOME"

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 1. Install system dependencies
log_info "Installing system dependencies..."
if ! sudo apt-get update; then
    log_error "Failed to update package list"
    exit 1
fi

if ! sudo apt-get install -y curl build-essential pkg-config libssl-dev git; then
    log_error "Failed to install system dependencies"
    exit 1
fi
log_success "System dependencies installed"

# 2. Install Rust if needed
if ! command_exists cargo; then
    log_info "Rust not found. Installing Rust..."
    if ! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
        log_error "Failed to install Rust"
        exit 1
    fi
    
    # Source Rust environment
    if [ -f "$ORIGINAL_HOME/.cargo/env" ]; then
        source "$ORIGINAL_HOME/.cargo/env"
    else
        log_error "Rust installation incomplete - env file not found"
        exit 1
    fi
    
    if ! rustup default stable; then
        log_error "Failed to set stable Rust toolchain"
        exit 1
    fi
    log_success "Rust installed successfully"
else
    log_info "Rust is already installed"
    # Make sure we have the environment loaded
    if [ -f "$ORIGINAL_HOME/.cargo/env" ]; then
        source "$ORIGINAL_HOME/.cargo/env"
    fi
fi

# Verify Rust installation
if ! command_exists cargo; then
    log_error "Cargo is not available even after installation"
    exit 1
fi

RUST_VERSION=$(rustc --version)
log_info "Using Rust: $RUST_VERSION"

# 3. Clone the repo to a temporary location
TEMP_DIR=$(mktemp -d)
log_info "Using temporary directory: $TEMP_DIR"
log_info "Cloning RustyShare from $REPO_URL..."

if ! git clone "$REPO_URL" "$TEMP_DIR/rustyshare"; then
    log_error "Failed to clone repository"
    rm -rf "$TEMP_DIR"
    exit 1
fi
log_success "Repository cloned successfully"

# 4. Build the project
log_info "Building RustyShare..."
cd "$TEMP_DIR/rustyshare"

if ! cargo build --release; then
    log_error "Build failed"
    rm -rf "$TEMP_DIR"
    exit 1
fi

BIN_PATH="$TEMP_DIR/rustyshare/target/release/$BIN_NAME"
if [ ! -f "$BIN_PATH" ]; then
    log_error "Build completed but binary not found at: $BIN_PATH"
    rm -rf "$TEMP_DIR"
    exit 1
fi
log_success "Build completed successfully"

# 5. Install to system location
log_info "Installing to system location..."
if [ -d "$INSTALL_DIR" ]; then
    log_warning "Removing previous installation at $INSTALL_DIR"
    sudo rm -rf "$INSTALL_DIR"
fi

sudo mkdir -p "$INSTALL_DIR"
sudo cp -r "$TEMP_DIR/rustyshare"/* "$INSTALL_DIR/"
sudo chown -R root:root "$INSTALL_DIR"
log_success "Installed to $INSTALL_DIR"

# Clean up temp directory
rm -rf "$TEMP_DIR"

# 6. Configuration prompts
echo ""
echo "==============================================="
echo "              Configuration"
echo "==============================================="

# Prompt with defaults matching config.rs
echo ""
log_info "Please configure your RustyShare installation:"
echo ""

read -p "Files directory (where uploads will be stored) [master]: " FILE_DIR
FILE_DIR=${FILE_DIR:-master}

read -p "Server port [3000]: " PORT  
PORT=${PORT:-3000}

echo -n "Password for file access (leave empty for no password): "
read -s PASSWORD
echo ""

read -p "Log level [info]: " RUST_LOG
RUST_LOG=${RUST_LOG:-info}

# Convert relative FILE_DIR to absolute path if needed
if [[ "$FILE_DIR" = "master" ]]; then
    # Use the default master directory in the install location
    FILE_DIR="$INSTALL_DIR/master"
    log_info "Using default files directory: $FILE_DIR"
elif [[ ! "$FILE_DIR" = /* ]]; then
    # If not absolute path, make it relative to install dir
    FILE_DIR="$INSTALL_DIR/$FILE_DIR"
    log_info "Converted to absolute path: $FILE_DIR"
fi

# 7. Create files directory
if [ ! -d "$FILE_DIR" ]; then
    log_info "Creating files directory: $FILE_DIR"
    sudo mkdir -p "$FILE_DIR"
    sudo chown -R nobody:nogroup "$FILE_DIR"
    sudo chmod 755 "$FILE_DIR"
else
    log_info "Files directory already exists: $FILE_DIR"
    sudo chown -R nobody:nogroup "$FILE_DIR"
fi

# 8. Write environment file
log_info "Creating environment configuration..."
sudo bash -c "cat > $ENV_FILE" <<EOF
FILE_DIR=$FILE_DIR
PORT=$PORT
PASSWORD=$PASSWORD
RUST_LOG=$RUST_LOG
EOF

sudo chmod 644 "$ENV_FILE"
log_success "Environment configuration saved to $ENV_FILE"

# 9. Create systemd service
log_info "Creating systemd service..."
sudo bash -c "cat > $SERVICE_FILE" <<EOF
[Unit]
Description=RustyShare File Sharing Server
After=network.target

[Service]
Type=simple
EnvironmentFile=$ENV_FILE
ExecStart=$INSTALL_DIR/target/release/$BIN_NAME
Restart=always
RestartSec=5
User=nobody
Group=nogroup
WorkingDirectory=$INSTALL_DIR

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$FILE_DIR

[Install]
WantedBy=multi-user.target
EOF

sudo chmod 644 "$SERVICE_FILE"
log_success "Systemd service created at $SERVICE_FILE"

# 10. Enable and start the service
log_info "Starting RustyShare service..."
sudo systemctl daemon-reload
sudo systemctl enable rustyshare

if sudo systemctl start rustyshare; then
    log_success "Service started successfully"
else
    log_error "Failed to start service"
    log_info "Check service status with: sudo systemctl status rustyshare"
    log_info "Check logs with: sudo journalctl -u rustyshare"
    exit 1
fi

# 11. Verify service is running
sleep 2
if sudo systemctl is-active --quiet rustyshare; then
    log_success "Service is running"
else
    log_warning "Service may not be running properly"
    log_info "Check status with: sudo systemctl status rustyshare"
fi

echo ""
echo "==============================================="
echo "           Installation Complete!"
echo "==============================================="
echo ""
log_success "RustyShare has been installed and started successfully!"
echo ""
echo "Configuration:"
echo "  Files directory: $FILE_DIR"
echo "  Server port: $PORT"
echo "  Log level: $RUST_LOG"
if [ -n "$PASSWORD" ]; then
    echo "  Password: *** (configured)"
else
    echo "  Password: (none)"
fi
echo ""
echo "Useful commands:"
echo "  Check status:  sudo systemctl status rustyshare"
echo "  View logs:     sudo journalctl -u rustyshare -f"
echo "  Stop service:  sudo systemctl stop rustyshare"
echo "  Start service: sudo systemctl start rustyshare"
echo "  Restart:       sudo systemctl restart rustyshare"
echo ""
echo "Access your server at: http://localhost:$PORT"
echo ""
