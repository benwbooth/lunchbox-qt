#!/usr/bin/env bash
set -euo pipefail

: "${LBPORT_SCUMMVM_ARGUMENT_LOG:?LBPORT_SCUMMVM_ARGUMENT_LOG is required}"
printf '%s\n' "$@" > "$LBPORT_SCUMMVM_ARGUMENT_LOG"
