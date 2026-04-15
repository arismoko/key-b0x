# key-b0x

`key-b0x` is a cross-platform Slippi keyboard controller based on the work of [agirardeau/b0xx-ahk](https://github.com/agirardeau/b0xx-ahk) and [JonnyHaystack/HayBox](https://github.com/JonnyHaystack/HayBox). It writes directly to Slippi's pipe controller backend instead of creating a virtual gamepad. This means you just need to download one executable to get up and running.

I made this because the other solutions typically require the user to fuss around with outdated drivers and autohotkey. This is a user friendly way to avoid all that for people who just want to play on Slippi with their keyboard.

I intend on adding Project M support/general dolphin support in the future but for now this is just targeting Slippi/Ishiiruka.

![key-b0x desktop app dashboard](docs/assets/key-b0x-dashboard.png)

The native stack is split into:

- `crates/core`: pure B0XX state machine and snapshot generation
- `crates/platform`: shared normalized-key model and backend traits
- `crates/platform-linux`: Linux keyboard capture and FIFO transport
- `crates/platform-windows`: Windows Raw Input capture and named-pipe transport
- `crates/app`: config, setup, profile install, runtime lifecycle, and state
  transitions for the desktop host
- `apps/desktop/src-tauri`: thin Tauri adapter and packaging config

## Desktop App

```bash
cd apps/desktop
npm install
npm run dev
```

Release builds use `npm run build`, which delegates to `tauri build`.

## Releases

CI and tagged release automation run through GitHub Actions. The maintainer
release runbook lives in [docs/releasing.md](docs/releasing.md).

## Config Notes

- Slippi user data defaults to `~/.config/SlippiOnline` on Linux and
  `%APPDATA%\Slippi Launcher\netplay\User` on Windows
- Linux creates `~/.config/SlippiOnline/Pipes/slippibot1` when the profile is installed
- The installed profile lives at
  `~/.config/SlippiOnline/Config/Profiles/GCPad/key-b0x.ini` on Linux and
  `%APPDATA%\Slippi Launcher\netplay\User\Config\Profiles\GCPad\key-b0x.ini`
  on Windows
- Both Linux and Windows capture from all active keyboards in the current
  session
- You still need to load the `key-b0x` profile in Slippi's controller UI
- For Linux in-app updates, keep `key-b0x.AppImage` in a stable writable
  location such as `~/Applications/key-b0x.AppImage`
