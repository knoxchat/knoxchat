## V1.3.1

- Enhance plan & task manager

### Checkpoints Implementation

- Fix: a new computeCheckpointDiff message
- Smarter checkpoint creation
    - Content-hash dedup
    - Correct created-vs-modified classification
    - Recognizable auto descriptions
    - Details consistency: getCheckpointDetails
- Better compare & list UI
    - Compare against any checkpoint
    - Change stats in the list
- Rewritten Enhanced Diff Viewer
    - Git-style hunks
    - Working word-level highlighting
    - Working navigation
    - Accurate counts
    - Copy as real patch
- Capture coverage — the 1 MB limit and untracked files are fixed
    - The per-file capture limit is now 5 MB by default and configurable
    - Text detection is no longer a small hard-coded allowlist
    - Both file watchers (CheckpointManager and AutoCheckpointSystem) now watch **/* instead of a fixed extension list, with a fast path filter so node_modules, .git, dist, etc. don't cause noise.
- Binary file support end to end
    - Images, fonts, PDFs, sqlite files, wasm, audio, etc.
    - Restore decodes base64 back to bytes, and conflict detection compares binary content correctly.

- New features
    - Compare against the current workspace: the "Compare against" selector now includes "Current workspace" — it reconstructs the checkpoint's full state and diffs it against what's on disk, effectively a preview of what a restore would change (including files created after the checkpoint).
    - Selective per-file restore: a new restoreCheckpointFiles message and manager API restores individual files (binary-aware, handles deletions) without touching the rest of the workspace. There's a restore button in the CodeViewer header in the File Snapshots tab.
    - Copy patch for all files: the enhanced diff viewer gained an "All" copy button that builds one multi-file unified patch across every changed text file.
    - Runtime config is applied live: saving the settings page immediately updates the manager's size limit, binary toggle, and tracked extensions (also loaded at startup from ~/.knox/checkpoint-config.json plus VS Code settings).



## V1.3.0

Fast, It's damn fast!

### Upgrades

- TypeScript 7
- Vite 8.1.4
- Node 24
- Esbuild 0.28.1
- Biome 2.5.4

### Updates & Improvements

- A better tool calling approach
- Merge all project root .knox into ~/.knox

## Fixes

- git diff HEAD --numstat
- Merges duplicate diff entries
- Real-time git state updates via `gitStateChanged` event
- Repo-relative path matching for accurate +/− stats
- Merge conflict files included in changed file list
- Click deleted files to open SCM diff view
- Smart display paths for duplicate filenames
- Sort changed files by total line delta

## V1.2.7

- Add git diff status
- Bugs fix

## V1.2.6

- move project root .knox/task directory to global ~/.knox/task
- move project root .knox/plans directory to global ~/.knox/plans
- Optimize the extension with faster initial loading
- Add todo/task UI

## V1.2.5

- Fix selected model state
- Fix message displaying state
- Fix runtime bugs, broken commands
- Fix match by model ID, not display label substring
- Added 120-second timeout that rejects with a descriptive error including partial output; clears listeners and data buffer on timeout
- Enhance font size resize
- Remove TTS

## V1.2.4

- Fix cross-platform compatible issues
- Fix checkpoints mass delete issue
- Improve Settings & Checkpoints Configuration UI/UX
- Adjust better context length
- Adjust better max-token management

## V1.2.3

### Features

- Add reasoning/thinking & web search for Anthropic Claude models
- Add reasoning/thinking for OpenAI models

### Upgrades

- uuid: 14.0.0
- zod: 4.4.2
- vite: 8.0.10
- axios: 1.16.0
- tailwindcss: 4.2.4


## V1.2.2

- Remove MCP
- Harness relevance improvements
- Support real-time code generation for all
- Memory system improvements
- Optimize DeepSeek V4 Pro & Flash models

## V1.2.1

- Fix Checkpoint Bugs

## V1.2.0

### Memory Brain System Upgrades

- **Input Sanitizer** — Security scanning on all memory writes (prompt injection, credential detection, invisible Unicode removal)
- **Context Fencing** — Recalled memories wrapped in safe boundary tags to prevent instruction injection
- **Tool Splitting** — Single 79-action memory tool split into 5 focused sub-tools: Memory, Memory Graph, Memory Sessions, Memory Manage, Memory Learn
- **FTS5 Fusion Search** — SQLite FTS5 full-text search with BM25 ranking, trigram fuzzy matching, and Bloom filter deduplication (replaces vector embeddings)
- **Multi-Strategy Retrieval** — Weighted fusion of FTS5, trigram, graph traversal, recency decay, and importance scoring with auto-detected query-type weights
- **LLM Reranking** — Optional LLM-based reranking pass on top-K retrieval candidates
- **Frozen Snapshot Pattern** — Session-scoped context caching with automatic invalidation after significant changes
- **Memory Provider Interface** — Plugin architecture for external memory backends (cloud, team-shared, vector DBs)
- **Cloud Sync** — Compressed backup/restore of memory database with cloud storage integration
- **Knox-MS Server Sync** — Background sync of local memories to Knox-MS server with change tracking and conflict resolution
- **Working Memory Persistence** — Serialize/restore working memory state across sessions with time-decay on restore
- **Capacity-Aware Auto-Pruning** — Proactive consolidation triggered when memory count approaches configured limits
- **Auto-Consolidation Scheduling** — Timer-based consolidation with capacity threshold checks

### Memory Dashboard UI

- **4-Tab Dashboard** — Overview, Memories, Graph, and Settings tabs with Lucide icons
- **Overview Page** — Health banner, 8 stat cards, tier distribution bars, category breakdown, entity types, recent sessions, consolidation stats, and memory timeline
- **Memory Browser** — Search with debounce, category/tier filters, sort options, expandable detail view, selection mode with bulk actions
- **Knowledge Graph** — Entity search, type filtering, interactive graph exploration with depth traversal
- **Settings Page** — Toggle/number controls for auto-memory, capacity, search engine (FTS5/trigram/bloom), and maintenance actions
- **Export Fixed** — Export now returns actual JSON data and saves file to ~/.knox/brain/
- **Delete Safety** — Delete uses proper forget() with checkpoint creation and event emission
- **Confirmation Dialogs** — All destructive actions (delete, bulk delete, purge, consolidate) require confirmation
- **Error Feedback** — Success/error toasts on all operations with auto-dismiss
- **Consolidation Feedback** — Shows promoted/demoted/pruned/merged counts after consolidation
- **Pagination** — Memory browser uses page-size 50 with "Load More" instead of hardcoded limit
- **Dashboard GraphStats Fix** — Proper field remapping for entity/edge counts in overview
- **Import Endpoint** — New brain/import protocol endpoint for memory restore
- **Full i18n** — All UI strings localized in English and Chinese (130+ keys)

### Checkpoint System Upgrade

- **Restore Safety & Storage Accuracy** — Interactive conflict prompting, restore fallback handling, verified backup flow, timestamp restoration, real compression and deduplication metrics, content reference counting, and storage garbage collection
- **Semantic Checkpoint Intelligence** — Symbol extraction and resolution, clone detection, pattern detection, dependency graph analysis, affected-feature/layer classification, and checkpoint risk and impact analysis
- **Advanced Diff & Comparison** — Word-level and syntax-highlighted diffs, semantic change annotations, JSON/CSS-specific diff modes, inline gutter restore actions, side-by-side checkpoint comparison, and 3-way compare against the current workspace
- **VS Code Checkpoint UX** — Checkpoint tree view with pagination, search/filter, grouping, icons, context actions, stronger native-module error handling, retry logic, and enterprise health monitoring with status bar indicators
- **Visualization & Export Tools** — Completed evolution timeline interactions, dependency graph layouts/tooltips/legends, and bulk checkpoint export as JSON, ZIP, and Markdown with progress feedback
- **Branching, Collaboration & Monitoring** — Incremental checkpoints, branch creation/switch/merge, shared bundles, cross-machine sync, audit trail, and performance dashboards for storage, restorations, and AI session productivity
- **Checkpoint Test Coverage** — Added backend, extension, and GUI test coverage for restore flows, conflicts, sessions, dashboards, comparison flows, and implementation-plan completeness



## V1.1.2

### Improvements

- knoxdev-package implementaion:  Speed up loading time faster

### Fix Bugs

- Fix 1 — The double-response bug in webviewProtocol.ts
- Fix 2 — Make the GUI retry treat

## CHANGELOG [Read More...](https://github.com/knoxchat/knoxchat/blob/main/CHANGELOG.md)