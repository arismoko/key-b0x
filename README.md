# key-b0x

`key-b0x` is a Linux-first Slippi keyboard controller runtime. It ports the
existing `b0xx-linux` controller logic into a Rust sidecar that captures
keyboard input and writes directly to Slippi's pipe controller backend instead
of creating a virtual gamepad.

This repository currently contains the native proof of concept:

- `crates/core`: pure B0XX state machine and snapshot generation
- `crates/platform-linux`: Linux keyboard capture and FIFO transport
- `crates/runtime`: CLI runtime for listing keyboards, installing the Slippi
  profile, printing default config, and running the input loop

The future Electron desktop app is intentionally deferred until the native
runtime is stable.

## Current Commands

```bash
cargo run -p key-b0x-runtime -- list-keyboards
cargo run -p key-b0x-runtime -- print-default-config
cargo run -p key-b0x-runtime -- install-profile
cargo run -p key-b0x-runtime -- run
```

## Linux Notes

- Slippi user data defaults to `~/.config/SlippiOnline`
- The runtime creates `~/.config/SlippiOnline/Pipes/slippibot1` if needed
- The installed profile lives at
  `~/.config/SlippiOnline/Config/Profiles/GCPad/key-b0x.ini`
- Exclusive keyboard grab is off by default; only opt into it deliberately
- You still need to load the `key-b0x` profile in Slippi's controller UI

## Next Steps

- Finish Windows named-pipe transport
- Add the Electron + React + TypeScript desktop UI
