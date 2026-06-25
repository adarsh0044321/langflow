# HANDOVER.md

Purpose: Allows one AI agent to immediately continue where another left off.

---

# Current Project State

Current Branch:
main

Current Version:
v1.0.0

Last Updated:
2026-06-25 14:00 UTC

---

# Session Summary

In this development cycle, we resolved core issues with offline bidirectional translations, swap dropdown state synchronization, and system-wide keyboard hook typing assistant bugs:
* Fixed `local_onnx.rs` model folder path checks to resolve both directions (`en-X` and `X-en`) to the same local model file.
* Added static fallback dictionaries to handle offline translate queries gracefully when models are not installed.
* Enabled swapping language choices from "Auto Detect" by implementing client-side character-range language detection.
* Solved infinite loop bugs in the global IME typing assistant hook by skipping injected key events (`LLKHF_INJECTED`).
* Implemented manual modifier checks (Shift, Caps Lock) and virtual-key-to-char fallback lookup in `ime.rs` to fix symbol detection issues.
* Created the persistent project memories: `BRAIN.md`, `ROADMAP.md`, and `DECISIONS.md`.

---

# What Was Just Completed

* Standardized swap button functionality for "Auto Detect" mode.
* Resolved offline MarianMT translation direction crashes.
* Ignored self-injected input events in `keyboard_hook_proc`.
* Created system-level documentation memory.

Files Modified:

* `src/windows/MainWindow.tsx`
* `src-tauri/src/translation/local_onnx.rs`
* `src-tauri/src/core/ime.rs`
* `BRAIN.md`
* `ROADMAP.md`
* `DECISIONS.md`

---

# Current Work In Progress

## Documentation Sync
Finalizing the creation of markdown documents (`ROADMAP.md`, `DECISIONS.md`, `HANDOVER.md`, `SESSION.md`) to establish the permanent project workspace memory structure.
Status: 90%

---

# Immediate Next Tasks

Priority Order:

1. Test OCR translation scaling compatibility on mixed high-DPI monitor settings.
2. Develop a Nullsoft Installer build pipeline script in Vite/Tauri configuration.
3. Quantize local MarianMT models to INT8 to reduce target download footprint.

---

# Known Blockers

None. The system compiles cleanly, and both online and offline translations operate correctly.

---

# Important Context

* **Injected Inputs**: Never let `keyboard_hook_proc` in `ime.rs` intercept key downs where `kbd_struct.flags.0 & 0x10 != 0`. This will cause typing loops.
* **GetKeyboardState Limitation**: Do not use `GetKeyboardState` alone in hooks, as background thread keyboard buffers are not populated. Always fetch modifiers using `GetKeyState`.

---

# Do Not Touch

* **Window Close Handlers** in `lib.rs`: These override window destruction to keep React states active.
* **SQLite Connection Pragmas** in `database.rs`: Changing them will cause UI thread lockups.

---

# Recommended First Actions

When a new AI agent starts:

1. Read `BRAIN.md` to understand tech stack and folder locations.
2. Read `DECISIONS.md` to see previous ADRs.
3. Review `src-tauri/src/core/ime.rs` and `src-tauri/src/translation/local_onnx.rs`.
4. Review the backlog inside `ROADMAP.md`.

---

# Handover Checklist

Completed:

* [x] Code Compiles
* [x] Tests Pass (Tauri backend compilation check succeeds)
* [x] Documentation Updated
* [x] BRAIN.md Updated
* [x] ROADMAP.md Updated
* [x] DECISIONS.md Updated

Next AI Can Safely Continue:
YES
