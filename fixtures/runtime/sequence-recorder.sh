#!/usr/bin/env bash
set -euo pipefail

: "${LBPORT_SEQUENCE_SMOKE_LOG:?LBPORT_SEQUENCE_SMOKE_LOG is required}"

if [[ ${1:-} != "--step" || -z ${2:-} || $# -ne 2 ]]; then
  printf 'Unexpected sequence-recorder arguments:' >&2
  printf ' <%s>' "$@" >&2
  printf '\n' >&2
  exit 64
fi

printf '%s\n' "$2" >> "$LBPORT_SEQUENCE_SMOKE_LOG"
