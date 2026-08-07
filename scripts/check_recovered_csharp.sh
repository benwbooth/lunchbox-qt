#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
project="$root/apps/lb-csharp-qt/recovered/Unbroken.LaunchBox.Recovered.csproj"
log=$(mktemp "${TMPDIR:-/tmp}/lunchbox-recovered-csharp.XXXXXX.log")
trap 'rm -f "$log"' EXIT

"$root/scripts/normalize_recovered_csharp.sh"

set +e
nix shell nixpkgs#dotnet-sdk_9 --command dotnet build "$project" -t:Rebuild -v:minimal >"$log" 2>&1
status=$?
set -e

printf '%s\n' 'Recovered C# compiler census:'
error_count=$(rg '^/.*\([0-9]+,[0-9]+\): error ' "$log" | sort -u | wc -l || true)
warning_count=$(rg '^/.*\([0-9]+,[0-9]+\): warning ' "$log" | sort -u | wc -l || true)
printf 'errors=%s warnings=%s\n' "$error_count" "$warning_count"
rg '^/.*\([0-9]+,[0-9]+\): error ' "$log" \
  | sort -u \
  | sed -E 's/.*: error ([A-Z0-9]+):.*/\1/' \
  | sort | uniq -c | sort -nr || true

if [[ "$status" -ne 0 ]]; then
  printf '%s\n' 'The recovered source does not compile; see the error census above.' >&2
  exit "$status"
fi

printf '%s\n' 'Recovered source compiled.'
