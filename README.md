# RustyStrem

A lightweight, cross-platform media streaming server built with Rust. Stream your local video and audio files to any device on your network using just a web browser—no client app required!

---

## Features

- 🎥 Stream video and audio files over your LAN
- 🌐 Modern, browser-based interface (no client install needed)
- 🔍 Automatic media file discovery
- 💻 Cross-platform (Linux, Windows, macOS)
- ⚡ Efficient and lightweight

---

## How It Works

1. **Drop your media files** (videos, music) into the `media/` folder in this project.
2. **Run the server** from the command line.
3. **Open a browser** on any device in your LAN and go to `http://<server-ip>:3000`.
4. **Browse and play** your media instantly!

---

## Requirements

- Rust 1.70 or higher
- A modern web browser (Chrome, Firefox, Edge, Safari, etc.)
- Local Area Network (LAN) access

---

## Installation

```bash
git clone https://github.com/yourusername/rustyStrem.git
cd rustyStrem
cargo build --release
```

---

## Usage

1. **Add your media files:**
   - Place your video and audio files in the `media/` directory (or set a custom folder, see below).

2. **Run the server:**
   ```bash
   cargo run --release
   ```

3. **Access from your browser:**
   - On the same machine: [http://localhost:3000](http://localhost:3000)
   - From another device on your LAN: `http://<your-server-ip>:3000`

---

## Configuration

You can customize the server using environment variables:

- `MEDIA_DIR` — Path to your media directory (default: `./media`)
- `PORT` — Server port (default: `3000`)
- `RUST_LOG` — Logging level (default: `info`)

**Example:**
```bash
MEDIA_DIR=/path/to/your/media PORT=8080 cargo run --release
```

---

## Folder Structure
