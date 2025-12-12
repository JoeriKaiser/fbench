# FBench

![fbench logo](./assets/logo_black.svg)

A fast, lightweight database explorer built with Rust and egui.

## Features

- **Multi-database**: PostgreSQL and MySQL support
- **Schema browser**: Tables, views, columns with row estimates
- **Table inspector**: Columns, indexes, constraints
- **Query editor**: Syntax highlighting, autocomplete
- **Results**: Sortable columns, export (CSV/JSON/XML)
- **Connections**: Save and manage multiple connections

## Build & Run

```bash
cargo run --release
```

## Install

```bash
make && sudo make install
```

Installs to `/usr/local/bin` by default. Override with `PREFIX=/custom/path`.

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
