# FBench

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

## Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+Enter` | Execute query |
| `Ctrl+S` | Save query |
| `Tab` | Accept autocomplete |

## Requirements

- Rust 1.70+
- PostgreSQL and/or MySQL server
