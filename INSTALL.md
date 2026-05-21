# Install and Support Matrix

## Quick Answer

For Rust UI/tool downloads, use the GitHub Actions artifact workflow:

1. Open the repo on GitHub.
2. Go to **Actions**.
3. Run **Build Rust Tool Artifacts**.
4. Download the artifact for your OS:
   - `MagicTrackpadRs-windows-tools`
   - `MagicTrackpadRs-macos`
   - `MagicTrackpadRs-linux-helper`

For Windows 11, use a complete release zip, open PowerShell as Administrator,
and run:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\install.ps1
```

The script detects `AMD64` / `x64` or `ARM64` and installs the matching
`AmtPtpDevice.inf` with `pnputil`.

## Supported Systems

| OS | AMD64 / x64 | ARM64 | Notes |
|---|---|---|---|
| Windows 11 | Supported | Package/build path exists | Main target |
| Windows 10 | Supported with workaround/package path | Not supported | Use the Windows 10 package flow |
| macOS | Native Apple driver | Native Apple driver | Optional Rust status UI only |
| Linux | Not supported yet | Not supported yet | Needs separate Linux HID/input work |

## Windows Install

1. Uninstall old Magic Trackpad drivers first, especially Magic Utilities,
   Trackpad++, imbushuo, or Apple's 2021 driver.
2. Download and unzip a complete release package.
3. Open PowerShell as Administrator in the unzipped folder.
4. Run:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\install.ps1
```

Manual install is also supported: open the `AMD64` or `ARM64` folder,
right-click `AmtPtpDevice.inf`, then choose Install.

## Windows 10 AMD64 Package Flow

Windows 10 AMD64 uses the workaround package flow:

```bat
cd build
make_win10.bat
```

Windows 10 ARM64 is not supported.

## Build From Source

Building Windows drivers requires Visual Studio, WDK, signing certificates, and
the usual Windows driver packaging tools. From a suitable Windows developer
prompt:

```bat
cd build
make.bat
```

`make.bat` builds both AMD64 and ARM64 packages. The resulting driver still
needs proper signing before normal Windows installation.

## macOS

macOS already supports Magic Trackpad devices through Apple's built-in driver.
This repo does not install a macOS driver.

The Rust desktop status UI can be run with:

```sh
./run-mac-ui.sh
```

It shows trackpad name, connection status, battery, transport, product ID,
serial, firmware, and current macOS trackpad settings.

## Linux

Linux is not supported by this Windows driver package. Running:

```sh
./install.sh
```

detects Linux and exits without changing the system.

Linux support needs a separate implementation using the Linux HID/input stack,
such as a `hidraw` + `uinput` userspace driver or a kernel HID driver.
