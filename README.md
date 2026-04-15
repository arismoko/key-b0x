# key-b0x

`key-b0x` is a cross-platform Slippi keyboard controller desktop app. It writes directly to Slippi's pipe controller backend instead of creating a virtual gamepad. This means you just need to download one executable to get up and running.

I made this because the other solutions typically require the user to fuss around with outdated drivers and autohotkey. This is a user friendly way to avoid all that for people who just want to play on Slippi with their keyboard.

I intend on adding Project M support/general dolphin support in the future but for now this is just targeting Slippi/Ishiiruka.

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

## Release Process

The repo uses three GitHub Actions workflows:

- `ci.yml`: fast validation on pull requests and pushes to `main`
- `bundle-smoke.yml`: real AppImage and NSIS packaging on `main` and manual runs, uploaded as workflow artifacts
- `release.yml`: tag-driven draft GitHub Releases with AppImage, NSIS, and `SHA256SUMS.txt`

Release steps:

1. Bump the desktop app version in `apps/desktop/package.json`.
2. Merge the release candidate to `main`.
3. Let `bundle-smoke.yml` build the real installers from `main`.
4. Manually validate the Linux AppImage and Windows NSIS installer before any public release.
5. Create a `vX.Y.Z` tag.
6. Let `release.yml` build the same packaging matrix and create a draft GitHub Release.
7. Validate the draft release artifacts and checksums, then publish it.

Public release remains gated on manual installer verification on both Linux and Windows.
Reserve the protected GitHub `release` environment for Windows signing secrets before the first public release.
Linux AppImage releases target modern Linux distributions rather than oldest-possible glibc compatibility.

## Config Notes

- Slippi user data defaults to `~/.config/SlippiOnline` on Linux and
  `%APPDATA%\Slippi Launcher\netplay\User` on Windows
- The config format is now `v2` and stores normalized physical key codes such
  as `BracketRight`, `Digit3`, `KeyV`, and `ArrowUp`
- Linux creates `~/.config/SlippiOnline/Pipes/slippibot1` when the profile is
  installed
- The installed profile lives at
  `~/.config/SlippiOnline/Config/Profiles/GCPad/key-b0x.ini` on Linux and
  `%APPDATA%\Slippi Launcher\netplay\User\Config\Profiles\GCPad\key-b0x.ini`
  on Windows
- Both Linux and Windows capture from all active keyboards in the current
  session
- You still need to load the `key-b0x` profile in Slippi's controller UI
