#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
patch_file="$root/apps/lb-csharp-qt/recovered/compatibility-shape.patch"

if [[ ! -d "$root/decompiled/Unbroken.LaunchBox" ]]; then
  printf 'Missing decompiled source: %s\n' "$root/decompiled/Unbroken.LaunchBox" >&2
  printf 'Run scripts/decompile.sh first.\n' >&2
  exit 1
fi

if git -C "$root" apply --check "$patch_file" >/dev/null 2>&1; then
  git -C "$root" apply "$patch_file"
  printf 'Applied compiler-shape normalization to the local decompiled tree.\n'
elif git -C "$root" apply --reverse --check "$patch_file" >/dev/null 2>&1; then
  printf 'Compiler-shape normalization is already applied.\n'
else
  printf 'The decompiled tree does not match the recorded normalization patch.\n' >&2
  printf 'Regenerate it with scripts/decompile.sh and retry.\n' >&2
  exit 1
fi
