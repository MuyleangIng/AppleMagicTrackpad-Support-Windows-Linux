# Rust Migration

This folder starts a gradual Rust migration. It does not replace the Windows
driver yet.

## Current Rust Components

| Crate | Platform | Status | Purpose |
|---|---|---|---|
| `mt2-core` | Cross-platform | Working tests | Magic Trackpad 2 report parsing |
| `mt2-settings` | Windows backend, cross-platform tests | Working tests | Shared presets and Windows registry writes |
| `mt2-control` | Windows | Early utility | Command-line settings tool |
| `mt2-win-ui` | Windows | Early desktop UI | Native Win32 settings UI |
| `mt2-mac-ui` | macOS | Early desktop UI | Native AppKit status UI |

The production Windows driver is still the existing C KMDF/UMDF driver. Rust
currently covers shared logic, settings tools, and native desktop UI prototypes.

`mt2-core` contains driver-independent Magic Trackpad 2 report parsing logic.
That is the safest first code to move because it can be tested on any platform
and reused later by:

- a Linux `hidraw` + `uinput` prototype
- a future Windows FFI boundary
- parser tests that validate behavior before changing driver code

The current Windows KMDF/UMDF driver remains C because Windows driver signing,
WDK integration, HID miniport behavior, and kernel APIs are all tied to the
existing C build.

`mt2-settings` contains the shared Rust settings backend for Windows registry
values and haptic/click presets.

`mt2-control` is the Rust command-line settings utility. It uses the shared
settings backend and can apply the same presets as the desktop UI.

`mt2-mac-ui` is a native macOS desktop UI launched from Rust through AppKit. It
shows Magic Trackpad status, battery, transport, product details, and current
trackpad settings. macOS already has native Magic Trackpad support, so this app
does not install Windows drivers.

`mt2-win-ui` is a native Windows desktop UI launched from Rust through Win32. It
can apply haptic and click presets through the shared Rust settings backend. Run
it as Administrator so Windows allows registry writes.

## Commands

Run the Rust tests:

```sh
cargo test
```

Build the Rust control utility:

```sh
cargo build -p mt2-control
```

Run the macOS desktop UI:

```sh
./run-mac-ui.sh
```

On Windows, build and run the Rust desktop UI from Administrator PowerShell:

```powershell
cargo run -p mt2-win-ui
```
