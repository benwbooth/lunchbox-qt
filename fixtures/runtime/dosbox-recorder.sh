#!/usr/bin/env bash
set -euo pipefail

: "${LBPORT_DOSBOX_ARGUMENT_LOG:?LBPORT_DOSBOX_ARGUMENT_LOG is required}"
printf '%s\n' "$@" > "$LBPORT_DOSBOX_ARGUMENT_LOG"
