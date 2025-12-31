# InfiniTrak

InfiniTrak is a terminal-based music tracker and modular synthesizer written in Rust. It features a classic vertical tracker interface and a flexible synthesis engine powered by `infinitedsp-core`.

This project was written together with **Gemini Code Assist**.

## Features

*   **Tracker Interface:** Classic vertical sequencing workflow.
*   **Modular Synthesis:** Build instruments using Oscillators, Filters, Envelopes (ADSR), and Gain modules.
*   **Real-time Audio:** Low-latency audio synthesis.
*   **Project Management:** Save and load projects (JSON format).
*   **Export:** Render your tracks to WAV files.
*   **TUI:** Text-based user interface built with `ratatui`.

## Getting Started

### Prerequisites

*   Rust toolchain (cargo, rustc)

### Build and Run

```bash
cargo run --release
```

Using `--release` is recommended for optimal audio performance.

## Controls

### General
*   **`Q`**: Quit the application.
*   **`Tab`**: Switch between **Pattern View** and **Instrument View**.
*   **`Space`**: Play / Stop.
*   **`Shift + Space`**: Play from current cursor position.

### Pattern View
*   **Arrow Keys**: Move cursor.
*   **`Z`, `S`, `X`, `D`, `C`, `V`, `G`, `B`...**: Input notes (Piano layout).
    *   `Z` = C
    *   `S` = C#
    *   `X` = D
    *   ...
    *   `,` = High C
*   **`Backspace` / `Delete` / `.`**: Delete note at cursor.
*   **`F1` / `F2`**: Change Octave (Down / Up).
*   **`F3` / `F4`**: Change Edit Step (0-16).
*   **`F7` / `F8`**: Change BPM (Decrease / Increase).

### Instrument View
*   **Arrow Keys (List)**: Select instrument.
*   **`0`-`9`**: Quick select instrument.
*   **`Right` / `Enter`**: Focus parameter table.
*   **`Left` / `Esc`**: Return to instrument list.
*   **`+` / `-`**: Adjust selected parameter value.

### Project & File Operations
*   **`F9`**: Load Project (Opens file dialog).
*   **`F10`**: Save as New Project (e.g., `project_01.json`).
*   **`F11`**: Save Current Project (Overwrites current file, or saves new if none loaded).
*   **`F12`**: Render to `output.wav`.

## Architecture

*   **Core**: Handles state, pattern data, and instrument definitions.
*   **Audio**: Runs on a high-priority thread, generating audio via `cpal` and `infinitedsp-core`.
*   **UI**: Runs on the main thread, rendering the TUI with `ratatui`.
