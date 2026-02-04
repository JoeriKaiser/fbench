# FBench

![fbench logo](./assets/logo_black.svg)

## A fast database explorer built with Rust and Dioxus.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/JoeriKaiser/fbench/main/install.sh | sh
```

Or build from source:

```bash
cargo build --release
```

## Usage

```bash
fbench              # Launch application
```

Connect to a database, browse schema, write SQL queries, and export results.

## Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+Enter` | Execute query |
| `Ctrl+Shift+Enter` | Execute statement at cursor |
| `Ctrl+S` | Save query |
| `Ctrl+P` | Open quick switcher |
| `Ctrl+D` | Duplicate current line |
| `Tab` | Indent selection |
| `Shift+Tab` | Outdent selection |
| `Ctrl+/` | Toggle comment |
| `Ctrl+]` | Indent |
| `Ctrl+[` | Outdent |
| `↑ / ↓` | Navigate autocomplete |
| `Tab / Enter` | Accept autocomplete |
| `Esc` | Dismiss autocomplete |

## Features

- **Multi-database**: PostgreSQL and MySQL support
- **Multi-tab Editor**: Work with multiple queries simultaneously, each with its own results and state
- **Schema browser**: Tables, views, columns with row estimates
- **Table inspector**: Columns, indexes, constraints
- **Query editor**: Syntax highlighting via Shiki, autocomplete
- **Results**: Sortable columns, export (CSV/JSON/XML)
- **Data Editing**: Edit cells inline, insert new rows, delete rows (for single-table queries with primary keys)
- **Foreign Key Navigation**: Click FK links to jump to related records
- **Data Import**: Import CSV/JSON data directly into tables
- **Connections**: Save and manage multiple connections
- **Query history**: Last 50 executed queries with persistence
- **Connection health monitoring**: Automatic health checks with visual status indicators
- **Query Bookmarks**: Star/favorite frequently used queries
- **Quick Switcher**: Command palette (Ctrl+P) for tables, queries, history
- **Recent Tables**: Track recently accessed tables
- **Query Templates**: Pre-built templates with variable substitution
- **Editor Drafts**: Auto-saved editor content
- **History Search**: Filter query history
- **Session Persistence**: Restore UI state on reconnect

## Data Storage

Configuration and query history are stored in:

- **Linux**: `~/.config/fbench/`
- **macOS**: `~/Library/Application Support/com.fbench.app/`
- **Windows**: `%APPDATA%\fbench\`

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

## Development

### Architecture

FBench uses **Dioxus 0.6** for cross-platform UI rendering with a modern web-based stack:

- **Frontend**: Dioxus (Rust-based React-like framework)
- **Styling**: Tailwind CSS for consistent, modern UI design
- **Syntax Highlighting**: Shiki (same engine as VS Code) for accurate SQL highlighting
- **Backend**: Rust async workers with sqlx for database operations
- **Desktop**: WebKit-based rendering via dioxus-desktop

### Build & Run

Prerequisites:
- Rust 1.70+ with Cargo
- System dependencies (see above)

Development build:

```bash
# Install Dioxus CLI (required for development)
cargo install dioxus-cli --locked

# Run in development mode with hot reloading
dx serve
```

Release build:

```bash
# Build optimized release binary
cargo build --release

# Run the release binary
./target/release/fbench
```

### Project Structure

```
src/
├── main.rs              # Entry point
├── app.rs               # Core app state, event loop
├── db/                  # Database layer
│   ├── mod.rs           # Types, enums, request/response structs
│   ├── connection.rs    # DbWorker, connection pools
│   └── query.rs         # Query utilities
├── components/          # UI components
│   ├── mod.rs           # Re-exports
│   ├── editor.rs        # SQL editor with Shiki highlighting
│   ├── results.rs       # Results table
│   ├── schema.rs        # Schema browser
│   └── ...              # Other UI components
├── config/              # Configuration persistence
└── export/              # Data export (CSV/JSON/XML)
```

### Linting and Formatting

```bash
cargo check              # Check code for errors
cargo clippy             # Run linter
cargo clippy --fix       # Auto-fix clippy warnings
cargo fmt                # Format code with rustfmt
```

### Key Technologies

- **Dioxus 0.6**: Reactive UI framework for Rust
- **Tailwind CSS**: Utility-first CSS framework
- **Shiki**: Syntax highlighter (TextMate grammar support)
- **sqlx**: Async SQL toolkit with compile-time checked queries
- **tokio**: Async runtime
- **serde**: Serialization framework

## Migration Notes (egui → Dioxus)

This project has migrated from egui to Dioxus 0.6 for improved cross-platform support and modern UI capabilities:

- **Immediate mode** (egui) → **Reactive/VDOM** (Dioxus)
- **Immediate rendering** → **Web-based rendering** via WebKit
- **Custom syntax highlighting** → **Shiki** (VS Code-quality highlighting)
- **Built-in styling** → **Tailwind CSS** for consistent design
- **Single codebase** → **Web and desktop** from the same code
