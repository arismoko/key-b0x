# Getting Started

This guide is for players who want to download `key-b0x` and get into Slippi
without digging through repo internals.

## Before You Start

- `key-b0x` currently supports Windows and Linux.
- It is currently aimed at Slippi / Ishiiruka.
- The app reads from all active keyboards in your current session. If another
  keyboard is sending inputs, `key-b0x` will see those too.

## Download

1. Open the [latest release](https://github.com/arismoko/key-b0x/releases/latest).
2. Click `Assets` if the download files are collapsed.
3. Download the file for your platform:

- Windows: `key-b0x_*_windows_x64-setup.exe`
- Linux: `key-b0x_*_linux_x86_64.AppImage`

## First-Time Setup

1. Open `key-b0x`.
2. Check the Slippi user folder shown in the app.
3. If you use the default Slippi location, you can usually leave it alone and
   press `Next`.
4. If you use a custom Slippi folder, press `Browse` and point `key-b0x` at the
   correct user folder before continuing.
5. `key-b0x` installs its controller profile for you.

Default Slippi user folders:

- Linux: `~/.config/SlippiOnline`
- Windows: `%APPDATA%\Slippi Launcher\netplay\User`

## Dolphin / Ishiiruka Setup

After `key-b0x` installs the profile:

1. Open Dolphin or Ishiiruka.
2. Open the controller settings.
3. Set Port 1 to `Standard Controller`.
4. Select the `key-b0x` profile.
5. Press `Load`.

## Keyboard Test

Before you play, open the in-app keyboard test and hold the combinations you
plan to use.

- If every held key appears at the same time, that keyboard is probably fine.
- If one key drops out when you hold several at once, that keyboard is likely
  not a good fit for `key-b0x`.

## Troubleshooting

### I cannot find the download button

Open the release page and click `Assets`. GitHub often collapses the files by
default.

### key-b0x says it is waiting for Slippi

In Slippi / Dolphin, open the controller settings, set Port 1 to `Standard
Controller`, select the `key-b0x` profile, and press `Load`. Restart Slippi /
Dolphin if key-b0x still does not connect.

### The Linux AppImage aborts with a GBM EGL display error

Current AppImage builds automatically disable WebKit's DMA-BUF renderer, which
avoids the known startup failure on some NVIDIA / Wayland systems. For older
builds, use this workaround:

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 ./key-b0x_*.AppImage
```

### The wrong Slippi folder was detected

Use the `Browse` button in `key-b0x` and select the Slippi user folder
manually.

### My keyboard misses some combinations

Use the keyboard test. If the test does not show all held keys together,
`key-b0x` cannot fix that in software. You will need a keyboard with better key
rollover.

### A held direction stops until I press it again

The default `2IP No Reactivation` SOCD mode follows B0XX Melee behavior. If you
hold one direction, press its opposite, and then release the newer direction,
the output intentionally stays neutral until you release and press the original
direction again. The keyboard test still shows the raw held key, which can make
this feel like a dropped input.

For keyboard-like holds, open Settings, find `SOCD Mode`, choose `2IP
Reactivation`, and save the Melee settings. Then releasing the newer opposite
direction returns to the direction you are still holding. Check the rules for
your tournament before changing SOCD behavior.

### Linux updates are not applying cleanly

Keep the AppImage in a stable writable location such as
`~/Applications/key-b0x.AppImage`.
