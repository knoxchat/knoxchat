# Memory Brain — Persistent AI Memory System

> **Give your AI a brain that remembers.**
>
> Memory Brain is a local-first, SQLite-powered memory system that gives Knox
> persistent, structured recall across conversations, sessions, and projects.
> It replaces the "goldfish memory" of stateless LLMs with a multi-layered
> cognitive architecture inspired by human memory science.

[**Try KnoxChat VS Code extension with Memory Brain here >>>**](https://open-vsx.org/extension/knoxchat/knoxchat)

## Table of Contents

- [Why Memory Brain?](#why-memory-brain)
- [How It Works](#how-it-works)
- [Architecture Overview](#architecture-overview)
- [Memory Types](#memory-types)
- [The 5-Tier Memory Hierarchy](#the-5-tier-memory-hierarchy)
- [Core Features](#core-features)
- [All 79 Actions Reference](#all-79-actions-reference)
- [Database Schema](#database-schema)
- [Advantages](#advantages)
- [Configuration](#configuration)
- [Test Coverage](#test-coverage)
- [Tool Integration](#tool-integration)

## Why Memory Brain?

Every conversation with an LLM starts from zero. The AI forgets your
preferences, past decisions, project context, and the patterns that
worked before. You end up repeating yourself, re-explaining your
codebase, and losing insights that took hours to reach.

**Memory Brain solves this by providing:**

| Problem | Solution |
|---------|----------|
| AI forgets everything between sessions | Semantic memory persists facts, preferences, and decisions |
| You repeat the same context every time | Context Builder auto-assembles relevant memories |
| No learning from past mistakes | Learning Engine tracks success/failure patterns |
| AI can't connect concepts across sessions | Knowledge Graph links entities with typed relationships |
| Conversation history is lost | Episodic memory stores full session transcripts |
| No way to undo destructive changes | Checkpoint system with full rollback capability |
| Memory grows unbounded | 5-tier hierarchy with Ebbinghaus decay auto-consolidation |

## How It Works

### The Memory Tool Call Flow

```
User or Agent
    │
    ▼
┌─────────────────────┐
│   memory tool call  │  action: "store", params: { title, content, ... }
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│   BrainManager      │  dispatch(action, params) → routes to handler
│   (Service Layer)   │  emits events, records audit log
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│   BrainStore        │  SQLite CRUD, BM25 search, consolidation
│   (Data Layer)      │  17 tables, 27 indexes, WAL mode
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│   brain.sqlite      │  ~/.knox/memory/brain.sqlite
│   (Local Database)  │  Your data. Your machine. No cloud.
└─────────────────────┘
```

1. An agent (or user) invokes the `memory` tool with an `action` and `params`.
2. `BrainManager.dispatch()` routes to one of **79 action handlers**.
3. The handler calls `BrainStore` (SQLite layer) or a specialized module
   (KnowledgeGraph, LearningEngine, CheckpointManager, etc.).
4. Events are emitted and logged in the audit trail.
5. A human-readable formatted string is returned to the caller.

### Automatic Memory Lifecycle

Memory Brain isn't just storage — it actively manages memory health:

```
 Store ──► Hot Tier ──► Warm Tier ──► Cold Tier ──► Pruned
           (active)     (1-7 days)   (7-90 days)   (>90 days)
              ▲                                        │
              └─── Promote (frequent retrieval) ◄──────┘

 Background: Ebbinghaus decay, spaced repetition, consolidation
```

Memories that are frequently accessed get promoted back to hot tier.
Memories that decay below importance thresholds are gradually consolidated
and eventually pruned. This keeps the system fast and relevant.

## Architecture Overview

### Module Map

```
core/context/memory/brain/
├── BrainManager.ts        # Service layer — 79-action dispatch, event bus
├── BrainStore.ts          # SQLite data layer — 17 tables, CRUD, search
├── types.ts               # TypeScript interfaces, enums, input/output models
├── ContextBuilder.ts      # Token-budgeted multi-source context assembly
├── KnowledgeGraph.ts      # Entity/edge CRUD, spreading activation traversal
├── LearningEngine.ts      # Pattern recording, Jaccard similarity suggestions
├── AutoMemory.ts          # Rule-based extraction, topic segmentation
├── CheckpointManager.ts   # Snapshots, rollback, event replay, undo
├── BatchOperations.ts     # Bulk store/delete/update, batch audit logging
├── AdvancedFeatures.ts    # SessionDiscovery, FiveTierHierarchy, SpacedRepetition,
│                          # RootCauseAnalyzer, LruCache, MetricsStorage
├── PerformanceMonitor.ts  # MetricsCollector, HealthScorer, PredictiveAnalytics,
│                          # HealingEngine, ConsolidationTracker
├── LlmMemoryService.ts    # LLM-powered entity extraction, summarization, scoring
├── LlmResilience.ts       # CircuitBreaker, rate limiting, decision caching
├── SleepConsolidation.ts  # Background scheduler, Ebbinghaus decay application
├── WorkingMemory.ts       # In-memory session state, short-term buffer
├── test-memory-tool.ts    # 94 integration tests — 100% pass rate
├── test-brain-debug.ts    # 202 ultra-comprehensive integration tests
├── test-brain-debug-modules.ts  # 59 module-level unit tests
└── test-brain-tool-calls.ts     # 88 end-to-end tool call tests
```

### Design Principles

- **Local-first**: All data in `~/.knox/memory/brain.sqlite`. Nothing leaves your machine.
- **Zero-config**: Works out of the box with sensible defaults.
- **LLM-optional**: Every LLM-enhanced feature has a rule-based fallback.
- **Audit everything**: Every write operation is logged with full undo support.
- **Fail gracefully**: Circuit breakers, rate limits, and resilient fallback chains.

## Memory Types

### 1. Semantic Memory — *What you know*

Long-term facts, decisions, and knowledge extracted from conversations.

| Category | Example |
|----------|---------|
| `fact` | "TypeScript strict mode enables stricter type checking" |
| `preference` | "User prefers tabs over spaces" |
| `decision` | "We chose PostgreSQL over MySQL for this project" |
| `code_pattern` | "Use `Result<T, E>` for error handling in Rust" |
| `error_fix` | "CORS error fixed by adding `Access-Control-Allow-Origin` header" |
| `insight` | "BM25 outperforms TF-IDF for short-document ranking" |
| `project_context` | "The knox-ms service handles user authentication" |
| `workflow` | "Deploy sequence: lint → test → build → push → tag" |
| `summary` | Session summary with key decisions and outcomes |

**Features**: BM25-inspired re-ranking, importance scoring, emotional valence
tagging (`positive`, `negative`, `neutral`, `surprise`, `urgency`, `curiosity`),
salience weighting, TTL expiration, keyword indexing.

### 2. Episodic Memory — *What happened*

Full conversation history: every user message, assistant response, tool call,
and tool result, stored per session.

- Tracks `role`, `token_count`, `importance_score`, `emotional_valence`
- Links to sessions via foreign key
- Supports cross-session search (`search_backlogs`)
- Auto-detects topic shifts within sessions (`get_session_topics`)

### 3. Knowledge Graph — *How things connect*

Entities and their relationships, modeled as a directed graph.

```
 ┌──────────┐   uses    ┌─────────────┐   depends_on   ┌──────────┐
 │  Knox    ├──────────►│ TypeScript  ├───────────────►│  SQLite  │
 │ (person) │           │ (technology)│                │  (tech)  │
 └──────────┘           └─────────────┘                └──────────┘
```

**13 entity types**: person, organization, technology, concept, project, file,
function, class, variable, location, event, product, custom.

**Graph traversal**: Spreading activation — explore from any entity and
discover connected nodes weighted by edge strength.

### 4. Learning Patterns — *What works and what doesn't*

Tracks approach success/failure rates by goal type (coding, debugging,
analysis, research, etc.). Uses Jaccard similarity to suggest the best
approach for new problems based on past outcomes.

### 5. Procedural Memory — *How to do things*

Named, ordered workflows with steps. Tracks execution count and success rate.

```json
{
  "name": "deploy-to-production",
  "steps": ["Run lint", "Run tests", "Build", "Push to registry", "Deploy"],
  "success_rate": 0.92,
  "execution_count": 13
}
```

### 6. Working Memory — *Right now*

In-memory session state and short-term buffer that flushes to disk.
Provides fast access to the current conversation context without DB queries.

## The 5-Tier Memory Hierarchy

Inspired by CPU cache hierarchies and Ebbinghaus forgetting curves.

```
┌─────────────────────────────────────────────────┐
│  TIER 1: ACTIVE   │ Currently in-session        │ Working Memory
├───────────────────┼─────────────────────────────┤
│  TIER 2: HOT      │ Frequently accessed, recent │ Full fidelity
├───────────────────┼─────────────────────────────┤
│  TIER 3: WARM     │ 1-7 days old, moderate use  │ Full fidelity
├───────────────────┼─────────────────────────────┤
│  TIER 4: COLD     │ 7-90 days old, low access   │ Compressed
├───────────────────┼─────────────────────────────┤
│  TIER 5: FROZEN   │ Archived, rarely accessed   │ Compressed
└───────────────────┴─────────────────────────────┘
```

### Automatic Transitions

| From | To | Trigger |
|------|----|---------|
| Hot | Warm | Age > 24h, importance < 0.7 |
| Warm | Cold | Age > 7 days, importance < 0.5 |
| Cold | Pruned | Age > 90 days, importance < 0.3 |
| Semantic Hot | Warm | No access for 7 days, importance < 0.6 |
| Semantic Warm | Cold | No access for 30 days, retrieval count < 3 |
| Cold | **Hot** (promote) | Retrieval count ≥ 5, accessed within 3 days |

### Spaced Repetition

Memories due for review are surfaced automatically. Each retrieval
boosts a memory's importance score, preventing decay — the more you
access it, the longer it stays.

## Core Features

### 1. Context Builder

Automatically assembles the most relevant memories within a token budget.
Used to prime the AI with context at the start of each conversation.

**Budget allocation (default 4000 tokens):**

| Source | Share | Purpose |
|--------|-------|---------|
| Semantic memories | 40% | Facts, decisions, code patterns |
| Episodic history | 25% | Recent conversation context |
| Knowledge graph | 15% | Entity relationships |
| Procedures | 10% | Relevant workflows |
| Learning patterns | 10% | Past approach success rates |

```
memory({ action: "build_context", query: "TypeScript migration", max_tokens: 4000 })
```

### 2. Checkpoint & Rollback

Full memory state snapshots with point-in-time recovery.

- **Create**: Snapshots all semantic memories, entities, and learning patterns
- **Diff**: See what changed since a checkpoint (added, removed, modified)
- **Rollback**: Restore memory to any checkpoint — conversation history preserved
- **Compress**: gzip compression for space-efficient long-term storage
- **Lifecycle cleanup**: Automatically prune old checkpoints

**Checkpoint strategies:**
- `manual` — Create checkpoints explicitly
- `time_based` — Auto-create at configurable intervals
- `adaptive` — Auto-create when change threshold is exceeded
- `hybrid` — Combine time and adaptive triggers

```
memory({ action: "create_checkpoint", label: "before-refactor-v2" })
memory({ action: "rollback_checkpoint", checkpoint_id: 4 })
```

### 3. Full Audit Trail

Every write operation is logged with target type, target ID, and details.
Supports filtering by action, time range, and target. Enables:

- **Undo**: Reverse any specific operation
- **Event replay**: Replay changes from a checkpoint forward
- **Forensics**: Understand exactly what changed and when

### 4. Auto-Extract

Pass raw text (conversation, code review, meeting notes) and Memory Brain
automatically extracts:

- Semantic memories (facts, decisions, insights)
- Named entities (people, technologies, projects)
- Emotional valence
- Topic segmentation

Works rule-based by default. LLM-enhanced extraction available when an LLM
is configured (with automatic fallback to rules if the LLM is unavailable).

### 5. Cross-Session Search

Search across ALL past conversation sessions with filters:

- Query text (full-text search)
- Date range (`from_date`, `to_date`)
- Role filter (user, assistant, tool)
- Session ID filter

### 6. Performance Monitor

Real-time health monitoring and self-healing capabilities:

- **Metrics**: Track operation counts, latency, cache hit rates
- **Health Score**: Weighted composite of fragmentation, stale data, capacity
- **Capacity Forecast**: Predict when storage or memory limits will be reached
- **Auto-Heal**: Detect and fix common issues (orphaned records, index fragmentation)
- **Trend Analysis**: Historical metrics with 24h lookback

### 7. Batch Operations

Bulk operations for efficiency:

- `batch_store` — Store multiple memories in one call
- `batch_delete` — Delete multiple memories by ID list
- `batch_update_importance` — Update importance scores in bulk
- `batch_move_tier` — Move memories between tiers
- `batch_audit_log` — Log multiple events at once

### 8. LLM-Enhanced Features (Optional)

When an LLM is connected, unlock advanced capabilities:

| Feature | With LLM | Without LLM (Fallback) |
|---------|----------|----------------------|
| Entity Extraction | NER-powered extraction | Regex + heuristic rules |
| Session Summary | Topic detection, decision extraction | Message count + title |
| Importance Scoring | Context-aware relevance | Keyword density + length |
| Post-Action Memory | LLM decides what to remember | Manual `store` calls |

All LLM features include:
- **Rate limiting**: Configurable calls/tokens per hour
- **Circuit breaker**: Automatically disables after repeated failures
- **Decision cache**: Avoids redundant LLM calls for similar inputs

## All 79 Actions Reference

### Core Memory (6 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `store` | Save a semantic memory | `category`, `title`, `content`, `keywords`, `importance`, `ttl_hours` |
| `recall` | Retrieve relevant memories (semantic + episodic + associations) | `query`, `category`, `limit` |
| `search` | Full-text search semantic memories | `query`, `category`, `limit` |
| `delete` | Remove a memory by ID | `id` |
| `build_context` | Token-budgeted context assembly | `query`, `max_tokens`, `include_graph`, `include_procedures`, `include_patterns` |
| `get_stats` | Comprehensive brain statistics | — |

### Session Management (5 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `summarize_session` | Generate session summary | `session_id`, `title`, `workspace_directory` |
| `list_sessions` | List tracked sessions | `limit`, `workspace_directory` |
| `get_session` | Session details + history | `session_id`, `limit` |
| `close_session` | Close and auto-summarize | `session_id` |
| `get_session_topics` | View topic shifts | `session_id` |

### Knowledge Graph (6 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `add_entity` | Add entity node | `name`, `entity_type`, `description`, `properties`, `confidence` |
| `search_entities` | Search entities | `query`, `entity_type`, `limit` |
| `add_edge` | Create relationship | `source_entity_id`, `target_entity_id`, `relationship`, `weight` |
| `explore_graph` | Spreading activation traversal | `entity_id`, `depth`, `min_weight` |
| `get_graph_stats` | Graph statistics | — |
| `extract_entities` | Auto-extract from text | `text` |

### Learning Engine (3 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `learn_pattern` | Record success/failure | `goal_type`, `description`, `success`, `tokens_used`, `pattern_signature` |
| `suggest_approach` | Get approach suggestions | `query`, `goal_type` |
| `get_patterns` | List learned patterns | `goal_type`, `limit` |

### Procedural Memory (3 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `store_procedure` | Store named workflow | `name`, `description`, `steps`, `trigger_pattern`, `category` |
| `get_procedures` | List procedures | `category`, `limit` |
| `execute_procedure` | Record execution | `procedure_id`, `success` |

### Tags & Collections (6 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `tag` | Add tag to memory | `memory_type`, `memory_id`, `tag` |
| `untag` | Remove tag | `memory_type`, `memory_id`, `tag` |
| `search_by_tag` | Find by tag | `tag`, `memory_type` |
| `create_collection` | Create collection | `name`, `description` |
| `list_collections` | List collections | — |
| `add_to_collection` | Add memory to collection | `collection_id`, `memory_type`, `memory_id` |

### Associations (1 action)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `associate` | Link two memories | `source_type`, `source_id`, `target_type`, `target_id`, `relationship`, `strength` |

### Auto-Memory (1 action)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `auto_extract` | Extract memories + entities from text | `text`, `session_id` |

### Maintenance (5 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `consolidate` | Run memory tiering + cleanup | — |
| `get_health` | Health check with recommendations | — |
| `optimize` | VACUUM + REINDEX + ANALYZE | — |
| `get_config` | View configuration | `key` |
| `update_config` | Update configuration | `key`, `value` |

### Checkpoint & Rollback (5 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `create_checkpoint` | Snapshot memory state | `label` |
| `list_checkpoints` | List all checkpoints | `limit` |
| `rollback_checkpoint` | Restore to checkpoint | `checkpoint_id` |
| `delete_checkpoint` | Remove checkpoint | `checkpoint_id` |
| `compress_checkpoint` | gzip-compress snapshot | `checkpoint_id` |

### Audit & Undo (4 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `get_audit_log` | View audit trail | `action`, `target_type`, `limit` |
| `undo_operation` | Reverse a specific operation | `audit_id` |
| `get_undoable_operations` | List undo-eligible entries | `limit` |
| `replay_events` | Replay events from checkpoint | `checkpoint_id`, `from_time`, `to_time` |

### Cross-Session Search (1 action)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `search_backlogs` | Search across all sessions | `query`, `from_date`, `to_date`, `role`, `session_id`, `limit` |

### Export / Import (2 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `export` | Export all memories to JSON | — |
| `import` | Import memories from JSON | `data` |

### Checkpoint Strategy (4 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `checkpoint_strategy_config` | Get strategy configuration | — |
| `update_checkpoint_strategy` | Set strategy mode | `mode`, `interval_minutes`, `max_checkpoints`, `compress_snapshots` |
| `checkpoint_lifecycle_cleanup` | Prune old checkpoints | — |
| `diff_checkpoint` | Show changes since checkpoint | `checkpoint_id` |

### Batch Operations (5 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `batch_store` | Store multiple memories | `items[]` |
| `batch_delete` | Delete by ID list | `ids[]` |
| `batch_update_importance` | Bulk importance update | `updates[]` |
| `batch_move_tier` | Move memories between tiers | `ids[]`, `target_tier` |
| `batch_audit_log` | Log multiple events | `events[]` |

### Advanced Features (7 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `find_related_sessions` | Discover related sessions | `session_id`, `limit` |
| `five_tier_consolidate` | Run 5-tier consolidation | — |
| `get_tier_distribution` | Memory counts per tier | — |
| `get_tier_configs` | View tier configurations | — |
| `update_tier_config` | Update tier settings | `tier`, `max_age_hours`, `importance_threshold` |
| `root_cause_analysis` | Analyze memory issues | — |
| `get_review_due` | Spaced repetition due list | `limit` |

### Performance & Health (11 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `boost_memory` | Spaced repetition boost | `memory_id` |
| `get_cache_stats` | LRU cache statistics | — |
| `clear_cache` | Clear LRU cache | — |
| `get_metrics` | Performance metrics | — |
| `get_health_score` | Weighted health score | — |
| `get_capacity_forecast` | Storage growth prediction | — |
| `heal` | Auto-heal detected issues | — |
| `get_healing_strategies` | Healing effectiveness | — |
| `get_consolidation_stats` | Consolidation metadata | — |
| `store_metrics_snapshot` | Store periodic metrics | — |
| `get_metrics_trend` | Historical metrics trend | `hours` |

### LLM-Enhanced (4 actions)

| Action | Description | Key Parameters |
|--------|-------------|----------------|
| `llm_extract_entities` | LLM NER entity extraction | `text` |
| `llm_summarize_session` | LLM session summary | `session_id` |
| `llm_evaluate_importance` | LLM importance scoring | `content`, `context` |
| `llm_post_action_memory` | LLM post-action memory | `action_type`, `input`, `output`, `context` |

## Database Schema

**17 tables** with **27 optimized indexes**, running on SQLite with WAL mode
and foreign key constraints enabled.

```
brain_sessions           — Conversation sessions
brain_episodic           — Conversation turns (messages, tool calls)
brain_semantic           — Long-term factual memories
brain_associations       — Cross-memory links
brain_entities           — Knowledge graph nodes
brain_graph_edges        — Knowledge graph relationships
brain_learning_patterns  — Success/failure patterns
brain_procedures         — Named workflows
brain_tags               — Memory tags (many-to-many)
brain_collections        — Named memory groups
brain_collection_items   — Collection membership
brain_config             — Configuration key/value store
brain_checkpoints        — Memory state snapshots
brain_rate_limits        — LLM rate limit tracking
brain_audit_log          — Full audit trail
brain_session_topics     — Auto-detected topic shifts
brain_metrics_snapshots  — Performance metrics history
```

**Storage**: Single file at `~/.knox/memory/brain.sqlite`. Typical size
is 1-10 MB after months of active use.

## Advantages

### vs. Vector Databases (Pinecone, Chroma, Weaviate)

| Aspect | Memory Brain | Vector DB |
|--------|-------------|-----------|
| **Setup** | Zero config, single SQLite file | Requires server, API keys, embeddings |
| **Privacy** | 100% local | Cloud-hosted or self-hosted server |
| **Cost** | Free | Embedding API costs + hosting |
| **Latency** | < 1ms for most queries | Network round-trip + embedding |
| **Search** | BM25 + TF-IDF (no embeddings needed) | Semantic vector similarity |
| **Structure** | Knowledge graph + typed categories | Flat vector space |
| **Portability** | Copy one file | Export/import pipeline |

### vs. File-Based Memory (.md files, JSON)

| Aspect | Memory Brain | File-Based |
|--------|-------------|------------|
| **Search** | Full-text with BM25 ranking | grep / string match |
| **Organization** | 9 semantic categories + tags + collections | Manual folder structure |
| **Lifecycle** | Automatic tiering + decay + promotion | Manual cleanup |
| **Relationships** | Knowledge graph + associations | None |
| **Undo** | Full audit trail + checkpoint rollback | Git history |
| **Performance** | 27 indexed queries, WAL mode | File I/O bottleneck |

### vs. No Memory (Stateless LLM)

| Capability | With Memory Brain | Without |
|------------|-------------------|---------|
| Remember past decisions | ✓ Automatically | ✗ Repeat every session |
| Learn from mistakes | ✓ Pattern tracking | ✗ Same mistakes repeatedly |
| Connect concepts | ✓ Knowledge graph | ✗ No cross-reference |
| Resume interrupted work | ✓ Session history | ✗ Start from scratch |
| Personalize responses | ✓ Preferences stored | ✗ Generic responses |
| Scale with usage | ✓ Auto-consolidation | N/A |

### Key Pros

1. **Privacy-first** — Everything stored locally. No API calls for core features.
2. **Self-maintaining** — Automatic consolidation, decay, promotion, and healing.
3. **Resilient** — Circuit breakers, fallbacks, rate limits. No single point of failure.
4. **Observable** — Full audit trail, health metrics, performance monitoring.
5. **Reversible** — Checkpoint/rollback for any destructive operation.
6. **Token-efficient** — Context Builder assembles only what fits in your budget.
7. **Structured** — Not just text blobs: typed entities, weighted edges, categorized facts.
8. **Tested** — 443 integration tests across 4 test suites covering all 79 actions at 100% pass rate.
9. **LLM-optional** — Every feature works without an LLM. Enhanced features degrade gracefully.
10. **Portable** — Export/import everything as JSON. Move between machines freely.

## Configuration

All settings are stored in `brain_config` table and can be read/updated
via tool calls.

| Key | Default | Description |
|-----|---------|-------------|
| `auto_extract_enabled` | `true` | Auto-extract memories from conversations |
| `consolidation_interval_hours` | `24` | Hours between auto-consolidation runs |
| `max_episodic_per_session` | `1000` | Max episodic entries per session |
| `max_hot_memories` | `500` | Hot memory cap (triggers consolidation) |
| `hot_to_warm_hours` | `24` | Hours before episodic demotion to warm |
| `warm_to_cold_days` | `7` | Days before demotion to cold |
| `cold_prune_days` | `90` | Days before cold memories are pruned |
| `graph_enabled` | `true` | Enable knowledge graph |
| `learning_enabled` | `true` | Enable learning pattern tracking |
| `context_max_tokens` | `4000` | Default token budget for context builder |
| `llm_calls_per_hour_limit` | `60` | Max LLM calls per hour |
| `llm_tokens_per_hour_limit` | `50000` | Max LLM tokens per hour |

```
memory({ action: "get_config" })
memory({ action: "update_config", key: "max_hot_memories", value: "1000" })
```

## Test Coverage

Four test suites exercise **443 integration tests** across all 79 actions,
calling every action through the actual dispatch pipeline against a real
SQLite database.

### Suite 1: `test-memory-tool.ts` — 94 tests

The original integration test covering all core dispatch actions.

| Section | Tests | Coverage |
|---------|-------|---------|
| Core Memory Operations | 6 | store, recall, search, delete |
| Session Operations | 2 | list, stats |
| Knowledge Graph | 8 | entities, edges, search, traverse, extract |
| Learning Engine | 4 | patterns, suggestions |
| Procedural Memory | 3 | store, list, execute |
| Auto-Extract | 1 | Full extraction pipeline |
| Context Builder | 2 | Full and minimal context |
| Tags & Collections | 5 | tag/untag, search, collections |
| Associations | 1 | Cross-memory links |
| Maintenance | 3 | health, optimize, config, consolidate |
| Checkpoint & Rollback | 5 | create, diff, compress, rollback, verify |
| Audit & Topics | 2 | Audit log, filtering |
| Cross-Session Search | 1 | Backlog search |
| Export / Import | 1 | JSON export |
| Checkpoint Strategy & Batch | 6 | Strategy config, batch operations |
| Advanced Features (Tier D) | 7 | 5-tier, spaced repetition, cache |
| Performance Monitor | 7 | Metrics, health, forecast, healing |
| LLM-Enhanced (Fallback) | 2 | Entity extraction, importance scoring |
| Edge Cases | 5 | Non-existent IDs, empty queries |
| Batch Delete | 2 | Batch delete, unknown action |
| **Total** | **94** | **100% pass rate** |

### Suite 2: `test-brain-debug.ts` — 202 tests

Ultra-comprehensive integration tests exercising every dispatch action with
deep output validation, edge cases, and multi-step workflows.

| Section | Tests | Coverage |
|---------|-------|---------|
| Core Memory (CRUD) | 12 | store, recall, search, delete, duplicates |
| Session Lifecycle | 10 | create, list, get, close, summarize, topics |
| Knowledge Graph | 16 | entities, edges, traversal, stats, extraction |
| Learning Engine | 10 | patterns, suggestions, goal types, similarity |
| Procedural Memory | 8 | store, list, execute, success tracking |
| Auto-Extract Pipeline | 6 | text extraction, entity detection, valence |
| Context Builder | 8 | token budgets, source allocation, filters |
| Tags & Collections | 12 | tag, untag, search, collections, membership |
| Associations | 6 | cross-memory links, relationship types |
| Maintenance | 10 | health, optimize, config CRUD, consolidation |
| Checkpoint & Rollback | 14 | create, diff, compress, rollback, lifecycle |
| Audit Trail & Undo | 12 | audit log, filtering, undo, replay |
| Cross-Session Search | 6 | backlog search, date/role filters |
| Export / Import | 4 | JSON export, import, round-trip |
| Batch Operations | 14 | batch store, delete, update, move, audit |
| Advanced Features | 18 | 5-tier, spaced rep, cache, root cause |
| Performance Monitor | 20 | metrics, health score, forecast, healing |
| LLM-Enhanced Fallback | 8 | entity extraction, summarize, importance |
| Checkpoint Strategies | 10 | manual, time, adaptive, hybrid modes |
| Edge Cases & Errors | 18 | invalid IDs, empty inputs, concurrency |
| **Total** | **202** | **100% pass rate** |

### Suite 3: `test-brain-debug-modules.ts` — 59 tests

Module-level unit tests for individual classes (BrainStore, KnowledgeGraph,
LearningEngine, ContextBuilder, CheckpointManager, BatchOperations,
AdvancedFeatures, PerformanceMonitor).

| Module | Tests | Coverage |
|--------|-------|---------|
| BrainStore | 10 | Table creation, CRUD, indexes, WAL mode |
| KnowledgeGraph | 8 | Entity/edge CRUD, spreading activation |
| LearningEngine | 6 | Pattern recording, Jaccard similarity |
| ContextBuilder | 8 | Token budgeting, source allocation |
| CheckpointManager | 7 | Snapshot, rollback, compression |
| BatchOperations | 6 | Bulk store, delete, update, move |
| AdvancedFeatures | 8 | 5-tier hierarchy, spaced rep, LRU cache |
| PerformanceMonitor | 6 | MetricsCollector, HealthScorer, healing |
| **Total** | **59** | **100% pass rate** |

### Suite 4: `test-brain-tool-calls.ts` — 88 tests

End-to-end tests verifying all 79 tool enum actions work through
`BrainManager.dispatch()`, organized by action tier.

| Tier | Tests | Coverage |
|------|-------|---------|
| Tier A — Core Memory | 6 | store, recall, search, delete, context, stats |
| Tier A — Sessions | 5 | summarize, list, get, close, topics |
| Tier A — Maintenance | 5 | consolidate, health, optimize, config |
| Tier A — Knowledge Graph | 6 | entity, edge, search, explore, stats, extract |
| Tier A — Learning Engine | 3 | learn, suggest, get patterns |
| Tier A — Procedures | 3 | store, list, execute |
| Tier A — Auto-Extract | 1 | Full pipeline |
| Tier A — Tags & Collections | 6 | tag, untag, search, collection CRUD |
| Tier A — Export/Import | 2 | JSON round-trip |
| Tier A — Self-Management | 5 | checkpoint, rollback, audit, undo |
| Tier A — LLM-Enhanced | 4 | entity extract, summarize, importance, post-action |
| Tier A — Checkpoints | 5 | list, delete, compress, diff, replay |
| Tier B — Performance | 6 | metrics, health score, forecast, heal, strategies |
| Tier C — Checkpoint Strategies | 4 | config, update, lifecycle, consolidation stats |
| Tier C — Event Replay/Undo | 3 | replay, undo, undoable operations |
| Tier C — Batch Operations | 5 | store, delete, update, move, audit |
| Tier D — Hierarchy | 5 | 5-tier, distribution, configs, update, related |
| Tier D — Root Cause & Rep | 3 | root cause, review due, boost |
| Tier D — Cache & Metrics | 5 | cache stats, clear, metrics snapshot, trend |
| Coverage Verification | 1 | Validates all 79 enum values dispatched |
| **Total** | **88** | **100% pass rate** |

### Summary

| Suite | Tests | Focus |
|-------|-------|-------|
| test-memory-tool.ts | 94 | Original integration tests |
| test-brain-debug.ts | 202 | Ultra-comprehensive deep validation |
| test-brain-debug-modules.ts | 59 | Module-level unit tests |
| test-brain-tool-calls.ts | 88 | End-to-end tool enum coverage |
| **Grand Total** | **443** | **All 79 actions — 100% pass rate** |

Each test suite uses a fresh temporary database, validates output format,
and cleans up after itself.

## Quick Start

### Store a memory

```
memory({ action: "store", category: "decision", title: "Use SQLite for memory",
         content: "Chose SQLite over Postgres for zero-config local storage.",
         keywords: "database,sqlite,architecture", importance: 0.9 })
```

### Recall relevant context

```
memory({ action: "recall", query: "database choice", limit: 5 })
```

### Build pre-conversation context

```
memory({ action: "build_context", query: "TypeScript migration project",
         max_tokens: 4000 })
```

### Create a safety checkpoint before risky work

```
memory({ action: "create_checkpoint", label: "before-major-refactor" })
// ... do risky work ...
memory({ action: "rollback_checkpoint", checkpoint_id: 1 })  // oops, undo
```

### Track what works

```
memory({ action: "learn_pattern", goal_type: "debugging",
         description: "Binary search through git history to isolate regression",
         success: true, tokens_used: 1200 })

// Next time:
memory({ action: "suggest_approach", query: "find regression", goal_type: "debugging" })
```

## Tool Integration

Memory Brain is fully integrated into the Knox tool call system as a
first-class built-in tool with its own UI category, interceptor validation,
and permission controls.

### Registration Architecture

```
core/tools/definitions/memory.ts    → Tool definition (schema, 79 actions, params)
core/tools/implementations/memory.ts → Implementation (routes to BrainManager.dispatch)
core/tools/callTool.ts              → Dispatch (BuiltInToolNames.Memory → memoryImpl)
core/tools/builtIn.ts               → Enum (Memory = "builtin_memory")
core/tools/index.ts                 → allTools array (includes memoryTool)
core/config/load.ts                 → Config (tools: [...allTools])
```

### Tool Identity

| Property | Value |
|----------|-------|
| **Internal name** | `builtin_memory` |
| **Display title** | `Memory` |
| **UI category** | `[Memory]` |
| **Permission group** | `Permissions` (built-in) |
| **Default permission** | `allowedWithoutPermission` (Auto-Approve) |
| **Read-only** | `false` |

### How It Appears in the UI

The memory tool shows up in the **Tools** panel under the **Permissions** group:

```
● Permissions  [12]                              ⬤ ON
  [Files] Read File                        Auto-Approve
  [Files] Create New File                  Auto-Approve
  [Terminal] Run Terminal Command          Auto-Approve
  [Files] View Subdirectory                Auto-Approve
  [Files] View Repo Structure             Auto-Approve
  [Search] Exact Search                    Auto-Approve
  [Search] Web Search                      Auto-Approve
  [Diff] View Diff                         Auto-Approve
  [Files] Read Currently Open File         Auto-Approve
  [Tool] Load Skill                        Auto-Approve
  [Tool] LSP                               Auto-Approve
  [Memory] Memory                          Auto-Approve
```

### File Locations

| Component | Path | Purpose |
|-----------|------|---------|
| Tool definition | `core/tools/definitions/memory.ts` | LLM-facing schema: 79 actions, all parameters |
| Tool implementation | `core/tools/implementations/memory.ts` | Routes `args.action` → `BrainManager.dispatch()` |
| Tool dispatch | `core/tools/callTool.ts` | `BuiltInToolNames.Memory` → `memoryImpl()` |
| Tool enum | `core/tools/builtIn.ts` | `Memory = "builtin_memory"` |
| Tool registry | `core/tools/index.ts` | `allTools` array, `allAvailableTools` |
| Config loader | `core/config/load.ts` | `tools: [...allTools]` in `KnoxConfig` |
| Interceptor | `extensions/vscode/src/agent/ToolCallInterceptor.ts` | Validation, logging, metrics |
| UI permissions | `gui/src/redux/slices/uiSlice.ts` | `toolSettings` initial state |
| UI category | `gui/src/util/toolNameFormatter.ts` | `[Memory]` category prefix |
| UI dialog | `gui/src/components/.../ToolPermissionsDialog.tsx` | Renders tool permission toggles |

### Permission States

Each tool can be in one of three permission states, cycled by clicking:

| State | Behavior |
|-------|----------|
| `allowedWithoutPermission` | **Auto-Approve** — tool calls execute without confirmation |
| `allowedWithPermission` | **Ask** — user must approve each tool call |
| `disabled` | **Disabled** — tool is never called |

The entire `Permissions` group can also be toggled on/off via the group switch.

### Tool Call Flow (End-to-End)

```
1. LLM generates tool call:
   { name: "builtin_memory", arguments: { action: "store", ... } }

2. ToolCallInterceptor validates:
   - Tool exists in registry ✓
   - Required params present (action) ✓
   - Logs call to history, tracks metrics

3. Permission check:
   - toolSettings["builtin_memory"] === "allowedWithoutPermission"
   - Auto-approved (no user prompt)

4. callTool() dispatches:
   - Matches BuiltInToolNames.Memory
   - Calls memoryImpl(args, extras)

5. memoryImpl() routes:
   - Gets BrainManager singleton
   - Calls brain.dispatch(args.action, args)

6. BrainManager.dispatch() handles:
   - Routes to appropriate handler (store, recall, search, etc.)
   - Records audit trail
   - Emits events
   - Returns formatted string

7. Result returned to LLM as ContextItem[]
```

[**Try KnoxChat VS Code extension with Memory Brain here >>>**](https://open-vsx.org/extension/knoxchat/knoxchat)

*Memory Brain is part of the KnoxCore platform. All data stays on your machine.*
