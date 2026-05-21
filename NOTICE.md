# Notice and Attribution

This repository is a Rust-first rebuild, but it still contains Windows driver
code derived from earlier open-source Magic Trackpad driver work.

## Current Code Status

- Rust parser/settings/UI code is new work in this repository.
- Windows driver code under `MagicTrackpadUsbDriver/` and
  `MagicTrackpadHidFilter/` is still C KMDF/UMDF code derived from the previous
  Magic Trackpad driver lineage.
- Because GPL-derived driver code is still present, this repository keeps the
  GPL license.

## Upstream Lineage

The Windows driver code is based on work from:

- `imbushuo/mac-precision-touchpad`
- later Magic Trackpad 2 Windows driver forks and improvements
- community reverse-engineering work around Magic Trackpad 2 haptic feedback

## Why The License Stays

The license cannot be removed while GPL-derived driver code remains in the
repository. A future fully independent rewrite could choose a different license
only after all derived code is removed or replaced with independently authored
code.

## Rebuild Goal

The goal is to make the project easier to understand and eventually move more
logic to Rust:

- Rust tested parser core.
- Rust settings backend.
- Rust desktop UI on Windows and macOS.
- Future Rust Linux HID/input implementation.
- Gradual driver-side Rust integration where it is safe and testable.
