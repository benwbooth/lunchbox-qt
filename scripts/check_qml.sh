#!/usr/bin/env bash
set -euo pipefail

workspace_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$workspace_root"

target_root=${CARGO_TARGET_DIR:-target}
qml_module_dir="$target_root/cxxqt/qml_modules"
if [[ ! -d "$qml_module_dir/LaunchBoxPort" ]]; then
  echo "Generated QML type metadata is missing; run cargo build -p lb-shell first." >&2
  exit 1
fi

import_args=(-I "$qml_module_dir" -I "$(qmake -query QT_INSTALL_QML)")
IFS=: read -ra nix_qml_paths <<< "${NIXPKGS_QT6_QML_IMPORT_PATH:-}"
for qml_path in "${nix_qml_paths[@]}"; do
  if [[ -n "$qml_path" ]]; then
    import_args+=(-I "$qml_path")
  fi
done

diagnostics=$(
  qmllint "${import_args[@]}" \
    apps/lb-shell/qml/LaunchBoxWindow.qml \
    apps/lb-shell/qml/BigBoxWindow.qml 2>&1
) || {
  printf '%s\n' "$diagnostics" >&2
  exit 1
}

if rg -q 'Failed to import|\[(missing-property|missing-type|missing-method)\]' <<< "$diagnostics"; then
  printf '%s\n' "$diagnostics" >&2
  exit 1
fi

echo "QML imports and generated controller members validated."

target_triple=${CARGO_BUILD_TARGET:-$(rustc -vV | sed -n 's/^host: //p')}
binary_dir=
for candidate in \
  "$target_root/$target_triple/release" \
  "$target_root/release" \
  "$target_root/$target_triple/debug" \
  "$target_root/debug"; do
  if [[ -x "$candidate/launchbox" && -x "$candidate/bigbox" ]]; then
    binary_dir=$candidate
    break
  fi
done

if [[ -z "$binary_dir" ]]; then
  echo "Shell binaries are missing; run cargo build -p lb-shell first." >&2
  exit 1
fi

test_config_root=$(mktemp -d)
empty_path_mappings="$test_config_root/empty-path-mappings.json"
trap 'rm -rf "$test_config_root"' EXIT

for shell in launchbox bigbox; do
  arguments=(--smoke-test --path-mappings-file "$empty_path_mappings")
  if [[ "$shell" == bigbox ]]; then
    arguments+=(--windowed)
  fi
  output=$(QT_QPA_PLATFORM=offscreen "$binary_dir/$shell" "${arguments[@]}" 2>&1) || {
    printf '%s\n' "$output" >&2
    exit 1
  }
  if ! rg -q 'MODEL_ROLE_SMOKE_COMPLETE rows=1' <<< "$output"; then
    printf '%s\n' "$output" >&2
    echo "$shell did not validate its role-based filtered model." >&2
    exit 1
  fi
done

echo "LaunchBox and BigBox role-model runtime smokes validated."

edit_root=$(mktemp -d)
crud_root=$(mktemp -d)
platform_crud_root=$(mktemp -d)
emulator_launch_root=$(mktemp -d)
direct_launch_root=$(mktemp -d)
sequence_launch_root=$(mktemp -d)
archive_launch_root=$(mktemp -d)
m3u_launch_root=$(mktemp -d)
dosbox_launch_root=$(mktemp -d)
scummvm_launch_root=$(mktemp -d)
trap 'rm -rf "$test_config_root" "$edit_root" "$crud_root" "$platform_crud_root" "$emulator_launch_root" "$direct_launch_root" "$sequence_launch_root" "$archive_launch_root" "$m3u_launch_root" "$dosbox_launch_root" "$scummvm_launch_root"' EXIT
mkdir -p "$edit_root/Data/Platforms" "$edit_root/Runtime"
edit_platform="$edit_root/Data/Platforms/Fixture Console.xml"
cp "fixtures/launchbox/Data/Platforms/Fixture Console.xml" "$edit_platform"
cp fixtures/runtime/argument-recorder.sh "$edit_root/Runtime/edited-recorder"
chmod +x "$edit_root/Runtime/edited-recorder"

edit_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$edit_root" --edit-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$edit_output" >&2
  exit 1
}
if ! rg -q 'EDIT_SMOKE_COMPLETE id=fixture-adventure title="Renamed Adventure" resets=3 data_changes=1 filtered=0' \
  <<< "$edit_output"; then
  printf '%s\n' "$edit_output" >&2
  echo "LaunchBox did not validate its transactional metadata and launch-configuration edits." >&2
  exit 1
fi
for expected in \
  '<Title>Renamed Adventure</Title>' \
  '<SortTitle>Adventure, Renamed</SortTitle>' \
  '<Notes>Edited notes from Qt.</Notes>' \
  '<Developer>Qt Forge</Developer>' \
  '<Genre>Action Adventure</Genre>' \
  '<MaxPlayers>6</MaxPlayers>' \
  '<PlayMode>Local Cooperative</PlayMode>' \
  '<Progress>75%</Progress>' \
  '<Publisher>Port Press</Publisher>' \
  '<Rating>T</Rating>' \
  '<Region>Europe</Region>' \
  '<ReleaseDate>2001-02-03</ReleaseDate>' \
  '<ReleaseType>Homebrew</ReleaseType>' \
  '<Source>Physical Media</Source>' \
  '<Status>Imported</Status>' \
  '<Version>2.0</Version>' \
  '<ApplicationPath>Runtime\edited-recorder</ApplicationPath>' \
  '<CommandLine>--edited "%gameid%" "two words"</CommandLine>' \
  '<Emulator>00000000-0000-0000-0000-000000000000</Emulator>' \
  '<UseDosBox>false</UseDosBox>' \
  '<UseScummVM>false</UseScummVM>' \
  '<ScummVMAspectCorrection>false</ScummVMAspectCorrection>' \
  '<ScummVMFullscreen>false</ScummVMFullscreen>' \
  '<Favorite>false</Favorite>' \
  '<Completed>true</Completed>' \
  '<StarRating>2</StarRating>' \
  '<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>' \
  '<Name>Adventure, Renamed Alias</Name>' \
  '<Name>Aventure Qt</Name>' \
  '<Region>France</Region>' \
  '<FutureAlternateNameElement>keep-alternate-name-data</FutureAlternateNameElement>' \
  '<Name>Cabinet Style</Name>' \
  '<Value>Cocktail</Value>' \
  '<Name>Port Status</Name>' \
  '<Value>Native Qt</Value>' \
  '<FutureCustomFieldElement>keep-custom-field-data</FutureCustomFieldElement>'; do
  if ! rg -q -F "$expected" "$edit_platform"; then
    echo "Edited platform XML is missing: $expected" >&2
    exit 1
  fi
