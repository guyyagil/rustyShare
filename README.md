# RustyStrem

![Rust](https://img.shields.io/badge/Rust-Stable-orange?logo=rust)
![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows%20%7C%20macOS-informational)

A lightweight, cross-platform media streaming server built with Rust. Stream your local video and audio files to any device on your network using just a web browser—no client app required!

---

## Features

* 🎥 Stream video and audio files over your LAN
* 🌐 Modern, browser-based interface (no client install needed)
* 🔍 Automatic media file discovery
* 💻 Cross-platform (Linux, Windows, macOS)
* ⚡ Efficient and lightweight
* 🔄 Real-time media directory monitoring
* 🚀 Runs as a system service (Linux)

---

## How It Works

1. **Drop your media files** (videos, music) into the `media/` folder in this project.
2. **Run the server** using `cargo run` or set it up as a system service.
3. **Open a browser** on any device in your LAN and go to `http://<server-ip>:3000`.
4. **Browse and play** your media instantly!
5. **Changes are reflected automatically** - no need to restart the server.

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
git clone https://github.com/guyyagil/rustyStrem.git

# Navigate to the project directory
cd rustyStrem

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

If you want RustyStrem to start automatically when your system boots:

1. **Create systemd service file:**
   Create `/etc/systemd/system/rustystrem.service` with:

   ```ini
   [Unit]
   Description=RustyStrem Media Server
   After=network.target

   [Service]
   Type=simple
   User=YOUR_USERNAME
   WorkingDirectory=/absolute/path/to/rustyStrem
   ExecStart=/absolute/path/to/rustyStrem/target/release/rustystrem
   Restart=always
   RestartSec=3

   [Install]
   WantedBy=multi-user.target
   ```

   Use `pwd` inside your project directory to find the correct absolute path.

2. **Enable and start the service:**

   ```bash
   sudo systemctl enable rustystrem
   sudo systemctl start rustystrem
   ```

3. **Check service status:**

   ```bash
   systemctl status rustystrem
   ```

4. **View logs:**

   ```bash
   journalctl -u rustystrem -f
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

### Managing Media Files

1. **Add media files:**

   * Place your video and audio files in the `media/` directory
   * Files are automatically detected and available in the web interface

2. **Real-time Updates:**

   * New files appear automatically
   * Removed files are removed from the listing
   * Modified files are updated in real-time
   * No manual refresh required

---

## Configuration

You can customize the server using environment variables:

* `MEDIA_DIR` — Path to your media directory (default: `./media`)
* `PORT` — Server port (default: `3000`)
* `RUST_LOG` — Logging level (default: `info`)

### Examples

1. **Running with custom port:**

   ```bash
   PORT=8080 cargo run
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
   ```

---

## Folder Structure

```text
rustyStrem/
├── src/                 # Main source code directory
│   ├── main.rs          # Application entry point
│   ├── server/          # Server-related code
│   │   ├── mod.rs       # Server module definition
│   │   └── routes.rs    # Route handlers
│   ├── media/           # Media handling code
│   │   ├── mod.rs       # Media module definition
│   │   └── scanner.rs   # Media file scanning logic
│   └── utils/           # Utility functions
│       ├── mod.rs       # Utils module definition
│       └── config.rs    # Configuration handling
├── templates/           # HTML templates
│   └── index.html       # Main web interface
├── static/              # Static assets
│   ├── css/             # Stylesheets
│   └── js/              # JavaScript files
├── tests/               # Test files
│   └── integration/     # Integration tests
├── media/               # Default media directory
├── config/              # Configuration files
│   └── systemd/         # System service configuration
└── docs/                # Documentation
```
