# fbench-tui: Go/Bubbletea Rewrite Design

## Overview

Port fbench from Rust/egui to a Go TUI using Bubbletea and Lipgloss. Full feature parity with novel enhancements for terminal workflows.

## Key Decisions

| Decision | Choice |
|----------|--------|
| Navigation | Panel-focused + command palette |
| Layout | Responsive splits (wide/medium/narrow) |
| Editor | Notebook mode with multi-cell support |
| Command palette | Context-aware with global `:` commands |
| Text editing | Vim bindings + `$EDITOR` escape hatch |
| Password storage | System keyring |
| Theme | Dark green palette |

## Architecture

```
┌─ Main Bubbletea Program
│  ├─ Root Model (orchestrates sub-models)
│  ├─ Layout Manager (responsive splits)
│  └─ Command Palette (context-aware)
│
├─ Panel Models (each a Bubbletea sub-model)
│  ├─ Schema Browser
│  ├─ Notebook Editor (multi-cell)
│  ├─ Results Viewer
│  └─ Connection Dialog
│
├─ Database Layer (goroutines + channels)
│  ├─ Connection Pool (sqlx)
│  ├─ Schema Introspection
│  └─ Query Execution
│
└─ LLM Layer (goroutines + channels)
   ├─ Ollama Client
   └─ OpenRouter Client
```

### Key Libraries

- `bubbletea` - TUI framework (Elm architecture)
- `lipgloss` - Styling and layout
- `bubbles` - Pre-built components (textinput, viewport, table, list)
- `sqlx` - Database driver (Postgres + MySQL)
- `go-keyring` - System credential storage

## Responsive Layout

### Breakpoints

- **Wide (≥120 cols):** Three columns - Schema (20%) | Notebook (50%) | Results (30%)
- **Medium (80-119 cols):** Two columns - Schema (25%) | Notebook+Results stacked (75%)
- **Narrow (<80 cols):** Single column, toggleable panels with Tab key

### Wide Layout

```
┌──────────┬─────────────────────┬────────────────┐
│ Schema   │ Notebook            │ Results        │
│ Browser  │ ┌─────────────────┐ │ ┌────────────┐ │
│          │ │ Cell 1 [SQL]    │ │ │ col1 │ col2│ │
│ ▼ Tables │ │ ► Result: 3 rows│ │ │──────┼─────│ │
│   users  │ ├─────────────────┤ │ │ val  │ val │ │
│   orders │ │ Cell 2 [SQL]    │ │ └────────────┘ │
│ ▼ Views  │ │ ► Result: 1 row │ │                │
│          │ └─────────────────┘ │ [Export: c j x]│
├──────────┴─────────────────────┴────────────────┤
│ [F1 Help] [Ctrl+P Palette] [Tab Focus] connected│
└─────────────────────────────────────────────────┘
```

### Narrow Layout

```
┌─────────────────────────────┐
│ Notebook [1/3]              │
│ ┌─────────────────────────┐ │
│ │ SELECT * FROM users     │ │
│ │ WHERE active = true;    │ │
│ ├─────────────────────────┤ │
│ │ ▼ 24 rows (12ms)        │ │
│ └─────────────────────────┘ │
├─────────────────────────────┤
│ Tab: Schema │ Results │ ... │
└─────────────────────────────┘
```

### Focus System

- `Tab` / `Shift+Tab` cycles between visible panels
- Focused panel gets highlighted border (accent color)
- Each panel captures its own keybindings when focused
- Global keys (Ctrl+P, Ctrl+Q, F1) work from anywhere

## Notebook Editor

### Cell Structure

Each cell is an independent unit with:
- SQL content (editable with vim bindings)
- Collapsed/expanded state
- Execution status (idle, running, success, error)
- Inline result preview (collapsible)
- Optional name for command palette jump

```
┌─────────────────────────────────────┐
│ ● users_query                    ▼ ▲│  ← Name + reorder handles
├─────────────────────────────────────┤
│ SELECT id, name, email              │
│ FROM users                          │
│ WHERE created_at > '2024-01-01'     │
│ LIMIT 100;                          │  ← Vim-enabled editor
├─────────────────────────────────────┤
│ ✓ 47 rows (23ms)        [expand ►] │  ← Inline result summary
└─────────────────────────────────────┘
```

### Keybindings (notebook focused)

| Key | Action |
|-----|--------|
| `j/k` | Navigate between cells |
| `Enter` | Edit current cell (insert mode) |
| `Esc` | Exit insert mode |
| `Ctrl+Enter` | Run current cell |
| `Ctrl+Shift+Enter` | Run all cells |
| `o` | New cell below |
| `O` | New cell above |
| `dd` | Delete cell (with confirmation) |
| `J/K` | Move cell down/up |
| `zc/zo` | Collapse/expand result |
| `Ctrl+E` | Open cell in `$EDITOR` |
| `/` | Name/rename cell |

## Command Palette

### Activation

