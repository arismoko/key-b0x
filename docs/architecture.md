# Architecture

`key-b0x` is split into a pure controller core, a shared platform boundary, an
application-service layer, and a thin desktop host.

- `key-b0x-core` owns the B0XX rules, SOCD handling, Firefox angles, shield
  behavior, and snapshot generation.
- `key-b0x-platform` owns normalized physical key codes, keyboard identifiers,
  and shared keyboard/transport traits.
- `key-b0x-platform-linux` owns Linux-only concerns: keyboard discovery, evdev
  capture across all active keyboards, FIFO creation, and FIFO writing.
- `key-b0x-platform-windows` owns Windows-only concerns: Raw Input keyboard
  capture across all active keyboards and Slippi named-pipe transport.
- `key-b0x-app` owns config loading, profile installation,
  snapshot diffing, runtime state transitions, and the worker-thread lifecycle.
- `apps/desktop/src-tauri` owns the Tauri adapter, commands, event emission,
  and native packaging.
- `apps/desktop/src` owns the React product surface, Slippi setup guidance, and
  config editing UI.

Rust remains the source of truth for gameplay behavior, config persistence, and
runtime lifecycle. The desktop renderer only talks to that Rust service through
Tauri commands and runtime state events.

Config is normalized around DOM-style physical key codes so the desktop UI can
share the same binding language on Linux and Windows. Platform crates translate
between those normalized codes and the OS-native keyboard APIs.
