# DECISIONS.md

Purpose: Stores architecture and engineering decisions.

---

# Architecture Decision Records

## ADR-001: SQLite WAL Mode for Concurrent Database Access

Date:
2026-06-20

Status:
Accepted

### Problem
Simultaneous database invocations from the Settings window (saving config / reading packs) and the Main window (reading cache / writing history) were throwing database locked errors in SQLite.

### Options Considered

#### Option A: Thread-blocking Mutex
Wrap all database connections in a global mutex lock.
* **Pros**: Simple to write.
* **Cons**: Introduces frontend UI freezes since one window blocks the other.

#### Option B: Enable WAL Mode
Configure SQLite connections with WAL (Write-Ahead Logging) and normal synchronization pragmas.
* **Pros**: Readers do not block writers and writers do not block readers. High concurrency.
* **Cons**: Slightly higher disk write frequency.

### Decision
Option B was chosen to ensure zero-lag UI performance.

### Reasoning
Concurrent reads/writes are critical in a multi-window desktop utility where user interface response times must remain under 16ms (60fps).

### Consequences
* Positive: Zero database locked errors; instant cache lookups.
* Negative: WAL files generated alongside the primary database file.

---

## ADR-002: Hide-on-Close Window Strategy

Date:
2026-06-20

Status:
Accepted

### Problem
Initializing a new WebView2 window in Tauri takes ~2 seconds, creating a noticeable lag when users open the settings or quick popup panel.

### Decision
Intercept the `CloseRequested` window event in `lib.rs`, call `window.hide()`, and prevent window destruction via `api.prevent_close()`.

### Consequences
* Positive: Sub-millisecond window reveal times; React states are completely preserved in the background.
* Negative: Running processes stay in memory (mitigated by automatically unloading ONNX models and forcing garbage collection trim on idle).

---

## ADR-003: Bidirectional Offline Path Resolution

Date:
2026-06-23

Status:
Accepted

### Problem
Local MarianMT models are downloaded English-centric under `en-X` folders (e.g. `en-ja`). Offline translations queried in the reverse direction (`ja` ➔ `en`) failed because the system searched for `ja-en` folder paths.

### Decision
Modify the `get_model_path` function to check for the model file existence in both directions (source-to-target and target-to-source), mapping both pairs to the same folder path (e.g., `en-ja`).

---

## ADR-004: In-Memory Fallback Offline Dictionary

Date:
2026-06-23

Status:
Accepted

### Problem
In strict Offline mode, if language packs are missing or fail to download, the application crashed or returned error messages in the translation pane.

### Decision
Added an in-memory dictionary fallback containing common words and phrase definitions for all 7 languages.

---

## ADR-005: LLKHF_INJECTED Hook Filter

Date:
2026-06-24

Status:
Accepted

### Problem
The global keyboard hook processed backspaces and letters injected by our own simulated text input events, creating an infinite loop of recursive typing.

### Decision
Checked the `LLKHF_INJECTED` bit (`0x10`) in the keyboard hook struct flags and ignored simulated inputs immediately.

---

# Decision History

* **ADR-001**: SQLite WAL Mode (2026-06-20)
* **ADR-002**: Hide-on-Close Window Strategy (2026-06-20)
* **ADR-003**: Bidirectional Offline Path Resolution (2026-06-23)
* **ADR-004**: In-Memory Fallback Offline Dictionary (2026-06-23)
* **ADR-005**: LLKHF_INJECTED Hook Filter (2026-06-24)