`Ctrl+P` from anywhere opens the fuzzy-searchable palette.

### Context-Aware Sections

```
┌─────────────────────────────────────────┐
│ >                                       │  ← Fuzzy search input
├─────────────────────────────────────────┤
│ ▸ Current Context (Schema Browser)      │
│   Select first 100 rows      Enter      │
│   View table structure       Ctrl+D     │
│   Copy table name            y          │
│   Generate INSERT template   gi         │
├─────────────────────────────────────────┤
│ ▸ Cells                                 │
│   → users_query              g1         │
│   → orders_summary           g2         │
│   → (unnamed cell 3)         g3         │
├─────────────────────────────────────────┤
│ ▸ Global                                │
│   :connect      New connection          │
│   :disconnect   Close connection        │
│   :export       Export results...       │
│   :ai           Generate SQL with AI    │
│   :settings     LLM settings            │
│   :help         Show keybindings        │
│   :quit         Exit application        │
└─────────────────────────────────────────┘
```

### Behavior

- Typing filters all sections simultaneously
- Arrow keys navigate, Enter executes
- Esc closes palette
- Global commands prefixed with `:` for quick access (`:q` → quit)
- Recently used commands float to top
- Shows associated keybinding so users learn shortcuts
- Typing `:` anywhere (not in insert mode) opens palette pre-filtered to global commands

## Schema Browser

### Tree Structure

```
┌─ Schema Browser ─────────────┐
│ 🔍 Filter: _____________     │  ← Live filter
├──────────────────────────────┤
│ ▼ Tables (12)                │
│   ├─ users          1,247    │  ← Row count estimate
│   ├─ orders        15,832    │
│   ├─ products         89     │
│   └─ ...                     │
│ ▶ Views (3)                  │  ← Collapsed section
│ ▶ Saved Queries (5)          │  ← Merged from sidebar
└──────────────────────────────┘
```

### Keybindings (schema focused)

| Key | Action |
|-----|--------|
| `j/k` | Navigate items |
| `Enter` | Load first 100 rows into new cell |
| `d` | Describe table (open detail modal) |
| `y` | Yank (copy) table name |
| `i` | Generate INSERT template in new cell |
| `s` | Generate SELECT * template |
| `/` | Focus filter input |
| `Esc` | Clear filter |
| `h/l` | Collapse/expand sections |
| `r` | Refresh schema |

### Table Detail Modal

```
┌─ users ──────────────────────────────┐
│ [Columns] [Indexes] [Constraints]    │  ← Tab navigation
├──────────────────────────────────────┤
│ name       │ type    │ null │ pk    │
│────────────┼─────────┼──────┼───────│
│ id         │ int     │ NO   │ ✓     │
│ email      │ varchar │ NO   │       │
│ created_at │ timestmp│ YES  │       │
└──────────────────────────────────────┘
│                          [Esc close] │
└──────────────────────────────────────┘
```

## Results Viewer & Export

### Results Table

```
┌─ Results ─ Cell: users_query ─ 47 rows (23ms) ─┐
│ id   │ name          │ email           │ stat │
│──────┼───────────────┼─────────────────┼──────│
│ 1    │ Alice Smith   │ alice@demo.com  │ act… │
│ 2    │ Bob Jones     │ bob@example.io  │ pen… │
│ 3    │ Carol White   │ carol@test.org  │ act… │
│ ...  │               │                 │      │
├──────────────────────────────────────────────┤
│ [c]sv [j]son [x]ml │ ← 1/47 │ Sort: id ▲    │
└──────────────────────────────────────────────┘
```

### Keybindings (results focused)

| Key | Action |
|-----|--------|
| `j/k` | Navigate rows |
| `h/l` | Scroll columns horizontally |
| `Enter` | Open cell detail modal |
| `s` | Cycle sort on current column |
| `H/L` | Move to prev/next column for sorting |
| `c` | Export as CSV |
| `J` (Shift) | Export as JSON |
| `x` | Export as XML |
| `y` | Yank current cell value |
| `Y` | Yank entire row as JSON |
| `gg/G` | Jump to first/last row |
| `Ctrl+U/D` | Page up/down |

### Export Flow

Pressing export key opens file path input with smart default:
`~/Downloads/users_query_2024-01-15.csv`

## AI Integration

### AI Prompt Modal

Activation: `:ai` command or `Ctrl+G`

```
┌─ Generate SQL ───────────────────────────────┐
│ Provider: Ollama (llama3.2)     [⚙ Settings] │
├──────────────────────────────────────────────┤
│ Describe what you want to query:             │
│ ┌──────────────────────────────────────────┐ │
│ │ Show me users who signed up last month   │ │
│ │ and have made at least one order         │ │
│ └──────────────────────────────────────────┘ │
├──────────────────────────────────────────────┤
│ Context: Schema will be included in prompt   │
│              [Esc Cancel]  [Enter Generate]  │
└──────────────────────────────────────────────┘
```

