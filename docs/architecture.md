# Architecture

`key-b0x` is split into a pure controller core and platform-specific runtime
layers.

- `key-b0x-core` owns the B0XX rules, SOCD handling, Firefox angles, shield
  behavior, and snapshot generation.
- `key-b0x-platform-linux` owns Linux-only concerns: keyboard discovery,
  optional exclusive grab, FIFO creation, and FIFO writing.
- `key-b0x-runtime` owns config loading, CLI commands, profile installation,
  diffing snapshots into Slippi pipe commands, and process lifecycle.

The Rust runtime is the source of truth for gameplay behavior. The future
desktop app should treat it as a managed child process rather than reimplement
input logic in JavaScript.
