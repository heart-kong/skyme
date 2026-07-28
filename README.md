# Skyme — Modern Rime Frontend (Rust)

A modern, modular, high-performance Windows IME frontend for Rime, built from scratch in Rust.

> 不重写 Rime Engine，而是重新设计 Weasel（小狼毫）的前端架构。

## Architecture

```
                   +----------------------+
                   |     Windows TSF      |
                   +----------+-----------+
                              |
                     ITfTextInputProcessor
                              |
                +-------------v-------------+
                |         ime-core          |  TSF composition, key events, focus
                +-------------+-------------+
                              |
       +----------------------+----------------------+
       |                      |                      |
+------v------+      +--------v-------+     +--------v-------+
| composition |      | session-manager|     | keyboard-hook  |
+------+------+\     +--------+-------+     +--------+-------+
       |                     |                       |
       +----------+----------+-----------------------+
                  |
         +--------v--------+
         |   rime-engine   |  ← librime (runtime-loaded via libloading)
         +--------+--------+
                  |
       +----------+----------+
       |                     |
+------v------+     +--------v-------+
| candidate-ui|     | config-center  |  ← hot-reload, ui.toml, theme.json
+------+------+\    +--------+-------+
       |                     |
+------v---------------------v------+
|        Direct2D / DirectWrite      |
+------------------------------------+
```

## Crates

| Crate | Lines | Description |
|-------|-------|-------------|
| `skyme-common` | 200 | EventBus, shared types, `Candidate`, `Rect`, `DisplayMode` |
| `skyme-rime-engine` | 900 | librime FFI via `libloading`, safe Rust API, deployer |
| `skyme-ime-core` | 500 | TSF text service, COM registration, key event dispatch |
| `skyme-candidate-ui` | 350 | Floating/Classic/Inline candidate renderers |
| `skyme-renderer` | 350 | `Renderer` trait, D2D/DWrite/DComp abstraction, `NullRenderer` |
| `skyme-settings` | 250 | File watcher, `ThemeConfig` (serde JSON), hot-reload |
| `skyme-config` | 200 | `ConfigCenter`, multi-source config merge |
| `skyme-plugins` | 200 | Plugin trait, `PluginRegistry`, 7 built-in stubs |
| `skyme-diagnostics` | 200 | Inspector, FPS/latency metrics, event log |

## Apps

| App | Type | Description |
|-----|------|-------------|
| `skyme-ime-service` | `cdylib` (DLL) | TSF COM DLL — the core IME |
| `skyme-settings-ui` | binary | Standalone settings GUI |
| `skyme-deploy` | binary | Schema deployment tool |

## Build

### Prerequisites

- Rust 1.75+ (`rustup target add x86_64-pc-windows-gnu`)
- MinGW-w64 (for cross-compilation from Linux):
  ```bash
  sudo apt install mingw-w64
  ```
- `rime.dll` from [Weasel](https://github.com/rime/weasel) (required at runtime, not build time)

### Cross-compile for Windows x86_64

```bash
# Debug build
cargo build --target x86_64-pc-windows-gnu --workspace

# Release build
cargo build --target x86_64-pc-windows-gnu --release --workspace
```

### Native check (no librime needed)

```bash
cargo check --workspace
```

### Package

```bash
bash scripts/package.sh release
# Then copy rime.dll into dist/skyme-ime-release/
```

## librime Integration

librime is loaded **dynamically at runtime** via `libloading`, not linked at compile time:

- `Engine::initialize()` calls `libloading::Library::new("rime.dll")` to find and load librime
- All function pointers (`RimeCreateSession`, `RimeProcessKey`, etc.) are resolved from the loaded library
- No `extern "C"` blocks, no `#[link]`, no build-time dependency on librime headers or libraries
- `rime.dll` must be in the `PATH`, the application directory, or `%ProgramFiles%\Skyme`

This decouples the Rust build from the C library, enabling:
- Cross-compilation without rime.dll at build time
- Independent rime.dll updates
- Cleaner distribution

## Windows Installation

```
dist/skyme-ime-release/
├── skyme_ime_service.dll     (1.2 MB)  TSF COM DLL
├── rime.dll                  (add this — from Weasel)
├── skyme-deploy.exe          (3.8 MB)  deployment tool
├── skyme-settings-ui.exe     (1.2 MB)  settings UI
├── install.bat                         admin installer
└── uninstall.bat                       cleanup
```

1. Copy `rime.dll` (from `%ProgramFiles(x86)%\Weasel\`) into `dist/skyme-ime-release/`
2. **Right-click `install.bat` → Run as administrator**
3. Open Windows Language Settings → Add "Skyme Input Method"
4. Switch with Win+Space

## Design Principles

- **Event-driven** — all modules communicate through `EventBus`
- **Unsafe boundary** — FFI calls are encapsulated in `rime-engine::raw`; all public API is safe
- **No GDI/GDI+** — GPU-only rendering (Direct2D + DirectWrite + DirectComposition)
- **Hot-reload** — config and theme changes take effect without deploy
- **Theme ≠ logic** — themes only control appearance (colors, fonts, shadows)
- **Plugin system** — `Plugin` trait with hooks for candidates, keys, commits

## Project Status

| Module | Status |
|--------|--------|
| EventBus | ✅ Done |
| librime FFI + safe API | ✅ Done (libloading) |
| TSF COM integration | ✅ Skeleton (Windows-only) |
| Candidate rendering | ✅ Floating + Classic + Inline |
| GPU rendering abstraction | ✅ Renderer trait + D2D |
| Configuration center | ✅ Hot-reload with file watching |
| Plugin system | ✅ Plugin trait + registry |
| Diagnostics panel | ✅ Inspector + metrics |
| Windows cross-compilation | ✅ x86_64-pc-windows-gnu |
| Installer | ✅ install.bat / uninstall.bat |
| Real TSF COM registration | 🔄 Pending (needs Windows testing) |
| NSIS installer | 🔄 Pending |
