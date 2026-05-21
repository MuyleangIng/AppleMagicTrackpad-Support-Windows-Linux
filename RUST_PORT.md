# Rust Port Plan

The goal is to move this project toward Rust without breaking the working
Windows driver package.

## What Can Move First

1. Report parsing and transformation logic.
   This is pure logic and can be tested on macOS, Linux, and Windows.

2. Settings logic.
   `mt2-settings` holds the shared Rust backend, with `mt2-control` and
   `mt2-win-ui` using it.

3. Linux support.
   Rust is a good fit for a Linux userspace prototype using HID input and uinput,
   but that is a separate implementation from the Windows WDF driver.

4. macOS helper UI.
   macOS already supports Magic Trackpad natively. The Rust macOS helper opens
   system Trackpad and Bluetooth settings and can later become the status/control
   surface for any cross-platform features.

5. Windows Rust desktop UI.
   The `mt2-win-ui` crate provides a native Win32 Rust window for settings
   presets.

## What Cannot Be Bulk Converted Safely

The Windows driver code depends on KMDF, UMDF, WDF object lifetimes, HID
miniport behavior, INF packaging, and Microsoft driver signing. Rust does not
make that automatically faster, and an unsafe line-by-line rewrite would be
riskier than the current C.

The practical path is:

1. Keep the C driver building.
2. Move pure parser code into Rust.
3. Add tests against captured trackpad reports.
4. Expose Rust parser code through a small C ABI only after behavior matches.
5. Expand the Rust desktop UIs.
6. Build a separate Rust Linux driver/userspace implementation.
