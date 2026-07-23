#!/usr/bin/env bash
set -euo pipefail

: "${LBPORT_M3U_ARGUMENT_LOG:?LBPORT_M3U_ARGUMENT_LOG is required}"
: "${LBPORT_M3U_CONTENT_LOG:?LBPORT_M3U_CONTENT_LOG is required}"
: "${LBPORT_M3U_LIFETIME_LOG:?LBPORT_M3U_LIFETIME_LOG is required}"

printf '%s\n' "$@" > "$LBPORT_M3U_ARGUMENT_LOG"

playlist_path=
playlist_directory=
while (($# > 0)); do
  case "$1" in
    --playlist)
      playlist_path=${2:?--playlist requires a value}
      shift 2
      ;;
    --dir)
      playlist_directory=${2:?--dir requires a value}
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [[ ! -f "$playlist_path" || ! -d "$playlist_directory" \
  || $(dirname "$playlist_path") != "$playlist_directory" ]]; then
  printf '%s\n' missing-playlist-before-exit > "$LBPORT_M3U_LIFETIME_LOG"
  exit 1
fi

cp "$playlist_path" "$LBPORT_M3U_CONTENT_LOG"
while IFS= read -r disc_path; do
  if [[ -z "$disc_path" || ! -f "$disc_path" ]]; then
    printf '%s\n' missing-disc-before-exit > "$LBPORT_M3U_LIFETIME_LOG"
    exit 1
  fi
done < "$playlist_path"

printf '%s\n' alive-before-exit > "$LBPORT_M3U_LIFETIME_LOG"
sleep 0.2

if [[ ! -f "$playlist_path" ]]; then
  printf '%s\n' missing-playlist-during-process > "$LBPORT_M3U_LIFETIME_LOG"
  exit 1
fi
while IFS= read -r disc_path; do
  if [[ -z "$disc_path" || ! -f "$disc_path" ]]; then
    printf '%s\n' missing-disc-during-process > "$LBPORT_M3U_LIFETIME_LOG"
    exit 1
  fi
done < "$playlist_path"

printf '%s\n' alive-until-exit > "$LBPORT_M3U_LIFETIME_LOG"
