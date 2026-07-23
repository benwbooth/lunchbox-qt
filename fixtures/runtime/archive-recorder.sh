#!/usr/bin/env bash
set -euo pipefail

: "${LBPORT_ARCHIVE_ARGUMENT_LOG:?LBPORT_ARCHIVE_ARGUMENT_LOG is required}"
: "${LBPORT_ARCHIVE_LIFETIME_LOG:?LBPORT_ARCHIVE_LIFETIME_LOG is required}"

printf '%s\n' "$@" > "$LBPORT_ARCHIVE_ARGUMENT_LOG"

rom_path=
rom_directory=
while (($# > 0)); do
  case "$1" in
    --rom)
      rom_path=${2:?--rom requires a value}
      shift 2
      ;;
    --dir)
      rom_directory=${2:?--dir requires a value}
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [[ ! -f "$rom_path" || ! -d "$rom_directory" ]]; then
  printf '%s\n' missing-before-exit > "$LBPORT_ARCHIVE_LIFETIME_LOG"
  exit 1
fi

printf '%s\n' alive-before-exit > "$LBPORT_ARCHIVE_LIFETIME_LOG"
sleep 0.2

if [[ ! -f "$rom_path" || ! -d "$rom_directory" ]]; then
  printf '%s\n' missing-during-process > "$LBPORT_ARCHIVE_LIFETIME_LOG"
  exit 1
fi

printf '%s\n' alive-until-exit > "$LBPORT_ARCHIVE_LIFETIME_LOG"
