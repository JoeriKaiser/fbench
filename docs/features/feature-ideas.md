# FBench Feature Ideas

Generated: 2026-02-04
Updated: 2026-02-04 (Phase 1 Complete)

## Current Features (as of today)

- [x] Multi-database support (PostgreSQL, MySQL)
- [x] Schema browser with table inspector
- [x] SQL editor with Shiki syntax highlighting and autocomplete
- [x] Query results with sorting and export (CSV/JSON/XML)
- [x] Saved connections with secure password storage
- [x] Query history (last 50 queries with persistence)
- [x] Connection health monitoring
- [x] Query bookmarks/favorites
- [x] Quick switcher (Ctrl+P command palette)
- [x] Recent tables tracking
- [x] Query templates with variable substitution
- [x] Editor drafts (auto-save)
- [x] History search/filtering
- [x] Session persistence
- [x] AI query generation (Ollama/OpenRouter)

---

## Phase 1: Query Experience Polish (COMPLETED ✓)

### Core Features
- [x] **Multi-tab query editor** - Open multiple query tabs instead of single editor
- [x] **Query formatting/beautification** - Auto-format SQL with configurable style
- [x] **JSON column viewer** - Pretty-print and edit JSON data in cells
- [x] **Query explain/execution plan** - Visualize query performance and optimization suggestions

### Implementation Details
- Created `QueryTab` and `TabState` structs for managing multiple tabs
- Added `TabBar` component for tab navigation with add/close functionality
- Refactored `SqlEditor` to use tab-aware state
- Updated `ResultsTable` to display results per-tab
- Added `JsonViewer` modal for viewing cell content with JSON pretty-printing
- Added `ExecutionPlanDialog` with PostgreSQL EXPLAIN and MySQL EXPLAIN ANALYZE support
- Updated `DraftStore` for multi-tab persistence
- Added `sqlformat` dependency for SQL formatting

---

## Features from Competitors (TablePlus/DBeaver)

### Query Editing Experience
- [ ] **Multi-tab Query Editor** - ✅ COMPLETED
- [ ] **Query Formatting/Beautification** - ✅ COMPLETED
- [ ] **Code Snippets Library** - Reusable SQL snippets beyond templates
- [ ] **Query Explain/Execution Plan** - ✅ COMPLETED
- [ ] **Query Profiling** - Track query execution statistics over time
- [ ] **Keyboard-First Navigation** - Vim-like keybindings option

### Data Management
- [ ] **Data Editing Inline** - Double-click cells to edit data directly in results grid
- [ ] **Foreign Key Navigation** - Click FK values to jump to related records
- [ ] **Table Data Filtering** - Filter results without modifying query
- [ ] **JSON Column Viewer** - ✅ COMPLETED
- [ ] **Data Masking for Sensitive Columns** - Auto-hide PII in results
- [ ] **Query Result Caching** - Cache expensive query results
- [ ] **Smart Data Export** - Export with automatic chunking for large datasets
- [ ] **Import Data** - Import CSV/JSON/SQL files into tables

### Administration & Schema
- [ ] **Structure Sync** - Compare and sync database schemas between connections
- [ ] **Database ERD/Diagrams** - Entity relationship diagrams
- [ ] **Backup/Restore** - Database backup and restore operations
- [ ] **Session Management** - View and manage active database sessions
- [ ] **Trigger/Procedure Editor** - Edit stored procedures and triggers
- [ ] **Migration Generator** - Generate migration scripts from schema changes
- [ ] **Data Compare** - Compare data between tables or databases

### Connectivity & Organization
- [ ] **SSH Tunneling** - Connect to databases through SSH tunnels
- [ ] **Connection Groups/Folders** - Organize connections into folders
- [ ] **Dark/Light Theme Toggle** - Manual theme switching
- [ ] **Cross-Database Queries** - Query across PostgreSQL and MySQL simultaneously

---

## Novel/AI-Powered Features (Unique to FBench)

### AI-Enhanced Development
- [ ] **AI-Powered Schema Insights** - LLM analyzes schema and suggests optimizations
- [ ] **Natural Language to Query** - "Show me users who signed up last week" → SQL
- [ ] **Query Performance Predictions** - AI estimates query cost before execution
- [ ] **Smart Query Recommendations** - Suggest queries based on schema patterns
- [ ] **Contextual Documentation** - Hover over tables/columns to see AI-generated descriptions
- [ ] **Auto-Complete from Schema Patterns** - Learn from your query patterns
- [ ] **Query Dependencies Graph** - Visualize which queries depend on which tables

### Collaboration & Automation
- [ ] **Collaborative Query Sharing** - Share queries via links/cloud
- [ ] **Query Diff/Versioning** - Git-like versioning for queries
- [ ] **Real-time Collaboration** - Multiple users editing same query
- [ ] **Query Schedule/Automation** - Run queries on schedule, email results
- [ ] **Voice-to-SQL** - Dictate queries naturally

### Monitoring & Operations
- [ ] **Database Health Dashboard** - Visual metrics on DB performance
- [ ] **Integration with Monitoring Tools** - Connect to Prometheus/Datadog
- [ ] **Query Performance Analytics** - Track slow queries and trends

---

## Prioritization Framework

### Impact vs Effort Matrix

**High Impact, Low Effort (Quick Wins):**
- ✅ Query formatting/beautification
- ✅ JSON column viewer
- Dark/light theme toggle
- Code snippets library
- Query result caching

**High Impact, High Effort (Major Features):**
- ✅ Multi-tab query editor
- Data editing inline
- Database ERD/diagrams
- AI natural language to query
- ✅ Query explain/execution plan

**Medium Impact, Low Effort:**
- Foreign key navigation
- Connection groups/folders
- Keyboard shortcuts (Vim mode)
- Data masking
- Table data filtering

**Strategic/Differentiation (Unique Value):**
- AI schema insights
- Query performance predictions
- Contextual documentation
- Query dependencies graph
- Natural language to SQL

---

## Recommended Phases

### Phase 1: Query Experience Polish (2-3 weeks) - ✅ COMPLETED
Focus on making the core query editing experience excellent:
1. ✅ Multi-tab query editor
2. ✅ Query formatting/beautification
3. ✅ JSON column viewer
4. ✅ Query explain/execution plan

### Phase 2: Data Management (2-3 weeks)
Enhance data manipulation capabilities:
1. Data editing inline
2. Foreign key navigation
3. Table data filtering
4. Import data

### Phase 3: AI Differentiation (3-4 weeks)
Leverage existing LLM infrastructure for unique features:
1. Natural language to query
2. AI schema insights
3. Contextual documentation
4. Query performance predictions

### Phase 4: Power User Features (Ongoing)
Advanced features for heavy users:
1. Database ERD/diagrams
2. Structure sync
3. Migration generator
4. Query versioning

---

## Notes

- Target audience: Developers (power users who write SQL)
- Goal: Both parity with competitors AND unique AI-powered differentiation
- All three areas: query editing, data management, and administration
