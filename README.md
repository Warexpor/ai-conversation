# AI Conversation

**v2.0** — Desktop multi-agent chat. Let 2–3 LLMs talk over OpenAI-compatible APIs.

![logo](public/logo.png)

## Features

- 2–3 agents with system prompts, models, API keys, and reasoning effort
- OpenCode Zen / Go / OpenAI presets, plus model list fetch
- SSE streaming with non-stream fallback
- Step / Auto modes, Stop (keeps transcript), Narrate next turn
- Thoughts panel for reasoning models
- Saved chats and API profiles (local)
- Markdown export and keyboard shortcuts (`?`)

## Requirements

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) (stable)
- Tauri system deps for your OS ([guide](https://v2.tauri.app/start/prerequisites/))

## Install & run

```bash
npm install
npm run tauri dev
```

Production installers:

```bash
npm run release:build
```

Outputs land under `src-tauri/target/release/bundle/` (NSIS / MSI on Windows).

## Scripts

| Command | Purpose |
|---------|---------|
| `npm run tauri dev` | Desktop app in development |
| `npm run build` | Frontend production build |
| `npm run release:build` | Frontend + Tauri release bundles |
| `npm run typecheck` | TypeScript check |
| `npm run test:rust` | Rust unit tests |
| `npm run icons` | Regenerate icons (`scripts/make_icons.py`) |

## Configuration

API keys are entered in **Settings** inside the app (not required in `.env`).  
See [`.env.example`](.env.example) only if you later wire env-based keys.

Chats persist in a local SQLite database (app data dir). Browser Vite preview can use a localStorage cache.

## Architecture

| Area | Location |
|------|----------|
| UI shell | `src/App.tsx` |
| Conversation / stream hooks | `src/hooks/` |
| Typed Tauri API | `src/lib/api.ts` |
| Config + prefs | `src/lib/config.ts` |
| Chat / API profile storage | `src/lib/storage.ts` |
| Rust engine + SQLite | `src-tauri/src/` |

## Shortcuts

| Key | Action |
|-----|--------|
| `Space` | Start/pause or step |
| `N` | Step one turn |
| `S` | Settings |
| `B` | Chats sidebar |
| `?` | Shortcuts |
| `Esc` | Stop → close overlays |
| `Ctrl+S` | Save chat |
| `Ctrl+E` | Export |
| `Ctrl+Shift+R` | New chat |

## Stack

Tauri 2 · React 18 · TypeScript · Tailwind · Rust (`reqwest` SSE, `rusqlite`)

## License

Private — see [LICENSE](LICENSE).
