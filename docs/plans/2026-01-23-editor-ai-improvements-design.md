# Editor & AI Improvements Design

## Overview

Targeted improvements to fbench focusing on editor usability and lightweight AI integration. Fixes immediate pain points without major architectural changes.

## Scope

### In Scope
- Editor keyboard navigation fixes
- Autocomplete Tab key fix
- AI quick actions on selected SQL
- Schema-aware query suggestions

### Out of Scope
- Custom editor widget replacement
- Chat sidebar
- Inline ghost-text completions

## 1. Editor Bug Fixes

### Ctrl+Arrow Word Navigation

**Problem:** Ctrl+Left/Right jumps to previous/next line before reaching word boundaries on current line.

**Fix:** Intercept Ctrl+Arrow before egui's default handling with custom logic:

1. Find current cursor position
2. For Ctrl+Right: scan forward to next word boundary (skip whitespace, then word chars, stop at whitespace/punctuation)
3. For Ctrl+Left: scan backward to previous word boundary
4. Only jump lines when already at line start/end

**Word boundaries:** whitespace, punctuation (`.`, `,`, `;`, `(`, `)`, etc.), transitions between alphanumeric and non-alphanumeric.

### Tab Key for Autocomplete

**Problem:** Tab moves UI focus instead of accepting autocomplete suggestion.

**Fix:** Capture Tab in input handling before egui's focus system:

```
if autocomplete.active && Tab pressed:
    consume the event (prevent focus change)
    apply selected suggestion
```

Use egui's event consumption API to prevent default Tab behavior when autocomplete is showing.

### Implementation Location

Both fixes in `src/ui/editor.rs`:
- New method `handle_navigation_keys()` called early in `show()`
- Modify autocomplete input handler to consume Tab events

## 2. AI Quick Actions on Selection

### Context Menu

Right-click on selected SQL shows:

| Action | Description |
|--------|-------------|
| Explain | Plain English explanation of the query |
| Optimize | Index usage suggestions, performance rewrites |
| Fix Error | Explains last error, suggests fixes |
| Add WHERE clause | Prompts for criteria, generates clause |
| Convert to... | Submenu: INSERT, UPDATE, DELETE |

### Interaction Flow

1. Select SQL text in editor
2. Right-click → context menu appears
3. Click action (e.g., "Explain")
4. Inline panel appears below editor:
   - Spinner while LLM responds
   - Result text with "Copy" and "Dismiss" buttons
   - For code suggestions: "Apply" button replaces selection

### Results Panel

- Collapsible panel between editor and results table
- Not a modal - query remains visible
- Auto-dismisses when user starts typing in editor

### Error Context

"Fix Error" automatically includes last error message from results panel.

## 3. Schema-Aware AI Suggestions

### UI Location

Below schema tree when a table is selected:

```
📁 Tables (12)
  ▪ users          ~1.2K    ← selected
  ▪ orders         ~15K
  ...

── Suggested Queries ──────────
  ▸ Recent users (last 7 days)
  ▸ Users by signup count
  ▸ Find user by email
```

### Generation Logic

1. On table selection, send to LLM:
   - Table name, columns (names + types)
   - Primary key, foreign keys
   - Row estimate

2. LLM returns 2-3 query descriptions with SQL

3. Cache suggestions per table for session

### Interaction

- Click suggestion → SQL inserted in editor
- Hover → tooltip shows actual SQL
- Refresh icon to regenerate

### Fallback (LLM unavailable or >3s timeout)

Generate based on table structure:
- Has `created_at` → "Recent rows"
- Has foreign key → "Join with [related table]"
- Has unique constraint → "Find by [column]"

## Implementation Order

1. **Editor fixes** - immediate usability improvement
2. **AI quick actions** - builds on existing LLM infrastructure
3. **Schema suggestions** - independent, can be added last

## Files Changed

| File | Changes |
|------|---------|
| `src/ui/editor.rs` | Navigation fixes, context menu, inline results panel |
| `src/ui/schema.rs` | Suggestions section below table list |
| `src/llm/mod.rs` | New request types: Explain, Optimize, Suggest |
| `src/app.rs` | Wire up new LLM responses |

## New LLM Request Types

```rust
enum LlmRequest {
    Generate { prompt: String, schema: SchemaInfo },  // existing
    Explain { sql: String },
    Optimize { sql: String, schema: SchemaInfo },
    FixError { sql: String, error: String, schema: SchemaInfo },
    SuggestQueries { table: TableInfo },
}

enum LlmResponse {
    Generated(String),  // existing
    Explanation(String),
    Optimization { suggestion: String, sql: Option<String> },
    ErrorFix { explanation: String, sql: Option<String> },
    QuerySuggestions(Vec<QuerySuggestion>),
    Error(String),
}

struct QuerySuggestion {
    label: String,
    sql: String,
}
```

## UI Mockups

### Quick Actions Context Menu

```
┌─────────────────────┐
│ ✦ Explain           │
│ ⚡ Optimize          │
│ 🔧 Fix Error        │
├─────────────────────┤
│ + Add WHERE clause  │
│ ▸ Convert to...     │
└─────────────────────┘
```

### Inline Results Panel

```
┌─ Editor ─────────────────────────────────────┐
│ SELECT * FROM users WHERE created_at > ...   │
└──────────────────────────────────────────────┘
┌─ Explanation ─────────────────────── ✕ ─────┐
│ This query retrieves all columns from the   │
│ users table where the creation date is...   │
│                          [Copy] [Dismiss]   │
└──────────────────────────────────────────────┘
┌─ Results ────────────────────────────────────┐
│ ...                                          │
```

### Schema Suggestions

```
┌─ Schema ─────────────────────────────────────┐
│ 📁 Tables (12)                               │
│   ▪ users          ~1.2K                     │
│   ▪ orders         ~15K                      │
│                                              │
│ ── Suggested Queries ─────────────── ↻ ──── │
│   ▸ Recent users (last 7 days)               │
│   ▸ Users by signup count                    │
│   ▸ Find user by email                       │
└──────────────────────────────────────────────┘
```
