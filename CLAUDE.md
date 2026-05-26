# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Tauri-based desktop application for storytelling/audio book generation. It combines:
- **Frontend**: Vue 3 + Quasar + TypeScript (Vite build)
- **Backend**: Rust (Tauri) with SQLite database
- **TTS Engine**: GPT-SoVITS (Python/FastAPI server) integrated as a submodule

## Build & Development Commands

### Frontend (Vue/Tauri)
```bash
npm run dev          # Start Vite dev server (port 8550)
npm run build        # Type-check and build frontend (vue-tsc + vite build)
npm run preview      # Preview production build
npm run tauri dev    # Start Tauri in development mode (runs beforeDevCommand: npm run dev)
npm run tauri build  # Build Tauri application for distribution
```

### Backend (Rust)
```bash
cd src-tauri
cargo build          # Build Rust backend
cargo test           # Run Rust tests
cargo clippy         # Lint Rust code
```

### TTS Engine (Python - optional)
The `engine/` directory contains a GPT-SoVITS submodule for text-to-speech functionality. If needed:
```bash
cd engine
# Activate venv first
python server.py     # Start TTS API server (FastAPI)
```

## Architecture

### Frontend Structure
- `src/main.ts` - Vue app entry point with Quasar configuration
- `src/App.vue` - Root component wrapping MainLayout
- `src/layouts/MainLayout.vue` - Main layout with sidebar, header, and footer player
- `src/routers/routes.ts` - Vue Router routes (booklist, character, list, bookDetail)
- `src/pages/*.vue` - Page components (BookCollection, Character, GenerateList, BookDetails)
- `src/generated/` - Auto-generated TypeScript types and commands from `tauri-ts-generator`

### Backend Structure (src-tauri/src/)
- `lib.rs` - Tauri app setup, plugin initialization, command registration
- `main.rs` - Entry point calling `o_storytelling_app_lib::run()`
- `config/appConfig.rs` - AppConfig struct with paths for novels and mp3s
- `state/appState.rs` - AppState managing config with Mutex, load/save functions
- `tools/upload.rs` - File upload command for importing novels (epub files)
- `tools/process_novel.rs` - Novel/Chapter data structures for database

### Key Integration Points
1. **Tauri Commands**: Rust functions exposed to frontend via `#[tauri::command]`, registered in `generate_handler![]`
2. **Type Generation**: `tauri-ts-generator` auto-generates TypeScript types in `src/generated/` matching Rust structs decorated with `#[derive(TS)]`
3. **State Management**: AppState is managed by Tauri (`app.manage(app_state)`) and accessed via `State<AppState>` in commands

## Important Patterns

### Adding New Tauri Commands
1. Create function with `#[tauri::command]` in appropriate module
2. Add to `generate_handler![]` macro in `lib.rs`
3. If returning custom types, derive `Serialize, Deserialize, TS` on structs
4. TypeScript wrapper will be auto-generated in `src/generated/commands.ts`

### Quasar UI Framework
- Uses Quasar components (QLayout, QDrawer, QPage, QCard, etc.)
- Custom theme colors defined in `src/main.ts` brand config (primary: #061931)
- SCSS variables in `src/styles/variables.scss` auto-injected via Vite config

### File Import Flow
1. User clicks "导入书籍" in BookCollection.vue
2. Opens file dialog via `@tauri-apps/plugin-dialog`
3. Calls `file_upload` Tauri command with selected file path
4. Rust copies file to configured novel_path and processes it

## Dependencies

- **Rust**: tauri 2.x, serde, rusqlite (bundled), hound (audio), html2text, epub
- **Frontend**: vue 3.x, vue-router 5.x, quasar 2.x, @tauri-apps/api, @tauri-apps/plugin-dialog, @tauri-apps/plugin-fs