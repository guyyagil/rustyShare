# RustyShare

<div align="center">
  <img src="static/logo/logo-transparent.png" alt="RustyShare Logo" width="300"/>
</div>

![Rust](https://img.shields.io/badge/Rust-Stable-orange?logo=rust)
![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows%20%7C%20macOS-informational)

**RustyShare** is a lightweight, cross-platform local LAN sharing server built with Rust.  
Host a master PC as a server and let any device on your network access, upload, download, update, and stream files directly from your browser—no client app required!

---

## Features

* 📁 **Share any file** over your LAN—upload, download, update, and manage files from any device
* 🎥 **Stream video and audio** files instantly in your browser
* 🌐 **Modern, browser-based interface** (no client install needed)
* 🔍 **Automatic media file discovery**
* 🔎 **Search for files** instantly using the built-in search bar in the web interface
* 💻 **Cross-platform** (Linux, Windows, macOS)
* ⚡ **Efficient and lightweight**
* 🔄 **Real-time media directory monitoring**
* 🚀 **Runs as a system service** (Linux)
* 🔒 **Access your files securely on your local network**
* 🗄️ **Use as external local storage** for your LAN devices—store, access, and manage files from any device on your network just like a shared drive

---

## How It Works

1. **Drop your files** (videos, music, documents, etc.) into the `media/` folder or upload them via the web interface.
2. **Run the server** using `cargo run` or set it up as a system service.
3. **Open a browser** on any device in your LAN and go to `http://<server-ip>:3000`.
4. **Browse, upload, download, and stream** your files instantly!
5. **Changes are reflected automatically**—no need to restart the server.

---

## Requirements

* Rust 1.70 or higher
* A modern web browser (Chrome, Firefox, Edge, Safari, etc.)
* Local Area Network (LAN) access
* Linux system for system service functionality (optional)

---

## 🚀 Quick Start (Recommended)

You can install and run RustyShare with a single command using the provided installer script:

```bash
# Download and run the installer
curl -sSL https://raw.githubusercontent.com/guyyagil/rustyShare/main/scripts/installer.sh | bash
```

Or, download and inspect the script first: 

```bash
# Download the installer
wget https://raw.githubusercontent.com/guyyagil/rustyShare/main/scripts/installer.sh
chmod +x installer.sh
./installer.sh
```

**Important:** Run the installer as a regular user (not with sudo). The script will prompt for sudo when needed.

The installer will:
- Install system dependencies (curl, git, build tools)
- Install Rust if not already installed
- Download and build RustyShare from source
- Prompt you for configuration (shared directory, port, password, etc.)
- Set up RustyShare as a systemd service (auto-starts on boot)
- Start the server automatically

---

## 🛠️ Manual Installation

If you prefer to install manually:

```bash
# Clone the repository
git clone https://github.com/guyyagil/rustyShare.git

# Navigate to the project directory
cd rustyShare

# Build the project
cargo build --release
```

---

## 🧹 Uninstall

To completely remove RustyShare, use the provided uninstaller:

```bash
curl -sSL https://raw.githubusercontent.com/guyyagil/rustyShare/main/scripts/uninstaller.sh | sudo bash
```

Or, if you already have the script:

```bash
# From the rustyShare directory
sudo ./scripts/uninstaller.sh
```

---

## 🔧 Configuration

RustyShare uses environment variables for configuration. These are set during installation but can be modified later:

- **FILE_DIR**: Directory where shared files are stored (default: `/var/lib/rustyshare`)
- **PORT**: Server port (default: `3000`)
- **PASSWORD**: Optional password for file access (default: none)
- **RUST_LOG**: Log level (default: `info`)

Configuration is stored in `/etc/rustyshare.env`. To modify:

```bash
sudo nano /etc/rustyshare.env
sudo systemctl restart rustyshare
```

---

## Usage

- After installation, access RustyShare in your browser at:  
  `http://localhost:3000`  
  or  
  `http://<your-server-ip>:3000` from any device on your LAN.

- To manage the service:
  ```bash
  sudo systemctl status rustyshare
  sudo systemctl restart rustyshare
  sudo systemctl stop rustyshare
  ```

---

## Configuration

The installer will prompt you for:
- **Files directory** (`FILE_DIR`)
- **Server port** (`PORT`)
- **Password** (`PASSWORD`)
- **Log level** (`RUST_LOG`)

You can later edit these in `/etc/rustyshare.env` and restart the service.

---

## Security Features

- **Password Protection:**  
  Access to the web interface is protected by a password, which you can set using the `PASSWORD` environment variable. This prevents unauthorized users on your network from accessing your files.

- **Cookie-based Authentication:**  
  After a successful login, the server issues a secure cookie that keeps you logged in for 12 hours. As long as the cookie is valid, you won't need to re-enter the password.

- **Session Expiry:**  
  The authentication cookie automatically expires after 12 hours, requiring users to log in again for continued access.

---

## Folder Structure

```text
rustyShare/
├── src/                         # Main source code directory
│   ├── main.rs                  # Application entry point
│   ├── server/                  # Server-related code
│   │   ├── mod.rs               # Server module definition
│   │   ├── startup.rs           # Server startup logic
│   │   ├── file_operations/     # File operations module
│   │   │   ├── mod.rs           # File operations module definition
│   │   │   └── streaming.rs     # Media streaming logic
│   │   └── routing/             # Modular routing system
│   │       ├── mod.rs           # Routing module definition
│   │       ├── router.rs        # Main router configuration
│   │       └── handlers/        # Route handlers organized by domain
│   │           ├── mod.rs       # Handlers module definition
│   │           ├── auth.rs      # Authentication & authorization
│   │           ├── file_operations.rs  # File management endpoints
│   │           ├── static_content.rs   # Static content serving
│   │           └── health.rs    # Health check endpoint
│   ├── file_manager/            # File management logic
│   │   ├── mod.rs               # File manager module definition
│   │   ├── file_tree.rs         # File tree operations
│   │   ├── file_utils.rs        # File utility functions
│   │   └── tree_watcher.rs      # Directory watching logic
│   └── utils/                   # Utility functions
│       ├── mod.rs               # Utils module definition
│       └── config.rs            # Configuration handling
├── static/                      # Static assets
│   ├── html/                    # HTML files for the web interface
│   │   ├── home.html            # Home/login page
│   │   └── master.html          # Main interface after login
│   ├── css/                     # Stylesheets
│   └── js/                      # JavaScript files
├── tests/                       # Test files
├── media/                       # Default media directory (user files)
├── config/                      # Configuration files
│   └── systemd/                 # System service configuration
├── README.md                    # Project documentation
└── Cargo.toml                   # Rust project manifest
```