done
for removed in \
  '<Series>' \
  '<WikipediaURL>' \
  '<CustomDosBoxVersionPath>' \
  '<DosBoxConfigurationPath>' \
  '<ScummVMGameDataFolderPath>' \
  '<ScummVMGameType>' \
  '<Name>The Fixture Adventure</Name>' \
  '<Name>Cabinet</Name>' \
  '<Value>Upright</Value>'; do
  if rg -q -F "$removed" "$edit_platform"; then
    echo "Edited platform XML retained cleared metadata: $removed" >&2
    exit 1
  fi
done

mapfile -t edit_backups < <(
  find "$edit_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#edit_backups[@]} -ne 2 ]]; then
  echo "Two successful edits did not retain exactly two transaction backups." >&2
  exit 1
fi
original_backups=0
state_backups=0
for backup in "${edit_backups[@]}"; do
  if cmp -s "$backup" "fixtures/launchbox/Data/Platforms/Fixture Console.xml"; then
    ((original_backups += 1))
  elif rg -q -F '<Title>Fixture Adventure</Title>' "$backup" \
    && rg -q -F '<Favorite>false</Favorite>' "$backup" \
    && rg -q -F '<Completed>true</Completed>' "$backup" \
    && rg -q -F '<StarRating>2</StarRating>' "$backup" \
    && rg -q -F '<Developer>Fixture Labs</Developer>' "$backup" \
    && rg -q -F '<Series>Fixture Saga</Series>' "$backup"; then
    ((state_backups += 1))
  fi
done
if [[ $original_backups -ne 1 || $state_backups -ne 1 ]]; then
  echo "Transaction backups do not form the expected original-to-state-edit chain." >&2
  exit 1
fi
if find "$edit_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful transactional edit left a recovery manifest behind." >&2
  exit 1
fi

edit_launch_log="$edit_root/edited-launch-arguments.txt"
edit_launch_output=$(
  LBPORT_LAUNCH_SMOKE_LOG="$edit_launch_log" \
    QT_QPA_PLATFORM=offscreen \
    "$binary_dir/launchbox" \
    --library "$edit_root" \
    --launch-smoke-test \
    --launch-game-id fixture-adventure \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$edit_launch_output" >&2
  exit 1
}
if ! rg -q 'LAUNCH_SMOKE_COMPLETE id=fixture-adventure launches=1' \
  <<< "$edit_launch_output"; then
  printf '%s\n' "$edit_launch_output" >&2
  echo "LaunchBox did not execute the launch configuration persisted by its editor." >&2
  exit 1
fi
if ! cmp -s "$edit_launch_log" \
  <(printf '%s\n' --edited fixture-adventure 'two words'); then
  printf 'Edited launch arguments were:\n' >&2
  sed 's/^/  /' "$edit_launch_log" >&2 || true
  exit 1
fi

echo "LaunchBox transactional metadata/launch/alias/custom-field edits, lexical Windows-path preservation, persisted Linux launch resolution, metadata search refresh, backup chain, and unknown XML preservation validated."

cp -R fixtures/launchbox/Data "$crud_root/Data"
crud_platform="$crud_root/Data/Platforms/Fixture Console.xml"
crud_playlist="$crud_root/Data/Playlists/Fixture Playlist.xml"
crud_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$crud_root" --crud-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$crud_output" >&2
  exit 1
}
if ! rg -q 'CRUD_SMOKE_COMPLETE blocked=5 inserts=1 removes=1 games=3' \
  <<< "$crud_output"; then
  printf '%s\n' "$crud_output" >&2
  echo "LaunchBox did not validate dependency-blocked delete and row CRUD." >&2
  exit 1
fi
if rg -q -F '<Title>Added Fixture</Title>' "$crud_platform"; then
  echo "CRUD smoke left its temporary added game in the platform XML." >&2
  exit 1
fi
if ! rg -q -F '<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>' \
  "$crud_platform"; then
  echo "CRUD smoke lost an unknown game element." >&2
  exit 1
fi
if ! cmp -s "$crud_playlist" \
  'fixtures/launchbox/Data/Playlists/Fixture Playlist.xml'; then
  echo "Dependency-blocked delete changed its referencing playlist." >&2
  exit 1
fi

