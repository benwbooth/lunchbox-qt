#!/usr/bin/env bash
set -euo pipefail

target=x86_64-pc-windows-gnu
cargo_command=${CARGO:-cargo}

"$cargo_command" check --target "$target" \
  -p lb-domain \
  -p lb-import \
  -p lb-integrations \
  -p lb-platform \
  -p lb-query \
  -p lb-storage
