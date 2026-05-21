# Magic-Tranpad-mac-OS-win-linux

Rust-first Apple Magic Trackpad tools for macOS, Windows, and future Linux support.

This repository is being rebuilt around Rust. The Rust parts are the shared parser,
settings backend, Windows desktop UI, Windows CLI, and macOS desktop status UI.
The Windows driver package is still C KMDF/UMDF code while the driver logic is
ported in safe stages.

## What This Is

- A Windows Precision Touchpad driver package for Apple Magic Trackpad devices.
- Rust desktop and CLI tools for settings/status.
- A Rust migration workspace for moving reusable driver logic out of C.
- A future base for Linux HID/input support.

## Important License Notice

The current Windows driver code is derived from previous GPL driver work,
including the `imbushuo/mac-precision-touchpad` lineage. Because that driver
code remains in this repository, the GPL license and attribution must stay.

See [NOTICE.md](NOTICE.md) for attribution and status.

## Current Support

| Platform | Architecture | Driver | UI/tooling | Status |
|---|---:|---|---|---|
| Windows 11 | AMD64 / x64 | Supported driver package | Rust desktop UI + Rust CLI | Main target |
| Windows 11 | ARM64 | Build/package path exists | Rust desktop UI + Rust CLI | Needs real-device testing |
| Windows 10 | AMD64 / x64 | Workaround/package path | Rust desktop UI + Rust CLI | Partial support |
| Windows 10 | ARM64 | Not supported | Not supported | Not planned yet |
| macOS | Apple Silicon / Intel | Uses Apple's built-in driver | Rust desktop status UI | Supported helper UI |
| Linux | AMD64 / ARM64 | Not implemented yet | Not implemented yet | Future Rust HID/input work |

## Current Features

### Windows Driver Package

- Magic Trackpad 2 Bluetooth support.
- Magic Trackpad 2 USB and USB-C support.
- Windows Precision Touchpad reports.
- Bluetooth battery query.
- Haptic feedback settings.
- Pointer precision settings.
- Near-finger, button-finger, and palm-rejection options.

### Rust Workspace

- `mt2-core`: Magic Trackpad 2 report parser with tests.
- `mt2-settings`: shared Rust settings and preset backend.
- `mt2-control`: Windows CLI settings utility.
- `mt2-win-ui`: native Windows desktop settings UI through Win32.
- `mt2-mac-ui`: native macOS desktop status UI through AppKit.

## Install

### Windows 11

Use a complete release package, open PowerShell as Administrator, then run:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\install.ps1
```

The installer detects AMD64/x64 or ARM64 and installs the matching INF package.
Manual install is also possible by right-clicking `AmtPtpDevice.inf` inside the
matching architecture folder.

### Windows 10

Windows 10 AMD64 uses the Windows 10 package flow documented in
[INSTALL.md](INSTALL.md). Windows 10 ARM64 is not supported.

### macOS

macOS already has a built-in Magic Trackpad driver. This repo does not install a
macOS driver. Run the Rust desktop status UI with:

```sh
./run-mac-ui.sh
```

### Linux

Linux driver support is not implemented yet. This command only detects Linux and
exits safely:

```sh
./install.sh
```

## Downloadable Builds

This repo can produce user-downloadable files through GitHub Actions:

| Artifact | Contents | Platform |
|---|---|---|
| `MagicTrackpadRs-windows-tools.zip` | `MagicTrackpadRs.exe`, `mt2-control.exe`, docs | Windows |
| `MagicTrackpadRs-macos.tar.gz` | `MagicTrackpadRs-mac`, launcher script, docs | macOS |
| `MagicTrackpadRs-linux-helper.tar.gz` | Linux helper CLI/scripts and docs | Linux |

Open the **Actions** tab on GitHub and run **Build Rust Tool Artifacts**. Tagged
releases named `v*` also trigger artifact builds.

The Windows driver package is separate from these Rust UI/tool artifacts and
still requires Windows driver signing.

## Build and Test

### Rust

Install Rust with `rustup`, then run:

```sh
cargo test
```

Build the Rust tools:

```sh
cargo build --workspace
```

Run the macOS UI:

```sh
./run-mac-ui.sh
```

Run the Windows desktop UI from Administrator PowerShell:

```powershell
cargo run -p mt2-win-ui
```

Run the Windows CLI:

```powershell
cargo run -p mt2-control -- help
```

### Windows Driver Package

Building the driver package requires Visual Studio, WDK, SDK, NuGet package
restore, signing certificates, and Windows driver packaging tools.

From a suitable Windows developer prompt:

```bat
cd build
make.bat
```

The driver package still needs proper signing before normal Windows installation.

## Repo Layout

```text
MagicTrackpadUsbDriver/     Windows USB UMDF driver source
MagicTrackpadHidFilter/     Windows HID filter driver source
rust/                       Rust parser, settings, and UI crates
build/                      Windows INF and packaging scripts
install.ps1                 Windows install helper
install.sh                  Linux/macOS safe detector
run-mac-ui.sh               macOS Rust UI launcher
```

## Rust Port Direction

The safe path is staged:

1. Keep the current Windows C driver building.
2. Move pure report parsing and settings logic into Rust.
3. Add tests against captured Magic Trackpad reports.
4. Expose Rust logic to the driver through a small C ABI only after behavior
   matches.
5. Build a separate Rust Linux HID/input implementation later.

See [RUST_PORT.md](RUST_PORT.md).
