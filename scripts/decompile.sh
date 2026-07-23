#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
core="$root/oracle/LaunchBox/Core"
out="$root/decompiled"

assemblies=(
    LaunchBox.dll
    BigBox.dll
    Unbroken.dll
    Unbroken.LaunchBox.dll
    Unbroken.LaunchBox.LocalDb.dll
    Unbroken.LaunchBox.Plugins.dll
    Unbroken.LaunchBox.SourceGenerators.dll
    Unbroken.LaunchBox.Windows.dll
    Unbroken.LaunchBox.Windows.BigPEmu.dll
    Unbroken.LaunchBox.Windows.Dolphin.dll
    Unbroken.LaunchBox.Windows.Mame.dll
    Unbroken.LaunchBox.Windows.Pcsx2.dll
    Unbroken.LaunchBox.Windows.PlaylistProvider.dll
    Unbroken.LaunchBox.Windows.RetroArch.dll
    Unbroken.LaunchBox.Windows.ScummVm.dll
    Unbroken.LaunchBox.Windows.Xemu.dll
)

if [[ ! -d "$core" ]]; then
    printf 'Installed Core directory not found: %s\n' "$core" >&2
    exit 1
fi

if [[ -e "$out" ]]; then
    printf 'Decompile output already exists at %s; refusing to mix runs.\n' "$out" >&2
    exit 1
fi

for assembly in "${assemblies[@]}"; do
    if [[ ! -f "$core/$assembly" ]]; then
        printf 'Required assembly not found: %s\n' "$core/$assembly" >&2
        exit 1
    fi
done

mkdir -p "$out"
export XDG_CACHE_HOME=${XDG_CACHE_HOME:-/tmp/lunchbox-port-nix-cache}
export LUNCHBOX_CORE="$core"
export LUNCHBOX_DECOMPILED="$out"
export LUNCHBOX_ASSEMBLIES
LUNCHBOX_ASSEMBLIES=$(printf '%s\n' "${assemblies[@]}")

nix shell nixpkgs#ilspycmd -c bash -c '
set -euo pipefail
while IFS= read -r assembly; do
    name=${assembly%.dll}
    printf "Decompiling %s\n" "$assembly"
    ilspycmd \
        --disable-updatecheck \
        --nested-directories \
        --project \
        --referencepath "$LUNCHBOX_CORE" \
        --outputdir "$LUNCHBOX_DECOMPILED/$name" \
        "$LUNCHBOX_CORE/$assembly"
done <<< "$LUNCHBOX_ASSEMBLIES"
'

uv run python "$root/scripts/build_static_inventory.py"
