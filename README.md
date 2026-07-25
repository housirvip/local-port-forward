# Port Forward Manager

**[中文文档](README_zh.md)**

A local port-forwarding tool with a web UI and desktop app.  
Bind a local port → forward all TCP connections to a configurable remote host:port.  
HTTP traffic is automatically detected and reverse-proxied with full request/response logging.

## Screenshots

![Rules Page](static/pic1.png)

![Logs Page](static/pic2.png)

## Features

- **Rules management** — create, edit, delete, and toggle forwarding rules via a browser or desktop UI
- **Protocol auto-detect** — peeks first 8 bytes to distinguish HTTP from raw TCP; explicit `http` / `tcp` modes available
- **HTTP reverse proxy** — full header and body logging for HTTP traffic; real-time streaming via SSE
- **TCP relay** — bidirectional raw TCP forwarding with byte-count and duration logging
- **Log viewer** — paginated request log table with per-rule filter, live SSE streaming mode, and expandable detail rows
- **Settings** — configure log retention (max rows + TTL days) and per-rule defaults from the UI
- **Desktop app** — native window on macOS / Windows / Linux via Tauri v2 (embedded server, no separate process)
- **i18n** — UI available in English and Chinese; language preference persisted in localStorage

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Tauri desktop (optional)  │  Browser (server mode) │
│  webview → http://127.0.0.1:<random>                │
└──────────────────┬──────────────────────────────────┘
                   │ HTTP
     ┌─────────────▼──────────────┐
     │   Axum API server          │  :8080 (configurable)
     │   GET/POST/PUT/DELETE       │
     │   /api/rules               │
     │   /api/logs  + SSE stream  │
     │   /api/settings            │
     │   /* (SPA, rust-embed)     │
     └─────────────┬──────────────┘
                   │ SQLite (sqlx)
     ┌─────────────▼──────────────┐
     │   Proxy Manager            │
     │   per-rule TcpListener     │
     │   HTTP path → hyper proxy  │
     │   TCP path  → relay        │
     └────────────────────────────┘
```

- **Backend**: Rust + Axum + sqlx (SQLite) + tokio
- **Frontend**: React 19 + TypeScript + Vite + Tailwind CSS v4 + shadcn/ui primitives
- **Desktop**: Tauri v2 — embeds the Axum server in-process, opens a webview on a random loopback port

## Quick Start

### Server mode (CLI)

**Prerequisites**: Rust toolchain (`rustup`), Node.js ≥ 20

```bash
# 1. Build the frontend (outputs to web/)
make frontend

# 2. Run (defaults: DB_PATH=portforward.db, LISTEN_ADDR=0.0.0.0:8080)
cargo run --release

# Open http://localhost:8080
```

Environment variables:

| Variable | Default | Description |
|---|---|---|
| `DB_PATH` | `portforward.db` | SQLite database file path |
| `LISTEN_ADDR` | `0.0.0.0:8080` | API + UI listen address |
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |

### Desktop app

**Prerequisites**: Rust toolchain, Node.js ≥ 20, `cargo install tauri-cli --version '^2' --locked`

Linux also requires:
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev patchelf build-essential libssl-dev
```

```bash
# Development (hot-rebuild)
make desktop-dev

# Production bundle for the current platform
make desktop-build
# macOS  → target/release/bundle/dmg/  and  target/release/bundle/macos/
# Linux  → target/release/bundle/deb/  and  target/release/bundle/appimage/
# Windows → target/release/bundle/msi/
```

The desktop app stores its database in the system app-data directory:
- **macOS**: `~/Library/Application Support/com.portforward.app/portforward.db`
- **Windows**: `%APPDATA%\com.portforward.app\portforward.db`
- **Linux**: `~/.local/share/com.portforward.app/portforward.db`

## Build Reference

```bash
make frontend       # build React → web/
make backend        # cargo build --release
make build          # frontend + backend

make dev            # frontend dev server (port 5173) + backend (port 8080) in parallel
make desktop-dev    # Tauri dev window
make desktop-build  # Tauri production bundle

make clean          # remove web/ dev.db target/
```

## API

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/rules` | List all rules |
| `POST` | `/api/rules` | Create a rule |
| `PUT` | `/api/rules/:id` | Update a rule |
| `DELETE` | `/api/rules/:id` | Delete a rule |
| `POST` | `/api/rules/:id/toggle` | Toggle enabled state |
| `GET` | `/api/logs` | Paginated request logs (`rule_id`, `page`, `page_size`) |
| `DELETE` | `/api/logs` | Clear logs (optional `rule_id` query param) |
| `GET` | `/api/logs/stream` | SSE stream of real-time request logs |
| `GET` | `/api/settings` | Get settings |
| `PUT` | `/api/settings` | Update settings |

## License

MIT — see [LICENSE](LICENSE)
