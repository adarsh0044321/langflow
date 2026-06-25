# Project Overview

## Project Name

LangFlow

## Purpose

LangFlow is a real-time desktop translation utility that allows users to translate text instantly. It supports hybrid/online translation (Google Translate, DeepL, Gemini) and strict offline translation utilizing local ONNX models. Features include a system-wide quick translation popup, screenshot-based OCR translation, and an inline keyboard typing assistant (IME mode) that translates words and sentences on-the-fly as they are typed.

## Current Status

Active development. Recent improvements focused on bidirectional offline translation, swap button upgrades, and fixing looping and race condition bugs in the low-level Windows keyboard hook typing assistant.

## Tech Stack

* **Frontend**: React (v19.1), TypeScript, Vite (v7.0)
* **Backend**: Rust, Tauri (v2)
* **Database**: SQLite (via `rusqlite` in Rust backend)
* **Infrastructure / OS APIs**: Win32 API (via `windows-rs` for low-level global hooks and input simulation)
* **Languages**: Rust, TypeScript, SQL (SQLite), CSS
* **Frameworks**: Tauri, React
* **Dependencies**: `tract-onnx` (local neural translation model runner), `reqwest` (online API requests), `image` & `tauri-plugin-opener` (assets/system hooks)

---

# Architecture

## High-Level Structure

Frontend (React) <== Tauri IPC ==> Backend (Rust) <== SQLite (cache/history)
                                            ||
                                            v
                                 Translation Engine (ONNX/APIs)

LangFlow utilizes Tauri's multi-window architecture:
1. **MainWindow**: Primary translator interface.
2. **SettingsWindow**: System preferences, hotkeys, and language pack manager.
3. **FloatingPopup**: Pop-up window displaying translations for global clipboard/hotkey grabs.
4. **ScreenshotOverlay**: Screen capture layer for local OCR.

The Rust backend handles low-level system integrations (global shortcuts, keyboard hooks, clipboard manipulation, memory management) and routes translation requests between SQLite caching, online providers, or the local ONNX inference engine.

---

# Directory Map

```
LangFlow/
├── src/                      # React Frontend Source
│   ├── components/           # Reusable UI components (e.g., TitleBar)
│   ├── windows/              # Tauri windows (Main, Settings, Screenshot, Popup)
│   ├── App.tsx               # Window routing entrypoint
│   ├── index.css             # CSS design token design system
│   └── main.tsx              # React mounting root
├── src-tauri/                # Tauri Rust Backend
│   ├── src/
│   │   ├── core/             # Core system configurations, SQLite db, Win32 hooks
│   │   │   ├── config.rs     # Configuration file serialization/deserialization
│   │   │   ├── database.rs   # SQLite history, cache, language pack registries
│   │   │   ├── hotkey.rs     # System-wide global hotkey monitor
│   │   │   ├── ime.rs        # Inline Typing Assistant hook & processing
│   │   │   └── inline_type.rs# Win32 SendInput keystroke simulator
│   │   │   └── memory.rs     # Idle model unloading & working RAM trimmer
│   │   ├── lang_pack/        # Language pack downloading & uninstallation
│   │   ├── ocr/              # Screenshot capture & Windows OCR runner
│   │   ├── translation/      # Translation logic, ONNX runner, Fallback Dict, APIs
│   │   ├── lib.rs            # Tauri IPC command registration & lifecycles
│   │   ├── tray.rs           # Windows System Tray menu builder
│   │   └── main.rs           # CLI launcher entrypoint
│   ├── Cargo.toml            # Rust cargo package manifest
│   └── tauri.conf.json       # Tauri window configurations & permissions
```

---

# Features

## Completed Features

### Multi-Engine Translation UI
Dual-pane translation with clipboard copy, clear, and text-to-speech (TTS) readback. History drawer displays translation cache with search capabilities, favorites, and purge options.

### Screenshot OCR Translate
Dims the screen, crops a user-selected area, performs native Windows OCR, and outputs the translated text onto the quick Floating Popup.

### Language Pack Manager
Allows users to download or uninstall local translation models (MarianMT) to their user folder to keep the initial application install footprint minimal.

### Real-Time Typing Translation (IME Mode)
Translates text globally as you type inside any application. Translates word-by-word on Space, or translates the whole sentence with proper context-aware grammar on typing punctuation (`.`, `?`, `!`) or Enter, erasing the draft and typing the final translation in place.

---

# Change Log

## 2026-06-23

### Added
* Bidirectional character-based language detection on the frontend (`MainWindow.tsx`) and the backend (`local_onnx.rs`).
* In-memory lookup fallback dictionary in `local_onnx.rs` supporting 7 target languages (Japanese, Chinese, Korean, Spanish, French, German, Russian) in both directions.
* Manual keyboard modifiers retrieval (Shift, Caps Lock) using `GetKeyState` inside the low-level hook thread.
* Virtual key fallback table in `ime.rs` to map virtual keys directly to characters if Win32 `ToUnicode` fails.

