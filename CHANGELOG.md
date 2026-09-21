## V1.5.0

### Knox OAuth2 — Connect without pasting an API key

Adding a KnoxChat model used to mean creating a key on knox.chat or knoxstudio.ai, copying it, and pasting it into **Add Chat Model** every time. That is easy to get wrong, leaves `sk-` secrets in `~/.knox/config.yaml`, and repeats the same secret on every model.

Sign in once with KnoxStudio instead. KnoxChat runs the same OAuth2 + PKCE flow as KnoxStudio Desktop, mints a local `sk-` key named **KnoxChat**, and reuses it for every new model.

- **Add Chat Model** shows **Sign in with KnoxStudio** instead of a required API key field
- After **Connected as @you**, Connect works with no paste; later models inherit the same session
- A yaml `apiKey` is still an override if you edit `~/.knox/config.yaml` by hand
- Sign out revokes the **KnoxChat** key on the server; you can also manage it at [knoxstudio.ai/keys](https://knoxstudio.ai/keys) or under Connected apps

`~/.knox/config.yaml` is a plaintext file. Anything with an `apiKey:` line is readable by other processes, easy to commit or back up, and duplicated on every model you add. The OAuth session key never goes there.

It is stored in the editor’s encrypted extension storage (AES-256-GCM), with the wrapping key in VS Code SecretStorage (the OS keychain). Account metadata (`@you`, token id) is kept separately so the UI never has to read the secret. Chat injects the key in memory only for `knoxchat` models that have no yaml `apiKey`.

Copy-paste is a shared secret sitting in a text file. OAuth is a one-time browser consent, a key encrypted at rest outside yaml, and the same login you already use for KnoxStudio.

### Jev — One Knox key, same API as chat

Jev (System One) now runs through Knox Chat with the API key you already use for models. There is no TypeSafe account, second key, or call to `api.typesafe.ai`.

- Enable with `jev.enabled: true` in `~/.knox/config.yaml`, or **Settings → Jev harness judgments**
- Reuses the `apiKey` on your `knoxchat` models, or the OAuth session key if yaml has none; optional `jev.apiKey` only if you want a Jev-only key
- Always calls `https://api.knoxstudio.ai/v1/systemone` (`jev.baseUrl` is ignored)
- Allow `jev-*` on the key at [knox.chat/keys](https://knox.chat/keys)
- Fail-open to the current heuristics if Jev is off, no Knox key is found, or the call times out (800 ms)
- Clearer errors for an invalid key (401), insufficient credits (402), and a key whose allowlist omits `jev-*` (403)

Jev still only fills harness judgments (Chat vs View/Read, skill hints, context and tool gates, citation checks, `auto` profile confirm). It does not write code or replace the Agent loop.

### Checkpoints

The Checkpoints list under Explorer (below the file tree) is gone. It duplicated the overlay from the status-bar **CP** button and was a worse place to browse history.

Use **CP** in the status bar (or the Knox sidebar Checkpoints tab) for list, details, restore, compare, pin, and delete. File history stays on Explorer / editor file context menus.

## V1.4.9

- Add Jev for harness

## V1.4.8

### Long chats — Stay open through hours of agent work

The sidebar used to go blank or freeze during long sessions. Chat and Agent mode now stay usable while replies and tool logs stream.

- The panel stays interactive: you can scroll, type, and open finished tool results while the agent is still working
- If the sidebar ever hangs, it reloads on its own and restores your last saved chat instead of staying gray
- Opening a huge past chat shows the latest messages first; scroll up or use Load earlier messages for the rest
- A notice appears when a chat is very large, so you know older tool output may be shortened on screen
- One broken message shows an error on that row; the rest of the chat and the input keep working

### Streaming — Read as it writes, without the thread shaking

- Only the live reply updates as new text arrives; earlier messages stay still
- Finished markdown, code, and tool cards no longer re-draw on every new word
- Auto-scroll follows the latest output; scroll up to pause and read, then jump back to the bottom when you are ready
- Your last sent prompt stays visible at the top while a reply is streaming, then returns to the thread if you scroll up
- Find in chat still finds text in older messages, even if they are not on screen yet
- Chat is saved after a reply finishes, not on every word, so the panel stays snappy

### Tool results — Expanded by default, heavy output stays light

- Finished tool cards stay open so you can read the result in place; click to collapse a card to a one-line summary
- Collapsed code shows a short preview instead of the whole file; expand for more, or open it in the editor
- Command output shows the recent lines; copy the full output or open the log when you need everything
- Directory listings and repo maps load when you expand them, not all at once when the chat opens
- Large tool dumps are shortened on screen so the chat stays fast; the agent still sees the full results

### Fixed

- Back arrow icon not in place

## V1.4.7

### Slash `/` commands — Type a name, insert a chip, run the command

- Type `/` for a grouped picker (Bookmarked, Recent, Commands, Prompts); descriptions sit next to the name instead of hugging the right edge
- Search matches the command name and the description (`/shell` finds `/cmd`); `/cmt` still ranks `/commit`
- Enter inserts a teal `/commit` chip; keep typing arguments after it. Hover shows the description, × removes the chip
- Bookmark from the row; recently used commands rise to the top of an empty `/`
- Arrow keys wrap, Home/End jump, Tab selects, Esc closes and keeps what you typed
- The `/query` you are typing is highlighted while the picker is open
- Empty search says no commands match, not “No files match”

### @ mentions — Type a file name, insert a chip, attach the file

- Type `@` plus a file or folder name (or a path like `gui/src`) to search the whole workspace; matching files show up in the top list without opening a submenu
- Empty `@` shows open files and providers; as you type, results group into Files, Folders, and Providers
- Open files rank first, then exact names, then path matches; matching characters are highlighted
- New files appear in the picker shortly after you create them; adding or removing a workspace folder updates results
- Mentions insert as teal chips with a file or folder icon; hover shows the path, × removes the chip, click opens it in the editor
- The `@query` you are typing is highlighted while the picker is open
- Clicking a chip on a sent message still opens the file and does not reopen the picker
- File and folder rows include Open in editor
- Folder mentions attach a directory listing; file mentions still attach the file contents
- `@repo-map` lists workspace folders again, not only the entire codebase
- Arrow keys wrap, Home/End jump, Tab selects, Esc closes and keeps what you typed; Back returns from a submenu
- The picker stays on screen in a narrow sidebar and stays readable in light and dark themes
- `@diff`, `@problems`, `@terminal`, and `@memory` still work as before

- Update VS Code 1.136.0
- Remove ONNX Runtime


## V1.4.6

- Fix: Cancel was stopping the parent stream but leaving child agents and background shells running

### Memory — Stay on the current question

Injected memories no longer cling to the previous topic after you switch tasks or say “continue”.

- Short follow-ups like “continue” and “ok” pull memories for the work you were just doing, not leftover stopwords
- A memory has to actually match the current question (words, a named thing, or a pin) before it is injected
- Switching topics clears the old scratchpad; “continue” keeps the current one
- The goal shown for the turn is your current ask, not the last stored fact
- Unrelated pinned items and old session summaries no longer crowd out what matches this turn
- Duplicate facts update the existing memory instead of creating a second copy; filler like “I'll update the file” is not stored
- Common words like “file” or “config” no longer drag in unrelated memories
- Wrong injects are not ranked higher next time just because they showed up once
- **Not relevant** in Memories used demotes an item for this topic without deleting it; Pin later brings it back. Forget still deletes
- Memories used shows why an item matched (score and evidence). In selective mode, weaker items stay collapsed
- Memory → Settings has a Retrieval Precision section to tune how strict injection is

## V1.4.5

- Task plan sits in the same attached stack above the input as Memories used and background jobs
- Task plan tracks each step live from later file writes and commands, not only builtin_plan updates
- Task plan steps use colored checkboxes instead of Pending / Done labels
- Plan dumps no longer appear as related context items in the chat stream
- Reloading a chat still injects the last Task Execution Plan into the model from history
- Chat streaming stays steady while long unwrapped code lines generate; the thread no longer shakes with each token
- Completed markdown sections stay put when the reply moves from a code fence to the next heading or list
- Thinking and reasoning blocks use the same smooth streaming path as the main reply
- Reasoning effort is remembered per model after you close and reopen the editor
- File-read tool cards stay collapsed by default instead of showing an empty line 1
- Context compacted sits in the same attached stack as Memories used and background jobs
- Fix bug with etBundledSkillsPath()
- Fix bug with createRequire
- Fix duplication messages
- Chat history list stays readable in a narrow sidebar: dates, titles, and actions no longer clip
- Agent and autonomous no longer stop at 40 / 120 tool rounds by default; both run until done or Cancel
- Autonomous outer iterations also default to unlimited (0); set a positive number in Memory settings to cap
- Turn meter still shows steps used, activity rows, and outer-loop count; a used/max bar appears only when a cap is set
- Switching Agent profile no longer overwrites a custom step cap with 40 / 60 / 120

## V1.4.4

- Agent no longer stops after editing a file
- Failed edits are returned to the model so it can retry instead of aborting the turn
- Writes to files already open in the editor apply through the buffer (no revert-dialog race)
- Make tool outcomes explicit
- Enhance Memory System
- Enhance Checkpoints System

## V1.4.3

- Beautify Checkpoints Timeline list UI
- File search no longer walks the whole workspace during startup
- System CA setup no longer blocks activate
- First paint no longer waits on IndexedDB migration

## V1.4.2

### Agent — One loop for chat, autonomous, and subagents

- Agent chat, `/autonomous`, child agents, and scripted eval share the same tool loop: streaming replies, permission prompts, stop, doom-loop, and step budget
- `/autonomous` now actually edits, searches, and runs commands instead of writing a text-only plan; live tool cards appear in Agent chat while it runs
- Ask / Accept / Auto use the same permission rules as chat; writes still wait on the bar unless Auto is on
- The turn meter shows outer-loop iteration banners for autonomous runs (started, each iteration, completed)
- Consecutive read-only tools in one turn still run in parallel; writes stay sequential

### Agent — Systems profile for kernel and QEMU work

- New **systems** profile: 120 tool steps per turn (default remains 40), doom-loop after 5 identical failures (default 3), and compile as the post-edit check
- Auto-selects systems when the workspace looks like a Linux kernel or QEMU; an explicit step cap or verify command still wins
- Turn meter shows a systems label; zero still means unlimited (hard cap 1000)
- Agent config covers profile, verify mode / command / iterations, job log directory, and await timeout

### Agent — Rust profile for Cargo workspaces

- New **rust** profile: 60 tool steps per turn, doom-loop after 4 identical failures, and `cargo check --workspace --all-targets` as the post-edit check
- Auto-selects rust when the workspace root has `Cargo.toml`; kernel / QEMU still win on mixed trees (a rust-for-linux tree stays systems)
- An explicit verify command still wins; GUI can pick Rust or Auto, and the turn meter shows a rust label
- First-turn **Codebase Card** for Cargo: crate / edition, workspace members, rust-toolchain / clippy / rustfmt files, and crate versions pinned from `Cargo.lock` (missing lock asks for `cargo generate-lockfile` — versions are not invented)
- Bundled rust skill: fmt → check → clippy `-D warnings` → test; prefer rust-analyzer hover over guessed method names; `rustc --explain` on the first error code
- Checkpoints ignore Cargo `target/`, rlib, rmeta, and incremental artifacts so restore stays source-only; `Cargo.lock` is kept so pinned crate versions stay accurate

### Agent — Compile and boot as the oracle

- After edits, a configured verify command (or the systems / rust profile) runs the compiler or test instead of waiting on editor diagnostics
- C, assembly, Kconfig, and device-tree edits skip language-server auto-fix even without a command, and the model is told to build
- Build output is parsed for gcc / clang / kbuild, rustc / clippy, linker, make / ninja / meson, cargo test / nextest, kselftest / KUnit failures, and Linux oops / panic / KASAN
- rustc and clippy JSON keep error codes, clippy lint ids, and machine-applicable `Suggested fix:` replacements the agent can apply instead of guessing
- A kernel panic, sanitizer hit, or kselftest failure counts as a failed check, not a clean verify
- Identical error signatures trip a circuit-breaker so the agent does not keep patching the same failure
- Clean builds and failed builds are remembered with high importance so restore and recall keep the last oracle

### Agent — Cargo as the compile oracle

- On the rust profile (or any `cargo …` verify command), post-edit truth is `cargo check`, not rust-analyzer auto-fix
- If rust-analyzer is live, its typed diagnostics are used; if it is missing or not ready, LSP is skipped and the model is told to `builtin_build` or install rust-analyzer (empty `.rs` hover never mentions `compile_commands.json`)
- After a **green** check: `cargo fmt --check`, then clippy `-D warnings` (library crates also deny `unwrap_used`, `expect_used`, and `await_holding_lock`). Red check skips fmt / clippy so the inner loop stays fast
- Edits under `crates/foo` compose `-p foo` (never `-p` together with `--workspace`); inner check / clippy never default `--all-features`
- Chat and `/autonomous` will not treat a no-tool “fixed” message as done while the last cargo / clippy / test oracle is red
- `builtin_build` refuses `cargo clean`, `publish`, `login`, `yank`, and `cargo fix --broken-code`; missing cargo returns an install-rustup message instead of a failed spawn
- Extra actions on the same build tool: clippy, fmt, test (optional doctests), rustc `--explain`, rustdoc lookup, `cargo fix --allow-dirty`, expand, miri, tree, deny, audit. Missing cargo-expand / cargo-deny / cargo-audit / miri return install hints
- rustdoc lookup reads `target/doc/<crate>.json` when present; otherwise it points at the locked `vendor/` or `~/.cargo/registry` source — not docs.rs
- Registry and git checkout trees are readable; writes under `~/.cargo/registry` and `~/.cargo/git` are denied

### Agent — Rust idiom gates

- Borrow-checker errors (E0502 and other named E04 / E05 / E06 codes) append an ownership remedy: split borrows, index access, `mem::take` — not `.clone()` / `Arc<Mutex<_>>` without a reason
- New `.clone()`, `Arc::new`, `Mutex`, `RefCell`, or `Rc` without `// share:` or `// owned:` is flagged (the edit is not auto-reverted)
- New `unsafe` (block, fn, impl, or trait) without `// SAFETY:` is flagged; the model is told to run miri before claiming soundness
- Deleting asserts, adding `#[ignore]`, or using `todo!()` / `unimplemented!()` to make tests green is rejected unless you explicitly asked to change tests
- Trait coherence errors (E0117 / E0119) remind the model to use a newtype, not thrash impl headers

### Agent — Rust-aware subagents

- `rust-review` (readonly): unwrap / expect, needless clone or Arc, non-Send futures, missing docs / examples on new `pub` items, semver
- `rust-borrowck` (readonly): ownership and lifetime errors after a failed borrow-check
- `rust-architect`: write an ADR with the plan tool before new public APIs or type topology — no nested `task`
- Soft routing only (no automatic spawn): borrowck after one failed borrow attempt; review before claiming done; architect for new public APIs

### Agent — Doom-loop understands rebuilds

- Rebuilds (make, ninja, QEMU boot, await, interactive read) fingerprint the **error**, not the command: a new compiler line or a new oops RIP is progress
- Mutating edits reset the streak; identical searches still stop the loop
- Three identical makes or QEMU boots with the same error still stop and force a summary (five on systems)

### Shell — Hour-long builds and job logs

- Make, ninja, cmake build, meson compile, configure, QEMU, and `cargo check|build|test|clippy|nextest|bench|doc|miri` start in the background immediately unless you ask to wait (`cargo metadata`, `cargo tree`, and `cargo fmt --check` stay in the foreground)
- Await-shell default wait is 10 minutes (override via job timeout); a multi-minute wait no longer hits the generic tool timeout
- Every job writes a full log; the jobs panel shows the path
- Await supports tail, grep, resume from a byte offset, and errors-only (parsed diagnostics instead of the whole make log)
- Truncated make / ninja / gcc / cargo / QEMU output returns parsed errors plus the last lines, not a huge compile transcript
- Completed jobs feed the terminal error classifier: compiler errors, undefined references, kernel panic, QEMU, ninja / kselftest failures; a kernel make does not suggest npm install

### Shell — Interactive sessions for QEMU, GDB, and consoles

- Start, send, and read interactive sessions (QEMU, GDB, kgdb, shells) with a job id and a full log
- Prefers a native TTY when available so Ctrl-C is a real interrupt; otherwise falls back to piped input
- Send supports interrupt and end-of-input; read waits from the last offset or a byte position
- Interactive jobs appear in the same jobs panel as background shells

### Agent — Compaction keeps the development loop

- When history is compacted, the last compiler errors (including rustc `E0xxx` and clippy lint ids), verify command (`make` or `cargo …`), background job ids, QEMU / GDB descriptors, failing test, oops RIP / call-trace, monitor VM status, and the task plan are pinned as development-loop state
- Verbose serial and make output keep RIP and job identifiers instead of collapsing to a short header
- Compiler spam is summarized as file counts plus file:line errors

### Agent — Persistent task plan

- Create, add, update, complete, skip, set current, list, and clear a multi-step plan that is injected every Agent turn (including autonomous iterations)
- The plan survives compaction so a kernel-scale checklist is still visible after hours of work

### Navigation — Large trees (kernel / QEMU scale)

- Directory walk always recurses; glob, repo map, and tree share the same ignore rules
- Walks honor git and Knox ignore files, and skip kernel images and modules even without an ignore file
- Repo map lists subsystem directory counts first so large trees survive the token budget; signatures only for the top-ranked files (maintainers, makefiles, Kconfig, git-touched, query, recent mtime)
- Incremental repo-map cache; zoom into a path
- Glob walk and result caps are explicit: truncation tells you to narrow the path, not a silent short list
- Out-of-tree build directories, dependencies, and object files stay skipped unless the pattern asks for them or the search is already rooted in a build tree
- Search default 50 hits; systems / kernel workspaces default 200; truncation says to pass max results, path, or file type
- Repo map and signatures understand assembly entry points, Makefile targets, Kconfig symbols, and linker entries
- Over-budget reads return line count, first and last lines, and a hint to use a line range — not a truncated middle; binaries, objects, kernel images, and modules are refused
- First-turn **Codebase Card** for kernel, QEMU, and Cargo: how to build and search, top-level dirs, architecture from config symbols only (the config is not dumped)
- Maintainers lookup / search / list from MAINTAINERS globs without dumping the rest of the file

### Navigation — Language server, tags, and compile database

- Workspace symbol search uses the query you passed (it no longer always searches empty)
- Empty C go-to-definition distinguishes **no compile database** (generate it once with bear or clang-tools) from **no language server**; include paths are not invented
- Empty Rust hover / go-to-definition tells you to install rust-analyzer and use `builtin_build` / the locked registry source; it does not mention `compile_commands.json`
- If workspace symbols are empty, fall back to ctags when present, and optionally cscope (not bundled)

### Git — Blame, pickaxe, and bisect

- Blame a file (optional line range, capped)
- Log pickaxe: search for added or removed strings, or a regex
- Bisect start / good / bad / skip / run / status / reset; run executes an oracle command until the first bad commit; always resets on abort; never force

### Debug — Debugger as an Agent tool

- Launch, attach, breakpoint, continue, step, backtrace, locals, evaluate, status, and disconnect through VS Code’s debugger
- Offered only on the systems profile or when a debug session is already active

### QEMU and serial

- Start a QEMU job from a command line (kernel, initrd, serial, optional gdbstub); optional human monitor on stdio with guest serial in the job log
- Stop kills the process group; status waits for serial and prepends a parsed oops when present
- Monitor commands prepend VM status and RIP
- `@serial` tails recent QEMU and interactive-session logs with oops parsed; the same tail can be injected as context
- Oops / panic / KASAN / UBSAN / QEMU guest fault / gcc ASan are parsed for RIP / symbol, tainted, and call-trace locations

## V1.4.0

### Agent — Live status rows follow

- Activity list above the input sticks to the latest row while the agent is working, same as chat streaming
- Scroll up to browse earlier steps without jumping; scroll back to the bottom to resume following new rows

### Checkpoints — Complete restore

- First checkpoint (and each agent-turn baseline) captures every tracked file, not only files that changed after the session started
- Restore rewinds the whole tree as it was at capture time, including files you never edited
- Evicting old checkpoints folds unique file content into the next one so newer restore points stay complete
- Restore reports the files actually written, not a partial change list

### Checkpoints — Safety

- Restore and import cannot write outside the workspace
- Secrets (`.env`, keys, credentials) are ignored by default unless you opt in
- Checkpoint writes are atomic with integrity checksums; tampered or truncated data is rejected instead of partially applied
- Restore is all-or-nothing: cancel or a crash rolls back instead of leaving a mixed tree
- Overlapping create / restore / delete operations wait or prompt so two restores cannot interleave

### Checkpoints — Storage

- Unique file contents are stored once; optional compression; checkpoint records no longer embed full source
- History stays a lightweight index; if it is lost, the list rebuilds from on-disk records
- Storage quotas evict the oldest unpinned checkpoints; pin important restore points so they survive cleanup
- Each workspace folder has its own isolated store, including multi-root windows
- Large repos scan faster: deep trees are included, ignored directories are skipped
- Binaries and UTF-16 text round-trip without corruption; oversized or unreadable files are skipped with a warning
- Optional encryption at rest for stored file contents (key in the OS keychain)
- Checkpoint IDs stay unique across export and import

### Checkpoints — Auto-capture and settings

- Advertised settings drive the engine (compression, quotas, auto-interval, smart tracking, inline diff)
- Auto-checkpoint timers and file-change thresholds apply without reload
- Create supports tags and include / exclude path filters; agent sessions can track only the files they touched
- One file watcher per folder; undo stack and session labels survive reload

### Checkpoints — Preview, selective restore, and file history

- Preview a restore before any write: counts of added, modified, and deleted files, then restore all or selected files
- Right-click a file for its checkpoint versions and restore that file only

### Checkpoints — Agent

- Agent can list, create, diff, preview restore, pin / unpin, restore, and delete
- Restore still requires confirmation and never auto-runs

### Checkpoints — Explorer, commands, and editor

- Checkpoints appear in the Explorer sidebar, grouped by date, session, or pin, with restore / preview / diff / pin / delete
- Command Palette uses one Checkpoints namespace (create, list, restore, stats, export / import, undo / redo, branches, health)
- Keyboard shortcuts for create, undo, redo, open panel, and file history
- Status bar shows health and last-checkpoint age; click opens the list
- Optional inline diff decorations in the editor against a chosen checkpoint

### Checkpoints — Overlay

- List stays usable at hundreds of checkpoints: pagination, search by tag / path / session, compare any two checkpoints, keyboard range / delete / restore
- Timeline, analysis, performance dashboard, and local share tabs show real data
- English and Chinese strings for all shipped checkpoint screens

### Checkpoints — Branching

- Named local branches; new checkpoints attach to the active line
- Switching branch does not auto-restore (you can restore to the branch head if you want)
- Merge reports conflicting paths instead of silently overwriting

### Checkpoints — Analysis, health, and sharing

- Local risk / impact hints and grouping by session, time, or path
- Performance dashboard uses real restoration history, storage stats, and AI session line / rollback counts
- Local audit log for create, restore, delete, pin, import, and export
- Health check verifies integrity and can rebuild the index, collect unused storage, and finish or roll back an interrupted restore
- Export / import uses checksummed local bundles; truncated or tampered files are rejected
- Share is a local bundle file (email, USB, PR artifact) — no cloud or network

## V1.3.9

### Agent — Auto-approve by default

- Catalog tools default to Auto-Approve; Ask on write remains a preset
- Auto-Approve (including Await Shell wait/kill) runs immediately — no Deny / Always / Approve popup
- Cancel is an X on the input bar only (no duplicate stop on the turn meter)

### Agent — Unexpected tool abort resumes

- If a tool is aborted by Core/IPC (not the user hitting Stop), write an error tool result and continue the model instead of ending the turn silently
- Canceled tool cards say Canceled, not Processing

### Agent — DSML tool-call leak

- Parse `<｜DSML｜…>` tool markup (fullwidth `｜`, U+FF5C) instead of printing the tags and stopping the turn
- Hold tags back while they stream so they never flash in chat; convert them to real tool calls (`builtin_read_file`, shell, etc.)
- Also accept `<｜DSML｜function_calls>` wrappers and incomplete blocks at end-of-stream
- Recover leaked markup from the current turn even when a thinking block is last, and from subagent / eval streams that skip the GUI recover path
- Strip DSML from chat history so it is not replayed to the model on the next turn

## V1.3.8

### Memory — Mass manage (checkpoint-style)

- Memories tab: Select / Select All / Clear / Exit, sticky action bar, and a header checkbox
- Bulk delete uses one transaction and one safety checkpoint (no per-row round trips)
- Bulk pin / unpin, plus copy selected memories as JSON or Markdown
- Filter by pin state, date grouping (Today / This Week / This Month / Earlier), Shift-click range select, Esc / ⌘A

### Agent — Soul (memory × checkpoints)

- One session id: GUI `session.id` binds AgentModeManager, workspace checkpoints, and Memory Brain
- SoulEvent after mutating tools, deny, doom-loop, maxSteps, ask_user, worktree apply/discard, and compaction
- First mutating tool of a turn always creates a workspace checkpoint (`soul-turn-…`)
- Restore injects a system note; optional files + memory rewind (`rewind_memory` or Restore files and memory). Memory rollback/forget offers the linked file checkpoint
- User Deny, Always (this chat), Ask/Edits/Auto flips, and background job complete/kill write a SoulEvent
- GUI overlay, chat checkpoint button (Shift), and timeline CP stamps use the same restore modes
- Post-turn memory includes settled tool names, status, and paths (not only assistant prose)
- `builtin_workspace_checkpoint` (`list` / `create` / `restore`); explore/review subagents may recall but not write memory
- Activity timeline shows workspace CP stamps; checkpoint overlay can filter to this session

### Agent — Eval & honesty gate

- CI golden tasks (no live LLM): StrReplace, multi-file `apply_patch`, test-fix loop, permission deny, maxSteps stop, abort mid-tool (`core/eval`)
- PR checklist + `honestyGate` test: no new tool def without a `callTool` impl; do not claim SmartToolRouter as the default path

### Agent — i18n

- Translate leftover Agent UI: permission presets, path/command policy editor, background-job labels, and tool-error cards (en / zh)
- Localize extension progress notifications (`Generating code`, structured-solve steps)

### Agent — Background jobs panel

- Agent tab **Jobs** chip lists detached shell jobs and in-flight `builtin_task` subagents (Claude Agent view lite)
- Jobs list above the input matches **files changed**: click the header to show/hide rows; click a command to inspect output; Clear finished
- Kill / dismiss from the panel; process-group kill so pipelines stop; richer PATH so `cargo` / Homebrew bins resolve from GUI-launched VS Code

### Agent — Worktree isolation

- Optional **Worktree** chip on the Agent tab (Claude `EnterWorktree`): edits and shell run in a `git worktree` until you **Apply** (copy files back) or **Discard**

### Agent — Activity timeline & turn meter

- Compact per-turn activity list after each user message: thinking → reads/searches → edits → tests/shell (click a step to jump to the tool card)
- Agent input bar shows steps used / max, estimated tokens, elapsed time, and a Stop control while the turn is active

### Agent — Subagents, Git, Ask User, Doom Loop

- Stop repeated identical tool calls or failure streaks (default 3) and force a text-only summary (`experimental.agentDoomLoopThreshold`, `0` = off)
- Add `builtin_task` child agents: `explore` / `review` (read-only) and `general` (full tools). Isolated context; parent gets a summary and files touched
- Add `builtin_ask_user` for mid-run multiple-choice or short answers (never auto-approved)
- Add `builtin_git_status` / `builtin_git_diff` / `builtin_git_log` / `builtin_git_commit` (no push/force). Prefer these over shell git
- `composite_health_check` reports workspace facts instead of invented scores

### Agent — Apply Patch, Permissions, Shell, Rules

- Add `builtin_apply_patch` (Codex-style `*** Begin Patch` multi-hunk / multi-file, applied atomically with unified-diff output)
- Session permission modes: **Ask** / **Edits** / **Auto** (cycle from the Agent tab or Shift+Tab). File edits auto-run in Edits; Auto is YOLO for the session without rewriting saved tool settings
- Tool cards: **Deny** / **Always** (this chat) / **Approve**
- Persistent terminal cwd across calls; tool results include exit code, duration, cwd, stdout, and stderr
- Stream terminal stdout into the tool card while a command runs; long commands auto-background after ~30s (or `background: true`) and are polled with `builtin_await_shell`
- Run independent read tools in parallel when the model emits multiple tool calls; file writes stay sequential
- Load `AGENTS.md`, `CLAUDE.md`, and `.knox/AGENTS.md` with `.knoxrules` (Knox-specific wins). Nested `AGENTS.md` is picked up from the open file’s folders
- Path & command policy: `allow` / `ask` / `deny` globs (editor in Tools permissions). Deny always wins, including in Auto. Defaults block `rm -rf`, `~/.ssh`, and similar; paths outside the workspace ask (configurable). AGENTS.md supports `always` / `ask` / `never` blocks

### Agent — One Switch

- Sync the Chat/Agent tab (`session.mode === "agent"`) with VS Code `AgentModeManager` — one Agent switch for tools, checkpoints, undo, shadow preview, and verification
- `knox.isAgentModeActive` now reports real status (not merely that the manager was constructed); command-palette toggle updates the tab and vice versa

### Agent — Edit & Discovery Tools

- Add `builtin_edit_file` (exact `old_string` → `new_string`; fail on 0 or multiple matches unless `replace_all`) on the default tool list, with undo snapshots and post-edit verification
- Add `builtin_write_file` for full-file create or overwrite; prefer over shell `cat` / `echo` / heredoc writes
- Teach the model the canonical edit path in the system prompt: read before edit, prefer StrReplace, never write files via the terminal; `composite_smart_edit` stays opt-in/legacy
- Add `builtin_glob` for file find by pattern (`**/*.ts`, `src/**/*.tsx`)
- Grep (`builtin_exact_search`): skip binary files (ripgrep default); document line numbers, path globs, context lines, and head limits
- Clarify `builtin_view_subdirectory`: depth limits, ignore globs, and stable sorted output; point pattern-only finds at Glob

### Agent — Permissions

- Safer defaults: reads auto-run; writes, terminal, and web ask first (unknown tools ask)
- One-click **Ask on write** and **YOLO** (full auto) presets in the tool permissions dialog; existing saved settings are unchanged until a preset is chosen

## V1.3.7

### Chat UI

- Move Chat History into the input toolbar overlay so sessions can be browsed without leaving chat; remove the title-bar History button
- Add Configuration tab to the in-chat Checkpoints overlay (full parity with the former restore page); remove the title-bar Checkpoints button
- Replace title-bar Memory icon with brain icon
- Replace VS Code blue button/badge accents with Knox teal — dark `#159994`, light `#0f7a76` — for primary buttons, focus rings, and selected toolbar chips (e.g. Checkpoints)

### Agent — Reliability

- Unify agent toggle on `knox.toggleAgentMode` (keybinding + command palette)
- Cap Agent tool-loop steps via `experimental.agentMaxSteps` (default **40**, `0` = unlimited) with Settings UI; on the cap, force a text-only summary turn and block further tools
- Harden cancel / Stop: clear dangling `calling` tool UI, abort in-flight tools via `tools/cancel` + `AbortSignal`, kill local terminal commands (SIGTERM → SIGKILL), reject half-applied streaming diffs, and do not continue the agent loop after cancel
- Keep Stop available while a tool is mid-flight (including after the LLM stream ends); Cancel on the tool card during `calling`

## V1.3.6

- Responsive Session History Browser
- Fix Node 24 crypto typing in brain export encrypt/decrypt and skill hash helper
- Align `brain/getEffectiveContext` protocol type with `CompressionRatios` (active/hot/warm/cold/frozen) instead of loose `Record<string, number>`
- Fix memory brain barrel export — export `calculateHierarchyEffective`, `getMemoryLevelSpecs`, and related types from `MemoryHierarchy` (removed invalid `MemoryHierarchy` class re-export)
- Type `BrainManager.consolidate()` as `SleepCycleResult` so sleep sub-phase counts (`sub_phases`) match runtime and tests
- Fix `knoxChatModels` Vitest mocks — typed `localStorage` stub and remove invalid `reasoning: null` on model metadata
- Update some phrases for multilingual
- Remove Legacy JSON config

## V1.3.5

- Beautify Checkpoints diff

### Memory Brain — Knox-MS Local Alignment

Complete local Knox-MS behavior parity — all memory in `~/.knox/memory/brain.sqlite`, no cloud sync or remote memory APIs. Full roadmap implemented and validated (70 automated regressions + checklist validator).

- **8-phase memory pipeline** (φ₁–φ₈) — pre-turn, post-turn, retrieval, and sleep consolidation orchestrated end-to-end; phase counts and cycle-invariant status on Memory Overview
- **Neural region pipeline** — sensory capture, attention gating, hippocampal encoding, prefrontal planning, amygdala salience boost into working memory, basal-ganglia procedural pattern record on post-turn extraction, hippocampus→prefrontal goal feedback
- **M₁ sensory buffer** (~250ms ring buffer, configurable `sensory_buffer_ms`); editor change events stream into sensory ingest; M₁ included in hierarchy metrics
- **5-tier hierarchy** (M₁–M₅) compression ratios (active/hot/warm/cold/frozen) wired to unlimited-context theorem `C_total = W_max + Σ |Mᵢ| / rᵢ`
- **Effective Context Capacity dashboard** — W_max, hierarchy effective tokens, M₁–M₅ tier breakdown, last-build window utilization bar, C_effective trend via metrics snapshots
- **Working memory** aligned to spec — configurable slots, 30K token budget cap, and TTL per session (`working_memory_max_slots`, `working_memory_token_budget`, `working_memory_ttl_seconds`)
- **Sleep consolidation** full φ₇ cycle with sub-phase counts (NREM replay, decay, REM distill) shown in Memory Overview
- **Ebbinghaus decay** — unified retention curve; configurable λ, θ_prune, and review-due list on Overview
- **Retrieval fusion** — θ=0.6 cutoff and top-k=20 defaults (configurable); optional rule-based enhanced semantic scoring (`enable_enhanced_semantic`)
- **Knowledge graph** — 5K entity cap with LRU refresh on re-mention, search, and fusion; depth-3 spreading activation (γ=0.7 decay); cap utilization bars on Overview and Graph view; graph explore respects configured max depth
- **Context assembly** — C_goal injection (todo plan → session summary → last user message) with provenance in Injected Memories panel; up to 100K token budget (2K–100K slider); compress-oldest overflow with `memory_tokens_saved` metrics
- **Memory modes** — `full` / `summarized` / `selective` (high-threshold minimal injection); persisted in local memory preferences
- **Session consistency** — IDE session ID is single source of truth; session switch closes prior brain session and restores working memory; `project_id` backfill on sessions
- **Local memory preferences** — expanded settings in Memory → Settings (memory mode, context budget, auto_summarize, summarize_threshold, enable_knowledge_extraction, `post_turn_min_chars`, `sensory_buffer_ms`, `memory_build_timeout_ms`, memory_scope, enhanced semantic, task-routing models); no network fetch
- **Post-turn knowledge extraction** — facts, concepts, and patterns categorized into semantic memory and linked to graph entities; fires when turn length ≥ `post_turn_min_chars` (or tool use); respects `enable_knowledge_extraction`
- **Cross-session history** — Session History tab with full episodic + semantic browse; debounced cross-session backlog search
- **Project-scoped memory** — `memory_scope`: `project` (workspace hash) | `global`; retrieval, context build, and backlog search filter to current workspace by default
- **Local autonomous loop** — `/autonomous <goal>` slash command; multi-step agent loop with memory pipeline per iteration; task router resolves easy/medium/hard models each iteration; responsive **cancel** via abort signal; checkpoints after each step; PlanTaskStatusPanel progress UI
- **Local task routing** — difficulty scoring (message length, code blocks, tool count) maps to user-configured easy/medium/hard models in Settings
- **Chat memory injection** — bounded context-build timeout; chat continues on timeout with “Memory unavailable” notice in Injected Memories panel
- **Agent tools** — memory pipeline, effective context, phase status, metrics trend, autonomous loop start/cancel, session history, and backlog search exposed to the agent

## V1.3.4

### Agent — Undo, Verification & Apply

- Real file snapshot undo/redo for mutating tools (`knox.undoLastOperation` / `knox.redoLastOperation`); GUI chat and agent share the same Core `tools/call` hooks
- Optional post-edit verification after edit/create/reapply (`knoxchat.enablePostEditVerification`) with per-file circuit breaker
- Shadow workspace Accept/Reject preview for Apply (`knoxchat.enableShadowPreview`) — Accept uses the chat apply path; Reject leaves the original untouched
- Fail-closed tool arg validation against each tool’s JSON Schema (no TODO/untitled placeholders written to disk)
- Refactoring via VS Code LSP (rename/move) and LLM extract; real `builtin_generate_tests` implementation
- ToolCallInterceptor quarantined (not on the live path); incomplete args rejected with model-visible errors

### Plan & Task Manager — Truthfulness

- Evidence-gated todo completion — no keyword bulk-skip; stream-end completes only evidenced items
- Verified shield requires real evidence (`Verified:` / tool-evidence); panel prefers local todos
- Medium+ complexity plans can require Approve/Reject before execution

### KnoxChat Models & Reasoning

- Persisted warm `/v1/models` cache (disk + localStorage); hydrate before capability lookups so cold start does not hide toggles
- Reasoning effort from API metadata → sidecar overrides → gateway default; sticky per-model effort in the UI
- Thinking indicator for any reasoning-capable model; `reasoning_effort` pass-through includes `"none"`

### Web Search

- Retrieval-first web search — never present unlabeled LLM fiction as “search results”
- Native `web_search` fallback only when the model advertises support (always labeled); otherwise a clear error
- Web-search toggle and `builtin_search_web` stay mutually exclusive and metadata-driven

### Memory Brain

- Injected memories merge into the leading system message and survive compaction; structured provenance with Pin/Forget in the UI
- Await session track / working-memory restore on load; post-turn memory write for substantial turns
- Optional AES-GCM encrypted export/import for local backup (still 100% local — no cloud sync)

### Compaction & Context

- Optional LLM summarization behind `experimental.useLlmSummarization` (timeout/input caps, heuristic fallback)
- Tool-call pairs kept atomic; memory/plan system blocks preserved
- CompactionStatusPanel shows when compaction ran (message counts; KnoxChat handles billing)
- Local context budget uses lightweight char÷4 estimate only — removed tiktoken/worker encoders from the extension

### Tools, Edit & Apply Hardening

- Every tool in the catalog has a `callTool` route; composite tools wired; SmartToolRouter unexported
- ToolTransaction default file rollback (create → remove; edit → restore prior contents)
- Lazy-apply handles top/bottom `UNCHANGED` markers; deterministic apply with confidence rejects
- Repo map signatures from real tree-sitter symbols (mock loop removed)
- `overwriteFile` with null previous content deletes the file on disk
- Anthropic adapter: completions + `list()` implemented; FIM omitted (autocomplete already removed)

### Skills, Rules & Context Providers

- Documented merge order for system message / rules / skills / prompts / memory
- Rules `applyTo` glob filter against open files; skill intent matching injects suggested-skill hints
- Remote skills hashed/pinned; default `@` providers always ensured (file/diff/problems/repo-map/terminal/memory)
- Niche integrations key-gated; dead CodeOutline/CodeHighlights stubs removed

### GUI Stream & Slash Robustness

- Cancel stream leaves tool conversations consistent (`clearDanglingMessages`)
- Prompt-based slash commands expand client-side and stream with tools
- Find widget supports regex (`.*`); checkpoint index association is deterministic
- Deprecated JSON `customCommands` warn toward removal in V1.4; `.prompt` / `.prompts` loaders unified

### Reliability, i18n & Tests

- Leveled `knoxLog` logger (prod/binary default quiet); strip noisy production debug logs
- en/zh locale key parity enforced in CI; ban raw English `show*Message` in agent/checkpoint paths
- Critical-path vitest coverage: models, tool routing, web search, compaction, todo evidence, config/org, middleware validation
- Debug tracker re-enabled with 300ms debounced `@debugger` refresh; DAP `continued` event fixed
- VsCodeWebviewProtocol `invoke` / `onError` implemented (no stub throws)

## V1.3.3


### Autocomplete — Removed

- Remove tab autocomplete / inline completion entirely (core engine, Rust module, VS Code provider, and native binary)
- Remove autocomplete model role, settings, commands, and keyboard shortcut (`Cmd/Ctrl + K, Cmd/Ctrl + A`)
- Remove autocomplete status bar item (the `Knox` checkmark in the status bar) and related battery-pause settings
- Remove Autocomplete section from Settings and model-role picker in the GUI
- Drop autocomplete from config schemas, dev-data events, and install/build scripts

### Plan & Task Manager — Lifecycle

- Clear stale plan/task UI before each new user message so completed work does not bleed into the next turn
- Auto-dismiss completion summary after ~6 seconds; completed todo sessions move to history instead of staying in the active view
- Full plan/task reset on new chat tab — no stale panels from prior conversations
- Cancel idle, active, or paused todo sessions when a fresh complex message supersedes them
- Fix conflicts where a prior completed session kept the plan panel visible on simple follow-up messages
- `fetchCurrentSession` no longer re-promotes completed sessions to the active UI

### Plan & Task Manager — Panel UX

- Pin expand/collapse on the task status panel — manual toggle sticks for the whole turn (including across tool-call rounds)
- Stop auto-collapsing or re-expanding the detail panel when streaming state flickers between tool rounds
- Use `session.isStreaming` as the stable in-progress signal so the panel does not flash into completion mode mid-turn

### Git File Changes Panel

- Pin expand/collapse on the “N files changed” list above the input — periodic git refreshes no longer force it back open
- Auto-expand only when file changes first appear from an empty state

## V1.3.2

### Memory Brain — 100% Local

- Remove CloudSync and KnoxMsSync entirely — memory is now fully local with no cloud or Knox-MS sync paths
- Fix session-scoped episodic search returning zero results (SQL parameter order in fusion FTS)
- Fix hyphenated queries silently disabling FTS5 (e.g. `build-cache` now quoted as valid FTS5 syntax)
- Wire the chat model into the Memory Brain on config load so LLM-enhanced extraction and summarization run outside memory-tool calls
- Close brain sessions when switching chats or starting a new session (topic flush, auto-summarize, working-memory persistence)
- Auto-consolidation now runs the full sleep cycle every tick and closes sessions idle for 24+ hours
- Restore working memory when reopening a previously closed session

### Memory Brain — Chat Integration

- Track sessions and record user/assistant turns in the Memory Brain during streaming
- Inject memory and plan context at the start of each turn; re-inject across tool-call rounds via a context cache so the LLM does not lose memory after the first tool round
- Auto-store task completions to the brain on stream completion
- Start auto-consolidation scheduler on Core startup

### Memory Brain — UI & Protocol

- Fix Memory Browser field names and pagination (`importance_score`, `retrieval_count`, `source_session_id`, offset browse)
- Rewrite Memory Settings to match core config keys; add tiering, features, and import-from-file
- Fix `brain/searchMemories` to query `brain_semantic` with category filter and offset pagination
- Extend `brain/import` to accept raw JSON from webviews; `brain/heal` accepts a specific action
- Remap task-completion auto-store to `summary` category with `task-completion` keyword

### Memory Brain — Hardening

- Unify legacy MemoryManager as a Brain adapter with one-time `memory.sqlite` → `brain.sqlite` migration
- Fix BrainStore init race with a single in-flight promise; fix `build_context` to skip stale snapshot cache when a query is provided
- Add Vitest regression suite and GitHub Actions workflow for core memory tests
- Remove dead code: MemoryProvider, ConventionExtractor, MemoryPanel

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
- **FTS5 Fusion Search** — SQLite FTS5 full-text search with BM25 ranking, trigram fuzzy matching, and Bloom filter deduplication
- **Multi-Strategy Retrieval** — Weighted fusion of FTS5 BM25, trigram, graph traversal, recency decay, and importance scoring with auto-detected query-type weights
- **Frozen Snapshot Pattern** — Session-scoped context caching with automatic invalidation after significant changes
- **Cloud Sync** — Compressed backup/restore of memory database with cloud storage integration *(removed in V1.3.2 — memory is local-only)*
- **Knox-MS Server Sync** — Background sync of local memories to Knox-MS server with change tracking and conflict resolution *(removed in V1.3.2)*
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