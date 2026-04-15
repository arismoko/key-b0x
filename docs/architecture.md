# Architecture

`key-b0x` is split into a pure controller core, a shared platform boundary, and
platform-specific runtime layers, plus a thin desktop shell.

- `key-b0x-core` owns the B0XX rules, SOCD handling, Firefox angles, shield
  behavior, and snapshot generation.
- `key-b0x-platform` owns normalized physical key codes, keyboard identifiers,
  and shared keyboard/transport traits.
- `key-b0x-platform-linux` owns Linux-only concerns: keyboard discovery, evdev
  capture across all active keyboards, FIFO creation, and FIFO writing.
- `key-b0x-platform-windows` owns Windows-only concerns: Raw Input keyboard
  capture across all active keyboards and Slippi named-pipe transport.
- `key-b0x-runtime` owns config loading, CLI commands, profile installation,
  diffing snapshots into Slippi pipe commands, and process lifecycle. The
  runtime chooses the active backend with `cfg` gates and keeps the input loop
  platform-neutral.
- `apps/desktop` owns the Electron shell, config editing UI, Slippi setup
  guidance, and runtime child-process lifecycle. It talks to the runtime over a
  small IPC layer instead of reimplementing gameplay behavior.

The Rust runtime is the source of truth for gameplay behavior. The desktop app
treats it as a managed child process rather than reimplementing input logic in
JavaScript.

Config is normalized around DOM-style physical key codes so the Electron GUI can
share the same binding language on Linux and Windows. Platform crates translate
between those normalized codes and the OS-native keyboard APIs.
