#!/usr/bin/env sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  echo "The macOS UI runs only on macOS."
  exit 1
fi

cargo run -p mt2-mac-ui
