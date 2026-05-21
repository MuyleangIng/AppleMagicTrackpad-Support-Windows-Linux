#!/usr/bin/env sh
set -eu

say() {
  printf '%s\n' "$*"
}

os="$(uname -s 2>/dev/null || printf unknown)"
arch="$(uname -m 2>/dev/null || printf unknown)"

case "$os" in
  Linux)
    say "Detected Linux $arch."
    say ""
    say "This repository contains Windows KMDF/UMDF drivers. They cannot be installed on Linux."
    say "Linux support needs a separate HID/input implementation, for example a hidraw/uinput userspace driver or a kernel HID driver."
    say ""
    say "Nothing was installed."
    exit 1
    ;;
  MINGW*|MSYS*|CYGWIN*)
    say "Detected a Windows shell."
    say "Run this from Administrator PowerShell instead:"
    say "  Set-ExecutionPolicy -Scope Process Bypass"
    say "  .\\install.ps1"
    exit 1
    ;;
  Darwin)
    say "Detected macOS $arch."
    say "This project is for using Magic Trackpad 2 on Windows. macOS does not need this driver."
    exit 1
    ;;
  *)
    say "Unsupported operating system: $os $arch"
    exit 1
    ;;
esac
