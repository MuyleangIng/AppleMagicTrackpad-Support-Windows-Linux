# Magic-Tranpad-mac-OS-win-linux

Rust-first tools and Windows driver package for Apple Magic Trackpad devices across Windows, macOS, and future Linux support.

The current Windows driver code is based on the excellent [imbushuo](https://github.com/imbushuo/mac-precision-touchpad) Magic Trackpad driver and supports Bluetooth. Compared to imbushuo or to the official 2021 Apple driver, this project adds:

- support for USB-C Magic Trackpad 2
- battery level reading
- haptic feedback control
- various options for controlling pointer precision
- Rust desktop and command-line tools for settings.

The previous version of this project used a hack to install itself in the DriverStore and couldn't support Bluetooth. At the beginning of this year, I decided to purchase an EV certificate to properly sign the driver: I paid 485 euros for it, including taxes that I have no way of recovering as an individual (btw, only organizations can request an EV certificate). I was tired of seeing people resorting to the wildest hacks to get the MT2 to work via Bluetooth 😀 (you can get a glimpse of this in the issues of this repo). **Windows drivers signing requirements and costs are unfair to open-source developers**.

## Current status

| Platform | Architecture | Driver support | UI support | Install path |
|---|---:|---|---|---|
| Windows 11 | AMD64 / x64 | Supported | Rust CLI and Rust desktop UI | `install.ps1` or right-click INF |
| Windows 11 | ARM64 | Package/build path exists | Rust CLI and Rust desktop UI | `install.ps1` or right-click INF |
| Windows 10 | AMD64 / x64 | Supported through workaround/package path | Rust CLI and Rust desktop UI | `build\make_win10.bat` package or `install.ps1` with a complete package |
| Windows 10 | ARM64 | Not supported | Not supported | No supported install |
| macOS | Apple Silicon / Intel | Uses Apple's built-in driver | Rust desktop status UI | `./run-mac-ui.sh` |
| Linux | AMD64 / ARM64 | Not supported yet | Not supported yet | `install.sh` detects Linux and stops safely |

## Features

### Windows driver features

- Magic Trackpad 2 Bluetooth support.
- Magic Trackpad 2 USB and USB-C support.
- Windows Precision Touchpad reporting.
- Battery level query for Bluetooth devices.
- Haptic feedback presets.
- Pointer precision options.
- Near-finger, button-finger, and palm-rejection options.

### Rust tools

- `mt2-core`: Magic Trackpad 2 report parsing logic.
- `mt2-settings`: shared Rust settings backend.
- `mt2-control`: Windows command-line settings utility.
- `mt2-win-ui`: native Windows desktop UI for settings presets.
- `mt2-mac-ui`: native macOS desktop UI for trackpad status.

The Windows driver itself is still the existing C KMDF/UMDF implementation. The Rust port is in progress and currently covers parser, settings, and UI/tooling pieces.

## Easy install

### Windows 11

0) Uninstall any previous versions of this driver, imbushuo or `official 2021 Apple driver`. Personally I use [DriverStore Explorer](https://github.com/lostindark/DriverStoreExplorer) for that, alternatively you can use Windows Device Manager. Also, **it's especially important to uninstall `Magic Utilities` and `Trackpad++`** before continuing with the installation!

1) Download the zip file of this project from this repo's Releases page and unzip it.

2) Select your architecture: AMD64 or ARM64. Right-click on the INF file and click "Install".

Or open PowerShell as Administrator from the unzipped release folder and run:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\install.ps1
```

The script detects AMD64/ARM64 and installs the matching driver package. See [INSTALL.md](INSTALL.md) for details.

### Windows 10

Windows 10 AMD64 uses the Windows 10 package flow documented in [INSTALL.md](INSTALL.md).

Windows 10 ARM64 is not supported.

### macOS

macOS already includes native Magic Trackpad support. This repo does not install a macOS driver. To run the Rust macOS status UI:

```sh
./run-mac-ui.sh
```

### Linux

Linux support is not implemented yet. This Windows driver cannot run on Linux because it uses KMDF/UMDF/WDF. Running the Linux helper is safe:

```sh
./install.sh
```

It detects Linux and exits without changing the system.

## Rust workspace

Run all Rust tests:

```sh
cargo test
```

Run the macOS desktop UI:

```sh
./run-mac-ui.sh
```

Run the Windows desktop UI from Administrator PowerShell:

```powershell
cargo run -p mt2-win-ui
```

Run the Windows CLI settings tool:

```powershell
cargo run -p mt2-control -- help
```

## Credits

- [This excellent PR](https://github.com/imbushuo/mac-precision-touchpad/pull/533) of [1Revenger1](https://github.com/1Revenger1) to the imbushuo repo, which fixes the "near field fingers" problem, cleans up the code, and removes the QueryPerformanceCounter call in the interrupt function.
- The haptic feedback control messages sent by the driver to the MT2 in this project are based on the excellent reverse engineering work of [dos1](https://github.com/dos1) ([here](https://github.com/mwyborski/Linux-Magic-Trackpad-2-Driver/issues/28#issuecomment-451625504)).
- My long-time friends at [Landlogic IT](https://landlogic.it/), who took care of the grueling process of gaining access to Microsoft's Hardware Dashboard and who take care of signing the driver packages for me.
- Community contributors who helped fund driver-signing work.
