#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
installer=${1:-/home/ben/Downloads/LaunchBox-13.27-Setup.exe}
expected_sha=19deeee55c135ffb1b720bcfcdecdd9e103ac86a6c47ffdc2b6b5a4af83b6481
oracle_dir="$root/oracle"
prefix="$oracle_dir/wine-prefix"
install_dir="$oracle_dir/LaunchBox"
log="$oracle_dir/installer.log"

if [[ ! -f "$installer" ]]; then
    printf 'Installer not found: %s\n' "$installer" >&2
    exit 1
fi

actual_sha=$(sha256sum "$installer" | cut -d' ' -f1)
if [[ "$actual_sha" != "$expected_sha" ]]; then
    printf 'Unexpected installer SHA-256: %s\n' "$actual_sha" >&2
    exit 1
fi

if [[ -e "$prefix" || -e "$install_dir" ]]; then
    printf 'Oracle already exists under %s; refusing to overwrite it.\n' "$oracle_dir" >&2
    exit 1
fi

mkdir -p "$oracle_dir"
export WINEPREFIX="$prefix"
export WINEARCH=win64
export WINEDEBUG=-all
export WINEDLLOVERRIDES='mscoree,mshtml='

wineboot --init
wineserver -w
windows_install=$(winepath -w "$install_dir")
windows_log=$(winepath -w "$log")

set +e
wine "$installer" \
    /VERYSILENT \
    /SUPPRESSMSGBOXES \
    /NORESTART \
    /SP- \
    "/DIR=$windows_install" \
    "/LOG=$windows_log"
wine_status=$?
set -e

wineserver -k 2>/dev/null || true

if ! grep -q 'Installation process succeeded.' "$log"; then
    printf 'Installer did not record success (Wine exit %s); inspect %s\n' "$wine_status" "$log" >&2
    exit 1
fi

for required in \
    "$install_dir/Core/LaunchBox.dll" \
    "$install_dir/Core/BigBox.dll" \
    "$install_dir/Core/Unbroken.LaunchBox.Windows.dll"; do
    if [[ ! -f "$required" ]]; then
        printf 'Required payload is missing: %s\n' "$required" >&2
        exit 1
    fi
done

printf 'LaunchBox 13.27 oracle installed successfully at %s\n' "$install_dir"
