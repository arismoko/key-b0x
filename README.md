# key-b0x

`key-b0x` is a cross-platform Slippi keyboard controller runtime. It ports the
existing `b0xx-linux` controller logic into a Rust sidecar that captures
keyboard input and writes directly to Slippi's pipe controller backend instead
of creating a virtual gamepad.

The native runtime is split into:

- `crates/core`: pure B0XX state machine and snapshot generation
- `crates/platform`: shared normalized-key model and backend traits
- `crates/platform-linux`: Linux keyboard capture and FIFO transport
- `crates/platform-windows`: Windows Raw Input capture and named-pipe transport
- `crates/runtime`: CLI runtime for listing keyboards, installing the Slippi
  profile, printing default config, and running the input loop

The repo now also includes an Electron desktop app in `apps/desktop`. It wraps
the Rust runtime with a wizard-style SPA for Slippi setup, binding edits, and
runtime start / stop.

## Current Commands

```bash
cargo run -p key-b0x-runtime -- list-keyboards
cargo run -p key-b0x-runtime -- print-default-config
cargo run -p key-b0x-runtime -- install-profile
cargo run -p key-b0x-runtime -- run
```

## Desktop App

```bash
cd apps/desktop
npm install
npm run dev
```

## Runtime Notes

- Slippi user data defaults to `~/.config/SlippiOnline` on Linux and
  `%APPDATA%\Slippi Launcher\netplay\User` on Windows
- The config format is now `v2` and stores normalized physical key codes such
  as `BracketRight`, `Digit3`, `KeyV`, and `ArrowUp`
- Existing `v1` configs using Linux `KEY_*` names are intentionally rejected;
  regenerate `config.toml` from `print-default-config`
- Linux creates `~/.config/SlippiOnline/Pipes/slippibot1` when the profile is
  installed
- The installed profile lives at
  `~/.config/SlippiOnline/Config/Profiles/GCPad/key-b0x.ini` on Linux and
  `%APPDATA%\Slippi Launcher\netplay\User\Config\Profiles\GCPad\key-b0x.ini`
  on Windows
- Both Linux and Windows capture from all active keyboards in the current
  session
- You still need to load the `key-b0x` profile in Slippi's controller UI

## Next Steps

- connect the desktop app to packaged runtime binaries
- expand desktop-side tests around IPC and runtime lifecycle
