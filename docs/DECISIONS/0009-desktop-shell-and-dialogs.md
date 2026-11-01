# 0009 — Desktop shell and native dialogs

Date: 2026-11-01
Status: Accepted

## Context

M4 needs a native desktop application with a non-blocking conversion flow, native file
selection, light/dark appearance, and headless UI tests. The conversion core is already
UI-independent. The project must not add Electron, a webview, a JavaScript runtime, or a
network dependency.

Two implementation choices were not obvious:

1. How to split UI state from rendering so it can be tested without a display.
2. Which file-dialog implementation to use on each platform.

## Options

### UI framework

- **egui/eframe** — immediate-mode, pure-Rust application code, native windowing/OpenGL,
  no webview. Chosen.
- Tauri/webview — rejected: violates the no-webview/no-JS rule and adds a web stack.
- GTK/Qt bindings directly — rejected: heavier C/C++ surface and more packaging work.

### File dialogs

- **`rfd`** — native AppKit dialogs on macOS, native Windows dialogs, GTK3 dialogs on
  Linux. Chosen.
- **XDG desktop portal (`ashpd`)** — pure-Rust DBus client, but the crate was not available
  in the offline dependency cache for this environment.
- **In-app path browser** — no native picker and significantly more UI code; kept as a
  fallback if GTK packaging becomes unacceptable.

## Decision

- `crates/desktop-app` uses `eframe` with the `glow` renderer, `default_fonts`, X11/Wayland,
  and accesskit.
- UI state (`model.rs`) contains no egui types and is unit-tested headlessly.
- Conversion and analysis run on `std::thread` workers; the UI communicates through
  `std::sync::mpsc` and only drains events non-blockingly. No conversion work runs on the
  render thread.
- `rfd` provides native file/folder dialogs. On Linux, `rfd` uses its GTK3 backend, so the
  Linux build and package need GTK3 development/runtime libraries. macOS and Windows use
  their native dialogs.
- The app accepts `.fig` paths on the command line and stages them in the queue. This also
  supports future file-association/open-with integration.

## Consequences

- **Pure Rust application logic** is preserved; GTK3 is a native platform library used only
  for Linux dialogs, not a JS/runtime dependency. This is a written C-binding exception as
  required by `AGENTS.md`.
- Linux CI and packaging must install GTK3 and the usual X11/Wayland/OpenGL development
  packages. CI now installs them explicitly.
- UI logic can be tested without a window: state transitions, settings round-trips,
  conversion-worker integration, and a smoke test that draws all eight screens through an
  `egui::Context`.
- Progress shows only real conversion stages. There is no percentage and no cancel button
  until the M5 batch queue provides cancellation semantics.
- Native dialog interaction and drag-and-drop still require manual QA; automated tests cover
  the state and worker paths, not the OS dialog itself.
