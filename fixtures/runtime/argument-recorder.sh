#!/usr/bin/env bash
set -euo pipefail

: "${LBPORT_LAUNCH_SMOKE_LOG:?LBPORT_LAUNCH_SMOKE_LOG is required}"
printf '%s\n' "$@" > "$LBPORT_LAUNCH_SMOKE_LOG"
sleep 1.05
