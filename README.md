# FBench

![fbench logo](./assets/logo_black.svg)

A fast, lightweight database explorer built with Rust and Dioxus.

## Features

- **Multi-database**: PostgreSQL and MySQL support
- **Schema browser**: Tables, views, columns with row estimates
- **Table inspector**: Columns, indexes, constraints
- **Query editor**: Syntax highlighting via Shiki, autocomplete
- **Results**: Sortable columns, export (CSV/JSON/XML)
- **Connections**: Save and manage multiple connections
- **Query history**: Last 50 executed queries with persistence
- **Connection health monitoring**: Automatic health checks with visual status indicators

## Install

### Quick Install (Recommended)

Install the latest release with one command:

```bash
curl -fsSL https://raw.githubusercontent.com/JoeriKaiser/fbench/main/install.sh | sh
```

Or with a custom install location:

```bash
curl -fsSL https://raw.githubusercontent.com/JoeriKaiser/fbench/main/install.sh | INSTALL_DIR=~/.local/bin sh
```

### From Source

```bash
make && sudo make install
```

Installs to `/usr/local/bin` by default. Override with `PREFIX=/custom/path`.

## Architecture

FBench uses **Dioxus 0.** for cross-platform UI rendering with a modern web-based stack:

- **Frontend**: Dioxus (Rust-based React-like framework)
- **Styling**: Tailwind CSS for consistent, modern UI design
- **Syntax Highlighting**: Shiki (same engine as VS Code) for accurate SQL highlighting
- **Backend**: Rust async workers with sqlx for database operations
- **Desktop**: WebKit-based rendering via dioxus-desktop

## System Dependencies

### Linux

Required packages for WebKit support:

```bash
# Fedora/RHEL/CentOS
sudo dnf install webkit2gtk3-devel gtk3-devel

# Ubuntu/Debian
sudo apt-get install libwebkit2gtk-4.0-dev libgtk-3-dev

# Arch Linux
sudo pacman -S webkit2gtk gtk3
```

### macOS

No additional dependencies required. WebKit is provided by the system.

### Windows

No additional dependencies required. WebKit2GTK is bundled.

## Build & Run

### Prerequisites

- Rust 1.70+ with Cargo
- System dependencies (see above)

### Development Build

```bash
# Install Dioxus CLI (required for development)
cargo install dioxus-cli --locked

# Run in development mode with hot reloading
dx serve
```

### Release Build

```bash
# Build optimized release binary
cargo build --release

# Run the release binary
./target/release/fbench
```

### Desktop Build

```bash
# Build native desktop application
cargo run --release
```

## Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+Enter` | Execute all |
| `Ctrl+Shift+Enter` | Execute statement at cursor |
| `Ctrl+S` | Save query |
| `Ctrl+/` | Toggle comment |
| `Ctrl+]` | Indent |
| `Ctrl+[` | Outdent |
| `↑ / ↓` | Navigate autocomplete |
| `Tab / Enter` | Accept autocomplete |
| `Esc` | Dismiss autocomplete |

## Configuration

Configuration and query history are stored in:

- **Linux**: `~/.config/fbench/`
- **macOS**: `~/Library/Application Support/com.fbench.app/`
- **Windows**: `%APPDATA%\fbench\`

## Development

### Project Structure

```
src/
├── main.rs              # Entry point
├── app.rs               # Core app state, event loop
├── db/                  # Database layer
│   ├── mod.rs           # Types, enums, request/response structs
│   ├── connection.rs    # DbWorker, connection pools
│   └── query.rs         # Query utilities
├── ui/                  # UI components
│   ├── mod.rs           # Re-exports
│   ├── editor.rs        # SQL editor with Shiki highlighting
│   ├── results.rs       # Results table
│   ├── schema.rs        # Schema browser
│   └── ...              # Other UI components
├── config/              # Configuration persistence
└── export/              # Data export (CSV/JSON/XML)
```

### Key Technologies

- **Dioxus 0.6**: Reactive UI framework for Rust
- **Tailwind CSS**: Utility-first CSS framework
- **Shiki**: Syntax highlighter (TextMate grammar support)
- **sqlx**: Async SQL toolkit with compile-time checked queries
- **tokio**: Async runtime
- **serde**: Serialization framework

### Linting and Formatting

```bash
cargo check              # Check code for errors
cargo clippy             # Run linter
cargo clippy --fix       # Auto-fix clippy warnings
cargo fmt                # Format code with rustfmt
```

## Migration Notes (egui → Dioxus)

This project has migrated from egui to Dioxus 0.6 for improved cross-platform support and modern UI capabilities:

- **Immediate mode** (egui) → **Reactive/VDOM** (Dioxus)
- **Immediate rendering** → **Web-based rendering** via WebKit
- **Custom syntax highlighting** → **Shiki** (VS Code-quality highlighting)
- **Built-in styling** → **Tailwind CSS** for consistent design
- **Single codebase** → **Web and desktop** from the same code
