# ROADMAP.md

Purpose: Defines the future direction of the project.

---

# Vision

Create the most responsive, zero-impact native Windows translation utility that combines local offline-first ONNX models, screenshot OCR, and seamless inline typing translation for international gamers, streamers, and developers.

---

# Current Milestone

Milestone:
v1.0.0

Goal:
Complete basic translations, screenshot OCR, and inline typing IME functionality with robust offline fallback dictionaries and keyboard hook loop fixes.

Status:
Completed

Completion:
100%

---

# Upcoming Milestones

## Milestone: v1.1.0

Target Date:
2026-07-15

Objectives:

* Fix OCR crop mapping errors on multi-monitor setups with mixed DPI scaling factors.
* Allow users to select custom translation UI fonts and adjust text area sizes.
* Add auto-start on Windows boot toggle inside settings UI.

Success Criteria:

* Perfect coordinate alignments for OCR bounds across dual-monitor arrays with 100% and 150% DPI scale factors.
* Font selection persists across restarts.

Risks:

* Win32 monitor boundaries returning virtual coordinates instead of physical pixels.

---

## Milestone: v1.2.0

Target Date:
2026-08-30

Objectives:

* Introduce inline Typing Assistant floating overlay for candidate selection (Google Input Tools style dropdown suggestion box).
* Enable user dictionary configuration (custom blacklist/whitelist words and phrases).

Dependencies:

* Tauri v2 window overlay borderless transparency fixes.

---

# Feature Backlog

## High Priority

### Auto-Update Pipeline
Description: Automatically fetch and install the latest compiled Nullsoft Installer (.exe) releases from GitHub releases.
Expected Impact: High
Estimated Complexity: Medium
Dependencies: GitHub API access, client updater script.

---

## Medium Priority

### Local Model Quantization (INT8)
Description: Compress MarianMT local ONNX files using INT8 quantization to reduce file size from ~150MB to < 40MB.
Expected Impact: High (decreases download bandwidth and RAM footprint)
Estimated Complexity: High
Dependencies: Python PyTorch/ONNX quantization pipeline.

---

## Low Priority

### Cross-Platform Support (macOS / Linux)
Description: Port the application to macOS and Linux.
Expected Impact: Medium
Estimated Complexity: High
Dependencies: Cross-platform equivalents for Win32 Low-Level Keyboard Hooks and Screen Capture APIs.

---

# Technical Debt

## Clipboard Lock Retries
Description: Wrap system clipboard copy-paste simulations with retry loops.
Reason: Some active applications lock the clipboard, causing occasional read/write errors.
Impact: Reliability
Priority: Medium

---

# Research Items

## Accessibility API Keystroke Injection
Questions:
* Can we inject translated text using UI Automation / accessibility APIs instead of simulating standard keystrokes?
* Does this bypass anti-cheat mechanisms in games that block Win32 `SendInput` events?

Potential Approaches:
* Implement Microsoft UI Automation patterns.
* Leverage Windows Input Injection API.

---

# Stretch Goals

* Translate speech audio input on-the-fly (dictation overlay).
* Local offline OCR using lightweight Tesseract/PP-OCR models instead of native Windows WinRT OCR.

---

# Recently Completed

## 2026-06-24

Completed:
* Fix swap button in Auto mode and resolve local model bidirectional file lookup.
* Fix IME hook typing assistant loops, modifiers, and backend language auto-detection.

Notes:
* Always filter out simulated inputs to protect keyboard hooks from recursive triggers.
