#!/bin/bash
# filepath: uninstall_rustyshare.sh

set -e

echo "=== RustyShare Uninstaller ==="

# Stop and disable the systemd service
if systemctl is-active --quiet rustyshare; then
    echo "Stopping RustyShare service..."
    sudo systemctl stop rustyshare
fi

if systemctl is-enabled --quiet rustyshare; then
    echo "Disabling RustyShare service..."
    sudo systemctl disable rustyshare
fi

# Remove the systemd service file
if [ -f /etc/systemd/system/rustyshare.service ]; then
    echo "Removing systemd service file..."
    sudo rm /etc/systemd/system/rustyshare.service
fi

# Remove the environment file
if [ -f /etc/rustyshare.env ]; then
    echo "Removing environment file..."
    sudo rm /etc/rustyshare.env
fi

# Remove the installation directory
if [ -d /opt/rustyshare ]; then
    echo "Removing installation directory..."
    sudo rm -rf /opt/rustyshare
fi

# Reload systemd to apply changes
sudo systemctl daemon-reload

echo "RustyShare has been completely uninstalled."