### Modified
* Enabled the language swap button for "Auto Detect" source mode.
* Instant text state swap in `MainWindow.tsx` to provide snappy user experience before translating the remainder.
* Bidirectional model path checks under `models/` (looks for both `en-ja` and `ja-en` folders) and cache key normalization.
* Updated `keyboard_hook_proc` to check for `LLKHF_INJECTED` (0x10) and ignore simulated keyboard events.

### Reason
* Language swap was disabled in Auto mode, and reverse-direction translations (`X-en`) failed offline because model files are saved only under `en-X`.
* Low-level typing assistant hook was looping infinitely by capturing its own injected characters, and failed to recognize modifiers/punctuation due to running on a background thread without GUI focus.

---

# Bug Fixes

## Language Swap Mismatch & Crash
* **Problem**: Swapping languages offline resulted in translation errors or no updates.
* **Root Cause**: MarianMT models are downloaded under `en-X` folders. Translating in the reverse direction (`X-en`) searched for the model under `X-en` folder, which doesn't exist. Thus, swapping broke offline mode.
* **Fix**: Updated `get_model_path` to check both directions. Added a robust dictionary translation fallback if ONNX model loading/running fails or isn't installed.
* **Files Modified**: `src-tauri/src/translation/local_onnx.rs`

## Typing Assistant Hook Loop & Modifier Failure
* **Problem**: Real-time typing assistant did not translate, or caused looping inputs and missed punctuation symbols like `?` or `!`.
* **Root Cause**: Hook thread processed simulated backspaces and injected keystrokes, entering an infinite loop. `GetKeyboardState` returned all zeroes on background threads, causing `ToUnicode` to miss modifier states (Shift/Caps Lock).
* **Fix**: Added checking of `LLKHF_INJECTED` to skip simulated keys, manually read Shift and Caps Lock states with `GetKeyState`, and added a fallback mapping from virtual keys to characters.
* **Files Modified**: `src-tauri/src/core/ime.rs`

---

# Technical Decisions

## Bidirectional Folder Check
* **Date**: 2026-06-23
* **Decision**: Let `get_model_path` check both `source-target` and `target-source` folders to determine which model file exists.
* **Reasoning**: MarianMT packs are English-centric (`en-ja`, `en-zh`). By checking both paths, both translation directions (`en` ➔ `X` and `X` ➔ `en`) can map to the same file.

## In-Memory Dictionary Fallback
* **Date**: 2026-06-23
* **Decision**: Add a static mapping dictionary of common phrases/words for all 7 languages.
* **Reasoning**: Prevents strict offline translation failures by gracefully falling back to a clean mock dictionary lookup if models are not downloaded or tract-onnx fails to load.

---

# Agent Notes

* **Win32 Hook Restrictions**: `keyboard_hook_proc` in `ime.rs` must not perform expensive block operations. Keep database lookups and translations inside spawned asynchronous runtimes (`tauri::async_runtime::spawn`).
* **Injected Key Events**: Always screen out events where `kbd_struct.flags.0 & 0x10 != 0` inside the keyboard hook, otherwise keyboard injections will cause lockups or infinite typing loops.
* **Tauri Windows**: Main and Settings windows are set up to hide on close requests instead of destroying, retaining their state and showing instantly. Do not change this behavior without review.

---

# Development Workflow

## Build Commands
* Backend: `cd src-tauri && cargo build`
* Frontend: `npm run build`
* Tauri bundle: `npm run tauri build`

## Run Commands
* Dev Mode (hot reloading): `npm run tauri dev`

---

# Dependency Notes

* **Dependency**: `tract-onnx`
  * **Purpose**: Performs local translation model inference using MarianMT ONNX files.
  * **Do Not Replace Because**: Essential for strict offline execution without network API keys.

* **Dependency**: `rusqlite`
  * **Purpose**: Manages SQLite backend caching and logs user history.
  * **Do Not Replace Because**: Embedded DB of choice, provides WAL mode for concurrent GUI reads/writes.

---

# AI Context Summary

1. **What the project does**: LangFlow is a desktop translator written in Rust (Tauri) and React, supporting quick clipboard translation, screen OCR, offline ONNX MarianMT model translations, and real-time keystroke translation (IME typing assistant).
2. **Current architecture**: React frontends invoke Rust Tauri IPC commands. The Rust backend handles system hooks, SQLite caching/history, OCR captures, and coordinates online translate endpoints and local `tract-onnx` inference.
3. **Recent major changes**: Enabled Auto-mode language swap, resolved reverse-pair model path issues offline, added dictionary offline fallbacks, and fixed looping/race condition bugs in the Typing Assistant hook.
4. **Important warnings**: Never let the global keyboard hook process injected keystrokes (checks `LLKHF_INJECTED`).
5. **Next priorities**: Verification of offline models in multi-lingual environments, user UI review of Settings window.

---

# Last Updated

* **Timestamp**: 2026-06-25 13:58 UTC
* **Updated By**: Antigravity AI Agent
* **Summary**: Initialized project persistent memory documentation `BRAIN.md`. Documented the overall architecture, tech stack, directory layouts, recent changes, bug fixes, and development workflows.
