# RustyShare Scripts

This directory contains the installation and management scripts for RustyShare.

## Scripts Overview

### Installation Scripts

#### `installer.sh`
The complete installer with enhanced error handling, better user experience, and comprehensive logging.

**Features:**
- Colored output and progress indicators
- Robust error handling and validation
- Security hardening for systemd service
- Better default configurations
- Comprehensive user guidance

**Usage:**
```bash
# Download and run directly
curl -sSL https://raw.githubusercontent.com/guyyagil/rustyShare/main/scripts/installer.sh | bash

# Or download, inspect, and run
wget https://raw.githubusercontent.com/guyyagil/rustyShare/main/scripts/installer.sh
chmod +x installer.sh
./installer.sh
```

### Management Scripts

#### `uninstaller.sh`
Completely removes RustyShare from the system, including:
- Stopping and disabling the systemd service
- Removing service files
- Removing installation directory
- Removing configuration files

**Usage:**
```bash
# Download and run directly
curl -sSL https://raw.githubusercontent.com/guyyagil/rustyShare/main/scripts/uninstaller.sh | sudo bash

# Or if you have the script locally
sudo ./scripts/uninstaller.sh
```

## Installation Process

The installer performs the following steps:

1. **Environment Check**: Verifies you're not running as root
2. **Dependencies**: Installs required system packages
3. **Rust Installation**: Downloads and installs Rust if needed
4. **Source Download**: Clones RustyShare from GitHub
5. **Build**: Compiles the project in release mode
6. **Installation**: Copies files to `/opt/rustyshare`
7. **Configuration**: Prompts for user preferences
8. **Service Setup**: Creates and starts systemd service

## Configuration

During installation, you'll be prompted for:

- **Files Directory**: Where shared files will be stored (default: `/var/lib/rustyshare`)
- **Port**: Server port (default: `3000`)
- **Password**: Optional password for file access (leave empty for none)
- **Log Level**: Logging verbosity (default: `info`)

## Post-Installation

After installation:

- **Access**: `http://localhost:3000` or `http://<server-ip>:3000`
- **Service Management**: `sudo systemctl [start|stop|restart|status] rustyshare`
- **Logs**: `sudo journalctl -u rustyshare -f`
- **Configuration**: Edit `/etc/rustyshare.env` and restart service

## Important Notes

- **Run as regular user**: Don't use `sudo` with the installer (it will prompt when needed)
- **Internet required**: Script downloads Rust and dependencies
- **Linux only**: Script is designed for Ubuntu/Debian systems
- **Clean installation**: Remove any existing installation before running installer

## Troubleshooting

If installation fails:

1. Check internet connectivity
2. Ensure you're running as regular user (not root)
3. Check system logs: `sudo journalctl -u rustyshare`
4. Try manual installation (see main README.md)

For more detailed installation instructions, see `INSTALL_GUIDE.md` in the project root.
