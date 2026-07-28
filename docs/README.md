# Skyme — Modern Rime Frontend (Rust)

A modern, modular, high-performance Windows IME frontend for Rime.

## Architecture

```
┌──────────────┐
│  Windows TSF │
├──────────────┤
│  ime-core    │  ← Text Services Framework integration
├──────────────┤
│  rime-engine │  ← librime FFI (safe Rust API)
├──────────────┤
│  candidate-ui│  ← Candidate rendering strategies
├──────────────┤
│  renderer    │  ← Direct2D/DirectWrite/DirectComposition
└──────────────┘
```

## Crates

| Crate | Description |
|-------|-------------|
| `skyme-common` | EventBus, shared types, utilities |
| `skyme-rime-engine` | Safe Rust bindings to librime |
| `skyme-ime-core` | TSF text service processor |
| `skyme-candidate-ui` | Candidate rendering (floating/inline/classic) |
| `skyme-renderer` | GPU rendering (Direct2D/DWrite/DComp) |
| `skyme-settings` | Hot-reloadable settings (theme, UI, IME) |
| `skyme-config` | Central configuration merger |
| `skyme-plugins` | Plugin system (emoji, AI, clipboard, etc.) |
| `skyme-diagnostics` | Developer inspection panel |

## Apps

| App | Description |
|-----|-------------|
| `skyme-ime-service` | TSF text service COM DLL |
| `skyme-settings-ui` | Standalone settings GUI |
| `skyme-deploy` | Schema deployment tool |

## Build

```bash
cargo build --workspace
```

Requires Rust 1.75+ and Windows SDK.

## Design Principles

- **Event-driven** — all communication goes through EventBus
- **Unsafe boundaries** — FFI lives only in `rime-engine`
- **No GDI/GDI+** — GPU-only rendering
- **Hot-reload** — config changes take effect without deploy
- **Theme ≠ logic** — themes only control appearance