mapfile -t crud_backups < <(
  find "$crud_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#crud_backups[@]} -ne 2 ]]; then
  echo "Successful add/remove did not retain exactly two transaction backups." >&2
  exit 1
fi
crud_original_backups=0
crud_added_backups=0
for backup in "${crud_backups[@]}"; do
  if cmp -s "$backup" 'fixtures/launchbox/Data/Platforms/Fixture Console.xml'; then
    ((crud_original_backups += 1))
  elif rg -q -F '<Title>Added Fixture</Title>' "$backup" \
    && rg -q -F '<ApplicationPath>Games\Added\added.rom</ApplicationPath>' "$backup"; then
    ((crud_added_backups += 1))
  fi
done
if [[ $crud_original_backups -ne 1 || $crud_added_backups -ne 1 ]]; then
  echo "CRUD transaction backups do not prove the expected add/remove chain." >&2
  exit 1
fi
if find "$crud_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful CRUD smoke left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox reference-gated add/remove CRUD and targeted Qt row signals validated."

cp -R fixtures/launchbox/Data "$platform_crud_root/Data"
platform_crud_catalog="$platform_crud_root/Data/Platforms.xml"
platform_crud_document="$platform_crud_root/Data/Platforms/Dragon 32_64.xml"
platform_crud_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$platform_crud_root" --platform-crud-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$platform_crud_output" >&2
  exit 1
}
if ! rg -q 'PLATFORM_CRUD_SMOKE_COMPLETE platform="Dragon 32/64" blocked=1 inserts=1 removes=1 games=3 platforms=1' \
  <<< "$platform_crud_output"; then
  printf '%s\n' "$platform_crud_output" >&2
  echo "LaunchBox did not validate dialog-driven platform creation and deletion." >&2
  exit 1
fi
if [[ -e "$platform_crud_document" ]]; then
  echo "Platform CRUD smoke retained the deleted portable platform document." >&2
  exit 1
fi
if rg -q -F 'Dragon 32/64' "$platform_crud_catalog"; then
  echo "Platform CRUD smoke retained the deleted catalog or folder records." >&2
  exit 1
fi
for media_directory in Images Videos Manuals Music; do
  if [[ -e "$platform_crud_root/$media_directory" ]]; then
    echo "Platform CRUD unexpectedly created the $media_directory media directory." >&2
    exit 1
  fi
done

