# LangFlow

LangFlow is a lightweight, high-performance, native Windows translation utility designed for gamers, streamers, multilingual users, and international communities. It runs unobtrusively in the system tray, offering instant global shortcut translations and screenshot OCR with minimal system resource footprint (< 50MB RAM idle).

---

## Key Features

*   🚀 **Fast & Responsive:** A frameless utility-first interface built for immediate accessibility and keyboard-driven workflows.
*   ⌨️ **Global Highlight Translation (`Ctrl+Shift+T`):** Instantly translate highlighted text in Chrome, Discord, WhatsApp Desktop, documents, and game launchers. Supports displaying a quick floating translation popup or directly replacing the selection in-place with the translated text.
*   📸 **Screenshot Area OCR (`Ctrl+Shift+S`):** Darken the screen, crop a custom bounding box, and automatically run native Windows OCR to translate text on top of game panels, quest descriptions, manga, or video streams.
*   ✍️ **Inline Typing Assistant (`Ctrl+Shift+I`):** Write in your native language inside a game-safe overlay. LangFlow translates and injects the output character-by-character into target window text fields using Win32 unicode input events (safe from anti-cheat bans).
*   📦 **Modular Language Packs:** Keeps initial download sizes tiny (< 10MB) by loading lightweight translation models on-demand. Install/uninstall packs through the Settings panel.
*   🔒 **Offline-First & Private:** Run translations locally using pure-Rust ONNX models (`tract-onnx`), with automatic cache storage in a local SQLite database.
*   🔌 **Hybrid Support:** Seamlessly verify local caches and models, falling back to online services (Google Translate, DeepL, or Gemini) when local models are unavailable or uninstalled.
*   ⚡ **Memory Optimized:** Automatically drops ONNX model handles after inactivity and invokes Win32 memory working set trimmers (`SetProcessWorkingSetSize`), dropping idle memory footprint down to < 5MB physical RAM.

---

## Technical Architecture

```text
LangFlow/
├── src-tauri/                  # Rust Backend
│   ├── src/
│   │   ├── core/               # config, SQLite DB, hotkeys, memory, keystroke injector
│   │   ├── ocr/                # GDI screenshot area crop, Windows.Media.Ocr bridges
│   │   ├── translation/        # tract-onnx loader, online API clients, caching engine
│   │   ├── lang_pack/          # background model downloader
│   │   └── tray.rs             # system tray builder
│   └── Cargo.toml              # Cargo configuration & dependencies
├── src/                        # React + TypeScript Frontend
│   ├── components/             # Reusable UI widgets
│   ├── windows/                # MainWindow, Settings, ScreenshotOverlay, FloatingPopup
│   ├── App.tsx                 # Client-side window routing
│   └── index.css               # Design tokens & layout styles
└── package.json                # Frontend package configuration
```

*   **GUI Shell:** Tauri v2 (Rust + WebView2).
*   **Frontend UI:** React 19 + TypeScript + Custom CSS.
*   **Database & Cache:** SQLite (WAL mode enabled for concurrent writes).
*   **OCR Pipeline:** Native Windows WinRT API (`Windows.Media.Ocr::OcrEngine`) for lightning-fast, zero-overhead text recognition, with coordinates adjusted by the monitor's DPI scaling factor.
*   **Local Inference:** Pure-Rust `tract-onnx` executing lightweight MarianMT models.

---

## Memory Management Strategy

LangFlow is designed to run in the background while playing modern resource-heavy 3D games. Memory transitions occur as follows:

| State | Target RAM | Trigger | Memory Actions |
| :--- | :--- | :--- | :--- |
| **Idle** | **< 35 MB** *(drops < 5MB)* | Application starts or minimized | Suspends webview threads, unloads ONNX structures, calls Win32 process working set trimmer |
| **Active typing** | **< 80 MB** | MainWindow opened | Activates webview, loads local SQLite cache maps |
| **Local ONNX Inference** | **< 120 MB** | Offline translation request | Lazily compiles the ONNX execution plan for the active language pair |
| **Inactivity (180s)** | **< 35 MB** | Inactivity timeout | Background scheduler drops model handles and forces memory reclamation |

---

## Getting Started

### Prerequisites

To compile the project from source, you will need the following tools installed on your Windows machine:

1.  **Rust (v1.75+):** [rustup.rs](https://rustup.rs/) (Ensure MSVC toolchain is selected)
2.  **Node.js (v18+):** [nodejs.org](https://nodejs.org/)
3.  **Windows Webview2 Runtime:** Standard on Windows 10/11 (or install via Microsoft Webview2 bootstrapper)

### Build Instructions

1.  **Clone the Repository:**
    ```bash
    git clone https://github.com/yourusername/LangFlow.git
    cd LangFlow
    ```

2.  **Install Frontend Dependencies:**
    ```bash
    npm install
    ```

3.  **Run in Development Mode:**
    Runs the Vite local dev server and opens the Tauri developer frame:
    ```bash
    npm run tauri dev
    ```

4.  **Build Production Executable:**
    Compiles the optimized release binary and bundles it into a Nullsoft Installer (NSIS) `.exe`:
    ```bash
    npm run tauri build
    ```
    The compiled executable will be located in:
    `src-tauri/target/release/LangFlow.exe`

---

## Open Source License

This project is licensed under the MIT License - see the `LICENSE` file for details.
