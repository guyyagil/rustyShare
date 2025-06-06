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

## Installation

```bash
# Clone the repository
git clone https://github.com/guyyagil/rustyShare.git

# Navigate to the project directory
cd rustyShare

# Build the project
cargo build --release
```

---

## Usage

### Quick Start

```bash
# Run the server directly
cargo run
```

### Running as a Service (Optional)

> ℹ️ System service setup is currently supported only on Linux.

If you want RustyShare to start automatically when your system boots:

1. **Create systemd service file:**
   Create `/etc/systemd/system/rustyshare.service` with:

   ```ini
   [Unit]
   Description=RustyShare File Sharing Server
   After=network.target

   [Service]
   Type=simple
   User=YOUR_USERNAME
   WorkingDirectory=/absolute/path/to/rustyShare
   ExecStart=/absolute/path/to/rustyShare/target/release/rustyshare
   Restart=always
   RestartSec=3

   [Install]
   WantedBy=multi-user.target
   ```

   Use `pwd` inside your project directory to find the correct absolute path.

2. **Enable and start the service:**

   ```bash
   sudo systemctl enable rustyshare
   sudo systemctl start rustyshare
   ```

3. **Check service status:**

   ```bash
   systemctl status rustyshare
   ```

4. **View logs:**

   ```bash
   journalctl -u rustyshare -f
   ```

### Accessing the Server

1. **On the same machine:**

   * Open your browser and go to [http://localhost:3000](http://localhost:3000)

2. **From another device on your LAN:**

   * Open your browser and go to `http://<your-server-ip>:3000`
   * Replace `<your-server-ip>` with your computer's IP address
   * To find your IP address on Linux, run:

     ```bash
     ip a | grep inet
     ```

   * **Recommendation:** For easier and more reliable access from other devices, consider setting a static IP address for your server machine on your router or network settings. This way, you can always access RustyShare using the same address (e.g., `http://192.168.1.100:3000`) without needing to check for changes.

### Managing Files

1. **Add files:**

   * Place your files in the media directory or upload them via the web interface
   * Files are automatically detected and available in the web interface

2. **Real-time Updates:**

   * New files appear automatically
   * Removed files are removed from the listing
   * Modified files are updated in real-time
   * No manual refresh required

3. **Search:**

   * Use the search bar at the top of the interface to quickly find files and folders by name

---

## Configuration

You can customize the server using environment variables:

* `FILE_DIR` — Path to your media directory (default: `./master`)
* `PORT` — Server port (default: `3000`)
* `PASSWORD` — Access password for the web interface (default: `changeme`)
* `RUST_LOG` — Logging level (default: `info`)

### Examples

1. **Running with custom port and password:**

   ```bash
   PORT=8080 PASSWORD=your_secure_password cargo run
   ```

2. **Running with custom media directory:**

   ```bash
   MEDIA_DIR=/path/to/your/media cargo run
   ```

3. **System service with custom configuration:**
   Add to your service file:

   ```ini
   [Service]
   Environment="MEDIA_DIR=/path/to/your/media"
   Environment="PORT=8080"
   Environment="PASSWORD=your_secure_password"
   ```

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