### Generation Flow

1. User types natural language request
2. System builds prompt with full schema context
3. Shows spinner: `⠋ Generating SQL...`
4. On success: Creates new notebook cell with generated SQL
5. On error: Shows error message, keeps modal open

### Settings Modal

```
┌─ LLM Settings ───────────────────────────────┐
│ Provider: [Ollama ▼]                         │
├──────────────────────────────────────────────┤
│ Ollama URL:    http://localhost:11434        │
│ Ollama Model:  llama3.2                      │
├──────────────────────────────────────────────┤
│ OpenRouter Key: ••••••••••••                 │
│ OpenRouter Model: openai/gpt-4o-mini         │
├──────────────────────────────────────────────┤
│             [Esc Cancel]  [Ctrl+S Save]      │
└──────────────────────────────────────────────┘
```

## Connection Management

### Connection Dialog

```
┌─ Connect to Database ────────────────────────┐
│ ▸ Saved Connections                          │
│   ├─ prod_postgres     PostgreSQL   ★ last   │
│   ├─ local_mysql       MySQL                 │
│   └─ staging_db        PostgreSQL            │
├──────────────────────────────────────────────┤
│ ▸ New Connection                             │
│   Type:     [PostgreSQL ▼]                   │
│   Name:     [________________________]       │
│   Host:     [localhost_____________] :[ 5432]│
│   Database: [________________________]       │
│   Schema:   [public__________________]       │
│   User:     [________________________]       │
│   Password: [••••••••________________]       │
│   ☑ Save password to keyring                 │
├──────────────────────────────────────────────┤
│ [T]est  [Esc Cancel]  [Enter Connect]        │
└──────────────────────────────────────────────┘
```

### Connection Status

Status bar always shows connection state:
- `○ Disconnected`
- `◐ Connecting...`
- `● prod_postgres (PostgreSQL)`
- `✗ Connection failed: timeout`

### Auto-reconnect

On connection drop, shows notification and offers quick reconnect:
`Connection lost. [r] Reconnect  [c] New connection`

## Project Structure

```
fbench-tui/
├── cmd/
│   └── fbench/
│       └── main.go              # Entry point
├── internal/
│   ├── app/
│   │   ├── app.go               # Root model, orchestration
│   │   ├── keys.go              # Keybinding definitions
│   │   └── layout.go            # Responsive layout manager
│   ├── ui/
│   │   ├── palette/
│   │   │   └── palette.go       # Command palette component
│   │   ├── notebook/
│   │   │   ├── notebook.go      # Notebook container
│   │   │   ├── cell.go          # Individual cell model
│   │   │   └── editor.go        # Vim-enabled text editor
│   │   ├── schema/
│   │   │   ├── browser.go       # Schema tree browser
│   │   │   └── detail.go        # Table detail modal
│   │   ├── results/
│   │   │   ├── table.go         # Results table view
│   │   │   └── export.go        # Export handlers
│   │   ├── connection/
│   │   │   └── dialog.go        # Connection modal
│   │   ├── ai/
│   │   │   └── prompt.go        # AI generation modal
│   │   └── components/
│   │       ├── modal.go         # Reusable modal wrapper
│   │       ├── input.go         # Styled text input
│   │       └── statusbar.go     # Bottom status bar
│   ├── db/
│   │   ├── connection.go        # Pool management
│   │   ├── schema.go            # Introspection queries
│   │   ├── query.go             # Query execution
│   │   └── types.go             # Shared types
│   ├── llm/
│   │   ├── client.go            # Provider interface
│   │   ├── ollama.go            # Ollama implementation
│   │   └── openrouter.go        # OpenRouter implementation
│   ├── config/
│   │   ├── connections.go       # Saved connections
│   │   ├── queries.go           # Saved queries
│   │   ├── llm.go               # LLM settings
│   │   └── keyring.go           # Password storage
│   └── export/
│       ├── csv.go
│       ├── json.go
│       └── xml.go
├── go.mod
├── go.sum
└── README.md
```

## Theme: Dark Green

```
Background:     #0a1210 (deep forest)
Surface:        #131f1a (panel backgrounds)
Border:         #2d4a3e (unfocused)
Border Focus:   #5faa8f (mint accent)
Text Primary:   #d4e6dc (soft mint white)
Text Muted:     #4a6b5d (muted sage)
Success:        #7fcc8e (bright green)
Error:          #e6736f (coral red)
Warning:        #d4a857 (gold)
Accent:         #5faa8f (mint)

SQL Syntax:
  Keywords:     #8fccb7 (seafoam)
  Strings:      #a8d98a (lime green)
  Numbers:      #e6b566 (amber)
  Functions:    #66c2cd (teal)
  Comments:     #4a6b5d (muted sage)
```

### Adaptive Styling

- Detects terminal color capability (truecolor/256/16)
- Falls back gracefully on limited terminals
- Respects `NO_COLOR` environment variable
