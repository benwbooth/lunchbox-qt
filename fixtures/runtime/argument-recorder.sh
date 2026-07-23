#!/usr/bin/env bash
set -euo pipefail

: "${LBPORT_LAUNCH_SMOKE_LOG:?LBPORT_LAUNCH_SMOKE_LOG is required}"
printf '%s\n' "$@" > "$LBPORT_LAUNCH_SMOKE_LOG"
exec sleep "${LBPORT_LAUNCH_SMOKE_SLEEP:-1.05}"