mapfile -t platform_catalog_backups < <(
  find "$platform_crud_root/Data" -maxdepth 1 -type f \
    -name 'Platforms.xml.lbport-transaction-backup-*' -print
)
if [[ ${#platform_catalog_backups[@]} -ne 2 ]]; then
  echo "Platform create/delete did not retain exactly two catalog backups." >&2
  exit 1
fi
platform_original_catalog_backups=0
platform_created_catalog_backups=0
for backup in "${platform_catalog_backups[@]}"; do
  if cmp -s "$backup" fixtures/launchbox/Data/Platforms.xml; then
    ((platform_original_catalog_backups += 1))
  elif rg -q -F '<Name>Dragon 32/64</Name>' "$backup" \
    && rg -q -F '<ScrapeAs>Dragon 32/64</ScrapeAs>' "$backup" \
    && [[ $(rg -c -F '<Platform>Dragon 32/64</Platform>' "$backup") -eq 51 ]] \
    && rg -q -F '<FolderPath>Images\Dragon 32_64\Box - Front</FolderPath>' "$backup"; then
    ((platform_created_catalog_backups += 1))
  fi
done
if [[ $platform_original_catalog_backups -ne 1 \
  || $platform_created_catalog_backups -ne 1 ]]; then
  echo "Platform catalog backups do not prove the expected portable create/delete chain." >&2
  exit 1
fi

mapfile -t platform_document_backups < <(
  find "$platform_crud_root/Data/Platforms" -maxdepth 1 -type f \
    -name 'Dragon 32_64.xml.lbport-transaction-backup-*' -print
)
if [[ ${#platform_document_backups[@]} -ne 3 ]]; then
  echo "Platform add-game/remove-game/delete did not retain exactly three document backups." >&2
  exit 1
fi
platform_game_backups=0
platform_empty_backups=0
for backup in "${platform_document_backups[@]}"; do
  if rg -q -F '<Title>Dragon Test</Title>' "$backup" \
    && rg -q -F '<Platform>Dragon 32/64</Platform>' "$backup" \
    && rg -q -F '<ApplicationPath>Games\Dragon 32_64\test.vdk</ApplicationPath>' "$backup"; then
    ((platform_game_backups += 1))
  elif ! rg -q -F '<Game>' "$backup"; then
    ((platform_empty_backups += 1))
  fi
done
if [[ $platform_game_backups -ne 1 || $platform_empty_backups -ne 2 ]]; then
  echo "Platform document backups do not prove empty/add/remove/delete transitions." >&2
  exit 1
fi
if find "$platform_crud_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful platform CRUD smoke left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox dialog-driven platform lifecycle, portable filenames, lexical Windows paths, reference gating, exact backups, and media isolation validated."

cp -R fixtures/launchbox/Data "$emulator_launch_root/Data"
mkdir -p "$emulator_launch_root/Emulators"
cp fixtures/runtime/argument-recorder.sh \
  "$emulator_launch_root/Emulators/fixture-emulator"
chmod +x "$emulator_launch_root/Emulators/fixture-emulator"

cp -R fixtures/launchbox-direct/Data "$direct_launch_root/Data"
mkdir -p "$direct_launch_root/LaunchTargets"
cp fixtures/runtime/argument-recorder.sh \
  "$direct_launch_root/LaunchTargets/argument-recorder"
chmod +x "$direct_launch_root/LaunchTargets/argument-recorder"

cp -R fixtures/launchbox-archive/Data "$archive_launch_root/Data"
mkdir -p \
  "$archive_launch_root/Emulators" \
  "$archive_launch_root/Games/Archive Fixture" \
  "$archive_launch_root/Runtime" \
  "$archive_launch_root/archive-source"
cp fixtures/runtime/archive-recorder.sh \
  "$archive_launch_root/Emulators/archive-recorder"
cp fixtures/runtime/noop.sh "$archive_launch_root/Runtime/archive-after"
cp fixtures/runtime/argument-recorder.sh \
  "$archive_launch_root/archive-source/Archive Racer.rom"
chmod +x \
  "$archive_launch_root/Emulators/archive-recorder" \
  "$archive_launch_root/Runtime/archive-after"
(
  cd "$archive_launch_root/archive-source"
  7z a -tzip \
    "$archive_launch_root/Games/Archive Fixture/Archive Racer.zip" \
    "Archive Racer.rom" >/dev/null
)

cp -R fixtures/launchbox-dosbox/Data "$dosbox_launch_root/Data"
mkdir -p \
  "$dosbox_launch_root/Runtime" \
  "$dosbox_launch_root/Config" \
  "$dosbox_launch_root/Games/DOS Fixture/BIN" \
  "$dosbox_launch_root/Media/CD Files" \
  "$dosbox_launch_root/Media"
cp fixtures/runtime/dosbox-recorder.sh \
  "$dosbox_launch_root/Runtime/dosbox-recorder"
chmod +x "$dosbox_launch_root/Runtime/dosbox-recorder"
touch \
  "$dosbox_launch_root/Config/dosbox.conf" \
  "$dosbox_launch_root/Games/DOS Fixture/BIN/PLAY.BAT" \
  "$dosbox_launch_root/Media/Disk One.img" \
  "$dosbox_launch_root/Media/Game.iso"

cp -R fixtures/launchbox-scummvm/Data "$scummvm_launch_root/Data"
mkdir -p \
  "$scummvm_launch_root/Runtime" \
  "$scummvm_launch_root/Games/Monkey Island 2"
cp fixtures/runtime/scummvm-recorder.sh \
  "$scummvm_launch_root/Runtime/scummvm"
chmod +x "$scummvm_launch_root/Runtime/scummvm"

cp -R fixtures/launchbox-m3u/Data "$m3u_launch_root/Data"
mkdir -p \
  "$m3u_launch_root/Emulators" \
  "$m3u_launch_root/Games/M3U Fixture" \
  "$m3u_launch_root/Runtime" \
  "$m3u_launch_root/m3u-source"
cp fixtures/runtime/m3u-recorder.sh \
  "$m3u_launch_root/Emulators/m3u-recorder"
cp fixtures/runtime/noop.sh "$m3u_launch_root/Runtime/m3u-after"
cp fixtures/runtime/noop.sh \
  "$m3u_launch_root/Games/M3U Fixture/Multi Disc Racer (Disc 1).chd"
cp fixtures/runtime/noop.sh \
  "$m3u_launch_root/Games/M3U Fixture/Multi Disc Racer (Disc 3).chd"
cp fixtures/runtime/noop.sh \
  "$m3u_launch_root/m3u-source/Multi Disc Racer (Disc 2).chd"
chmod +x \
  "$m3u_launch_root/Emulators/m3u-recorder" \
  "$m3u_launch_root/Runtime/m3u-after"
(
  cd "$m3u_launch_root/m3u-source"
  7z a -tzip \
    "$m3u_launch_root/Games/M3U Fixture/Multi Disc Racer (Disc 2).zip" \
    "Multi Disc Racer (Disc 2).chd" >/dev/null
)

run_launch_smoke() {
  local shell_name=$1
  local launch_root=$2
  local game_id=$3
  local launch_log=$4
  shift 4
  local -a arguments=(
    --library "$launch_root"
    --launch-smoke-test
    --launch-game-id "$game_id"
    --path-mappings-file "$empty_path_mappings"
  )
  if [[ "$shell_name" == bigbox ]]; then
    arguments+=(--windowed)
  fi
  if [[ "$game_id" == fixture-direct ]]; then
    arguments+=(--map-windows-drive "Z=$launch_root")
  fi
  rm -f "$launch_log"
  local output
  output=$(
    LBPORT_LAUNCH_SMOKE_LOG="$launch_log" \
      QT_QPA_PLATFORM=offscreen \
      "$binary_dir/$shell_name" "${arguments[@]}" 2>&1
  ) || {
    printf '%s\n' "$output" >&2
    exit 1
  }
  if ! rg -q "LAUNCH_SMOKE_COMPLETE id=$game_id launches=1" <<< "$output"; then
    printf '%s\n' "$output" >&2
    echo "$shell_name did not report a successful process spawn for $game_id." >&2
    exit 1
  fi
  local attempt
  for ((attempt = 0; attempt < 200; ++attempt)); do
    if [[ -f "$launch_log" ]]; then
      break
    fi
    sleep 0.01
  done
  if [[ ! -f "$launch_log" ]]; then
    printf '%s\n' "$output" >&2
    echo "$shell_name spawned $game_id but its argument recorder did not run." >&2
    exit 1
  fi
  local -a actual_arguments
  mapfile -t actual_arguments < "$launch_log"
  if [[ ${#actual_arguments[@]} -ne $# ]]; then
    printf 'Expected %d arguments but captured %d for %s/%s:\n' \
      "$#" "${#actual_arguments[@]}" "$shell_name" "$game_id" >&2
    printf '  %s\n' "${actual_arguments[@]}" >&2
    exit 1
  fi
  local index=0
  local expected
  for expected in "$@"; do
    if [[ "${actual_arguments[$index]}" != "$expected" ]]; then
      printf 'Argument %d for %s/%s was <%s>, expected <%s>.\n' \
        "$index" "$shell_name" "$game_id" \
        "${actual_arguments[$index]}" "$expected" >&2
      exit 1
    fi
    ((index += 1))
  done
}

emulator_log="$emulator_launch_root/emulator-arguments.txt"
run_launch_smoke launchbox "$emulator_launch_root" fixture-racer "$emulator_log" \
  --platform fixture \
  "$emulator_launch_root/Games/Fixture Racer/racer.rom"
run_launch_smoke bigbox "$emulator_launch_root" fixture-racer "$emulator_log" \
  --platform fixture \
  "$emulator_launch_root/Games/Fixture Racer/racer.rom"

direct_log="$direct_launch_root/direct-arguments.txt"
run_launch_smoke launchbox "$direct_launch_root" fixture-direct "$direct_log" \
  --direct "two words"

run_archive_launch_smoke() {
  local shell_name=$1
  local argument_log="$archive_launch_root/archive-$shell_name-arguments.txt"
  local lifetime_log="$archive_launch_root/archive-$shell_name-lifetime.txt"
  local -a arguments=(
    --library "$archive_launch_root"
    --launch-smoke-test
    --launch-game-id fixture-archive
    --path-mappings-file "$empty_path_mappings"
  )
  if [[ "$shell_name" == bigbox ]]; then
    arguments+=(--windowed)
  fi
  rm -f "$argument_log" "$lifetime_log"
  local output
  output=$(
    LBPORT_ARCHIVE_ARGUMENT_LOG="$argument_log" \
      LBPORT_ARCHIVE_LIFETIME_LOG="$lifetime_log" \
      QT_QPA_PLATFORM=offscreen \
      "$binary_dir/$shell_name" "${arguments[@]}" 2>&1
  ) || {
    printf '%s\n' "$output" >&2
    exit 1
  }
  if ! rg -q 'LAUNCH_SMOKE_COMPLETE id=fixture-archive launches=1' \
    <<< "$output"; then
    printf '%s\n' "$output" >&2
    echo "$shell_name did not launch the extracted archive fixture." >&2
    exit 1
  fi
  if [[ ! -f "$argument_log" || ! -f "$lifetime_log" ]]; then
    printf '%s\n' "$output" >&2
    echo "$shell_name did not execute the archive lifecycle recorder." >&2
    exit 1
  fi
  if ! cmp -s "$lifetime_log" <(printf '%s\n' alive-until-exit); then
    printf 'Archive lifecycle result for %s was:\n' "$shell_name" >&2
    sed 's/^/  /' "$lifetime_log" >&2 || true
    exit 1
  fi

  local -a actual_arguments
  mapfile -t actual_arguments < "$argument_log"
  if [[ ${#actual_arguments[@]} -ne 4 \
    || "${actual_arguments[0]}" != --rom \
    || "${actual_arguments[2]}" != --dir ]]; then
    printf 'Archive arguments for %s were:\n' "$shell_name" >&2
    printf '  %s\n' "${actual_arguments[@]}" >&2
    exit 1
  fi
  local extracted_rom=${actual_arguments[1]}
  local extracted_directory=${actual_arguments[3]}
  if [[ $(basename "$extracted_rom") != 'Archive Racer.rom' \
    || $(basename "$extracted_directory") != 'Archive Racer' \
    || $(dirname "$extracted_rom") != "$extracted_directory" ]]; then
    printf 'Archive extraction did not preserve its archive folder and ROM name: %s\n' \
      "$extracted_rom" >&2
    exit 1
  fi
  if [[ "$extracted_rom" == "$archive_launch_root"/* ]]; then
    echo "Archive extraction wrote temporary data into the library." >&2
    exit 1
  fi
  for ((attempt = 0; attempt < 300; ++attempt)); do
    if [[ ! -e "$extracted_rom" && ! -e "$extracted_directory" ]]; then
      break
    fi
    sleep 0.01
  done
  if [[ -e "$extracted_rom" || -e "$extracted_directory" ]]; then
    echo "$shell_name retained its extracted ROM after the emulator exited." >&2
    exit 1
  fi
}

run_archive_launch_smoke launchbox
run_archive_launch_smoke bigbox

run_m3u_launch_smoke() {
  local shell_name=$1
  local argument_log="$m3u_launch_root/m3u-$shell_name-arguments.txt"
  local content_log="$m3u_launch_root/m3u-$shell_name-content.txt"
  local lifetime_log="$m3u_launch_root/m3u-$shell_name-lifetime.txt"
  local -a arguments=(
    --library "$m3u_launch_root"
    --launch-smoke-test
    --launch-game-id fixture-m3u
    --path-mappings-file "$empty_path_mappings"
  )
  if [[ "$shell_name" == bigbox ]]; then
    arguments+=(--windowed)
  fi
  rm -f "$argument_log" "$content_log" "$lifetime_log"
  local output
  output=$(
    LBPORT_M3U_ARGUMENT_LOG="$argument_log" \
      LBPORT_M3U_CONTENT_LOG="$content_log" \
      LBPORT_M3U_LIFETIME_LOG="$lifetime_log" \
      QT_QPA_PLATFORM=offscreen \
      "$binary_dir/$shell_name" "${arguments[@]}" 2>&1
  ) || {
    printf '%s\n' "$output" >&2
    exit 1
  }
  if ! rg -q 'LAUNCH_SMOKE_COMPLETE id=fixture-m3u launches=1' \
    <<< "$output"; then
    printf '%s\n' "$output" >&2
    echo "$shell_name did not launch the generated M3U fixture." >&2
    exit 1
  fi
  if [[ ! -f "$argument_log" || ! -f "$content_log" \
    || ! -f "$lifetime_log" ]]; then
    printf '%s\n' "$output" >&2
    echo "$shell_name did not execute the M3U lifecycle recorder." >&2
    exit 1
  fi
  if ! cmp -s "$lifetime_log" <(printf '%s\n' alive-until-exit); then
    printf 'M3U lifecycle result for %s was:\n' "$shell_name" >&2
    sed 's/^/  /' "$lifetime_log" >&2 || true
    exit 1
  fi

  local -a actual_arguments
  mapfile -t actual_arguments < "$argument_log"
  if [[ ${#actual_arguments[@]} -ne 4 \
    || "${actual_arguments[0]}" != --playlist \
    || "${actual_arguments[2]}" != --dir ]]; then
    printf 'M3U arguments for %s were:\n' "$shell_name" >&2
    printf '  %s\n' "${actual_arguments[@]}" >&2
    exit 1
  fi
  local playlist_path=${actual_arguments[1]}
  local playlist_directory=${actual_arguments[3]}
  if [[ $(basename "$playlist_path") != 'Multi Disc Racer (Disc 1).m3u' \
    || $(dirname "$playlist_path") != "$playlist_directory" \
    || "$playlist_path" == "$m3u_launch_root"/* ]]; then
    printf 'Generated playlist location for %s was: %s\n' \
      "$shell_name" "$playlist_path" >&2
    exit 1
  fi

  local -a disc_paths
  mapfile -t disc_paths < "$content_log"
  if [[ ${#disc_paths[@]} -ne 3 \
    || $(basename "${disc_paths[0]}") != 'Multi Disc Racer (Disc 1).chd' \
    || $(basename "${disc_paths[1]}") != 'Multi Disc Racer (Disc 2).chd' \
    || $(basename "${disc_paths[2]}") != 'Multi Disc Racer (Disc 3).chd' \
    || "${disc_paths[0]}" != "$m3u_launch_root/Games/M3U Fixture/Multi Disc Racer (Disc 1).chd" \
    || "${disc_paths[2]}" != "$m3u_launch_root/Games/M3U Fixture/Multi Disc Racer (Disc 3).chd" \
    || "${disc_paths[1]}" == "$m3u_launch_root"/* \
    || $(basename "$(dirname "${disc_paths[1]}")") != 'Multi Disc Racer (Disc 2)' ]]; then
    printf 'M3U content for %s was:\n' "$shell_name" >&2
    printf '  %s\n' "${disc_paths[@]}" >&2
    exit 1
  fi

  for ((attempt = 0; attempt < 300; ++attempt)); do
    if [[ ! -e "$playlist_path" \
      && ! -e "$playlist_directory" \
      && ! -e "${disc_paths[1]}" \
      && ! -e "$(dirname "${disc_paths[1]}")" ]]; then
      break
    fi
    sleep 0.01
  done
  if [[ -e "$playlist_path" || -e "$playlist_directory" \
    || -e "${disc_paths[1]}" || -e "$(dirname "${disc_paths[1]}")" ]]; then
    echo "$shell_name retained its M3U or extracted disc after emulator exit." >&2
    exit 1
  fi
}

run_m3u_launch_smoke launchbox
run_m3u_launch_smoke bigbox

run_dosbox_launch_smoke() {
  local shell_name=$1
  local argument_log="$dosbox_launch_root/dosbox-$shell_name-arguments.txt"
  local -a arguments=(
    --library "$dosbox_launch_root"
    --launch-smoke-test
    --launch-game-id fixture-dosbox
    --path-mappings-file "$empty_path_mappings"
  )
  if [[ "$shell_name" == bigbox ]]; then
    arguments+=(--windowed)
  fi
  rm -f "$argument_log"
  local output
  output=$(
    LBPORT_DOSBOX_ARGUMENT_LOG="$argument_log" \
      QT_QPA_PLATFORM=offscreen \
      "$binary_dir/$shell_name" "${arguments[@]}" 2>&1
  ) || {
    printf '%s\n' "$output" >&2
    exit 1
  }
  if ! rg -q 'LAUNCH_SMOKE_COMPLETE id=fixture-dosbox launches=1' \
    <<< "$output"; then
    printf '%s\n' "$output" >&2
    echo "$shell_name did not launch the DOSBox mount fixture." >&2
    exit 1
  fi
  for ((attempt = 0; attempt < 200; ++attempt)); do
    if [[ -f "$argument_log" ]]; then
      break
    fi
    sleep 0.01
  done
  local expected_log="$dosbox_launch_root/dosbox-expected-arguments.txt"
  printf '%s\n' \
    -noconsole \
    -conf \
    "$dosbox_launch_root/Config/dosbox.conf" \
    -noautoexec \
    -c \
    '@ECHO OFF' \
    -c \
    CLS \
    -c \
    "MOUNT C \"$dosbox_launch_root/Games/DOS Fixture\"" \
    -c \
    "MOUNT D \"$dosbox_launch_root/Media/CD Files\" -t cdrom -fs iso" \
    -c \
    "IMGMOUNT A \"$dosbox_launch_root/Media/Disk One.img\" -t floppy -fs fat" \
    -c \
    "IMGMOUNT E \"$dosbox_launch_root/Media/Game.iso\" -t iso -fs iso" \
    -c \
    'C:' \
    -c \
    'CD "BIN"' \
    -c \
    'CALL "PLAY.BAT" -fast' \
    -c \
    EXIT > "$expected_log"
  if ! cmp -s "$argument_log" "$expected_log"; then
    printf 'DOSBox arguments for %s were:\n' "$shell_name" >&2
    sed 's/^/  /' "$argument_log" >&2 || true
    diff -u "$expected_log" "$argument_log" >&2 || true
    exit 1
  fi
}

run_dosbox_launch_smoke launchbox
run_dosbox_launch_smoke bigbox

run_scummvm_launch_smoke() {
  local shell_name=$1
  local argument_log="$scummvm_launch_root/scummvm-$shell_name-arguments.txt"
  local -a arguments=(
    --library "$scummvm_launch_root"
    --launch-smoke-test
    --launch-game-id fixture-scummvm
    --path-mappings-file "$empty_path_mappings"
  )
  if [[ "$shell_name" == bigbox ]]; then
    arguments+=(--windowed)
  fi
  rm -f "$argument_log"
  local output
  output=$(
    PATH="$scummvm_launch_root/Runtime:$PATH" \
      LBPORT_SCUMMVM_ARGUMENT_LOG="$argument_log" \
      QT_QPA_PLATFORM=offscreen \
      "$binary_dir/$shell_name" "${arguments[@]}" 2>&1
  ) || {
    printf '%s\n' "$output" >&2
    exit 1
  }
  if ! rg -q 'LAUNCH_SMOKE_COMPLETE id=fixture-scummvm launches=1' \
    <<< "$output"; then
    printf '%s\n' "$output" >&2
    echo "$shell_name did not launch the legacy ScummVM fixture." >&2
    exit 1
  fi
  for ((attempt = 0; attempt < 200; ++attempt)); do
    if [[ -f "$argument_log" ]]; then
      break
    fi
    sleep 0.01
  done
  local data_folder="$scummvm_launch_root/Games/Monkey Island 2"
  local expected_log="$scummvm_launch_root/scummvm-expected-arguments.txt"
  printf '%s\n' \
    --no-console \
    "--savepath=$data_folder" \
    "--extrapath=$data_folder" \
    -p \
    "$data_folder" \
    -f \
    --aspect-ratio \
    monkey2 > "$expected_log"
  if ! cmp -s "$argument_log" "$expected_log"; then
    printf 'ScummVM arguments for %s were:\n' "$shell_name" >&2
    sed 's/^/  /' "$argument_log" >&2 || true
    diff -u "$expected_log" "$argument_log" >&2 || true
    exit 1
  fi
}

run_scummvm_launch_smoke launchbox
run_scummvm_launch_smoke bigbox

persisted_path_mappings="$test_config_root/persisted-path-mappings.json"
path_mapping_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --path-mapping-smoke-test \
    --path-mapping-host-root "$direct_launch_root" \
    --path-mappings-file "$persisted_path_mappings" 2>&1
) || {
  printf '%s\n' "$path_mapping_output" >&2
  exit 1
}
if ! rg -q 'PATH_MAPPING_SMOKE_COMPLETE mappings=1 settings=' \
  <<< "$path_mapping_output"; then
  printf '%s\n' "$path_mapping_output" >&2
  echo "LaunchBox did not complete persisted path-mapping CRUD." >&2
  exit 1
fi
expected_path_mappings="$test_config_root/expected-path-mappings.json"
sed "s|@HOST_ROOT@|$direct_launch_root|g" > "$expected_path_mappings" <<'EOF'
{
  "version": 1,
  "windows_drives": [
    {
      "drive": "Z",
      "host_root": "@HOST_ROOT@"
    }
  ],
  "windows_unc": []
}
EOF
if ! cmp -s "$persisted_path_mappings" "$expected_path_mappings"; then
  echo "Persisted host path mappings do not match the canonical versioned document." >&2
  diff -u "$expected_path_mappings" "$persisted_path_mappings" >&2 || true
  exit 1
fi

rm -f "$direct_log"
persisted_launch_output=$(
  LBPORT_LAUNCH_SMOKE_LOG="$direct_log" \
    QT_QPA_PLATFORM=offscreen \
    "$binary_dir/bigbox" \
    --windowed \
    --library "$direct_launch_root" \
    --launch-smoke-test \
    --launch-game-id fixture-direct \
    --path-mappings-file "$persisted_path_mappings" 2>&1
) || {
  printf '%s\n' "$persisted_launch_output" >&2
  exit 1
}
if ! rg -q 'LAUNCH_SMOKE_COMPLETE id=fixture-direct launches=1' \
  <<< "$persisted_launch_output"; then
  printf '%s\n' "$persisted_launch_output" >&2
  echo "BigBox did not use the persisted Windows drive mapping after restart." >&2
  exit 1
fi
for ((attempt = 0; attempt < 200; ++attempt)); do
  if [[ -f "$direct_log" ]]; then
    break
  fi
  sleep 0.01
done
if ! cmp -s "$direct_log" <(printf '%s\n' --direct 'two words'); then
  printf 'Persisted-mapping launch arguments were:\n' >&2
  sed 's/^/  /' "$direct_log" >&2 || true
  exit 1
fi

cp -R fixtures/launchbox-sequence/Data "$sequence_launch_root/Data"
mkdir -p "$sequence_launch_root/Runtime"
cp fixtures/runtime/sequence-recorder.sh \
  "$sequence_launch_root/Runtime/sequence-recorder"
chmod +x "$sequence_launch_root/Runtime/sequence-recorder"

sequence_log="$sequence_launch_root/sequence-order.txt"
sequence_output=$(
  LBPORT_SEQUENCE_SMOKE_LOG="$sequence_log" \
    QT_QPA_PLATFORM=offscreen \
    "$binary_dir/launchbox" \
    --library "$sequence_launch_root" \
    --launch-smoke-test \
    --launch-game-id fixture-sequence \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$sequence_output" >&2
  exit 1
}
if ! rg -q 'LAUNCH_SMOKE_COMPLETE id=fixture-sequence launches=1' \
  <<< "$sequence_output"; then
  printf '%s\n' "$sequence_output" >&2
  echo "LaunchBox did not complete the automatic additional-app lifecycle." >&2
  exit 1
fi
for ((attempt = 0; attempt < 200; ++attempt)); do
  if [[ -f "$sequence_log" ]] \
    && [[ $(wc -l < "$sequence_log") -eq 3 ]]; then
    break
  fi
  sleep 0.01
done
if ! cmp -s "$sequence_log" <(printf '%s\n' before main after); then
  printf 'Automatic launch order was:\n' >&2
  sed 's/^/  /' "$sequence_log" >&2 || true
  exit 1
fi

rm -f "$sequence_log"
manual_output=$(
  LBPORT_SEQUENCE_SMOKE_LOG="$sequence_log" \
    QT_QPA_PLATFORM=offscreen \
    "$binary_dir/bigbox" \
    --windowed \
    --library "$sequence_launch_root" \
    --launch-smoke-test \
    --launch-game-id fixture-sequence \
    --launch-additional-application-id fixture-manual \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$manual_output" >&2
  exit 1
}
if ! rg -q \
  'ADDITIONAL_APP_LAUNCH_SMOKE_COMPLETE game=fixture-sequence application=fixture-manual launches=1' \
  <<< "$manual_output"; then
  printf '%s\n' "$manual_output" >&2
  echo "BigBox did not complete its selected additional-app launch." >&2
  exit 1
fi
for ((attempt = 0; attempt < 200; ++attempt)); do
  if [[ -f "$sequence_log" ]]; then
    break
  fi
  sleep 0.01
done
if ! cmp -s "$sequence_log" <(printf '%s\n' manual); then
  printf 'Selected additional-app launch recorded:\n' >&2
  sed 's/^/  /' "$sequence_log" >&2 || true
  exit 1
fi

xml_record_field() {
  local file=$1
  local record_name=$2
  local id_field=$3
  local id=$4
  local field=$5
  awk \
    -v record_name="$record_name" \
    -v id_field="$id_field" \
    -v id="$id" \
    -v field="$field" '
      $0 ~ "<" record_name ">" {
        in_record = 1
        buffer = $0 "\n"
        next
      }
      in_record {
        buffer = buffer $0 "\n"
      }
      in_record && $0 ~ "</" record_name ">" {
        id_element = "<" id_field ">" id "</" id_field ">"
        if (index(buffer, id_element)) {
          field_start = "<" field ">"
          field_end = "</" field ">"
          start = index(buffer, field_start)
          if (start) {
            value = substr(buffer, start + length(field_start))
            finish = index(value, field_end)
            if (finish) {
              print substr(value, 1, finish - 1)
              exit
            }
          }
        }
        in_record = 0
        buffer = ""
      }
    ' "$file"
}

assert_play_stats() {
  local file=$1
  local record_name=$2
  local id_field=$3
  local id=$4
  local expected_count=$5
  local minimum_time=$6
  local last_played_field=$7
  local play_count
  local play_time
  local last_played
  play_count=$(xml_record_field "$file" "$record_name" "$id_field" "$id" PlayCount)
  play_time=$(xml_record_field "$file" "$record_name" "$id_field" "$id" PlayTime)
  last_played=$(
    xml_record_field \
      "$file" "$record_name" "$id_field" "$id" "$last_played_field"
  )
  if [[ -z "$play_time" && "$minimum_time" == 0 ]]; then
    play_time=0
  fi
  if [[ "$play_count" != "$expected_count" ]]; then
    echo "$record_name $id has PlayCount <$play_count>, expected <$expected_count>." >&2
    exit 1
  fi
  if [[ ! "$play_time" =~ ^[0-9]+$ ]] \
    || ((play_time < minimum_time)); then
    echo "$record_name $id has PlayTime <$play_time>, expected at least <$minimum_time>." >&2
    exit 1
  fi
  if [[ ! "$last_played" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{7}[+-][0-9]{2}:[0-9]{2}$ ]]; then
    echo "$record_name $id has a non-LaunchBox LastPlayed value: <$last_played>." >&2
    exit 1
  fi
}

assert_play_stats \
  "$emulator_launch_root/Data/Platforms/Fixture Console.xml" \
  Game ID fixture-racer 10 14402 LastPlayedDate
assert_play_stats \
  "$direct_launch_root/Data/Platforms/Direct Fixture.xml" \
  Game ID fixture-direct 2 2 LastPlayedDate
assert_play_stats \
  "$sequence_launch_root/Data/Platforms/Sequence Fixture.xml" \
  Game ID fixture-sequence 1 0 LastPlayedDate
assert_play_stats \
  "$sequence_launch_root/Data/Platforms/Sequence Fixture.xml" \
  AdditionalApplication Id fixture-manual 1 0 LastPlayed
assert_play_stats \
  "$archive_launch_root/Data/Platforms/Archive Fixture.xml" \
  Game ID fixture-archive 2 0 LastPlayedDate
assert_play_stats \
  "$m3u_launch_root/Data/Platforms/M3U Fixture.xml" \
  Game ID fixture-m3u 2 0 LastPlayedDate
assert_play_stats \
  "$dosbox_launch_root/Data/Platforms/DOSBox Fixture.xml" \
  Game ID fixture-dosbox 2 0 LastPlayedDate
assert_play_stats \
  "$scummvm_launch_root/Data/Platforms/ScummVM Fixture.xml" \
  Game ID fixture-scummvm 2 0 LastPlayedDate

if ! rg -q -F '<FutureRootElement>preserve-me</FutureRootElement>' \
  "$emulator_launch_root/Data/Platforms/Fixture Console.xml"; then
  echo "Play-statistics writes lost an unknown platform element." >&2
  exit 1
fi

for launch_root in \
  "$emulator_launch_root" \
  "$direct_launch_root" \
  "$sequence_launch_root" \
  "$archive_launch_root" \
  "$m3u_launch_root" \
  "$dosbox_launch_root" \
  "$scummvm_launch_root"; do
  if ! find "$launch_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print -quit | rg -q .; then
    echo "A successful launch did not retain a recoverable statistics backup in $launch_root." >&2
    exit 1
  fi
  if find "$launch_root" -type f \
    -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
    echo "A successful launch left a recovery manifest behind in $launch_root." >&2
    exit 1
  fi
done

echo "Persisted host mappings, direct/emulator/archive/M3U/DOSBox/ScummVM argv, folder/image mounts, leased resource cleanup, launch ordering, selected additional apps, and transactional play statistics validated."
