# Smart Clipboard

An intelligent clipboard for Windows — a *second brain* for everything you copy.
Every copy is a signal: the app understands what you copied, why, and what
you'll need next.

Built with **Tauri v2** (Rust core + React/TypeScript UI) for a native, tiny
(~30 MB) footprint, with **Appwrite** for cloud sync and a local **Ollama** LLM
for the semantic layer.

> Status: **Phase 1 foundation.** Universal capture, type detection, history UI,
> system tray, global overlay, sensitive-clip detection, Power-Paste transforms,
> and optional Appwrite sync are in. The AI layer, workspaces, E2E sync and deep
> Windows integration are being built out in subsequent phases.

## Features in this phase

- **Universal capture** — a background watcher records text, URLs, code, hex
  colors and images from the OS clipboard automatically.
- **Auto-detect type** — each clip is classified (`text` / `url` / `code` /
  `color` / `image`) with metadata (url host, code language, image dimensions).
- **History UI** — searchable, filterable grid of clips with live updates.
- **System tray** — runs in the tray; left-click toggles the window, menu has
  Show / Pause capture / Quit.
- **Power Paste overlay** — `Ctrl+Shift+V` opens a quick overlay; transform a
  clip before pasting (UPPER / lower / trim / URL-encode / JSON pretty).
- **Sensitive-clip detection** — passwords, OTPs and card numbers are flagged,
  masked in the UI, and excluded from cloud sync.
- **Incognito** — pause capture from the tray or the header toggle.
- **Optional Appwrite sync** — configure `.env` to mirror clips to your project.

## Tech stack

| Layer | Choice |
|---|---|
| Shell | Tauri v2 |
| Core | Rust (clipboard watcher, tray, global shortcut, detection) |
| UI | React 19 + TypeScript + Vite |
| Cloud DB / sync | Appwrite |
| Local AI (upcoming) | Ollama |

## Prerequisites (Windows)

- [Rust](https://rustup.rs/) (MSVC toolchain)
- Microsoft C++ Build Tools (VS 2022)
- [WebView2 runtime](https://developer.microsoft.com/microsoft-edge/webview2/)
- Node.js 20+

## Development

```bash
npm install
npm run tauri dev      # launches the desktop app with hot reload
```

Frontend only:

```bash
npm run dev            # vite dev server on http://localhost:1420
npm run build          # type-check + production build
```

## Appwrite sync (optional)

1. `cp .env.example .env` and fill in your endpoint, project ID and a server API key.
2. Provision the database/collection: `npm run setup:appwrite`
3. Restart the app — clips now sync to Appwrite (sensitive clips are never synced).

Without a `.env`, the app runs fully locally.

## Project layout

```
src/                 React + TypeScript UI
  lib/api.ts         typed wrappers around Tauri commands
  lib/appwrite.ts    optional Appwrite sync layer
src-tauri/src/
  clipboard.rs       background clipboard watcher + state
  detect.rs          type detection + sensitive-content heuristics
  commands.rs        Tauri commands exposed to the UI
  models.rs          Clip data model
scripts/
  setup-appwrite.mjs one-time Appwrite provisioning
```

## Roadmap

2. AI semantic layer (Ollama): auto-tag, summarize, embeddings + smart search.
3. Spaces / workspaces + templates.
4. Power Paste: multi-paste queue, more transforms, paste-as.
5. Sync & security: E2E encryption, auto-expiring secrets.
6. Developer mode: snippet library, variable templates, copy diff.
7. Windows integration: Explorer context menu, PowerToys-style action bar.
