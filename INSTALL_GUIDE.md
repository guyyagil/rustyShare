# RustyShare Installation Guide

This guide covers the installation of RustyShare using the provided installer script.

## Quick Installation

```bash
curl -sSL https://raw.githubusercontent.com/guyyagil/rustyShare/main/scripts/installer.sh | bash
```

## Safe Installation (Inspect First)

If you prefer to inspect the script before running:

```bash
# Download the installer
wget https://raw.githubusercontent.com/guyyagil/rustyShare/main/scripts/installer.sh

# Make it executable
chmod +x installer.sh

# Inspect the script (optional)
cat installer.sh

# Run the installer
./installer.sh
```

## What the Installer Does

1. **Checks Environment**: Verifies you're not running as root
2. **Installs Dependencies**: Updates package list and installs required system packages
3. **Installs Rust**: Downloads and installs Rust if not already present
4. **Downloads Source**: Clones RustyShare from GitHub to a temporary location
5. **Builds Project**: Compiles RustyShare in release mode
6. **System Installation**: Copies files to `/opt/rustyshare`
7. **Configuration**: Prompts for your preferences:
   - Files directory (where uploads are stored)
   - Server port (default: 3000)
   - Optional password protection
   - Log level
8. **Service Setup**: Creates systemd service for auto-start
9. **Service Start**: Enables and starts the RustyShare service

## Configuration Options

During installation, you'll be prompted for:

- **Files Directory**: Where shared files will be stored (default: `/var/lib/rustyshare`)
- **Port**: Server port (default: `3000`)
- **Password**: Optional password for file access (leave empty for none)
- **Log Level**: Logging verbosity (default: `info`)

## Post-Installation

After successful installation:

- **Access**: Open `http://localhost:3000` in your browser
- **Service Control**: Use `sudo systemctl [start|stop|restart|status] rustyshare`
- **Logs**: View with `sudo journalctl -u rustyshare -f`
- **Configuration**: Edit `/etc/rustyshare.env` and restart service

## Troubleshooting

### Common Issues

1. **Permission Errors**: Don't run the installer with `sudo` - it will prompt when needed
2. **Rust Installation Failed**: Ensure you have internet access and try again
3. **Build Failed**: Check that all dependencies are installed
4. **Service Won't Start**: Check logs with `sudo journalctl -u rustyshare`

### Getting Help

1. Check service status: `sudo systemctl status rustyshare`
2. View logs: `sudo journalctl -u rustyshare -f`
3. Test manually: `cd /opt/rustyshare && sudo -u nobody ./target/release/rustyShare`

## Uninstallation

To remove RustyShare completely:

```bash
curl -sSL https://raw.githubusercontent.com/guyyagil/rustyShare/main/scripts/uninstaller.sh | sudo bash
```

## Manual Installation

If you prefer manual installation, see the main README.md file for detailed instructions.
