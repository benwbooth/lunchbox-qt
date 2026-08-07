#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
fixture_xml="$repo_root/fixtures/launchbox/Data/Platforms/Fixture Console.xml"
run_log=$(mktemp "${TMPDIR:-/tmp}/lunchbox-csharp-qt.XXXXXX.log")
mono_build=$(mktemp -d "${TMPDIR:-/tmp}/lunchbox-csharp-mono.XXXXXX")
trap 'rm -f "$run_log"; rm -rf "$mono_build"' EXIT

if [[ -z "${DOTNET_ROOT:-}" ]]; then
  dotnet_path=$(readlink -f "$(command -v dotnet)")
  export DOTNET_ROOT=$(dirname "$dotnet_path")
fi
export DOTNET_ROOT_X64="${DOTNET_ROOT_X64:-$DOTNET_ROOT}"

qt_dir="${QT_BRIDGE_QT_DIR:-${QtDir:-}}"
if [[ -z "$qt_dir" ]]; then
  qt_dir=$(qmake -query QT_INSTALL_PREFIX)
fi
if [[ ! -f "$qt_dir/lib/cmake/Qt6/Qt6Config.cmake" ]]; then
  echo "QtDir must contain lib/cmake/Qt6/Qt6Config.cmake: $qt_dir" >&2
  exit 1
fi

export QT_QPA_PLATFORM="${QT_QPA_PLATFORM:-offscreen}"
export LAUNCHBOX_QT_SMOKE=1

dotnet run --no-launch-profile \
  --project "$repo_root/apps/lb-csharp-qt/LaunchBox.QtPort.csproj" \
  --configuration Release \
  -p:QtDir="$qt_dir" \
  -- --smoke "$fixture_xml" | tee "$run_log"
grep -F 'CSHARP_QT_MANAGED_SMOKE_COMPLETE games=3 changed=1' "$run_log" >/dev/null

# The model has no Qt or WPF dependency and remains independently usable from
# Mono. The official Qt Bridge UI itself currently targets .NET 8 on Linux and
# Windows, so Mono is deliberately a model-only compatibility gate here.
mcs -out:"$mono_build/ManagedSmoke.exe" \
  -r:System.Xml.Linq -r:System.Xml \
  "$repo_root/apps/lb-csharp-qt/ManagedLibrary.cs" \
  "$repo_root/apps/lb-csharp-qt/ManagedSmoke.cs"
mono "$mono_build/ManagedSmoke.exe" "$fixture_xml" | tee -a "$run_log"
grep -F 'CSHARP_QT_MANAGED_MONO_SMOKE_COMPLETE games=3 changed=1' "$run_log" >/dev/null
