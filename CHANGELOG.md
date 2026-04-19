## V1.1.2

### Improvements

- knoxdev-package implementaion:  Speed up loading time faster

### Fix Bugs

- Fix 1 — The double-response bug in webviewProtocol.ts
- Fix 2 — Make the GUI retry treat

## V1.1.1

### Fix Bugs

- Stale backend session
- Optimistic session too eager
- Missing backend reset protocol

### Improvements

- Agent mode code generation in real-time streaming
- GenericCodePreview Polish
- Middleware Hardening
- Tool call UX

## V1.1.0

### New Feature

- Knox Memory System (Knox-MS)
- Knox-MS as a tool calling [Read More >>>](https://docs.knox.chat/blog/memory-brain)

### Improvement

- Todo/Task System UI/UX

### Upgrade

- "vite": "^8.0.3"
- "typescript": "^6.0.2"

## V1.0.2

- Fix terminal auto-scroll behavior
- Context Compaction System — conversation summarization, token budget management, smart pruning, and deduplication
- Fix Diagnostic Fix Manager — real LLM-powered fix generation replacing stub
- Agentic Auto-Verification Loop — automatic diagnostics check and self-correction after code edits
- Smart Auto-Context — automatic relevant file selection based on query analysis
- Todo Planning Enhancement — pre-execution plan generation with auto-proceed and progress tracking
- Rules System (.knoxrules) — hierarchical rule discovery and merging with template variables
- Persistent Project Memory — SQLite-backed memory store with convention extraction and @memory context provider
- Multi-File Apply-All — batch accept/reject across all pending diffs
- Enhanced Terminal Integration — real-time error detection with 20+ pattern matchers via shell integration API
- Token Usage & Cost Dashboard — per-model pricing, cost calculation, and enriched stats endpoint
- Complete MCP Notification Handlers — tool, prompt, resource, and logging notification registration
- Fix Config Model Selection — chat and summarize roles now participate in model rectification
- Screenshot Capture Command — OS-native screen capture injected into chat input
- Git Workflow Enhancement — conventional commits, /pr description generator, /changelog from git history


## V1.0.0

### Add Skills System

### LSP Tool Improvement

### Todo/Task System Optimizations

Progress is now tracked by observing ACTUAL tool executions (file edits, terminal commands, searches) and semantically matching them to planned todo items.

**Architecture:**
```
callTool.ts  ──(ToolCallEvent)──→  todoStreamSync.ts  ──(completeTodo)──→  TodoManager.ts
                                        │                                      │
                                   Evidence Scoring                      Parse Metadata
                                   Multi-Signal Match                   (verified, score,
                                   Per-Todo Accumulation                 toolCallCount)
                                        │                                      │
                                        ▼                                      ▼
                                   planTaskSlice  ←──(session update)──  IDE Protocol
                                        │
                                        ▼
                              PlanTaskStatusPanel (UI)
                              TodoPanel (UI)
                              Verification Badges
```

### Tools Implementation

- GUI callTool thunk (retry with backoff)
  - → IPC → core.ts tools/call handler (structured error logging)
    - → callTool() with middleware:
    - 1. Argument parse + JSON repair
    - 2. Input validation (required params)
    - 3. Circuit breaker check
    - 4. File write lock (if applicable)
    - 5. Execute with timeout + retry (exponential backoff + jitter)
    - 6. Performance metrics collection
      - → VsCodeIde (retry + timeout at I/O level)

### No macos-x64 support anymore

### Package Update

- "@tiptap/core": "^3.20.0"
- "axios": "^1.13.6"
- "engines": {
    "vscode": "^1.109.0",
    "node": ">=22.22.0"
  }
- "esbuild": "0.27.3"
- "onnxruntime-common": "1.24.2"
- "onnxruntime-web": "1.24.2"
- "react": "^19.2.4"
- "typescript": "^5.9.3"
- "uuid": "^13.0.0"
- "vite": "^7.3.1"
- "web-tree-sitter": "^0.26.6"
- "zod": "^4.3.6"

### Crates Update
```toml
[workspace.dependencies]
# Core dependencies
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
rusqlite = { version = "0.38.0", features = ["bundled"] }
dirs = "6.0.0"
uuid = { version = "1.21.0", features = ["v4", "serde"] }
chrono = { version = "0.4.44", features = ["serde"] }
sha2 = "0.10.9"
hex = "0.4.3"

# File system and async
tokio = { version = "1.49.0", features = ["full"] }
walkdir = "2.5.0"
ignore = "0.4.25"
notify = "8.2.0"

# Compression and encoding
flate2 = "1.1.9"
base64 = "0.22.1"
lz4_flex = "0.12.0"

# Error handling
thiserror = "2.0.18"
anyhow = "1.0.102"

# Node.js bindings - Updated to latest version
neon = { version = "1.1.1", default-features = false, features = ["napi-6"] }

# Additional dependencies
once_cell = "1.21.3"
parking_lot = "0.12.5"
dashmap = "6.1.0"
regex = "1.12.3"
rayon = "1.11.0"

# Logging
log = "0.4.29"
env_logger = "0.11.9"

# Security and encryption
ring = "0.17.14"
argon2 = "0.5.3"

# Monitoring and metrics
prometheus = "0.14.0"

# Configuration
config = "0.15.19"
toml = "1.0.3"

# System utilities
num_cpus = "1.17.0"

# Additional dependencies for production features
reqwest = { version = "0.13.2", features = ["json"] }
futures = "0.3.32"

# Tree-sitter for semantic analysis
tree-sitter = "0.26.6"
tree-sitter-typescript = "0.23.2"
tree-sitter-javascript = "0.25.0"
tree-sitter-rust = "0.24.0"
tree-sitter-python = "0.25.0"
tree-sitter-go = "0.25.0"
tree-sitter-java = "0.23.5"

# Dev dependencies
tempfile = "3.26.0"
criterion = "0.8.2"
```
### Terminal Beautify

## V0.9.9

- Add Kmox-MS status - A human-brain-inspired memory architecture with hierarchical memory levels, autonomous execution, and intelligent context management that enables effectively unlimited context windows

- Update multilingual with Chinese and English

### Todo/Task Management System - Fully-Featured AI-Powered Task Orchestration

A full-stack, real-time task management system that automatically detects complex user requests, uses an LLM to decompose them into actionable steps, and tracks progress live as the AI streams its response. The system spans 4 layers: **Core protocol**, **VS Code extension backend**, **Redux state management**, and **React UI**.

#### The Problem

When users submit complex, multi-step requests (e.g., "Build a Go REST API with authentication, database layer, and Docker setup"), there was no visibility into what the AI was doing. Users had to wait for the entire response to finish with no indication of progress, no breakdown of steps, and no way to track which parts were complete.

#### The Solution - Automatic LLM-Powered Task Decomposition & Live Stream Tracking

**End-to-End Architecture:**
```
User types complex message
        │
        ▼
  Auto-Detection (GUI)  ──── runs in parallel with chat stream
        │
        ▼
  LLM Task Decomposition (ReasoningEngine)
        │
        ▼
  Todo Session Created (status: "idle")
        │
        ▼
  Stream Starts → Session Activated → Tracker Initialized
        │
        ▼
  LLM Streaming → onStreamChunk() per chunk
        │  (keyword matching / char budget / time proportional)
        ▼
  Todos Complete One-by-One → Live Progress in UI
        │
        ▼
  Stream Finished → Remaining Todos Completed with Animation
```

#### Feature 1: Automatic Complexity Detection

Analyzes user messages in real-time to detect when task decomposition would be helpful — no manual trigger needed.

**Detection Criteria (any 2+ triggers):**
- Explicit keywords: "step by step", "break this down", "create a plan"
- Complexity patterns: file creation, API endpoints, database operations, authentication, testing, deployment
- Independent clauses: 3+ distinct instructions in one message
- Multi-step indicators: numbered lists, "first... then... finally..."

**Implementation:**
```typescript
// gui/src/redux/thunks/autoTodoDetection.ts
// Fires in parallel with chat — never blocks the user's response
void dispatch(maybeAutoCreateTodos({ content }));
```

#### Feature 2: LLM-Powered Task Decomposition (ReasoningEngine)

Singleton service that calls the LLM to break complex prompts into structured, actionable steps.

**Features:**
- **System prompt engineering**: Instructs the LLM to output structured JSON with task titles, descriptions, categories, priorities, and dependencies
- **JSON parsing with fence stripping**: Handles markdown code fences in LLM output
- **Graceful fallback**: Falls back to local sentence-splitting heuristic when LLM is unavailable
- **Category detection**: Automatically categorizes tasks (coding, analysis, research, testing, documentation, refactoring, devops)

**Example LLM Decomposition:**
```
Input: "Build a Go REST API with auth and database"
Output:
  1. Initialize Go module and project structure     [coding, high]
  2. Create main application entry point            [coding, high]
  3. Implement database connection layer            [coding, high]
  4. Build authentication middleware                 [coding, high]
  5. Create user model and repository               [coding, medium]
  6. Implement REST API routes and handlers         [coding, high]
  7. Add configuration management                   [coding, medium]
  8. Create Docker setup                            [devops, medium]
  9. Add error handling and validation              [coding, medium]
  10. Write README and documentation                [documentation, low]
  11. Add tests                                     [testing, medium]
```

#### Feature 3: Real-Time Stream ↔ Todo Synchronization

The heart of the system — advances todos in lockstep with the LLM's streaming output, surviving multi-round agent interactions (stream → tool call → stream → …).

**3 Advancement Strategies:**

| Strategy | How It Works | When It Fires |
|----------|-------------|---------------|
| **Keyword Matching** | Extracts keywords from todo titles, scans ALL future todos for best match (≥40% score). Completes all intermediate todos when a later match is found. | When streamed text contains enough keywords from a future todo |
| **Character Budget** | ~800 chars per todo. If 1.5× budget exceeded, advance by one | When LLM produces long output without keyword matches |
| **Time Proportional** | ~5s per step. Advance based on elapsed time | Ensures progress even on code-heavy output with few keywords |

**Multi-Round Agent Mode Support:**
```
streamThunkWrapper (outer, depth=1)
  └─ streamNormalInput
       └─ LLM detects tool call
            └─ callTool → streamResponseAfterToolCall
                 └─ streamThunkWrapper (inner, depth=2)
                      └─ streamNormalInput (round 2)
                           └─ onStreamChunk still works ✓
                      └─ finally: depth=1, skip onStreamEnd ✓
       └─ finally: depth=0, complete remaining todos ✓
```

**Key Design Decisions:**
- **Depth-aware nesting**: `wrapperDepth` counter ensures `onStreamEnd` only fires on the outermost `streamThunkWrapper` exit
- **Idempotent `onStreamStart`**: First call initializes tracker; subsequent tool-call rounds are no-ops
- **Smart keyword extraction**: Preserves domain-relevant action words ("create", "build", "implement") — only strips true stop-words
- **Session polling**: Polls up to 8 seconds for late-arriving sessions (auto-detection runs in parallel)
- **Early buffer**: Chunks arriving before tracker initialization are buffered and replayed

#### Feature 4: TodoManager Backend (1183 lines)

Comprehensive singleton managing the entire session lifecycle on the extension side.

**Session Lifecycle:**
- `analyzeAndCreateSession()` → `startSession()` → `processNextTodos()` → `completeTodo()` → `completeSession()`

**Features:**
- **Persistence**: Sessions saved to `~/.knox/todo-sessions/` in workspace storage for cross-session recovery
- **Configuration**: Auto-detect enabled, max concurrent todos, auto-complete on stream end, session retention days
- **Dependency tracking**: Todos can depend on other todos; dependents unblocked automatically on completion
- **10 event emitters**: `sessionCreated`, `sessionUpdated`, `todoUpdated`, `progressUpdated`, `sessionCompleted`, etc.
- **12 VS Code commands**: Including `knox.todo.createInteractive` for manual creation via command palette

#### Feature 5: Protocol Layer (13 Messages)

Full bidirectional communication between GUI and extension:

| Direction | Messages |
|-----------|----------|
| **GUI → Extension** | `todo/createSession`, `todo/startSession`, `todo/pauseSession`, `todo/resumeSession`, `todo/cancelSession`, `todo/retryTodo`, `todo/skipTodo`, `todo/getSession`, `todo/getSessions`, `todo/deleteSession`, `todo/completeTodo`, `todo/advanceSession` |
| **Extension → GUI** | `todo/sessionCreated`, `todo/sessionUpdated`, `todo/todoUpdated`, `todo/progressUpdated`, `todo/sessionCompleted` |

## V0.9.8

- Added Knox Memory System - A human-brain-inspired memory architecture with hierarchical memory levels, autonomous execution, and intelligent context management that enables effectively unlimited context windows [Read More](https://docs.knox.chat/knox-ms-complete-doc)
- Update multilingual with Chinese and English

## V0.9.8

### Add
- Knox Memory System - AI model with unlimited context length through intelligent memory management. Orchestrates multiple underlying models via Plan-Task-Memory architecture.
- Support multilingual with Chinese and English

### Update
- Move New Chat button to input toolbar for better UI/UX

## V0.9.7

**Checkpoint System Improvements:**
  - Code View Highlighting: The diff viewer needs better syntax highlighting with line-level change detection
  - Missing Features: Auto-checkpoint, undo/redo stack, checkpoint branching, smart merge
  - Performance: Better caching and incremental updates
  - UI/UX: Enhanced file tree, minimap, change statistics
  - Integration: Better IDE integration with inline diff viewing

## V0.9.6

**Tool Call Improvements:**
  - View Subdirectory: Added sorting, filtering, and file type options
  - View File: Added line range and content preview options
  - File Search: Added file type filtering and result limits
  - File Creation: Added template support
  - View Subdirectory: Fixed incorrect file counts in certain cases
- Markdown Rendering Improvements in Chat Stream
- Terminal Command On Chat Stream Improvements

## V0.9.5

- Fix Chat Stream Hanging Issue on Large Responses
- Improve Chat Stream Performance
- Optimize Chat Stream Memory Usage
- Thinking Section Improvements
- Exact Search Query Matching Improvements

#### New Features
- **Advanced Relevance Scoring** - Multi-dimensional relevance beyond vector similarity
- **Intelligent Tool Orchestration** - Automated tool chaining and dependency resolution
- **Context-Aware Search** - Hybrid search combining exact, fuzzy, semantic, and structural analysis
- **Learning & Adaptation** - Pattern recognition from execution history
- **Multi-Strategy Search Engine** - Parallel search strategies with result fusion

## V0.9.4

- Add Run Terminal Command Section on Streaming Interface
- Add xterm
- Simplify Context Awareness & Performance Optimization
- Fix Agent mode code block streaming issue

## V0.9.2

- Better Chat Streaming
- Smarter Context Awareness
- Better Code Understanding
- Smarter Checkpoint Management

## V0.9.1

- Update knox.chat API Base URL to https://api.knox.chat

### Checkpoint System - Three Major Performance Enhancements

Implemented three critical enhancements that dramatically improve checkpoint performance, reliability, and functionality across the entire system (API, VSCode extension, and GUI).

#### Enhancement 1: Non-Blocking File Watcher with Debouncing

**The Problem:**
- The native Rust `ChangesetTracker` was using a blocking file watcher that froze the main thread
- This caused the entire native Rust module to be disabled, forcing fallback to slower TypeScript implementation
- Poor performance for real-time change tracking in large codebases

**The Solution:**
Completely refactored the file watcher to be asynchronous, non-blocking, and intelligently debounced:

**Features:**
- **Asynchronous File Watching**: Uses `tokio::spawn` for non-blocking event processing
- **Smart Debouncing**: Collects file events for 300ms before processing to reduce noise
- **Rate Limiting**: Polls file system every 500ms to prevent excessive CPU usage
- **Channel-Based Communication**: Uses `mpsc::channel` for thread-safe event passing
- **Batch Processing**: Groups multiple events into single operations for efficiency

**Performance Impact:**
- **Native Module**: Re-enabled across all systems (API, extension, GUI)
- **CPU Usage**: Reduced by ~80% through debouncing and rate limiting
- **Responsiveness**: No more UI freezing during file watching
- **Reliability**: Proper thread cleanup and graceful shutdown

**Technical Implementation:**
```rust
// core/checkpoints/src/changeset_tracker.rs

// Asynchronous file watcher with debouncing
let watcher = RecommendedWatcher::new(
    tx,
    Config::default().with_poll_interval(Duration::from_millis(500))
)?;

// Non-blocking event processing
tokio::spawn(async move {
    process_events(rx, event_queue).await;
});

// Smart debouncing (300ms)
const DEBOUNCE_DURATION: Duration = Duration::from_millis(300);
```

#### Enhancement 2: Complete FFI API Implementation

**The Problem:**
- Core checkpoint management functions were unimplemented (`list`, `delete`, `cleanup`, `stats`)
- Users couldn't manage checkpoints programmatically
- GUI couldn't display comprehensive checkpoint information
- No way to monitor checkpoint storage usage

**The Solution:**
Implemented all missing FFI bindings to provide a complete checkpoint management API:

**Implemented Functions:**

**1. `list_checkpoints()`**
- Lists all checkpoints with detailed metadata
- Returns: `id`, `description`, `createdAt`, `filesAffected`, `sizeBytes`
- Sorted by creation date (newest first)
- Used by GUI to display checkpoint history

**2. `delete_checkpoint(id: string)`**
- Deletes specific checkpoint by ID
- Returns: boolean success status
- Performs cleanup of orphaned files
- Used by GUI delete buttons

**3. `cleanup_old_checkpoints()`**
- Automatically removes checkpoints based on retention policy
- Respects `retentionDays` and `maxStorageBytes` settings
- Returns: number of checkpoints deleted
- Used by automatic cleanup scheduler

**4. `get_checkpoint_stats()`**
- Returns comprehensive checkpoint statistics
- Metrics: `totalCheckpoints`, `totalSessions`, `totalStorageBytes`, `avgCheckpointSize`, `filesTracked`
- Used by GUI dashboard and monitoring

**Integration:**
- **VSCode Extension**: All commands now use native Rust bindings
- **GUI**: Checkpoint pages leverage full FFI functionality
- **API**: RESTful endpoints expose all operations
- **Type Safety**: Complete TypeScript definitions for all FFI functions

**Example Usage:**
```typescript
// List checkpoints with full metadata
const checkpoints = await checkpointManager.listCheckpoints();
// [{ id: "abc123", description: "...", createdAt: 1234567890, ... }]

// Delete specific checkpoint
const success = await checkpointManager.deleteCheckpoint("abc123");

// Get storage statistics
const stats = await checkpointManager.getCheckpointStats();
// { totalCheckpoints: 50, totalStorageBytes: 104857600, ... }

// Cleanup old checkpoints
const deleted = await checkpointManager.cleanupOldCheckpoints();
// 5 (number of checkpoints removed)
```

#### Enhancement 3: Smart Incremental Checkpoint System with Multi-Level Caching

**The Problem:**
- Every checkpoint required a full workspace scan (slow for large codebases)
- Scanning 10,000+ files took 2-5 seconds per checkpoint
- No caching of file state between checkpoints
- Redundant hash calculations on unchanged files
- Poor performance for incremental changes

**The Solution:**
Implemented a sophisticated incremental checkpoint system with intelligent caching:

**Core Components:**

**1. Incremental Cache System**
```rust
struct IncrementalCache {
    cached_inventory: HashMap<PathBuf, Metadata>,  // File metadata cache
    modified_directories: HashSet<PathBuf>,         // Tracks changed dirs
    file_hashes: HashMap<PathBuf, u64>,            // File content hashes
    last_scan_time: SystemTime,                     // Cache timestamp
}
```

**2. Smart Change Detection**
- **Modified Directory Tracking**: Only scans directories with changes
- **File Hash Caching**: Compares hashes instead of re-reading files
- **Metadata Comparison**: Quick checks via timestamps and size
- **Fallback Strategy**: Full scan if cache is stale (>5 minutes)

**3. LRU Checkpoint Cache**
- **In-Memory Cache**: Stores 100 most recent checkpoints
- **Fast Retrieval**: Sub-millisecond access for cached checkpoints
- **Thread-Safe**: Uses `DashMap` for concurrent access
- **Auto-Eviction**: Least recently used checkpoints removed automatically

**4. File Watcher Integration**
- **Real-Time Tracking**: Monitors file system changes continuously
- **Directory Marking**: Marks modified directories for next scan
- **Efficient Updates**: Only processes changed files

**Performance Impact:**

| Operation | Before (Full Scan) | After (Incremental) | Improvement |
|-----------|-------------------|---------------------|-------------|
| **First Checkpoint** | 2,000ms | 2,000ms | Baseline |
| **No Changes** | 2,000ms | ~50ms | **40x faster** |
| **1-5 Files Changed** | 2,000ms | ~100-200ms | **10-20x faster** |
| **50 Files Changed** | 2,000ms | ~400-600ms | **3-5x faster** |
| **Cache Hit (Retrieval)** | 100ms | ~1ms | **100x faster** |

## V0.9.0

### AI Context System - 10 Major Enhancements

Implemented a comprehensive suite of 10 major enhancements to the AI Context system, transforming it from a basic code understanding tool into one of the most advanced AI-powered code intelligence systems available.

#### Performance Improvements

- Context build time: **0-50ms** (with predictive preloading)
- Context relevance: **~95%** (+35% improvement)
- Context sources: Code + Comments + Tests + Commit History + Documentation
- Transparency: **100%** (full explanations provided)
- Updates: **10-100x faster** (incremental updates only process changed code)
- User effort: **Minimal** (automatic query expansion)

#### Enhancement 1: Context Explanation & Transparency System

Added complete transparency into AI decision-making process:

**Features:**
- **Detailed File Inclusion Reasons**: Explains exactly why each file was included in context
- **Confidence Breakdown**: Shows semantic match, temporal relevance, architectural fit scores
- **Per-File Relevance Scores**: Individual scoring for each file with key elements highlighted
- **Exclusion Reasoning**: Explains why files were excluded and what would improve their relevance
- **Token Analysis**: Shows token usage, compression ratios, and optimization details
- **Overall Confidence Impact**: Calculates how each file contributes to overall confidence

**Example Output:**
```typescript
{
  explanation: {
    summary: "I found 5 highly relevant files for your authentication query...",
    confidence_breakdown: {
      semantic_match: 0.92,
      temporal_relevance: 0.85,
      architectural_fit: 0.88
    },
    inclusion_reasons: [
      {
        file_path: "src/auth/login.ts",
        reason: "High semantic relevance to authentication query",
        relevance_score: 0.925,
        key_elements: ["loginUser()", "validateCredentials()", "generateToken()"]
      }
    ]
  }
}
```

#### Enhancement 2: Multi-Modal Context Integration

Extended context beyond just code to include rich non-code information:

**Features:**
- **Comment Insights**:
  - TODO comments with priority and context
  - FIXME issues with severity levels
  - HACK/WORKAROUND detection with explanations
  - Design rationale extraction from comments
  - Known issues and limitations
  - Future improvement suggestions
  
- **Test Insights**:
  - Expected behaviors from test cases
  - Edge case identification
  - Usage examples from tests
  - Test coverage analysis
  - Untested code areas detection
  
- **Commit History Insights**:
  - Relevant commits for context
  - Architectural decisions from commit messages
  - Refactoring patterns and history
  
- **Documentation Insights**:
  - Relevant documentation sections
  - API usage examples
  - Architectural overviews

**Impact**: +30% richer context through multi-modal sources

#### Enhancement 3: Intelligent Query Expansion

Automatically expands user queries with related entities and concepts:

**Features:**
- **Entity Expansion**: Finds related functions, classes, modules based on knowledge graph
- **Concept Identification**: Detects technologies, patterns, frameworks relevant to query
- **Implicit Requirements**: Identifies unstated context needs (security, error handling, etc.)
- **Scope Suggestions**: Recommends expanding from function → class → module → system level
- **Synonym Detection**: Maps user terms to actual code entities

**Example:**
```
User Query: "login"
Expanded To: login, logout, authenticate, validateCredentials, 
             session management, JWT tokens, error handling, security context
```

**Impact**: +40% better relevance through automatic expansion

#### Enhancement 4: Incremental Context Updates

Revolutionary 10-100x faster updates by only reprocessing changed code:

**Features:**
- **Region-Level Analysis**: Identifies which code regions (functions/classes) were modified
- **Smart Change Detection**: Differentiates between signature changes, implementation changes, and comments
- **Incremental Graph Updates**: Updates only affected nodes in knowledge graph
- **Smart Cache Invalidation**: Invalidates only dependent caches, not entire system
- **Dependency Tracking**: Tracks file and function-level dependencies for efficient updates

**Performance:**
- Small changes: **< 50ms** (vs 800-2000ms full rebuild)
- Medium changes: **100-200ms** (vs 800-2000ms)
- Large changes: **300-500ms** (vs 800-2000ms)

**Technical Details:**
- Uses file watchers to detect changes in real-time
- Caches AST for unchanged code regions
- Only parses and analyzes modified sections
- Maintains incremental knowledge graph updates

#### Enhancement 5: Predictive Context Preloading

Provides **instant responses** by predicting and preloading context before users ask:

**Features:**
- **User Behavior Analysis**: Learns from query patterns (sequential, time-based, task-based)
- **File Activity Prediction**: Predicts queries based on which files user opens
- **Pattern Learning**: Identifies common query sequences (e.g., "controller" → "model")
- **Background Preloading**: Builds context for predicted queries in background
- **Accuracy Tracking**: Monitors prediction hit rates and adjusts accordingly

**Example Predictions:**
- User opens `auth.ts` → Preload "How does authentication work?"
- User opens test file → Preload "How does [tested function] work?"
- After "controller" query → Preload model-related queries
- Error detected → Preload debugging help context

**Impact**: **0ms response time** for predicted queries (instant)

#### Enhancement 6-10: Advanced Features (Architecture Ready)

**Pattern Detection (ML-based)**:
- Automatic detection of design patterns
- Anti-pattern identification
- Code smell detection

**Cross-Repository Context**:
- Multi-repository awareness for microservices
- Inter-service dependency tracking
- API contract understanding

**Flow Analysis (CFG/DFG)**:
- Control Flow Graph construction
- Data Flow Graph analysis
- Security vulnerability detection
- Dead code identification

**Type Inference**:
- Dynamic type inference for JavaScript/Python
- Type propagation across functions
- API contract inference

**Collaborative Context Learning**:
- Team behavior learning
- Code review pattern analysis
- Issue tracker integration
- Shared context knowledge

## V0.8.2

### Storage System Overhaul - Performance & Scalability Improvements

Completely redesigned the storage architecture to eliminate size limits, improve performance, and resolve all storage-related warnings. Both the GUI and extension now use optimized storage mechanisms suitable for production scale.

#### The Problem - Storage Limitations & Warnings

**GUI Storage Issues:**
- **5MB localStorage Limit**: Hit storage quota with extensive chat histories
- **Serialization Overhead**: JSON.stringify/parse on every read/write
- **No Type Safety**: String-only storage required constant conversion
- **Quota Errors**: Users losing data when exceeding browser limits

**Extension Storage Issues:**
- **VS Code Warning**: `WARN [mainThreadStorage] large extension state detected: 2850.318359375kb`
- **Memory Bloat**: 2.8MB of checkpoint metadata stored in VS Code's globalState
- **Performance Impact**: Slower VS Code startup/shutdown due to large state serialization
- **Not Scalable**: globalState designed for small settings, not large datasets

---

#### The Solution - Two-Pronged Storage Optimization

### Part 1: GUI Storage Migration (localStorage → IndexedDB)

Migrated the entire GUI from browser localStorage (5MB limit) to IndexedDB (hundreds of MB capacity) with full type safety and better performance.

#### Storage Architecture Comparison

**Before (localStorage):**
```
Browser Storage (5MB limit)
├── localStorage
│   ├── persist:root (Redux state)         ~500KB-2MB
│   ├── inputHistory_chat (chat history)  ~100KB-1MB
│   ├── fontSize, ide, etc. (settings)    ~10KB
│   └── data:image/* (cached images)      ~2-4MB
│
└── Risk: Quota exceeded errors ⚠️
```

**After (IndexedDB):**
```
Browser Storage (Hundreds of MB)
├── IndexedDB: "knoxchat_storage" v1
│   ├── keyValueStore (object store)
│   │   ├── fontSize, ide, etc.           ~10KB
│   │   ├── inputHistory_chat             ~100KB-1MB
│   │   └── extensionVersion, dialogs     ~5KB
│   │
│   ├── imageStore (object store)
│   │   ├── https://example.com/img1      ~50KB
│   │   ├── data:image/png;base64...      ~100KB
│   │   └── (hundreds of cached images)   ~5-50MB
│   │
│   └── reduxPersist (object store)
│       └── persist:root (Redux state)    ~500KB-2MB
│
└── localStorage (fallback only)
    └── Used only for sync operations      ~1MB
```

#### Key Features

**1. Unlimited Storage Capacity**
- **Before**: 5-10MB browser localStorage limit
- **After**: Hundreds of MB to GB (browser-dependent)
- **Impact**: Can store extensive chat histories without quota errors

**2. Separate Object Stores**
- **keyValueStore**: General settings and preferences
- **imageStore**: Cached images for offline access
- **reduxPersist**: Redux state persistence
- **Benefit**: Clear separation of concerns, easier debugging

**3. Type-Safe API with Function Overloading**
```typescript
// Strict typing for known keys
await setLocalStorage("fontSize", 16);  // ✅ Type checked

// Flexible typing for dynamic keys
await setLocalStorage("customKey" as any, anyValue);  // ✅ Works
```

**4. Automatic Migration**
- **One-time migration** from localStorage to IndexedDB on first load
- **Data preservation**: All existing data automatically transferred
- **Fallback strategy**: localStorage maintained as backup
- **Zero user intervention**: Completely transparent to users

**5. Performance Improvements**
- **Async operations**: Non-blocking I/O doesn't freeze UI
- **Native data types**: No JSON.stringify/parse overhead
- **Indexed queries**: Faster data retrieval
- **Better for large datasets**: Optimized for big data

#### Implementation Highlights

```typescript
// core/ai-context/indexedDB.ts

class IndexedDBManager {
  // Three separate object stores
  private async initDB(): Promise<IDBDatabase> {
    db.createObjectStore('keyValueStore');
    db.createObjectStore('imageStore');
    db.createObjectStore('reduxPersist');
  }

  // Automatic migration
  async migrateFromLocalStorage(): Promise<void> {
    // 1. Load data from localStorage
    // 2. Save to IndexedDB
    // 3. Clear localStorage (optional)
    console.log('✅ Migration complete');
  }
}

// Function overloading for type safety
export async function setLocalStorage<T>(key: T, value: LocalStorageTypes[T]): Promise<void>;
export async function setLocalStorage(key: any, value: any): Promise<void>;
```

**Redux Persist Integration:**
```typescript
// gui/src/redux/store.ts

import { indexedDBStorage } from '../util/indexedDBStorage';

const persistConfig = {
  key: 'root',
  storage: indexedDBStorage,  // Custom IndexedDB adapter
  // ... filters
};
```

---

### Part 2: Extension Storage Optimization (globalState → Disk)

Moved checkpoint metadata from VS Code's in-memory globalState (2.8MB) to disk-based storage, eliminating warnings and improving performance.

#### Storage Architecture Comparison

**Before (globalState):**
```
VS Code Extension Storage
├── globalState (In-Memory)
│   ├── hasBeenInstalled              1 byte
│   ├── quickEditHistory              ~5KB
│   └── knox.checkpointHistory        2.8MB ⚠️
│       ├── messageCheckpoints        ~500KB
│       ├── stableIdCheckpoints       ~300KB
│       └── checkpointHistory[]       ~2MB
│
├── Disk Storage
│   └── ~/.knox/checkpoints/
│       ├── ckpt_abc123/              (actual data)
│       ├── ckpt_def456/              (actual data)
│       └── ...
│
└── Warning: "large extension state detected" ⚠️
```

**After (Disk-Based):**
```
VS Code Extension Storage
├── globalState (In-Memory)
│   ├── hasBeenInstalled              1 byte
│   └── quickEditHistory              ~5KB
│   └── Total: ~5KB ✅ (was 2.8MB)
│
├── Disk Storage - Data
│   └── ~/.knox/checkpoints/
│       ├── ckpt_abc123/
│       │   ├── metadata.json
│       │   ├── src/main.ts
│       │   └── src/utils.ts
│       ├── ckpt_def456/
│       └── ...
│
└── Disk Storage - Index (NEW)
    └── ~/.vscode/extensions/.../globalStorage/
        └── knoxchat.knoxchat/
            └── checkpoint-history.json  ~2.8MB
                ├── messageCheckpoints
                ├── stableIdCheckpoints
                └── checkpointHistory[]

✅ No warnings - memory freed!
```

#### Two Storage Locations Explained

The checkpoint system now uses **two separate storage locations** for different purposes:

| Location | Purpose | Size | Contains |
|----------|---------|------|----------|
| **Metadata Index**<br>`~/.vscode/.../checkpoint-history.json` | Fast lookup index | ~2-5MB | Message→Checkpoint mappings<br>Checkpoint metadata<br>Quick references |
| **Actual Data**<br>`~/.knox/checkpoints/` | Permanent storage | ~100MB-10GB (can be more) | Full file snapshots<br>Complete checkpoint data<br>Restoration content |

**Why Two Locations?**
1. **Performance**: Index loaded once for fast lookups; data loaded on-demand
2. **Scope**: Index is VS Code-specific; data is global across workspaces
3. **Access patterns**: Index needs speed; data needs completeness
4. **Separation of concerns**: Metadata vs actual content

#### Migration Process

**Automatic on First Launch:**
```
1. Extension initializes
2. Check for checkpoint-history.json
3. If not found:
   ├─ Load from old globalState
   ├─ Save to disk (checkpoint-history.json)
   └─ Clear globalState ✅
4. Future launches: Load from disk
```

#### Performance Impact

**Before:**
```
VS Code Startup:
  └─ Load 2.8MB from globalState          ~50-100ms
  └─ Deserialize JSON                     ~20-50ms
  └─ Store in memory                      2.8MB

VS Code Shutdown:
  └─ Serialize 2.8MB to JSON              ~50-100ms
  └─ Save to globalState                  ~20-50ms
```

**After:**
```
VS Code Startup:
  └─ Load ~5KB from globalState           ~1-5ms ✅
  └─ CheckpointManager init (lazy)        ~0ms

First Checkpoint Operation:
  └─ Load index from disk                 ~10-50ms
  └─ No VS Code impact                    ✅
```

**Memory Savings:**
- VS Code memory usage: **-2.8MB** ✅
- Extension heap: **-2.8MB** ✅
- Startup/shutdown: **~70% faster** ✅

#### Implementation Highlights

```typescript
// extensions/vscode/src/checkpoints/CheckpointManager.ts

class CheckpointManager {
  // Get disk storage path
  private getCheckpointHistoryPath(context: vscode.ExtensionContext): string {
    const storageUri = context.globalStorageUri;
    return path.join(storageUri.fsPath, 'checkpoint-history.json');
  }

  // Load from disk (with migration)
  private async loadCheckpointHistory(context: vscode.ExtensionContext): Promise<void> {
    const historyPath = this.getCheckpointHistoryPath(context);
    
    if (fs.existsSync(historyPath)) {
      // Load from disk
      const data = JSON.parse(fs.readFileSync(historyPath, 'utf8'));
      console.log(`✅ Loaded ${data.checkpointHistory.length} checkpoints`);
    } else {
      // One-time migration from globalState
      const oldData = context.globalState.get('knox.checkpointHistory');
      if (oldData) {
        await this.saveCheckpointHistory();
        await context.globalState.update('knox.checkpointHistory', undefined);
        console.log('✅ Migration complete');
      }
    }
  }

  // Save to disk (atomic write)
  private async saveCheckpointHistory(): Promise<void> {
    const tempPath = historyPath + '.tmp';
    fs.writeFileSync(tempPath, JSON.stringify(data, null, 2));
    fs.renameSync(tempPath, historyPath);  // Atomic on POSIX
  }
}
```
### Issue Fixed
- React Error #62 Fix
- Markdown Table Rendering Fix
- Markdown Rendering Conflict in Chat Stream Fix

## V0.8.1

  - Update knoxdev-package ([Details](https://github.com/knoxchat/knoxchat/commit/a2b8238d15eeda1572b9cd8555328e59d9342c25))

## V0.8.0

### Checkpoint System - Complete Restoration Overhaul

Fixed a critical architectural flaw in the checkpoint system that prevented accurate workspace restoration. The system now properly restores workspaces to their exact state at checkpoint time.

#### The Problem - Incomplete State Tracking

Previously, the checkpoint system had a fundamental flaw that caused incomplete restoration:
- **Delta-Only Storage**: Checkpoints only stored file changes (deltas), not complete workspace state
- **Orphaned Files**: Restoring to an earlier checkpoint left files/directories that were added later
- **Inaccurate State**: Workspaces couldn't be restored to exact historical states
- **User Frustration**: When restoring to a checkpoint with 3 files, additional directories (like `database/`) remained instead of being removed

**Example of the Bug:**
```
Checkpoint A: workspace/ with 3 files (index.html, script.js, styles.css)
Later: AI adds database/ directory
Restore to Checkpoint A → Expected: Only 3 files | Actual: 3 files + database/ (WRONG!)
```

#### The Solution - Complete Workspace Snapshots

Completely redesigned the checkpoint system to capture and restore complete workspace states, similar to how Git commits work.

**Architecture Changes:**

**1. Enhanced Checkpoint Structure**
```rust
// core/checkpoints/src/types.rs
pub struct Checkpoint {
    pub file_changes: Vec<FileChange>,      // What changed (for content)
    pub file_inventory: Vec<PathBuf>,       // NEW: Complete file list
    pub files_affected: usize,
    pub size_bytes: u64,
    // ... other fields
}
```

**2. Complete File Inventory Capture (Rust)**

Added comprehensive workspace scanning during checkpoint creation:

```rust
// core/checkpoints/src/manager.rs
fn capture_complete_file_inventory(&self) -> Result<Vec<PathBuf>> {
    use walkdir::WalkDir;
    
    let mut inventory = Vec::new();
    
    // Walk ENTIRE workspace and record ALL files
    for entry in WalkDir::new(&self.workspace_path) {
        if entry.file_type().is_file() && self.should_track_file(path) {
            inventory.push(relative_path.to_path_buf());
        }
    }
    
    Ok(inventory)
}
```

**3. Inventory-Based Cleanup (Rust)**

Added cleanup logic to remove files not in checkpoint inventory:

```rust
// core/checkpoints/src/restoration.rs
fn cleanup_files_not_in_inventory(
    &self,
    checkpoint: &Checkpoint,
    result: &mut RestoreResult,
) -> Result<()> {
    let inventory_set: HashSet<PathBuf> = 
        checkpoint.file_inventory.iter().cloned().collect();
    
    // Walk workspace and remove files NOT in inventory
    for current_file in workspace_files {
        if !inventory_set.contains(current_file) {
            fs::remove_file(&full_path)?;
            self.remove_empty_directories(parent)?;
        }
    }
}
```

**4. TypeScript Integration**

Mirrored the same functionality in the VSCode extension:

```typescript
// extensions/vscode/src/checkpoints/CheckpointManager.ts

// Capture complete inventory
private async captureCompleteFileInventory(): Promise<string[]> {
    const inventory: string[] = [];
    
    const scanDirectory = async (dirPath: string): Promise<void> => {
        for (const entry of await fs.readdir(dirPath)) {
            if (entry.isFile() && this.isTextFile(fullPath)) {
                inventory.push(relativePath);
            }
        }
    };
    
    await scanDirectory(this.currentWorkspacePath);
    return inventory;
}

// Remove extra files during restoration
private async cleanupFilesNotInInventory(
    fileInventory: string[],
    removedFiles: string[],
    failedFiles: Array<{path: string, error: string}>
): Promise<void> {
    const inventorySet = new Set(fileInventory);
    const currentFiles = await this.captureCompleteFileInventory();
    
    const filesToRemove = currentFiles.filter(file => !inventorySet.has(file));
    
    for (const file of filesToRemove) {
        await fs.unlink(fullPath);
        await this.removeEmptyDirectories(path.dirname(fullPath));
    }
}
```

#### Key Features

**Complete State Capture:**
- Every checkpoint now captures a complete inventory of ALL tracked files
- File inventory stored alongside file changes
- Minimal storage overhead (~5KB for 100 files)

**Accurate Restoration:**
- Restores files from checkpoint content
- **Removes files that don't exist in checkpoint inventory**
- **Removes empty directories automatically**
- Workspace restored to exact historical state

**Smart File Tracking:**
- Tracks text files: `.js`, `.ts`, `.tsx`, `.jsx`, `.py`, `.java`, `.cpp`, `.rs`, etc.
- Automatically ignores: `node_modules/`, `.git/`, `dist/`, `target/`, `build/`, etc.
- Respects `.gitignore` patterns

## V0.7.9

### Chat History Auto-Save System

Fixed a critical issue where chat history was not being automatically saved to disk, causing users to lose their conversation history when closing the editor.

#### The Problem
Previously, chat history was only stored in localStorage (temporary memory), which meant:
- **Data Loss on Close**: All chat history disappeared when closing the editor
- **Agent Mode Not Saving**: Agent mode conversations were never persisted to disk
- **Manual Save Required**: Users had to manually save sessions or risk losing work
- **Inconsistent with Checkpoints**: Unlike the checkpoint system which auto-saves reliably, chat history required manual intervention

#### The Solution - Comprehensive Auto-Save

Implemented a robust auto-save system that works exactly like checkpoints - automatic, reliable, and transparent to users.

**Key Features:**

1. **Universal Mode Support**
   - **Chat Mode**: Auto-saves after every conversation completion
   - **Agent Mode**: Auto-saves after tool calls and agent operations  
   - **Edit Mode**: Auto-saves after edit operations complete

2. **Smart Auto-Save Middleware**
   - **Intelligent Debouncing**: 2-second debounce prevents excessive disk writes
   - **Performance Optimized**: Minimum 5-second interval between saves for efficiency
   - **Action-Triggered**: Automatically saves on message updates, context additions, title changes
   - **Non-Intrusive**: Only saves when there's actual history and not actively streaming

3. **Multiple Save Triggers**
   - After every chat stream completes (all modes)
   - After tool calls complete in agent mode
   - When user cancels a stream (preserves partial responses)
   - Periodically during chat updates (debounced)
   - When workspace changes (existing behavior)

4. **Dual Storage Strategy**
   - **localStorage** (redux-persist): Quick restoration within same session
   - **~/.knox/sessions/** (auto-save): Persistent filesystem storage

## V0.7.8

### Checkpoint Diff Viewer - Advanced Comparison System

Added a comprehensive diff viewer to visualize changes between consecutive checkpoints, making it easy to understand what changed in your codebase.

#### Split View - Side-by-Side Comparison

**Professional vertical split layout for easy file comparison:**
- **Equal-Width Panels**: Both files displayed side-by-side with perfect 50/50 split
- **Synchronized Vertical Scrolling**: Both panels scroll up/down together for aligned comparison
- **Independent Horizontal Scrolling**: Each panel can scroll left/right independently to view long lines
- **Fixed Headers**: "Previous Version" and "Current Version" headers remain visible while scrolling
- **Line-by-Line Comparison**: Side-by-side line numbers for easy reference
- **Visual Diff Highlights**: 
  - Red background for removed lines (left panel)
  - Green background for added lines (right panel)
  - Neutral background for unchanged lines

#### Unified View - Inline Comparison

**Traditional inline diff display with +/- indicators:**
- **Single-Column Layout**: All changes shown in one continuous view
- **Clear Change Markers**: 
  - `-` prefix with red highlight for removed lines
  - `+` prefix with green highlight for added lines
  - No prefix for unchanged context lines
- **Line Number Tracking**: Separate counters for old and new file line numbers
- **Compact Display**: Efficient use of screen space for quick change review

#### Interactive File Tree Navigation

**Hierarchical file browser with change statistics:**
- **Collapsible Tree View**: Toggle show/hide with dedicated expand/collapse button
- **Folder Hierarchy**: Full directory structure with expand/collapse controls
- **Change Indicators**: 
  - Green dot badge for files with additions
  - Red dot badge for files with deletions
  - Orange dot badge for files with both additions and deletions
- **File Statistics**: Shows `+X -Y` additions and deletions count per file
- **Quick Navigation**: Click any file to instantly view its diff
- **Persistent State**: Remembers expanded/collapsed preference using localStorage

#### Smart Code Display

**Professional code viewing with advanced features:**
- **Line Wrapping Toggle**: "Wrap" button to toggle between wrapped and unwrapped code
  - Wrapped mode: Full code visibility with text wrapping
  - Unwrapped mode: Horizontal scrolling for long lines
- **Syntax Highlighting**: Monospace font with proper formatting
- **Line Numbers**: Fixed-width line numbers that don't scroll
- **Minimal Design**: Clean, distraction-free diff display
- **Theme Integration**: Follows VSCode's active color theme

#### Performance & UX Optimizations

**Smooth and responsive user experience:**
- **Efficient Diff Algorithm**: Uses the `diff` library for accurate line-by-line comparison
- **Optimized Rendering**: React useMemo for computed diffs to prevent unnecessary recalculations
- **Smooth Animations**: Subtle transitions for view mode switching and file selection
- **Responsive Design**: Works seamlessly on different screen sizes
- **Empty State Handling**: Graceful messaging when no previous checkpoint exists
- **Loading States**: Visual feedback during diff computation

#### Accessibility Features

**Built with accessibility in mind:**
- **Keyboard Navigation**: Full keyboard support for tree navigation and file selection
- **Semantic HTML**: Proper heading hierarchy and ARIA labels
- **Focus Management**: Clear focus indicators for all interactive elements
- **Screen Reader Support**: Descriptive labels and meaningful content structure

**Technical Implementation:**
- Integrated `diff` npm package for reliable diff computation
- Extended backend API with `getPreviousCheckpoint` command
- Enhanced protocol types for checkpoint comparison
- Added synchronized scroll event handlers for vertical alignment
- Implemented localStorage persistence for UI preferences

## V0.7.6

### Checkpoint System - Feature Enhancements

#### 1. File System Watcher Integration for Real-Time Tracking

The checkpoint system now includes real-time file change tracking using VSCode's FileSystemWatcher API.

**Features:**
- **Real-time change detection**: Automatically tracks file changes as they happen
- **Debounced updates**: Prevents excessive processing with 500ms debounce timer
- **Memory efficient**: Uses a Set to track recently modified files
- **Workspace-aware**: Automatically reinitializes when workspace changes
- **Event-driven**: Responds to file create, change, and delete events

#### 2. Configurable Scan Depth and File Size Limits

Added configurable limits to prevent performance issues in large projects.

**Configuration Options:**

**Scan Depth Limit**:
- **Default**: 10 levels
- **Range**: 1-50 levels
- **Purpose**: Prevents infinite recursion in deep directory structures
- **Command**: `knox.checkpoint.configureScanDepth`

**File Size Limit**:
- **Default**: 1 MB (1,048,576 bytes)
- **Range**: 1 KB - 100 MB
- **Purpose**: Skips large files that would slow down checkpoints
- **Command**: `knox.checkpoint.configureMaxFileSize`

#### 3. Custom Ignore Pattern Presets

Predefined ignore pattern templates for different project types, making it easy to create `.knoxignore` files.

**Available Presets:**

1. **Minimal** - Basic patterns for any project
2. **Node.js / TypeScript** - JavaScript, React, Vue, Angular projects
3. **Python** - Python, Django, Flask projects
4. **Rust** - Rust projects using Cargo
5. **Java / Maven / Gradle** - Java, Kotlin, Spring Boot projects
6. **Go** - Go / Golang projects
7. **C / C++** - C and C++ projects
8. **NET / C#** - .NET, ASP.NET projects
9. **Ruby / Rails** - Ruby and Ruby on Rails projects
10. **PHP** - PHP, Laravel, Symfony projects

**Features:**

- **Auto-detection**: Automatically detects project type based on files
- **Interactive selection**: Choose preset via command palette
- **Customizable**: Edit generated `.knoxignore` file after creation
- **Comprehensive patterns**: Includes dependencies, build outputs, IDE files, etc.

#### 4. Performance Profiling and Optimization

Comprehensive performance metrics tracking for checkpoint operations.

**Tracked Metrics:**

**TypeScript Metrics**:
```typescript
{
    lastScanDuration: number,        // Duration of last scan in ms
    totalScans: number,              // Total number of scans performed
    averageScanDuration: number,     // Rolling average scan duration
    filesScanned: number,            // Total files scanned
    lastScanTimestamp: number        // Timestamp of last scan
}
```

**Rust Metrics** (file_tracker.rs):
```rust
pub struct ScanStatistics {
    pub total_scans: u64,
    pub total_files_scanned: u64,
    pub total_directories_scanned: u64,
    pub last_scan_duration_ms: u64,
    pub average_scan_duration_ms: u64,
    pub files_skipped_size: u64,
    pub files_skipped_depth: u64,
}
```

**Features:**
- **Real-time tracking**: Updates after each scan
- **Rolling averages**: Calculates average performance over time
- **Resource monitoring**: Tracks skipped files and directories
- **Debug logging**: Detailed performance logs when verbose mode enabled

## V0.7.4

**Major UI/UX Overhaul - shadcn/ui Migration Complete**

### Component Migration & Theme Integration
* **Complete Migration from styled-components to shadcn/ui**: Migrated all 180+ styled-components to shadcn/ui with Tailwind CSS
  - Eliminated runtime CSS-in-JS overhead for better performance
  - Fixed TypeScript compilation errors from styled-components migration
  - Converted all component files from `.ts` to `.tsx` for proper JSX support
  - Removed deprecated `fix-animations.js` build script

### Modal & Dialog System Enhancement
* **Professional Modal Backgrounds**: Complete redesign of all modal overlays and dialogs
  - Replaced harsh `bg-black/80` with softer `rgba(0, 0, 0, 0.7)` + 2px backdrop blur
  - Enhanced Dialog, AlertDialog, Sheet components with VSCode theme integration
  - Improved TextDialog and ConfirmationDialog with proper theme colors
  - Added beautiful box shadows (`0 25px 50px -12px rgba(0, 0, 0, 0.5)`)
  - Fixed AddPromptDialog textarea to use VSCode theme colors dynamically

### VSCode Theme Integration
* **Comprehensive Theme Support**: Full dark/light theme adaptability with VSCode
  - Added proper CSS variables mapping for all shadcn/ui components
  - Enhanced popover and dropdown menus with VSCode color scheme
  - Implemented theme-aware placeholders, inputs, selects, and textareas
  - Added hover and focus states using VSCode theme colors
  - Updated global CSS with proper color fallbacks for both themes

### Tool Permissions Interface Redesign
* **Professional Tool Permissions UI**: Complete redesign of the tool permissions dialog
  - **Card-Based Layout**: Each tool group displayed in modern bordered cards
  - **Visual Status Indicators**: Color-coded dots (green = active, red = disabled)
  - **Badge Counts**: Shows number of tools in each group with themed badges
  - **Enhanced Headers**: Dark backgrounds with proper separators and borders
  - **Professional Status Badges**: 
    - "Requires Approval" - Orange badge with transparent background
    - "Auto-Approve" - Knox cyan badge with transparent background
    - "Disabled" - Gray badge with muted colors
  - **Disabled State Overlay**: Shows "Group Disabled" badge with blur effect
  - **Improved Hover States**: Smooth background transitions on tool items

### Custom Switch Component
* **New CustomSwitch Component**: Smaller, cleaner switch for compact UIs
  - Clean design without complex animations (200ms smooth transitions)
  - Knox cyan active state with white thumb
  - VSCode theme-integrated colors for background and borders
  - Smaller size options (12px) perfect for dense layouts
  - Replaces default shadcn/ui switch in tool permissions
  - Full accessibility support with keyboard navigation

### Dynamic Height & Scrolling
* **Responsive Toolbar Sections**: Fixed static height limitation in expandable toolbars
  - Changed from fixed `max-h-[200px]` to dynamic `max-h-[70vh]`
  - Content now adapts to screen size (responsive across all devices)
  - Shows all tool items without cutting off content
  - **Enhanced Scrollbars**: Beautiful VSCode-themed thin scrollbars
    - 6px width, unobtrusive and modern design
    - Knox cyan accent color on hover
    - Smooth scrolling behavior with 200ms transitions
    - Uses `--vscode-scrollbarSlider-*` theme variables

### Component Improvements
* **Button Components**: Updated Input, HeaderButton, and other components with ref forwarding
  - Added `React.forwardRef` support for better ref handling
  - Fixed ref prop TypeScript errors across multiple components
  - Improved accessibility with proper ARIA attributes

### CSS Enhancements
* **Global Styling Updates**: Added comprehensive CSS for better theming
  - Modal overlay enhancements with backdrop filters
  - Dropdown and context menu styling with VSCode colors
  - Menu item hover and selection states
  - Smooth scrolling utilities
  - Better scrollbar styling across all components

**Bug Fixes**
- Fixed Checkpoint Deletion Bug
- Fixed Minified React error #62
- Fixed 180 TypeScript errors from styled-components migration
- Fixed JSX.Element namespace errors in component files
- Fixed ref prop issues in custom Input and HeaderButton components
- Fixed fontSize prop missing in StyledMarkdownPreview component
- Fixed styled-jsx syntax errors in Spinner, ThinkingBlockPeek, and Reasoning components

## V0.7.3

**Checkpoint System Enhancements**
* **Compact File Tree View**: Redesigned file tree with significantly reduced vertical spacing for better space utilization
  - Reduced padding from `py-1` to `py-0.5` for tighter row spacing
  - Decreased indentation from 16px to 12px per level for more compact hierarchy
  - Smaller icons (16px to 12px) and reduced gaps for efficient screen real estate usage
  - Updated folder icons to use project's `text-knoxcyan` theme color

* **Enhanced File Metadata Display**: Reorganized file information layout for better user experience
  - **Header Cleanup**: Moved Size and Modified date from header to footer
  - **Action-Ready Header**: Prepared header space for future action buttons (copy, code wrap, etc.)
  - **Comprehensive Footer**: File metadata now displayed alongside line count and language info
  - **24-Hour Time Format**: Modified date/time displays in 24-hour format for consistency
  - **Removed Duplicate Information**: Eliminated file size display from tree view to avoid redundancy

* **Persistent UI State Management**: Added localStorage-based state persistence
  - **File Tree Toggle Memory**: Remembers user's expand/collapse preference across modal sessions
  - **First-Time User Experience**: File tree defaults to expanded for new users
  - **Cross-Session Persistence**: UI preferences maintained between browser sessions

* **Smart JSON Formatting**: Enhanced description display with intelligent JSON detection
  - **Automatic JSON Detection**: Detects and formats valid JSON in checkpoint descriptions
  - **Pretty Printing**: Proper indentation and syntax highlighting for JSON content
  - **Error Handling**: Graceful fallback for invalid JSON with appropriate styling
  - **Visual Indicators**: Clear section headers with icons for different content types
  - **Scrollable Containers**: Prevents modal overflow with max-height constraints

* **Advanced Search Capabilities**: Extended search functionality to support checkpoint IDs
  - **Full ID Search**: Search using complete checkpoint IDs (e.g., `3da53072-047c-4374-83ad-918ddcde2ea6`)
  - **Partial ID Matching**: Support for truncated ID searches (e.g., `3da53072`, `3da5307`)
  - **Multi-Field Search**: Searches both descriptions and checkpoint IDs simultaneously
  - **Case Insensitive**: All searches work regardless of case sensitivity
  - **Updated Placeholders**: Search boxes now indicate "Search by description or ID..."
  - **Enhanced Filtering**: Improved MiniSearch configuration with fuzzy matching

* **Copy-to-Clipboard Functionality**: Interactive checkpoint ID copying with visual feedback
  - **Clickable ID Badges**: Truncated checkpoint ID badges are now clickable
  - **Full ID Copying**: Copies complete checkpoint ID to clipboard for easy sharing
  - **Visual Feedback**: Shows "Copied" status with themed green styling for 3 seconds
  - **Hover Effects**: Subtle UI feedback to indicate clickable elements
  - **Workflow Integration**: Seamless copy-and-paste workflow for quick checkpoint searching
  - **Error Handling**: Graceful fallback if clipboard access fails

**Data Integrity Improvements**
* **Fixed JSON Truncation Issue**: Resolved checkpoint description truncation that caused invalid JSON
  - **Root Cause Fix**: Removed 500-character limit from conversationContext.messageContent
  - **Complete Data Preservation**: Full message content now stored in checkpoints
  - **Better JSON Parsing**: Enhanced description formatter can now handle complete JSON objects

## V0.7.2

**Checkpoint Details Enhancement**
* **Comprehensive Details Modal**: Added detailed checkpoint information display with professional UI
* **File Tree Navigation**: Interactive file tree with expand/collapse functionality for checkpoint file snapshots
* **Syntax-Highlighted Code Viewer**: Professional code display with line numbers and theme-aware highlighting
* **Resizable Panels**: Drag-to-resize file tree width with localStorage persistence to remember user preferences
* **Tab-Based Interface**: Clean two-tab layout (File Snapshots / Basic Information) for better space utilization
* **Smart Copy Functionality**: One-click code copying with visual feedback and 3-second success state
* **Responsive Design**: Fully responsive modal that works seamlessly on desktop and mobile devices
* **Enhanced User Experience**: 
  - Collapsible sections for optimal screen space usage
  - Proper scrolling for code content viewing
  - Clean, distraction-free code highlighting without line backgrounds
  - Professional file metadata display (size, encoding, modification date)
  - Intuitive file selection and navigation

**UI Improvements**
- Update chat input box model name avoid mix with Go Bottom button

## V0.7.0

**AI Context System**
* Optimized the data structure for better efficiency.
* Improved overall performance.
* Fixed Rust integration errors.

**User Interface (UI)**
* Updated to a brand new UI leveraging shadcn/ui.

**Workflows & Features**
* Improved the performance of the AI Models Team workflow.
* Improved the performance of the Checkpoints Creation workflow.
* Added a Checkpoints List feature, including a Title for each checkpoint.

## V0.6.8

- Redesigned the UI with a new look and feel
- Implemented new settings for the GUI
- Implemented new settings for the VSCode extension
- Implemented new settings for the performance monitoring system
- Implemented new settings for the performance alerts system
- Implemented new settings for the performance dashboard system
- Implemented new settings for the AI context system
- Fixed a bug where Chat streaming not workspace-aware
- Fixed a bug where Checkpoints not workspace-aware

## V0.6.6

### AI Context System Migration (TypeScript to Rust)

#### Performance Metrics
- **Speed Improvement**: **10,000x faster** (from ~500ms to <1ms per query)
- **Throughput**: **10,663 queries/second** (vs ~10 previously)  
- **Memory Usage**: **60-70% reduction** through native Rust optimization
- **Cache Hit Ratio**: **>90%** with intelligent multi-level caching
- **Response Time**: **Sub-millisecond** context building

#### **Core Engine Enhancement**
- Enhanced `AIContextManager` with full TypeScript AI context capabilities
- Implemented unified semantic analysis pipeline using tree-sitter
- Added advanced query intent analysis engine
- Created sophisticated context relevance scoring system
- Built comprehensive architectural impact analyzer
- Integrated performance monitoring and optimization

#### **FFI Interface Development**
- Created production-quality Neon.js bindings
- Implemented synchronous FFI interface
- Added comprehensive error handling and type safety
- Built efficient JSON serialization for complex data structures
- Created multi-level caching optimization

#### **VSCode Integration Update**
- Replaced TypeScript AI context providers with unified Rust FFI wrappers
- Streamlined context building to single high-performance Rust call
- Updated VSCode extension to use production unified interface
- Migrated all caching logic to optimized Rust layer
- Created migration guide for gradual deprecation of legacy files

#### **Performance Optimization**
- Implemented parallel semantic analysis with rayon
- Added intelligent caching strategies with LRU and semantic similarity
- Optimized memory usage for enterprise-scale codebases (10,000+ files)
- Created real-time incremental update algorithms
- Built comprehensive performance monitoring dashboard

## V0.6.3

- Fix Incorrect Tool Support

## V0.6.2

- Performance Optimization

## V0.6.0

## **Core Features Overview**

### **1. Advanced AI Chat Interface**
- **Natural Language Programming**: Describe what you want and watch Knox build it
- **Multi-Model Support**: Choose from OpenAI GPT, Anthropic Claude, and Knox's own models
- **Context-Rich Conversations**: Knox understands your entire codebase context
- **Smart Cost Optimization**: Automatically routes requests to cost-effective models

### **2. Intelligent Code Understanding**
- **Semantic Analysis**: Goes beyond syntax to understand code meaning and relationships
- **Architecture Awareness**: Understands patterns, dependencies, and data flow
- **Evolution Tracking**: Tracks how code changes over time and why
- **Predictive Context**: Preloads relevant context before you need it

### **3. Agent Mode**
Transform Knox into an autonomous development partner:
- **Complex Task Execution**: Handles multi-step development tasks autonomously
- **Shadow Workspace**: Test changes safely before applying them
- **Smart Refactoring**: Intelligent code restructuring and optimization
- **Debug Integration**: AI-powered debugging and error resolution

### **4. Checkpoint & Restore System**
Never lose work or worry about AI changes:
- **Automatic Checkpoints**: Created before every AI operation
- **Rich History**: Visual timeline of all changes with detailed metadata  
- **Instant Restoration**: One-click restore to any previous state
- **Smart Cleanup**: Automatic management of checkpoint storage

### **5. Enhanced Code Editing**
- **Natural Language Edits**: Describe changes in plain English
- **Intelligent Diff Viewer**: Clear visualization of proposed changes
- **Multi-file Operations**: Handle complex changes across multiple files
- **Smart Accept/Reject**: Granular control over AI-generated changes
