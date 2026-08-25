# The Agent. The Memory. The Solution.

**Knox** is an AI coding environment for VS Code — not a chat overlay. An autonomous agent, a local memory brain, and git-independent checkpoints share one session, so the model can plan, act, remember, and rewind as a single system.

Bring your own models. Anthropic Claude, OpenAI GPT, DeepSeek, Gemini, Qwen, Grok, GLM, Codestral, Sonar Pro, and Knox's On-Demand model `knox/knox-ms` are supported out of the box.

[![Video Title](https://docs.knox.chat/img/main-ui.png)](https://www.youtube.com/watch?v=jx2tMqUGcuk)

**Three systems. One loop.**

| | | |
|---|---|---|
| **Agent** | Plans, edits, searches, and verifies — 29 built-in tools, Ask / Edits / Auto permissions, worktree isolation | `Cmd/Ctrl + Shift + Alt + A` |
| **Memory Brain** | Local SQLite cognition — hierarchy, knowledge graph, sleep consolidation. No cloud. | Brain icon in the sidebar |
| **Checkpoints** | Instant rewind of files — or files *and* what the agent remembered | Restore icon in chat |

## Memory Brain

Most assistants keep a handful of notes. Knox runs a **local cognitive stack** — a SQLite brain at `~/.knox/memory/brain.sqlite` that spans every session, compresses what it keeps, forgets what it should, and injects only what the next turn needs.

Nothing leaves your machine. There is no cloud sync and no remote memory API. Optional AES-256-GCM export is for local backup only.

[More Details >>>](https://docs.knox.chat/blog/knoxchat-soul)

## Internationalization

- **English** (default)
- **Chinese** (中文)

Change language from Settings in the sidebar. Agent UI, permissions, jobs, and tool-error cards are localized.

## Requirements

- **VS Code** ≥ 1.125.0
- **Node.js** ≥ 24.19.0
- An API key for at least one supported LLM provider (Anthropic, OpenAI, or Knox)

## Documentation & Support

- **Docs**: [https://docs.knox.chat/](https://docs.knox.chat/)
- **GitHub**: [https://github.com/knoxchat/knoxchat](https://github.com/knoxchat/knoxchat)
- **Email**: support@knox.chat
- **Homepage**: [https://knox.chat](https://knox.chat)
