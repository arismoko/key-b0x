# Architecture

`key-b0x` is split into a pure controller core, a shared platform boundary, and
platform-specific runtime layers.

- `key-b0x-core` owns the B0XX rules, SOCD handling, Firefox angles, shield
  behavior, and snapshot generation.
- `key-b0x-platform` owns normalized physical key codes, keyboard identifiers,
  backend capabilities, and shared keyboard/transport traits.
- `key-b0x-platform-linux` owns Linux-only concerns: keyboard discovery,
  optional exclusive grab, FIFO creation, and FIFO writing.
- `key-b0x-platform-windows` owns Windows-only concerns: Raw Input keyboard
  capture and Slippi named-pipe transport.
- `key-b0x-runtime` owns config loading, CLI commands, profile installation,
  diffing snapshots into Slippi pipe commands, and process lifecycle. The
  runtime chooses the active backend with `cfg` gates and keeps the input loop
  platform-neutral.

The Rust runtime is the source of truth for gameplay behavior. The future
desktop app should treat it as a managed child process rather than reimplement
input logic in JavaScript.

Config is normalized around DOM-style physical key codes so the future Electron
GUI can share the same binding language on Linux and Windows. Platform crates
translate between those normalized codes and the OS-native keyboard APIs.
