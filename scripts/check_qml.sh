#!/usr/bin/env bash
set -euo pipefail

workspace_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$workspace_root"

nix_build_sandbox=false
if [[ -n ${NIX_BUILD_TOP:-} && -z ${IN_NIX_SHELL:-} ]]; then
  nix_build_sandbox=true
fi

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
    apps/lb-shell/qml/BigBoxWindow.qml \
    apps/lb-shell/qml/GameImageViewer.qml \
    apps/lb-shell/qml/BoxArtView.qml \
    apps/lb-shell/qml/BoxModelViewer.qml \
    apps/lb-shell/qml/GameMusicPlayer.qml \
    apps/lb-shell/qml/BackgroundMusicPlayer.qml \
    apps/lb-shell/qml/BigBoxStartupPresentation.qml \
    apps/lb-shell/qml/BigBoxAttractMode.qml \
    apps/lb-shell/qml/BigBoxScreensaver.qml \
    apps/lb-shell/qml/BigBoxInputRouter.qml \
    apps/lb-shell/qml/BigBoxInputSettings.qml \
    apps/lb-shell/qml/BigBoxMarqueeSettings.qml \
    apps/lb-shell/qml/BigBoxPinPopup.qml \
    apps/lb-shell/qml/BigBoxPlaylistPopup.qml \
    apps/lb-shell/qml/BigBoxRelatedGamesPopup.qml \
    apps/lb-shell/qml/BigBoxDiscoveryPage.qml \
    apps/lb-shell/qml/BigBoxStarRatingPopup.qml \
    apps/lb-shell/qml/BigBoxSecuritySettings.qml \
    apps/lb-shell/qml/BigBoxMarqueeWindow.qml \
    apps/lb-shell/qml/LaunchStartupOverlay.qml \
    apps/lb-shell/qml/LaunchShutdownOverlay.qml \
    apps/lb-shell/qml/LaunchPauseOverlay.qml \
    apps/lb-shell/qml/LaunchBoxSystemTray.qml 2>&1
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
    if [[ -z "$binary_dir" ]] \
      || { [[ "$candidate/launchbox" -nt "$binary_dir/launchbox" ]] \
        && [[ "$candidate/bigbox" -nt "$binary_dir/bigbox" ]]; }; then
      binary_dir=$candidate
    fi
  fi
done

if [[ -z "$binary_dir" ]]; then
  echo "Shell binaries are missing; run cargo build -p lb-shell first." >&2
  exit 1
fi
process_fixture="$binary_dir/lb-process-fixture"
if [[ ! -x "$process_fixture" ]]; then
  echo "Portable process fixture is missing; run cargo build --workspace first." >&2
  exit 1
fi
stale_source=$(
  find apps/lb-shell crates tools -type f \
    \( -name '*.rs' -o -name '*.qml' -o -name '*.svg' \
      -o -name Cargo.toml -o -name build.rs \) \
    -newer "$binary_dir/launchbox" -print -quit
)
if [[ -n "$stale_source" || Cargo.toml -nt "$binary_dir/launchbox" \
  || Cargo.lock -nt "$binary_dir/launchbox" ]]; then
  echo "Shell binaries are older than the checked source; run cargo build -p lb-shell first." >&2
  exit 1
fi

install_process_fixture() {
  local destination=$1
  cp "$process_fixture" "$destination"
  chmod +x "$destination"
}

run_rendered_smoke() {
  if [[ $(uname -s) == Linux ]]; then
    local runtime_root="$test_config_root/model-viewer-runtime"
    mkdir -p \
      "$runtime_root/home" \
      "$runtime_root/cache" \
      "$runtime_root/config" \
      "$runtime_root/runtime"
    chmod 700 "$runtime_root/runtime"
    if "$nix_build_sandbox"; then
      env \
        HOME="$runtime_root/home" \
        XDG_CACHE_HOME="$runtime_root/cache" \
        XDG_CONFIG_HOME="$runtime_root/config" \
        XDG_RUNTIME_DIR="$runtime_root/runtime" \
        QT_QPA_PLATFORM=offscreen \
        "$@"
      return
    fi
    xvfb-run -a -s '-screen 0 1280x800x24' \
      env \
      HOME="$runtime_root/home" \
      XDG_CACHE_HOME="$runtime_root/cache" \
      XDG_CONFIG_HOME="$runtime_root/config" \
      XDG_RUNTIME_DIR="$runtime_root/runtime" \
      LIBGL_ALWAYS_SOFTWARE=1 \
      QT_QPA_PLATFORM=xcb \
      "$@"
  else
    QT_QPA_PLATFORM=offscreen "$@"
  fi
}

run_software_rendered_smoke() {
  if [[ $(uname -s) == Linux ]]; then
    local runtime_root="$test_config_root/software-rendered-runtime"
    mkdir -p \
      "$runtime_root/home" \
      "$runtime_root/cache" \
      "$runtime_root/config" \
      "$runtime_root/runtime"
    chmod 700 "$runtime_root/runtime"
    xvfb-run -a -s '-screen 0 1280x800x24' \
      env \
      HOME="$runtime_root/home" \
      XDG_CACHE_HOME="$runtime_root/cache" \
      XDG_CONFIG_HOME="$runtime_root/config" \
      XDG_RUNTIME_DIR="$runtime_root/runtime" \
      LIBGL_ALWAYS_SOFTWARE=1 \
      QT_QUICK_BACKEND=software \
      QT_QPA_PLATFORM=xcb \
      "$@"
  else
    QT_QUICK_BACKEND=software QT_QPA_PLATFORM=offscreen "$@"
  fi
}

validate_rendered_model_viewport() {
  local screenshot=$1
  if [[ $(uname -s) != Linux ]]; then
    return
  fi

  local unique_colors
  unique_colors=$(
    magick "$screenshot" -crop 1230x480+25+110 +repage \
      -format '%k' info:
  )
  if [[ ! "$unique_colors" =~ ^[0-9]+$ ]] \
    || ((unique_colors < 64)); then
    echo "3D model viewport is blank or insufficiently rendered: $screenshot ($unique_colors unique colors)." >&2
    exit 1
  fi
}

test_config_root=$(mktemp -d)
media_root=$(mktemp -d)
empty_path_mappings="$test_config_root/empty-path-mappings.json"
trap 'rm -rf "$test_config_root" "$media_root"' EXIT

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

cp -a fixtures/launchbox/. "$media_root/"
fixture_video="$media_root/Videos/Fixture Console/fixture-adventure.mp4"
fixture_marquee_video="$media_root/Videos/Fixture Console/Marquee/Fixture Adventure-01.mp4"
fixture_manual="$media_root/Manuals/Fixture Console/fixture-adventure.pdf"
fixture_music_first="$media_root/Music/Fixture Console/Fixture Adventure-01.mp3"
fixture_music_second="$media_root/Music/Fixture Console/Fixture Adventure-02.mp3"
fixture_startup_sound="$test_config_root/fixture-startup.wav"
mkdir -p \
  "$(dirname "$fixture_video")" \
  "$(dirname "$fixture_marquee_video")" \
  "$(dirname "$fixture_manual")" \
  "$(dirname "$fixture_music_first")"
base64 --decode fixtures/media/fixture-video.mp4.base64 > "$fixture_video"
cp "$fixture_video" "$fixture_marquee_video"
base64 --decode fixtures/media/fixture-manual.pdf.base64 > "$fixture_manual"
base64 --decode fixtures/media/fixture-music.mp3.gz.base64 \
  | gzip --decompress > "$fixture_music_first"
base64 --decode fixtures/media/fixture-startup.wav.gz.base64 \
  | gzip --decompress > "$fixture_startup_sound"
cp "$fixture_music_first" "$fixture_music_second"
background_music_files=(
  "$media_root/Music/Background/Default-01.mp3"
  "$media_root/Music/Background/Default-02.mp3"
  "$media_root/Music/Background/Platforms/Fixture Console/Platform-01.mp3"
  "$media_root/Music/Background/Platforms/Fixture Console/Platform-02.mp3"
  "$media_root/Music/Background/Playlists/Fixture Favorites/Playlist-01.mp3"
  "$media_root/Music/Background/Playlists/Fixture Favorites/Playlist-02.mp3"
  "$media_root/Music/Background/Platform Categories/Fixture Category/Category-01.mp3"
  "$media_root/Music/Background/Platform Categories/Fixture Category/Category-02.mp3"
)
for background_music_file in "${background_music_files[@]}"; do
  mkdir -p "$(dirname "$background_music_file")"
  cp "$fixture_music_first" "$background_music_file"
done
if [[ $(sha256sum "$fixture_video" | cut -d' ' -f1) \
  != d415ca3d0511bb16cbbc5a508fe831f7a0f080e9c187475de907ba070431a205 ]]; then
  echo "Decoded selected-game video fixture does not match its pinned source." >&2
  exit 1
fi
if [[ $(sha256sum "$fixture_manual" | cut -d' ' -f1) \
  != 52a03172ce1339ed39ad214396e16a10d65af40543ae4f7a4289545cc554bae1 ]]; then
  echo "Decoded game-manual fixture does not match its pinned source." >&2
  exit 1
fi
if [[ $(sha256sum "$fixture_startup_sound" | cut -d' ' -f1) \
  != ff01eea0f9e23153752ee48cc0a74771dc234f7a84a2da7a879c72538ddffec7 ]]; then
  echo "Decoded startup-sound fixture does not match its pinned source." >&2
  exit 1
fi
for fixture_music in \
  "$fixture_music_first" \
  "$fixture_music_second" \
  "${background_music_files[@]}"; do
  if [[ $(sha256sum "$fixture_music" | cut -d' ' -f1) \
    != 89d64dd51662c9c3c41629582028828cf53f0d66608404b69e178310e1174fd3 ]]; then
    echo "Decoded game-music fixture does not match its pinned source." >&2
    exit 1
  fi
done
media_files_manifest="$test_config_root/media-files.before.sha256"
(
  cd "$media_root"
  find Images Videos Manuals Music -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$media_files_manifest"
media_platform="$media_root/Data/Platforms/Fixture Console.xml"
cp "$media_platform" "$media_platform.before-media-smoke"
for shell in launchbox bigbox; do
  arguments=(
    --library "$media_root"
    --media-smoke-test
    --path-mappings-file "$empty_path_mappings"
  )
  if [[ "$shell" == bigbox ]]; then
    arguments+=(--windowed)
  fi
  output=$(QT_QPA_PLATFORM=offscreen \
    "$binary_dir/$shell" "${arguments[@]}" 2>&1) || {
    printf '%s\n' "$output" >&2
    exit 1
  }
  if ! rg -q \
    'MEDIA_SMOKE_COMPLETE images=1 id=fixture-adventure file=Fixture-Adventure-01.svg' \
    <<< "$output"; then
    printf '%s\n' "$output" >&2
    echo "$shell did not decode and render its indexed front artwork." >&2
    exit 1
  fi
  cmp "$media_platform.before-media-smoke" "$media_platform"
done

echo "LaunchBox and BigBox native-path front artwork indexing, URL delivery, decoding, and rendering validated without library writes."

frontend_handoff_recorder="$test_config_root/frontend-handoff-recorder"
frontend_handoff_ui_state="$test_config_root/frontend-handoff-ui-state.json"
frontend_handoff_model_state="$test_config_root/frontend-handoff-model-state.json"
install_process_fixture "$frontend_handoff_recorder"

for shell_name in launchbox bigbox; do
  frontend_handoff_log="$test_config_root/frontend-handoff-$shell_name-arguments.txt"
  output=$(
    LBPORT_FRONTEND_HANDOFF_LOG="$frontend_handoff_log" \
      QT_QPA_PLATFORM=offscreen \
      "$binary_dir/$shell_name" \
      --library "$media_root" \
      --frontend-handoff-smoke-test \
      --frontend-peer-executable "$frontend_handoff_recorder" \
      --select-game-id fixture-racer \
      --windowed \
      --path-mappings-file "$empty_path_mappings" \
      --map-windows-drive "Z=$media_root" \
      --ui-state-file "$frontend_handoff_ui_state" \
      --model-viewer-state-file "$frontend_handoff_model_state" \
      --frontend-handoff-drop-me drop-me 2>&1
  ) || {
    printf '%s\n' "$output" >&2
    exit 1
  }
  target_name=bigbox
  if [[ "$shell_name" == bigbox ]]; then
    target_name=launchbox
  fi
  if ! rg -q \
    "FRONTEND_HANDOFF_STARTED source=$shell_name target=$target_name selected=fixture-racer forwarded_arguments=12" \
    <<< "$output"; then
    printf '%s\n' "$output" >&2
    echo "$shell_name did not activate its real $target_name handoff control." >&2
    exit 1
  fi
  for ((attempt = 0; attempt < 200; ++attempt)); do
    if [[ -f "$frontend_handoff_log" ]]; then
      break
    fi
    sleep 0.01
  done
  if ! cmp -s "$frontend_handoff_log" <(
    printf '%s\n' \
      --path-mappings-file "$empty_path_mappings" \
      --map-windows-drive "Z=$media_root" \
      --ui-state-file "$frontend_handoff_ui_state" \
      --model-viewer-state-file "$frontend_handoff_model_state" \
      --library "$media_root" \
      --select-game-id fixture-racer
  ); then
    printf '%s handoff arguments were:\n' "$shell_name" >&2
    sed 's/^/  /' "$frontend_handoff_log" >&2 || true
    exit 1
  fi
done

frontend_handoff_locked_root="$test_config_root/frontend-handoff-locked-library"
cp -a "$media_root" "$frontend_handoff_locked_root"
frontend_handoff_locked_settings="$frontend_handoff_locked_root/Data/BigBoxSettings.xml"
sed -i \
  '/<ShowGameLockUnlock>true<\/ShowGameLockUnlock>/a\    <LockPin>2580</LockPin>' \
  "$frontend_handoff_locked_settings"
cp "$frontend_handoff_locked_settings" \
  "$frontend_handoff_locked_settings.before"
frontend_handoff_log="$test_config_root/frontend-handoff-locked-arguments.txt"
output=$(
  LBPORT_FRONTEND_HANDOFF_LOG="$frontend_handoff_log" \
    QT_QPA_PLATFORM=offscreen \
    "$binary_dir/bigbox" \
    --windowed \
    --library "$frontend_handoff_locked_root" \
    --frontend-handoff-blocked-smoke-test \
    --frontend-peer-executable "$frontend_handoff_recorder" \
    --select-game-id fixture-racer \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_LOCKED_ACTION_BLOCKED action=BigBoxExit' \
  <<< "$output"; then
  printf '%s\n' "$output" >&2
  echo "Locked BigBox did not deny its Desktop Mode handoff." >&2
  exit 1
fi
if [[ -e "$frontend_handoff_log" ]]; then
  echo "Locked BigBox started a frontend peer despite the denied exit permission." >&2
  exit 1
fi
cmp "$frontend_handoff_locked_settings.before" \
  "$frontend_handoff_locked_settings"

echo "LaunchBox and BigBox direct-process handoffs preserved the live library, selected stable game, path mappings, and state files; dropped smoke/UI-only arguments; and enforced locked-mode exit permission without a shell."

input_settings="$media_root/Data/BigBoxSettings.xml"
input_bindings="$media_root/Data/InputBindings.xml"
cp "$input_settings" "$input_settings.before-input-smoke"
cp "$input_bindings" "$input_bindings.before-input-smoke"
output=$(QT_QPA_PLATFORM=offscreen \
  "$binary_dir/bigbox" \
    --library "$media_root" \
    --bigbox-input-smoke-test \
    --windowed \
    --path-mappings-file "$empty_path_mappings" 2>&1) || {
  printf '%s\n' "$output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_INPUT_SMOKE_COMPLETE actions=59 keyboard_slots=4 .*controller_rules=18 select=1 back=1 navigation=2 images=1 image_back=1 final=fixture-adventure' \
  <<< "$output"; then
  printf '%s\n' "$output" >&2
  echo "BigBox did not route recovered keyboard and semantic controller input through its live surfaces." >&2
  exit 1
fi
cmp "$input_settings.before-input-smoke" "$input_settings"
cmp "$input_bindings.before-input-smoke" "$input_bindings"
(
  cd "$media_root"
  sha256sum --check "$media_files_manifest"
)

echo "BigBox four-slot keyboard mappings, native gamepad boundary, semantic controller rules, distinct Select/Play actions, navigation, nested Back, and image entry validated without library writes."

input_editor_root="$test_config_root/input-editor-library"
mkdir -p "$input_editor_root"
cp -a "$media_root/." "$input_editor_root/"
editor_settings="$input_editor_root/Data/BigBoxSettings.xml"
editor_bindings="$input_editor_root/Data/InputBindings.xml"
editor_settings_before="$test_config_root/BigBoxSettings.before-input-editor.xml"
editor_bindings_before="$test_config_root/InputBindings.before-input-editor.xml"
input_editor_screenshot="$test_config_root/bigbox-input-editor.png"
cp "$editor_settings" "$editor_settings_before"
cp "$editor_bindings" "$editor_bindings_before"
output=$(run_software_rendered_smoke \
  "$binary_dir/bigbox" \
    --library "$input_editor_root" \
    --bigbox-input-editor-smoke-test \
    --bigbox-input-editor-screenshot "$input_editor_screenshot" \
    --windowed \
    --path-mappings-file "$empty_path_mappings" 2>&1) || {
  printf '%s\n' "$output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_INPUT_EDITOR_SMOKE_COMPLETE actions=59 keyboard=Z controller_rules=18 hold=Button7 transaction=2 revision=' \
  <<< "$output"; then
  printf '%s\n' "$output" >&2
  echo "BigBox did not edit and transactionally reload keyboard and controller mappings through its Qt dialog." >&2
  exit 1
fi
if ! rg -q '<UseAllControllers>true</UseAllControllers>' "$editor_settings" \
  || ! rg -q '<KeyboardSelect2>69</KeyboardSelect2>' "$editor_settings" \
  || ! rg -q -U \
    '<InputAction>BigBoxExit</InputAction>\s*<ControllerBinding>Button8</ControllerBinding>\s*<ControllerHoldBinding>Button7</ControllerHoldBinding>' \
    "$editor_bindings" \
  || [[ $(rg -c '<InputBinding>' "$editor_bindings") -ne 18 ]]; then
  printf '%s\n' "$output" >&2
  echo "BigBox input editor did not persist the expected LaunchBox XML semantics." >&2
  exit 1
fi
editor_settings_backups=(
  "$input_editor_root"/Data/BigBoxSettings.xml.lbport-transaction-backup-*
)
editor_bindings_backups=(
  "$input_editor_root"/Data/InputBindings.xml.lbport-transaction-backup-*
)
if [[ ${#editor_settings_backups[@]} -ne 1 \
  || ${#editor_bindings_backups[@]} -ne 1 ]]; then
  echo "BigBox input editor did not retain exactly one backup for each transaction participant." >&2
  exit 1
fi
cmp "$editor_settings_before" "${editor_settings_backups[0]}"
cmp "$editor_bindings_before" "${editor_bindings_backups[0]}"
if [[ ! -s "$input_editor_screenshot" ]] \
  || [[ $(wc -c < "$input_editor_screenshot") -lt 4096 ]]; then
  echo "BigBox input editor did not save a rendered screenshot." >&2
  exit 1
fi
input_editor_colors=$(magick "$input_editor_screenshot" -format '%k' info:)
if [[ ! "$input_editor_colors" =~ ^[0-9]+$ ]] \
  || ((input_editor_colors < 64)); then
  echo "BigBox input editor screenshot is blank or insufficiently rendered ($input_editor_colors colors)." >&2
  exit 1
fi

echo "BigBox rendered Qt input editor, logical key capture, controller/hold editing, two-document transaction, exact backups, and committed live-policy reload validated."

security_root="$test_config_root/security-library"
mkdir -p "$security_root"
cp -a fixtures/launchbox/. "$security_root/"
security_settings="$security_root/Data/BigBoxSettings.xml"
sed -i \
  's#<AllowChangeFilterPlatformsWhileLocked>true</AllowChangeFilterPlatformsWhileLocked>#<AllowChangeFilterPlatformsWhileLocked>false</AllowChangeFilterPlatformsWhileLocked>#' \
  "$security_settings"
sed -i \
  '/<ShowGameLockUnlock>true<\/ShowGameLockUnlock>/a\    <LockPin>2580</LockPin>' \
  "$security_settings"
security_settings_before="$test_config_root/BigBoxSettings.before-security.xml"
security_pin_screenshot="$test_config_root/bigbox-security-pin.png"
security_editor_screenshot="$test_config_root/bigbox-security-editor.png"
cp "$security_settings" "$security_settings_before"
security_immutable_manifest="$test_config_root/security-immutable.before.sha256"
(
  cd "$security_root"
  find Data Images Metadata Music -type f \
    ! -path 'Data/BigBoxSettings.xml' -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$security_immutable_manifest"
output=$(run_software_rendered_smoke \
  "$binary_dir/bigbox" \
    --library "$security_root" \
    --bigbox-security-smoke-test \
    --bigbox-security-pin-screenshot "$security_pin_screenshot" \
    --bigbox-security-editor-screenshot "$security_editor_screenshot" \
    --windowed \
    --path-mappings-file "$empty_path_mappings" 2>&1) || {
  printf '%s\n' "$output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_SECURITY_SMOKE_COMPLETE permissions=32 blocked=2 failures=2 unlocks=2 writes=1 pin=configured locked=0 revision=' \
  <<< "$output"; then
  printf '%s\n' "$output" >&2
  echo "BigBox did not complete its locked-mode, keypad, PIN replacement, and permission transaction flow." >&2
  exit 1
fi
if rg -q '2580|8642|0000' <<< "$output"; then
  printf '%s\n' "$output" >&2
  echo "BigBox exposed a PIN value in runtime diagnostics." >&2
  exit 1
fi
if ! rg -q '<LockPin>8642</LockPin>' "$security_settings" \
  || rg -q '<LockPin>2580</LockPin>' "$security_settings" \
  || ! rg -q '<ShowGameLockUnlock>true</ShowGameLockUnlock>' \
    "$security_settings" \
  || ! rg -q '<AllowExitWhileUnlocked>true</AllowExitWhileUnlocked>' \
    "$security_settings" \
  || ! rg -q \
    '<AllowChangeFilterPlatformsWhileLocked>false</AllowChangeFilterPlatformsWhileLocked>' \
    "$security_settings" \
  || [[ $(rg -c '    <Allow' "$security_settings") -ne 32 ]] \
  || ! rg -q '<Theme>Fixture BigBox Theme</Theme>' "$security_settings"; then
  printf '%s\n' "$output" >&2
  echo "BigBox security did not persist the complete expected LaunchBox XML contract." >&2
  exit 1
fi
security_backups=(
  "$security_root"/Data/BigBoxSettings.xml.lbport-transaction-backup-*
)
if [[ ${#security_backups[@]} -ne 1 ]]; then
  echo "BigBox security did not retain exactly one transaction backup." >&2
  exit 1
fi
cmp "$security_settings_before" "${security_backups[0]}"
for screenshot in \
  "$security_pin_screenshot" \
  "$security_editor_screenshot"; do
  if [[ ! -s "$screenshot" ]] \
    || [[ $(wc -c < "$screenshot") -lt 4096 ]]; then
    echo "BigBox security did not save a rendered screenshot: $screenshot" >&2
    exit 1
  fi
  security_colors=$(magick "$screenshot" -format '%k' info:)
  if [[ ! "$security_colors" =~ ^[0-9]+$ ]] \
    || ((security_colors < 64)); then
    echo "BigBox security screenshot is blank or insufficiently rendered: $screenshot ($security_colors colors)." >&2
    exit 1
  fi
done
(
  cd "$security_root"
  sha256sum --check "$security_immutable_manifest"
) >/dev/null
if find "$security_root" -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "BigBox security left a recovery manifest behind." >&2
  exit 1
fi

echo "BigBox native PIN keypad, startup lock, per-action and navigation gates, wrong/correct unlocks, rendered security editor, PIN replacement, 32-permission transaction, exact backup, redacted diagnostics, and immutable peer data validated."

game_actions_root="$test_config_root/game-actions-library"
mkdir -p "$game_actions_root"
cp -a fixtures/launchbox/. "$game_actions_root/"
game_actions_settings="$game_actions_root/Data/BigBoxSettings.xml"
game_actions_platform="$game_actions_root/Data/Platforms/Fixture Console.xml"
sed -i \
  's#<AllowSettingStarRatingsWhileLocked>false</AllowSettingStarRatingsWhileLocked>#<AllowSettingStarRatingsWhileLocked>true</AllowSettingStarRatingsWhileLocked>#' \
  "$game_actions_settings"
sed -i \
  '/<ShowGameLockUnlock>true<\/ShowGameLockUnlock>/a\    <LockPin>2580</LockPin>' \
  "$game_actions_settings"
game_actions_platform_before="$test_config_root/Fixture Console.before-game-actions.xml"
game_actions_screenshot="$test_config_root/bigbox-star-rating.png"
cp "$game_actions_platform" "$game_actions_platform_before"
game_actions_immutable_manifest="$test_config_root/game-actions-immutable.before.sha256"
(
  cd "$game_actions_root"
  find Data Images Metadata Music -type f \
    ! -path 'Data/Platforms/Fixture Console.xml' -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$game_actions_immutable_manifest"
output=$(run_software_rendered_smoke \
  "$binary_dir/bigbox" \
    --library "$game_actions_root" \
    --bigbox-game-actions-smoke-test \
    --bigbox-game-actions-screenshot "$game_actions_screenshot" \
    --windowed \
    --path-mappings-file "$empty_path_mappings" 2>&1) || {
  printf '%s\n' "$output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_GAME_ACTIONS_SMOKE_COMPLETE game=fixture-adventure favorite=0 rating=2.5 integer=2 completed=0 favorite_first=1 popup=1 blocked=1 unlocks=1 writes=2 revision=' \
  <<< "$output"; then
  printf '%s\n' "$output" >&2
  echo "BigBox did not complete its locked rating, unlock, and favorite transaction flow." >&2
  exit 1
fi
if rg -q '2580' <<< "$output"; then
  printf '%s\n' "$output" >&2
  echo "BigBox game-action smoke exposed its PIN in runtime diagnostics." >&2
  exit 1
fi
if ! rg -q -U \
  '(?s)<Game>.*?<Completed>false</Completed>.*?<Favorite>false</Favorite>.*?<ID>fixture-adventure</ID>.*?<StarRating>2</StarRating>.*?<StarRatingFloat>2.5</StarRatingFloat>.*?<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>' \
  "$game_actions_platform"; then
  printf '%s\n' "$output" >&2
  echo "BigBox did not persist the expected favorite, half-star, integer companion, completion, and unknown XML fields." >&2
  exit 1
fi
game_actions_backups=(
  "$game_actions_root"/Data/Platforms/Fixture\ Console.xml.lbport-transaction-backup-*
)
if [[ ${#game_actions_backups[@]} -ne 2 ]]; then
  echo "BigBox game actions did not retain exactly two transaction backups." >&2
  exit 1
fi
original_game_action_backups=0
intermediate_game_action_backups=0
for backup in "${game_actions_backups[@]}"; do
  if cmp -s "$game_actions_platform_before" "$backup"; then
    ((original_game_action_backups += 1))
  fi
  if rg -q -U \
    '(?s)<Game>.*?<Completed>false</Completed>.*?<Favorite>true</Favorite>.*?<ID>fixture-adventure</ID>.*?<StarRating>2</StarRating>.*?<StarRatingFloat>2.5</StarRatingFloat>' \
    "$backup"; then
    ((intermediate_game_action_backups += 1))
  fi
done
if ((original_game_action_backups != 1 \
  || intermediate_game_action_backups != 1)); then
  echo "BigBox game-action backups do not form the expected original-to-rating transaction chain." >&2
  exit 1
fi
if [[ ! -s "$game_actions_screenshot" ]] \
  || [[ $(wc -c < "$game_actions_screenshot") -lt 4096 ]]; then
  echo "BigBox star-rating popup did not save a rendered screenshot." >&2
  exit 1
fi
game_actions_colors=$(magick "$game_actions_screenshot" -format '%k' info:)
if [[ ! "$game_actions_colors" =~ ^[0-9]+$ ]] \
  || ((game_actions_colors < 48)); then
  echo "BigBox star-rating screenshot is blank or insufficiently rendered ($game_actions_colors colors)." >&2
  exit 1
fi
(
  cd "$game_actions_root"
  sha256sum --check "$game_actions_immutable_manifest"
) >/dev/null
if find "$game_actions_root" -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "BigBox game actions left a recovery manifest behind." >&2
  exit 1
fi

echo "BigBox recovered favorite/rating settings, favorite-first projection, locked permission gates, rendered half-star popup, two committed state transactions, exact backup chain, lossless XML, and immutable peer data validated."

playlist_actions_root="$test_config_root/playlist-actions-library"
mkdir -p "$playlist_actions_root"
cp -a fixtures/launchbox/. "$playlist_actions_root/"
playlist_actions_settings="$playlist_actions_root/Data/BigBoxSettings.xml"
playlist_actions_parents="$playlist_actions_root/Data/Parents.xml"
playlist_actions_manual="$playlist_actions_root/Data/Playlists/Manual Picks.xml"
playlist_actions_generated="$playlist_actions_root/Data/Playlists/Generated Picks.xml"
playlist_actions_screenshot="$test_config_root/bigbox-playlist-actions.png"
sed -i \
  '/<ShowGameMenuFlipBox>true<\/ShowGameMenuFlipBox>/a\    <ShowGameMenuPlaylistActions>true</ShowGameMenuPlaylistActions>' \
  "$playlist_actions_settings"
sed -i \
  '/<ShowGameLockUnlock>true<\/ShowGameLockUnlock>/a\    <LockPin>2580</LockPin>' \
  "$playlist_actions_settings"
sed -i \
  '/<\/LaunchBox>/i\  <Parent>\n    <PlaylistId>manual-playlist</PlaylistId>\n  </Parent>\n  <Parent>\n    <PlaylistId>generated-playlist</PlaylistId>\n  </Parent>' \
  "$playlist_actions_parents"
printf '%s\n' \
  '<?xml version="1.0" encoding="utf-8"?>' \
  '<LaunchBox>' \
  '  <Playlist>' \
  '    <PlaylistId>manual-playlist</PlaylistId>' \
  '    <Name>Manual Picks</Name>' \
  '    <NestedName>Manual Picks</NestedName>' \
  '    <Notes>Manual BigBox action fixture.</Notes>' \
  '    <HideInBigBox>false</HideInBigBox>' \
  '    <LocalDbParsed>true</LocalDbParsed>' \
  '    <AutoPopulate>false</AutoPopulate>' \
  '    <IncludeWithPlatforms>false</IncludeWithPlatforms>' \
  '    <IsAutogenerated>false</IsAutogenerated>' \
  '    <SortBy>Title</SortBy>' \
  '    <FuturePlaylistField>keep-manual</FuturePlaylistField>' \
  '  </Playlist>' \
  '  <PlaylistGame>' \
  '    <GameId>fixture-puzzle</GameId>' \
  '    <GameTitle>Fixture Puzzle</GameTitle>' \
  '    <GamePlatform>Fixture Console</GamePlatform>' \
  '    <GameFileName>fixture-puzzle.rom</GameFileName>' \
  '    <LaunchBoxDbId>4321</LaunchBoxDbId>' \
  '    <ManualOrder>4</ManualOrder>' \
  '    <FutureGameField>keep-puzzle</FutureGameField>' \
  '  </PlaylistGame>' \
  '  <FutureRootField>keep-manual-root</FutureRootField>' \
  '</LaunchBox>' > "$playlist_actions_manual"
printf '%s\n' \
  '<?xml version="1.0" encoding="utf-8"?>' \
  '<LaunchBox>' \
  '  <Playlist>' \
  '    <PlaylistId>generated-playlist</PlaylistId>' \
  '    <Name>Generated Picks</Name>' \
  '    <NestedName>Generated Picks</NestedName>' \
  '    <Notes>Generated exclusion fixture.</Notes>' \
  '    <HideInBigBox>false</HideInBigBox>' \
  '    <LocalDbParsed>true</LocalDbParsed>' \
  '    <AutoPopulate>false</AutoPopulate>' \
  '    <IncludeWithPlatforms>false</IncludeWithPlatforms>' \
  '    <IsAutogenerated>true</IsAutogenerated>' \
  '    <SortBy>Title</SortBy>' \
  '    <FutureGeneratedField>keep-generated</FutureGeneratedField>' \
  '  </Playlist>' \
  '  <PlaylistGame>' \
  '    <GameId>fixture-racer</GameId>' \
  '    <GameTitle>Fixture Racer</GameTitle>' \
  '    <GamePlatform>Fixture Console</GamePlatform>' \
  '    <GameFileName>fixture-racer.rom</GameFileName>' \
  '    <ManualOrder>3</ManualOrder>' \
  '  </PlaylistGame>' \
  '</LaunchBox>' > "$playlist_actions_generated"
playlist_actions_manual_before="$test_config_root/Manual Picks.before-playlist-actions.xml"
cp "$playlist_actions_manual" "$playlist_actions_manual_before"
playlist_actions_immutable_manifest="$test_config_root/playlist-actions-immutable.before.sha256"
(
  cd "$playlist_actions_root"
  find Data Images Metadata Music -type f \
    ! -path 'Data/Playlists/Manual Picks.xml' -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$playlist_actions_immutable_manifest"
output=$(run_software_rendered_smoke \
  "$binary_dir/bigbox" \
    --library "$playlist_actions_root" \
    --bigbox-playlist-actions-smoke-test \
    --bigbox-playlist-actions-screenshot "$playlist_actions_screenshot" \
    --windowed \
    --path-mappings-file "$empty_path_mappings" 2>&1) || {
  printf '%s\n' "$output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_PLAYLIST_ACTIONS_SMOKE_COMPLETE game=fixture-adventure playlist=manual-playlist targets=1 popup=1 blocked=1 unlocks=1 writes=2 revision=.* final_visible=fixture-puzzle' \
  <<< "$output"; then
  printf '%s\n' "$output" >&2
  echo "BigBox did not complete its locked add, manual-playlist navigation, and remove transaction flow." >&2
  exit 1
fi
if rg -q '2580' <<< "$output"; then
  printf '%s\n' "$output" >&2
  echo "BigBox playlist-action smoke exposed its PIN in runtime diagnostics." >&2
  exit 1
fi
if rg -q '<GameId>fixture-adventure</GameId>' \
    "$playlist_actions_manual" \
  || ! rg -q -U \
    '(?s)<PlaylistGame>.*?<GameId>fixture-puzzle</GameId>.*?<ManualOrder>4</ManualOrder>.*?<FutureGameField>keep-puzzle</FutureGameField>.*?</PlaylistGame>' \
    "$playlist_actions_manual" \
  || ! rg -q '<FuturePlaylistField>keep-manual</FuturePlaylistField>' \
    "$playlist_actions_manual" \
  || ! rg -q '<FutureRootField>keep-manual-root</FutureRootField>' \
    "$playlist_actions_manual"; then
  printf '%s\n' "$output" >&2
  echo "BigBox playlist actions did not preserve the final manual membership and unknown XML contract." >&2
  exit 1
fi
playlist_actions_backups=(
  "$playlist_actions_root"/Data/Playlists/Manual\ Picks.xml.lbport-transaction-backup-*
)
if [[ ${#playlist_actions_backups[@]} -ne 2 ]]; then
  echo "BigBox playlist actions did not retain exactly two transaction backups." >&2
  exit 1
fi
playlist_actions_original_backups=0
playlist_actions_intermediate_backups=0
for backup in "${playlist_actions_backups[@]}"; do
  if cmp -s "$playlist_actions_manual_before" "$backup"; then
    ((playlist_actions_original_backups += 1))
  fi
  if rg -q -U \
    '(?s)<PlaylistGame>.*?<GameId>fixture-adventure</GameId>.*?<GameTitle>Fixture Adventure</GameTitle>.*?<GamePlatform>Fixture Console</GamePlatform>.*?<GameFileName>adventure.rom</GameFileName>.*?<LaunchBoxDbId>1234</LaunchBoxDbId>.*?<ManualOrder>-1</ManualOrder>.*?</PlaylistGame>' \
    "$backup" \
    && rg -q '<FuturePlaylistField>keep-manual</FuturePlaylistField>' \
      "$backup" \
    && rg -q '<FutureGameField>keep-puzzle</FutureGameField>' "$backup"; then
    ((playlist_actions_intermediate_backups += 1))
  fi
done
if ((playlist_actions_original_backups != 1 \
  || playlist_actions_intermediate_backups != 1)); then
  echo "BigBox playlist-action backups do not form the expected original-to-added membership chain." >&2
  exit 1
fi
if [[ ! -s "$playlist_actions_screenshot" ]] \
  || [[ $(wc -c < "$playlist_actions_screenshot") -lt 4096 ]]; then
  echo "BigBox playlist popup did not save a rendered screenshot." >&2
  exit 1
fi
playlist_actions_colors=$(
  magick "$playlist_actions_screenshot" -format '%k' info:
)
if [[ ! "$playlist_actions_colors" =~ ^[0-9]+$ ]] \
  || ((playlist_actions_colors < 32)); then
  echo "BigBox playlist popup screenshot is blank or insufficiently rendered ($playlist_actions_colors colors)." >&2
  exit 1
fi
(
  cd "$playlist_actions_root"
  sha256sum --check "$playlist_actions_immutable_manifest"
) >/dev/null
if find "$playlist_actions_root" -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "BigBox playlist actions left a recovery manifest behind." >&2
  exit 1
fi

echo "BigBox recovered manual-only add/remove playlist actions, fail-closed locked gate, rendered list popup, two committed transactions, exact backup chain, ManualOrder append sentinel, lossless XML, and immutable automatic/generated peer playlists validated."

related_games_root="$test_config_root/related-games-library"
mkdir -p "$related_games_root"
cp -a "$media_root/." "$related_games_root/"
related_games_settings="$related_games_root/Data/BigBoxSettings.xml"
related_games_database="$related_games_root/Metadata/LaunchBox.Metadata.db"
related_games_screenshot="$test_config_root/bigbox-related-games.png"
sed -i \
  '/<ShowGameMenuFlipBox>true<\/ShowGameMenuFlipBox>/a\    <ShowGameMenuViewRelatedGames>true</ShowGameMenuViewRelatedGames>' \
  "$related_games_settings"
sqlite3 "$related_games_database" \
  < fixtures/launchbox/Metadata/fixture.sql
sqlite3 "$related_games_database" \
  "DELETE FROM Games;
   INSERT INTO Games VALUES (
     5555, 'Fixture Adventure', 'FIXTURE ADVENTURE',
     '2001-02-03 00:00:00', 2001,
     'A database-only port used by the Related Games rendered smoke.',
     2, 'Released', 1, NULL, 4.8, 64, NULL,
     'Other Console', 'E', 'Adventure', 'Fixture Labs',
     'Fixture Publishing'
   );"
related_games_manifest="$test_config_root/related-games.before.sha256"
(
  cd "$related_games_root"
  find Data Images Metadata Music -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$related_games_manifest"
output=$(run_software_rendered_smoke \
  "$binary_dir/bigbox" \
    --library "$related_games_root" \
    --bigbox-related-games-smoke-test \
    --bigbox-related-games-screenshot "$related_games_screenshot" \
    --windowed \
    --path-mappings-file "$empty_path_mappings" 2>&1) || {
  printf '%s\n' "$output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_RELATED_GAMES_SMOKE_COMPLETE seed=fixture-adventure selected=fixture-racer sections=3 metadata=1 revision=' \
  <<< "$output"; then
  printf '%s\n' "$output" >&2
  echo "BigBox did not complete the lazy Related Games and stable-ID selection flow." >&2
  exit 1
fi
if [[ ! -s "$related_games_screenshot" ]] \
  || [[ $(wc -c < "$related_games_screenshot") -lt 4096 ]]; then
  echo "BigBox Related Games popup did not save a rendered screenshot." >&2
  exit 1
fi
related_games_colors=$(
  magick "$related_games_screenshot" -format '%k' info:
)
if [[ ! "$related_games_colors" =~ ^[0-9]+$ ]] \
  || ((related_games_colors < 32)); then
  echo "BigBox Related Games screenshot is blank or insufficiently rendered ($related_games_colors colors)." >&2
  exit 1
fi
(
  cd "$related_games_root"
  sha256sum --check "$related_games_manifest"
) >/dev/null

echo "BigBox lazy Related Games, three recovered tabs, read-only metadata candidates, dimmed cloud rows, rendered popup, and stable installed-game navigation validated."

discovery_root="$test_config_root/discovery-library"
mkdir -p "$discovery_root"
cp -a "$media_root/." "$discovery_root/"
discovery_screenshot="$test_config_root/bigbox-discovery.png"
discovery_manifest="$test_config_root/discovery.before.sha256"
(
  cd "$discovery_root"
  find Data Images Metadata Music Videos Manuals -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$discovery_manifest"
output=$(run_software_rendered_smoke \
  "$binary_dir/bigbox" \
    --library "$discovery_root" \
    --bigbox-discovery-smoke-test \
    --bigbox-discovery-screenshot "$discovery_screenshot" \
    --windowed \
    --path-mappings-file "$empty_path_mappings" 2>&1) || {
  printf '%s\n' "$output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_DISCOVERY_SMOKE_COMPLETE selected=fixture-racer contracts=6 visible=6 revision=' \
  <<< "$output"; then
  printf '%s\n' "$output" >&2
  echo "BigBox did not complete its recovered Discovery Center list and stable-ID selection flow." >&2
  exit 1
fi
if [[ ! -s "$discovery_screenshot" ]] \
  || [[ $(wc -c < "$discovery_screenshot") -lt 4096 ]]; then
  echo "BigBox Discovery Center did not save a rendered screenshot." >&2
  exit 1
fi
discovery_colors=$(
  magick "$discovery_screenshot" -format '%k' info:
)
if [[ ! "$discovery_colors" =~ ^[0-9]+$ ]] \
  || ((discovery_colors < 48)); then
  echo "BigBox Discovery Center screenshot is blank or insufficiently rendered ($discovery_colors colors)." >&2
  exit 1
fi
(
  cd "$discovery_root"
  sha256sum --check "$discovery_manifest"
) >/dev/null

echo "BigBox recovered Discovery Center order, local projections, bounded provider manual/automatic rows, priority ordering, keyboard navigation, rendered provider content, and stable installed-game navigation validated with immutable inputs."

marquee_root="$test_config_root/marquee-library"
mkdir -p "$marquee_root"
cp -a "$media_root/." "$marquee_root/"
marquee_settings="$marquee_root/Data/BigBoxSettings.xml"
marquee_settings_before="$test_config_root/BigBoxSettings.before-marquee.xml"
marquee_game_screenshot="$test_config_root/bigbox-marquee-game.png"
marquee_platform_screenshot="$test_config_root/bigbox-marquee-platform.png"
cp "$marquee_settings" "$marquee_settings_before"
marquee_manifest="$test_config_root/marquee-media.before.sha256"
(
  cd "$marquee_root"
  find Images Videos Manuals Music -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$marquee_manifest"
output=$(run_software_rendered_smoke \
  "$binary_dir/bigbox" \
    --library "$marquee_root" \
    --bigbox-marquee-smoke-test \
    --bigbox-marquee-game-screenshot "$marquee_game_screenshot" \
    --bigbox-marquee-platform-screenshot "$marquee_platform_screenshot" \
    --windowed \
    --path-mappings-file "$empty_path_mappings" 2>&1) || {
  printf '%s\n' "$output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_MARQUEE_SMOKE_COMPLETE screens=[1-9][0-9]* monitor=0 game=fixture-adventure video=mp4 image=svg platform=Fixture Console banner=svg stretch=1 theme_override=1 compatibility=TopHalfCutOff transaction=1 revision=' \
  <<< "$output"; then
  printf '%s\n' "$output" >&2
  echo "BigBox did not complete its native secondary-marquee display transaction and rendered game/platform flow." >&2
  exit 1
fi
if ! rg -q '<PrimaryMonitorIndex>0</PrimaryMonitorIndex>' "$marquee_settings" \
  || ! rg -q '<MarqueeMonitorIndex>0</MarqueeMonitorIndex>' "$marquee_settings" \
  || ! rg -q '<MarqueeIgnoreThemeViews>true</MarqueeIgnoreThemeViews>' "$marquee_settings" \
  || ! rg -q '<MarqueeStretchImages>true</MarqueeStretchImages>' "$marquee_settings" \
  || ! rg -q '<MarqueeScreenCompatibilityMode>TopHalfCutOff</MarqueeScreenCompatibilityMode>' "$marquee_settings" \
  || ! rg -q '<Theme>Fixture BigBox Theme</Theme>' "$marquee_settings"; then
  printf '%s\n' "$output" >&2
  echo "BigBox marquee settings did not preserve the expected LaunchBox XML semantics." >&2
  exit 1
fi
marquee_backups=(
  "$marquee_root"/Data/BigBoxSettings.xml.lbport-transaction-backup-*
)
if [[ ${#marquee_backups[@]} -ne 1 ]]; then
  echo "BigBox marquee settings did not retain exactly one transaction backup." >&2
  exit 1
fi
cmp "$marquee_settings_before" "${marquee_backups[0]}"
for screenshot in "$marquee_game_screenshot" "$marquee_platform_screenshot"; do
  if [[ ! -s "$screenshot" ]] \
    || [[ $(wc -c < "$screenshot") -lt 256 ]]; then
    echo "BigBox marquee did not save a rendered screenshot: $screenshot" >&2
    exit 1
  fi
  marquee_colors=$(magick "$screenshot" -format '%k' info:)
  if [[ ! "$marquee_colors" =~ ^[0-9]+$ ]] \
    || ((marquee_colors < 48)); then
    echo "BigBox marquee screenshot is blank or insufficiently rendered: $screenshot ($marquee_colors colors)." >&2
    exit 1
  fi
done
(
  cd "$marquee_root"
  sha256sum --check "$marquee_manifest"
) >/dev/null

echo "BigBox native Qt marquee window, host-screen routing, silent video priority, direct game/platform art, display editor, compatibility geometry, one-document transaction, exact backup, live-policy reload, and immutable media validated."

for shell in launchbox bigbox; do
  screenshot="$media_root/$shell-supplemental-media.png"
  arguments=(
    --library "$media_root"
    --supplemental-media-smoke-test
    --supplemental-media-screenshot "$screenshot"
    --path-mappings-file "$empty_path_mappings"
  )
  marker=LAUNCHBOX_SUPPLEMENTAL_MEDIA_SMOKE_COMPLETE
  if [[ "$shell" == bigbox ]]; then
    arguments+=(--windowed)
    marker=BIGBOX_SUPPLEMENTAL_MEDIA_SMOKE_COMPLETE
  fi
  output=$(QT_QPA_PLATFORM=offscreen \
    "$binary_dir/$shell" "${arguments[@]}" 2>&1) || {
    printf '%s\n' "$output" >&2
    exit 1
  }
  if ! rg -q \
    "$marker id=fixture-adventure manuals=1 tracks=2 manual=pdf audio=mp3 playlist=m3u controls=1" \
    <<< "$output"; then
    printf '%s\n' "$output" >&2
    echo "$shell did not validate its manual and game-music controls." >&2
    exit 1
  fi
  if [[ ! -s "$screenshot" ]] \
    || [[ $(wc -c < "$screenshot") -lt 1024 ]]; then
    echo "$shell did not save a rendered supplemental-media screenshot." >&2
    exit 1
  fi
  supplemental_colors=$(magick "$screenshot" -format '%k' info:)
  if [[ ! "$supplemental_colors" =~ ^[0-9]+$ ]] \
    || ((supplemental_colors < 64)); then
    echo "$shell supplemental-media screenshot is blank or insufficiently rendered ($supplemental_colors colors)." >&2
    exit 1
  fi
  cmp "$media_platform.before-media-smoke" "$media_platform"
done

echo "LaunchBox and BigBox manual opening, typed music policy, M3U expansion, Qt audio decode, pause/next/stop controls, and rendered player UI validated without library writes."

attract_root="$test_config_root/attract-mode-library"
attract_screenshot="$test_config_root/bigbox-attract-mode.png"
attract_move_folder="$attract_root/Sounds/Fixture Sounds/Move"
mkdir -p "$attract_root"
cp -a "$media_root/." "$attract_root/"
mkdir -p "$attract_move_folder"
cp "$fixture_startup_sound" "$attract_move_folder/MOVE001.wav"
cp "$fixture_startup_sound" "$attract_move_folder/MOVE002.wav"
attract_settings="$attract_root/Data/BigBoxSettings.xml"
sed -i \
  's#<AttractModeDelay>120</AttractModeDelay>#<AttractModeDelay>1</AttractModeDelay>#' \
  "$attract_settings"
sed -i \
  's#<AttractModeTimePerMovement>5</AttractModeTimePerMovement>#<AttractModeTimePerMovement>1</AttractModeTimePerMovement>#' \
  "$attract_settings"
sed -i \
  's#<AttractModeMinimumSpeed>200</AttractModeMinimumSpeed>#<AttractModeMinimumSpeed>80</AttractModeMinimumSpeed>#' \
  "$attract_settings"
sed -i \
  's#<PlayMoveInAttractMode>false</PlayMoveInAttractMode>#<PlayMoveInAttractMode>true</PlayMoveInAttractMode>#' \
  "$attract_settings"
sed -i \
  's#<VolumeAttractModeNavigationSound>15</VolumeAttractModeNavigationSound>#<VolumeAttractModeNavigationSound>40</VolumeAttractModeNavigationSound>#' \
  "$attract_settings"
sed -i \
  's#<VolumeAttractModeMaster>100</VolumeAttractModeMaster>#<VolumeAttractModeMaster>50</VolumeAttractModeMaster>#' \
  "$attract_settings"
cp "$attract_settings" "$attract_settings.before-attract-mode-smoke"
attract_manifest="$test_config_root/attract-mode.before.sha256"
(
  cd "$attract_root"
  find Images Videos Manuals Music Sounds -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$attract_manifest"
attract_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --windowed \
    --library "$attract_root" \
    --bigbox-attract-mode-smoke-test \
    --bigbox-attract-mode-screenshot "$attract_screenshot" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$attract_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_ATTRACT_MODE_SMOKE_COMPLETE enabled=1 wheel_steps=[0-9]+ movement_cycles=[0-9]+ filter_switches=[0-9]+ auto_delay_ms=[0-9]+ manual=1 input_exit=[0-9]+ sounds=2 sound=wav volume=20 curve=80-20-80' \
  <<< "$attract_output"; then
  printf '%s\n' "$attract_output" >&2
  echo "BigBox did not complete automatic and manual Attract Mode interaction." >&2
  exit 1
fi
if [[ ! -s "$attract_screenshot" ]] \
  || [[ $(wc -c < "$attract_screenshot") -lt 1024 ]]; then
  echo "BigBox did not save a rendered Attract Mode screenshot." >&2
  exit 1
fi
attract_colors=$(magick "$attract_screenshot" -format '%k' info:)
if [[ ! "$attract_colors" =~ ^[0-9]+$ ]] \
  || ((attract_colors < 64)); then
  echo "BigBox Attract Mode screenshot is blank or insufficiently rendered ($attract_colors colors)." >&2
  exit 1
fi
cmp "$attract_settings.before-attract-mode-smoke" "$attract_settings"
(
  cd "$attract_root"
  sha256sum --check "$attract_manifest"
) >/dev/null

attract_disabled_root="$test_config_root/attract-mode-disabled-library"
attract_disabled_move_folder="$attract_disabled_root/Sounds/Fixture Sounds/Move"
mkdir -p "$attract_disabled_root"
cp -a "$media_root/." "$attract_disabled_root/"
mkdir -p "$attract_disabled_move_folder"
cp "$fixture_startup_sound" \
  "$attract_disabled_move_folder/MOVE001.wav"
cp "$fixture_startup_sound" \
  "$attract_disabled_move_folder/MOVE002.wav"
attract_disabled_settings="$attract_disabled_root/Data/BigBoxSettings.xml"
sed -i \
  's#<EnableAttractMode>true</EnableAttractMode>#<EnableAttractMode>false</EnableAttractMode>#' \
  "$attract_disabled_settings"
sed -i \
  's#<AttractModeDelay>120</AttractModeDelay>#<AttractModeDelay>1</AttractModeDelay>#' \
  "$attract_disabled_settings"
sed -i \
  's#<AttractModeTimePerMovement>5</AttractModeTimePerMovement>#<AttractModeTimePerMovement>1</AttractModeTimePerMovement>#' \
  "$attract_disabled_settings"
sed -i \
  's#<AttractModeMinimumSpeed>200</AttractModeMinimumSpeed>#<AttractModeMinimumSpeed>80</AttractModeMinimumSpeed>#' \
  "$attract_disabled_settings"
sed -i \
  's#<PlayMoveInAttractMode>false</PlayMoveInAttractMode>#<PlayMoveInAttractMode>true</PlayMoveInAttractMode>#' \
  "$attract_disabled_settings"
sed -i \
  's#<VolumeAttractModeNavigationSound>15</VolumeAttractModeNavigationSound>#<VolumeAttractModeNavigationSound>40</VolumeAttractModeNavigationSound>#' \
  "$attract_disabled_settings"
sed -i \
  's#<VolumeAttractModeMaster>100</VolumeAttractModeMaster>#<VolumeAttractModeMaster>50</VolumeAttractModeMaster>#' \
  "$attract_disabled_settings"
cp "$attract_disabled_settings" \
  "$attract_disabled_settings.before-attract-mode-smoke"
attract_disabled_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --windowed \
    --library "$attract_disabled_root" \
    --bigbox-attract-mode-disabled-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$attract_disabled_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_ATTRACT_MODE_SMOKE_COMPLETE enabled=0 wheel_steps=[0-9]+ movement_cycles=0 filter_switches=0 auto_delay_ms=0 manual=1 input_exit=[0-9]+ sounds=2 sound=wav volume=20 curve=80-20-80' \
  <<< "$attract_disabled_output"; then
  printf '%s\n' "$attract_disabled_output" >&2
  echo "BigBox did not keep automatic Attract Mode disabled while retaining manual start." >&2
  exit 1
fi
cmp "$attract_disabled_settings.before-attract-mode-smoke" \
  "$attract_disabled_settings"

echo "BigBox typed Attract Mode policy, delayed automatic and explicit manual entry, bounded native wheel curve, filter switching, key/button exit layer, decoded move sounds, attract-specific volume, rendering, disabled-auto behavior, and immutable library data validated."

screensaver_root="$test_config_root/screensaver-library"
screensaver_screenshot_prefix="$test_config_root/bigbox-screensaver"
mkdir -p "$screensaver_root"
cp -a "$media_root/." "$screensaver_root/"
screensaver_settings="$screensaver_root/Data/BigBoxSettings.xml"
sed -i \
  's#<ScreensaverDelay>300</ScreensaverDelay>#<ScreensaverDelay>1</ScreensaverDelay>#' \
  "$screensaver_settings"
sed -i \
  's#<ScreensaverMinimumSwapTime>30000</ScreensaverMinimumSwapTime>#<ScreensaverMinimumSwapTime>300</ScreensaverMinimumSwapTime>#' \
  "$screensaver_settings"
sed -i \
  's#<ScreensaverMaximumSwapTime>60000</ScreensaverMaximumSwapTime>#<ScreensaverMaximumSwapTime>500</ScreensaverMaximumSwapTime>#' \
  "$screensaver_settings"
sed -i \
  's#<ScreensaverSkipGamesMissingVideo>false</ScreensaverSkipGamesMissingVideo>#<ScreensaverSkipGamesMissingVideo>true</ScreensaverSkipGamesMissingVideo>#' \
  "$screensaver_settings"
sed -i \
  's#<ScreensaverView>Screensaver1View</ScreensaverView>#<ScreensaverView>Screensaver3View</ScreensaverView>#' \
  "$screensaver_settings"
sed -i \
  's#<VolumeVideo>75</VolumeVideo>#<VolumeVideo>40</VolumeVideo>#' \
  "$screensaver_settings"
cp "$screensaver_settings" "$screensaver_settings.before-screensaver-smoke"
screensaver_manifest="$test_config_root/screensaver.before.sha256"
(
  cd "$screensaver_root"
  find Images Videos Manuals Music -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$screensaver_manifest"
screensaver_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --windowed \
    --library "$screensaver_root" \
    --bigbox-screensaver-smoke-test \
    --bigbox-screensaver-screenshot-prefix "$screensaver_screenshot_prefix" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$screensaver_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_SCREENSAVER_SMOKE_COMPLETE enabled=1 candidates=1 swaps=[0-9]+ selections=[0-9]+ auto_delay_ms=[0-9]+ manual=1 input_exit=[0-9]+ explore=[0-9]+ views=1-2-3-4 video=h264 volume=20 range=300-500' \
  <<< "$screensaver_output"; then
  printf '%s\n' "$screensaver_output" >&2
  echo "BigBox did not complete automatic and manual screensaver interaction." >&2
  exit 1
fi
for view in 1 2 3 4; do
  screenshot="$screensaver_screenshot_prefix-view$view.png"
  if [[ ! -s "$screenshot" ]] \
    || [[ $(wc -c < "$screenshot") -lt 1024 ]]; then
    echo "BigBox did not save rendered screensaver view $view." >&2
    exit 1
  fi
  screensaver_colors=$(magick "$screenshot" -format '%k' info:)
  if [[ ! "$screensaver_colors" =~ ^[0-9]+$ ]] \
    || ((screensaver_colors < 64)); then
    echo "BigBox screensaver view $view is blank or insufficiently rendered ($screensaver_colors colors)." >&2
    exit 1
  fi
done
cmp "$screensaver_settings.before-screensaver-smoke" "$screensaver_settings"
(
  cd "$screensaver_root"
  sha256sum --check "$screensaver_manifest"
) >/dev/null

screensaver_disabled_root="$test_config_root/screensaver-disabled-library"
mkdir -p "$screensaver_disabled_root"
cp -a "$media_root/." "$screensaver_disabled_root/"
screensaver_disabled_settings="$screensaver_disabled_root/Data/BigBoxSettings.xml"
sed -i \
  's#<EnableScreensaver>true</EnableScreensaver>#<EnableScreensaver>false</EnableScreensaver>#' \
  "$screensaver_disabled_settings"
sed -i \
  's#<ScreensaverDelay>300</ScreensaverDelay>#<ScreensaverDelay>1</ScreensaverDelay>#' \
  "$screensaver_disabled_settings"
sed -i \
  's#<ScreensaverMinimumSwapTime>30000</ScreensaverMinimumSwapTime>#<ScreensaverMinimumSwapTime>300</ScreensaverMinimumSwapTime>#' \
  "$screensaver_disabled_settings"
sed -i \
  's#<ScreensaverMaximumSwapTime>60000</ScreensaverMaximumSwapTime>#<ScreensaverMaximumSwapTime>500</ScreensaverMaximumSwapTime>#' \
  "$screensaver_disabled_settings"
sed -i \
  's#<ScreensaverSkipGamesMissingVideo>false</ScreensaverSkipGamesMissingVideo>#<ScreensaverSkipGamesMissingVideo>true</ScreensaverSkipGamesMissingVideo>#' \
  "$screensaver_disabled_settings"
sed -i \
  's#<ScreensaverView>Screensaver1View</ScreensaverView>#<ScreensaverView>Screensaver3View</ScreensaverView>#' \
  "$screensaver_disabled_settings"
sed -i \
  's#<VolumeVideo>75</VolumeVideo>#<VolumeVideo>40</VolumeVideo>#' \
  "$screensaver_disabled_settings"
cp "$screensaver_disabled_settings" \
  "$screensaver_disabled_settings.before-screensaver-smoke"
screensaver_disabled_manifest="$test_config_root/screensaver-disabled.before.sha256"
(
  cd "$screensaver_disabled_root"
  find Images Videos Manuals Music -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$screensaver_disabled_manifest"
screensaver_disabled_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --windowed \
    --library "$screensaver_disabled_root" \
    --bigbox-screensaver-disabled-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$screensaver_disabled_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_SCREENSAVER_SMOKE_COMPLETE enabled=0 candidates=1 swaps=[0-9]+ selections=[0-9]+ auto_delay_ms=0 manual=1 input_exit=[0-9]+ explore=0 views=1-2-3-4 video=h264 volume=20 range=300-500' \
  <<< "$screensaver_disabled_output"; then
  printf '%s\n' "$screensaver_disabled_output" >&2
  echo "BigBox did not keep automatic screensaver entry disabled while retaining manual start." >&2
  exit 1
fi
cmp "$screensaver_disabled_settings.before-screensaver-smoke" \
  "$screensaver_disabled_settings"
(
  cd "$screensaver_disabled_root"
  sha256sum --check "$screensaver_disabled_manifest"
) >/dev/null

echo "BigBox typed screensaver policy, guarded candidate projection, automatic and manual entry, bounded random swapping, all four recovered views, decoded video, composed volume, input return/explore behavior, disabled-auto behavior, and immutable library data validated."

background_music_root="$test_config_root/background-music-library"
background_music_screenshot="$test_config_root/bigbox-background-music.png"
mkdir -p "$background_music_root"
cp -a "$media_root/." "$background_music_root/"
background_music_settings="$background_music_root/Data/BigBoxSettings.xml"
sed -i \
  's#<EnableBackgroundMusic>false</EnableBackgroundMusic>#<EnableBackgroundMusic>true</EnableBackgroundMusic>#' \
  "$background_music_settings"
sed -i \
  's#<VolumeBackgroundMusic>75</VolumeBackgroundMusic>#<VolumeBackgroundMusic>63</VolumeBackgroundMusic>#' \
  "$background_music_settings"
sed -i \
  's#<ShuffleBackgroundMusic>true</ShuffleBackgroundMusic>#<ShuffleBackgroundMusic>false</ShuffleBackgroundMusic>#' \
  "$background_music_settings"
cp "$background_music_settings" \
  "$background_music_settings.before-background-music-smoke"
background_music_manifest="$test_config_root/background-music.before.sha256"
(
  cd "$background_music_root"
  find Images Videos Manuals Music -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$background_music_manifest"
background_music_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --windowed \
    --library "$background_music_root" \
    --bigbox-background-music-smoke-test \
    --bigbox-background-music-screenshot \
    "$background_music_screenshot" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$background_music_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_BACKGROUND_MUSIC_SMOKE_COMPLETE tracks=8 default=2 platform=2 playlist=2 category=2 audio=mp3 playlist_format=m3u controls=1 osd=1 video_audio=0' \
  <<< "$background_music_output"; then
  printf '%s\n' "$background_music_output" >&2
  echo "BigBox did not validate context-specific background music." >&2
  exit 1
fi
if [[ ! -s "$background_music_screenshot" ]] \
  || [[ $(wc -c < "$background_music_screenshot") -lt 1024 ]]; then
  echo "BigBox did not save a rendered background-music OSD screenshot." >&2
  exit 1
fi
background_music_colors=$(
  magick "$background_music_screenshot" -format '%k' info:
)
if [[ ! "$background_music_colors" =~ ^[0-9]+$ ]] \
  || ((background_music_colors < 64)); then
  echo "BigBox background-music screenshot is blank or insufficiently rendered ($background_music_colors colors)." >&2
  exit 1
fi
cmp "$background_music_settings.before-background-music-smoke" \
  "$background_music_settings"
(
  cd "$background_music_root"
  sha256sum --check "$background_music_manifest"
) >/dev/null

background_music_overlap_root="$test_config_root/background-music-overlap-library"
mkdir -p "$background_music_overlap_root"
cp -a "$media_root/." "$background_music_overlap_root/"
background_music_overlap_settings="$background_music_overlap_root/Data/BigBoxSettings.xml"
sed -i \
  's#<EnableBackgroundMusic>false</EnableBackgroundMusic>#<EnableBackgroundMusic>true</EnableBackgroundMusic>#' \
  "$background_music_overlap_settings"
sed -i \
  's#<VolumeBackgroundMusic>75</VolumeBackgroundMusic>#<VolumeBackgroundMusic>63</VolumeBackgroundMusic>#' \
  "$background_music_overlap_settings"
sed -i \
  's#<ShuffleBackgroundMusic>true</ShuffleBackgroundMusic>#<ShuffleBackgroundMusic>false</ShuffleBackgroundMusic>#' \
  "$background_music_overlap_settings"
sed -i \
  's#<PlayVideoAudioWithBackgroundMusic>false</PlayVideoAudioWithBackgroundMusic>#<PlayVideoAudioWithBackgroundMusic>true</PlayVideoAudioWithBackgroundMusic>#' \
  "$background_music_overlap_settings"
cp "$background_music_overlap_settings" \
  "$background_music_overlap_settings.before-background-music-smoke"
background_music_overlap_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --windowed \
    --library "$background_music_overlap_root" \
    --bigbox-background-music-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$background_music_overlap_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_BACKGROUND_MUSIC_SMOKE_COMPLETE tracks=8 default=2 platform=2 playlist=2 category=2 audio=mp3 playlist_format=m3u controls=1 osd=1 video_audio=1' \
  <<< "$background_music_overlap_output"; then
  printf '%s\n' "$background_music_overlap_output" >&2
  echo "BigBox did not retain background music with video audio enabled." >&2
  exit 1
fi
cmp "$background_music_overlap_settings.before-background-music-smoke" \
  "$background_music_overlap_settings"
(
  cd "$background_music_overlap_root"
  sha256sum --check "$background_music_manifest"
) >/dev/null

echo "BigBox default/platform/playlist/category background music, typed sound policy, context fallback, Qt decode, previous/pause/next controls, both video-audio coexistence policies, game-music interruption and resume, OSD rendering, and read-only media behavior validated."

startup_video_root="$test_config_root/startup-video-library"
startup_video_screenshot="$test_config_root/bigbox-startup-video.png"
mkdir -p "$startup_video_root"
cp -a "$media_root/." "$startup_video_root/"
mkdir -p "$startup_video_root/Videos/Startup"
cp "$fixture_video" "$startup_video_root/Videos/Startup/Startup-01.mp4"
cp "$fixture_video" "$startup_video_root/Videos/Startup/Startup-02.mp4"
startup_video_settings="$startup_video_root/Data/BigBoxSettings.xml"
sed -i \
  's#<VolumeVideo>75</VolumeVideo>#<VolumeVideo>61</VolumeVideo>#' \
  "$startup_video_settings"
cp "$startup_video_settings" \
  "$startup_video_settings.before-startup-video-smoke"
startup_video_manifest="$test_config_root/startup-video.before.sha256"
(
  cd "$startup_video_root"
  find Images Videos Manuals Music -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$startup_video_manifest"
startup_video_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --windowed \
    --library "$startup_video_root" \
    --bigbox-startup-video-smoke-test \
    --bigbox-startup-video-index 1 \
    --bigbox-startup-video-screenshot "$startup_video_screenshot" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$startup_video_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_STARTUP_VIDEO_SMOKE_COMPLETE videos=2 selected=Startup-02.mp4 decode=h264 completion=skip volume=61 probe_before_load=1' \
  <<< "$startup_video_output"; then
  printf '%s\n' "$startup_video_output" >&2
  echo "BigBox did not decode and skip the selected randomized startup video." >&2
  exit 1
fi
if [[ ! -s "$startup_video_screenshot" ]] \
  || [[ $(wc -c < "$startup_video_screenshot") -lt 1024 ]]; then
  echo "BigBox did not save a rendered startup-video screenshot." >&2
  exit 1
fi
startup_video_colors=$(
  magick "$startup_video_screenshot" -format '%k' info:
)
if [[ ! "$startup_video_colors" =~ ^[0-9]+$ ]] \
  || ((startup_video_colors < 64)); then
  echo "BigBox startup-video screenshot is blank or insufficiently rendered ($startup_video_colors colors)." >&2
  exit 1
fi

startup_video_natural_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --windowed \
    --library "$startup_video_root" \
    --bigbox-startup-video-smoke-test \
    --bigbox-startup-video-natural-end \
    --bigbox-startup-video-index 0 \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$startup_video_natural_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_STARTUP_VIDEO_SMOKE_COMPLETE videos=2 selected=Startup-01.mp4 decode=h264 completion=natural volume=61 probe_before_load=1' \
  <<< "$startup_video_natural_output"; then
  printf '%s\n' "$startup_video_natural_output" >&2
  echo "BigBox did not finish the selected randomized startup video naturally." >&2
  exit 1
fi
cmp "$startup_video_settings.before-startup-video-smoke" \
  "$startup_video_settings"
(
  cd "$startup_video_root"
  sha256sum --check "$startup_video_manifest"
) >/dev/null

legacy_startup_video_root="$test_config_root/legacy-startup-video-library"
mkdir -p "$legacy_startup_video_root"
cp -a "$media_root/." "$legacy_startup_video_root/"
cp "$fixture_video" "$legacy_startup_video_root/Videos/Startup.mp4"
legacy_startup_video_settings="$legacy_startup_video_root/Data/BigBoxSettings.xml"
sed -i \
  's#<VolumeVideo>75</VolumeVideo>#<VolumeVideo>61</VolumeVideo>#' \
  "$legacy_startup_video_settings"
cp "$legacy_startup_video_settings" \
  "$legacy_startup_video_settings.before-startup-video-smoke"
legacy_startup_video_manifest="$test_config_root/legacy-startup-video.before.sha256"
(
  cd "$legacy_startup_video_root"
  find Images Videos Manuals Music -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$legacy_startup_video_manifest"
legacy_startup_video_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --windowed \
    --library "$legacy_startup_video_root" \
    --bigbox-startup-video-smoke-test \
    --bigbox-startup-video-natural-end \
    --bigbox-startup-video-index 0 \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$legacy_startup_video_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_STARTUP_VIDEO_SMOKE_COMPLETE videos=1 selected=Startup.mp4 decode=h264 completion=natural volume=61 probe_before_load=1' \
  <<< "$legacy_startup_video_output"; then
  printf '%s\n' "$legacy_startup_video_output" >&2
  echo "BigBox did not decode and naturally finish the legacy startup video." >&2
  exit 1
fi
cmp "$legacy_startup_video_settings.before-startup-video-smoke" \
  "$legacy_startup_video_settings"
(
  cd "$legacy_startup_video_root"
  sha256sum --check "$legacy_startup_video_manifest"
) >/dev/null

startup_splash_root="$test_config_root/startup-splash-library"
startup_splash_screenshot="$test_config_root/bigbox-startup-splash.png"
startup_splash_sound_folder="$startup_splash_root/Sounds/Fixture Sounds/Startup"
mkdir -p "$startup_splash_root"
cp -a "$media_root/." "$startup_splash_root/"
mkdir -p "$startup_splash_sound_folder"
cp "$fixture_startup_sound" \
  "$startup_splash_sound_folder/STARTUP001.wav"
cp "$fixture_startup_sound" \
  "$startup_splash_sound_folder/STARTUP002.wav"
cp "$fixture_startup_sound" \
  "$startup_splash_root/Sounds/Fixture Sounds/Startup.wav"
startup_splash_settings="$startup_splash_root/Data/BigBoxSettings.xml"
cp "$startup_splash_settings" \
  "$startup_splash_settings.before-startup-splash-smoke"
startup_splash_manifest="$test_config_root/startup-splash.before.sha256"
(
  cd "$startup_splash_root"
  find Images Videos Manuals Music Sounds -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$startup_splash_manifest"
startup_splash_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --windowed \
    --library "$startup_splash_root" \
    --bigbox-startup-splash-smoke-test \
    --bigbox-startup-sound-index 1 \
    --bigbox-startup-splash-screenshot "$startup_splash_screenshot" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$startup_splash_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_STARTUP_SPLASH_SMOKE_COMPLETE enabled=1 sounds=2 selected=STARTUP002.wav decode=wav probe_before_load=1 splash=1 audio=1 volume=32' \
  <<< "$startup_splash_output"; then
  printf '%s\n' "$startup_splash_output" >&2
  echo "BigBox did not render its startup splash and decode the selected randomized startup sound." >&2
  exit 1
fi
if [[ ! -s "$startup_splash_screenshot" ]] \
  || [[ $(wc -c < "$startup_splash_screenshot") -lt 1024 ]]; then
  echo "BigBox did not save a rendered startup-splash screenshot." >&2
  exit 1
fi
startup_splash_colors=$(
  magick "$startup_splash_screenshot" -format '%k' info:
)
if [[ ! "$startup_splash_colors" =~ ^[0-9]+$ ]] \
  || ((startup_splash_colors < 64)); then
  echo "BigBox startup-splash screenshot is blank or insufficiently rendered ($startup_splash_colors colors)." >&2
  exit 1
fi
cmp "$startup_splash_settings.before-startup-splash-smoke" \
  "$startup_splash_settings"
(
  cd "$startup_splash_root"
  sha256sum --check "$startup_splash_manifest"
) >/dev/null

startup_splash_disabled_root="$test_config_root/startup-splash-disabled-library"
mkdir -p "$startup_splash_disabled_root"
cp -a "$startup_splash_root/." "$startup_splash_disabled_root/"
startup_splash_disabled_settings="$startup_splash_disabled_root/Data/BigBoxSettings.xml"
sed -i \
  's#<ShowStartupSplashScreen>true</ShowStartupSplashScreen>#<ShowStartupSplashScreen>false</ShowStartupSplashScreen>#' \
  "$startup_splash_disabled_settings"
sed -i \
  's#<PlayStartupSound>true</PlayStartupSound>#<PlayStartupSound>false</PlayStartupSound>#' \
  "$startup_splash_disabled_settings"
cp "$startup_splash_disabled_settings" \
  "$startup_splash_disabled_settings.before-startup-splash-smoke"
startup_splash_disabled_manifest="$test_config_root/startup-splash-disabled.before.sha256"
(
  cd "$startup_splash_disabled_root"
  find Images Videos Manuals Music Sounds -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$startup_splash_disabled_manifest"
startup_splash_disabled_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --windowed \
    --library "$startup_splash_disabled_root" \
    --bigbox-startup-splash-disabled-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$startup_splash_disabled_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_STARTUP_SPLASH_SMOKE_COMPLETE enabled=0 sounds=2 selected=none decode=wav probe_before_load=1 splash=0 audio=0 volume=32' \
  <<< "$startup_splash_disabled_output"; then
  printf '%s\n' "$startup_splash_disabled_output" >&2
  echo "BigBox did not honor disabled startup splash and sound settings." >&2
  exit 1
fi
cmp "$startup_splash_disabled_settings.before-startup-splash-smoke" \
  "$startup_splash_disabled_settings"
(
  cd "$startup_splash_disabled_root"
  sha256sum --check "$startup_splash_disabled_manifest"
) >/dev/null

legacy_startup_sound_root="$test_config_root/legacy-startup-sound-library"
legacy_startup_sound_folder="$legacy_startup_sound_root/Sounds/Fixture Sounds"
mkdir -p "$legacy_startup_sound_root"
cp -a "$media_root/." "$legacy_startup_sound_root/"
mkdir -p "$legacy_startup_sound_folder"
cp "$fixture_startup_sound" \
  "$legacy_startup_sound_folder/Startup.wav"
legacy_startup_sound_settings="$legacy_startup_sound_root/Data/BigBoxSettings.xml"
cp "$legacy_startup_sound_settings" \
  "$legacy_startup_sound_settings.before-startup-splash-smoke"
legacy_startup_sound_manifest="$test_config_root/legacy-startup-sound.before.sha256"
(
  cd "$legacy_startup_sound_root"
  find Images Videos Manuals Music Sounds -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum
) > "$legacy_startup_sound_manifest"
legacy_startup_sound_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --windowed \
    --library "$legacy_startup_sound_root" \
    --bigbox-startup-splash-smoke-test \
    --bigbox-startup-sound-index 0 \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$legacy_startup_sound_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_STARTUP_SPLASH_SMOKE_COMPLETE enabled=1 sounds=1 selected=Startup.wav decode=wav probe_before_load=1 splash=1 audio=1 volume=32' \
  <<< "$legacy_startup_sound_output"; then
  printf '%s\n' "$legacy_startup_sound_output" >&2
  echo "BigBox did not decode the legacy single-file startup sound." >&2
  exit 1
fi
cmp "$legacy_startup_sound_settings.before-startup-splash-smoke" \
  "$legacy_startup_sound_settings"
(
  cd "$legacy_startup_sound_root"
  sha256sum --check "$legacy_startup_sound_manifest"
) >/dev/null

echo "BigBox early startup probe, randomized and legacy startup-video discovery, native file URLs, real H.264 decode, shared key/tap skip action, natural completion, typed video volume, port-owned cross-platform splash rendering, randomized-folder and legacy WAV startup sounds, disabled presentation settings, master-adjusted sound volume, and read-only media behavior validated."

game_details_settings="$media_root/Data/Settings.xml"
game_details_screenshot="$media_root/launchbox-game-details.png"
cp "$game_details_settings" "$game_details_settings.before-game-details-smoke"
game_details_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$media_root" \
    --game-details-smoke-test \
    --game-details-screenshot "$game_details_screenshot" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$game_details_output" >&2
  exit 1
}
if ! rg -q \
  'GAME_DETAILS_SMOKE_COMPLETE id=fixture-adventure play_count=3 play_time=5400 community_rating=4.25 applications=1 saves=1' \
  <<< "$game_details_output"; then
  printf '%s\n' "$game_details_output" >&2
  echo "LaunchBox did not validate the selected-game details pane." >&2
  exit 1
fi
if [[ ! -s "$game_details_screenshot" ]] \
  || [[ $(wc -c < "$game_details_screenshot") -lt 1024 ]] \
  || [[ $(od -An -tx1 -N8 "$game_details_screenshot" | tr -d ' \n') \
    != 89504e470d0a1a0a ]]; then
  printf '%s\n' "$game_details_output" >&2
  echo "LaunchBox did not render a valid selected-game details PNG." >&2
  exit 1
fi
cmp "$media_platform.before-media-smoke" "$media_platform"
cmp "$game_details_settings.before-game-details-smoke" "$game_details_settings"

echo "LaunchBox stable-ID selection, resizable game details, metadata actions, play statistics, community rating, and rendered artwork validated without library writes."

game_details_media_screenshot="$test_config_root/launchbox-game-details-media.png"
game_details_media_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$media_root" \
    --game-details-media-smoke-test \
    --game-details-media-screenshot "$game_details_media_screenshot" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$game_details_media_output" >&2
  exit 1
}
if ! rg -q \
  'GAME_DETAILS_MEDIA_SMOKE_COMPLETE id=fixture-adventure items=10 image=Box-Front full=Box-Full video=Video-Snap autoplay=1' \
  <<< "$game_details_media_output"; then
  printf '%s\n' "$game_details_media_output" >&2
  echo "LaunchBox did not validate selected-game image and video media." >&2
  exit 1
fi
if [[ ! -s "$game_details_media_screenshot" ]] \
  || [[ $(wc -c < "$game_details_media_screenshot") -lt 1024 ]] \
  || [[ $(od -An -tx1 -N8 "$game_details_media_screenshot" | tr -d ' \n') \
    != 89504e470d0a1a0a ]]; then
  printf '%s\n' "$game_details_media_output" >&2
  echo "LaunchBox did not render a valid selected-game media PNG." >&2
  exit 1
fi
cmp "$media_platform.before-media-smoke" "$media_platform"
cmp "$game_details_settings.before-game-details-smoke" "$game_details_settings"
(
  cd "$media_root"
  sha256sum --check "$media_files_manifest"
) >/dev/null

echo "LaunchBox selected-game image thumbnails, decoded H.264 video, autoplay, real play/pause controls, selection, and preview rendering validated without media or library writes."

launchbox_image_viewer_screenshot="$test_config_root/launchbox-image-viewer.png"
launchbox_image_viewer_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$media_root" \
    --launchbox-image-viewer-smoke-test \
    --launchbox-image-viewer-screenshot \
    "$launchbox_image_viewer_screenshot" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$launchbox_image_viewer_output" >&2
  exit 1
}
if ! rg -q \
  'LAUNCHBOX_IMAGE_VIEWER_SMOKE_COMPLETE id=fixture-adventure images=8 first=Box-Front next=Screenshot-Gameplay zoom=1 pan=1 switch=1 controls=1' \
  <<< "$launchbox_image_viewer_output"; then
  printf '%s\n' "$launchbox_image_viewer_output" >&2
  echo "LaunchBox did not validate its full-screen image viewer." >&2
  exit 1
fi
if [[ ! -s "$launchbox_image_viewer_screenshot" ]] \
  || [[ $(wc -c < "$launchbox_image_viewer_screenshot") -lt 1024 ]] \
  || [[ $(od -An -tx1 -N8 "$launchbox_image_viewer_screenshot" \
      | tr -d ' \n') != 89504e470d0a1a0a ]]; then
  printf '%s\n' "$launchbox_image_viewer_output" >&2
  echo "LaunchBox did not render a valid zoomed full-screen image PNG." >&2
  exit 1
fi
cmp "$media_platform.before-media-smoke" "$media_platform"
cmp "$game_details_settings.before-game-details-smoke" "$game_details_settings"
(
  cd "$media_root"
  sha256sum --check "$media_files_manifest"
) >/dev/null

echo "LaunchBox full-screen image entry, image-type switching, bounded zoom, fit reset, pan, focus return, native-path rendering, and read-only media behavior validated."

bigbox_game_details_media_screenshot="$test_config_root/bigbox-game-details-media.png"
bigbox_game_details_media_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --library "$media_root" \
    --windowed \
    --bigbox-game-details-media-smoke-test \
    --bigbox-game-details-media-screenshot \
    "$bigbox_game_details_media_screenshot" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$bigbox_game_details_media_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_GAME_DETAILS_MEDIA_SMOKE_COMPLETE id=fixture-adventure items=10 image=Box-Front full=Box-Full video=Video-Snap autoplay=1 controls=1' \
  <<< "$bigbox_game_details_media_output"; then
  printf '%s\n' "$bigbox_game_details_media_output" >&2
  echo "BigBox did not validate its full-screen selected-game media controls." >&2
  exit 1
fi
if [[ ! -s "$bigbox_game_details_media_screenshot" ]] \
  || [[ $(wc -c < "$bigbox_game_details_media_screenshot") -lt 1024 ]] \
  || [[ $(od -An -tx1 -N8 "$bigbox_game_details_media_screenshot" \
      | tr -d ' \n') != 89504e470d0a1a0a ]]; then
  printf '%s\n' "$bigbox_game_details_media_output" >&2
  echo "BigBox did not render a valid full-screen selected-game media PNG." >&2
  exit 1
fi
cmp "$media_platform.before-media-smoke" "$media_platform"
cmp "$game_details_settings.before-game-details-smoke" "$game_details_settings"
(
  cd "$media_root"
  sha256sum --check "$media_files_manifest"
) >/dev/null

echo "BigBox full-screen selected-game details, image/video thumbnails, decoded H.264 autoplay, real previous/play-pause/back controls, keyboard media navigation, stopped-on-close lifecycle, and preview rendering validated without media or library writes."

bigbox_image_viewer_screenshot="$test_config_root/bigbox-image-viewer.png"
bigbox_image_viewer_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --library "$media_root" \
    --windowed \
    --bigbox-image-viewer-smoke-test \
    --bigbox-image-viewer-screenshot \
    "$bigbox_image_viewer_screenshot" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$bigbox_image_viewer_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_IMAGE_VIEWER_SMOKE_COMPLETE id=fixture-adventure images=8 first=Box-Front next=Screenshot-Gameplay zoom=1 pan=1 switch=1 controls=1' \
  <<< "$bigbox_image_viewer_output"; then
  printf '%s\n' "$bigbox_image_viewer_output" >&2
  echo "BigBox did not validate its standalone full-screen image viewer." >&2
  exit 1
fi
if [[ ! -s "$bigbox_image_viewer_screenshot" ]] \
  || [[ $(wc -c < "$bigbox_image_viewer_screenshot") -lt 1024 ]] \
  || [[ $(od -An -tx1 -N8 "$bigbox_image_viewer_screenshot" \
      | tr -d ' \n') != 89504e470d0a1a0a ]]; then
  printf '%s\n' "$bigbox_image_viewer_output" >&2
  echo "BigBox did not render a valid zoomed full-screen image PNG." >&2
  exit 1
fi
cmp "$media_platform.before-media-smoke" "$media_platform"
cmp "$game_details_settings.before-game-details-smoke" "$game_details_settings"
(
  cd "$media_root"
  sha256sum --check "$media_files_manifest"
) >/dev/null

echo "BigBox standalone image entry, image-type switching, bounded zoom, fit reset, pan, nested back navigation, native-path rendering, and read-only media behavior validated."

box_flip_bigbox_settings="$media_root/Data/BigBoxSettings.xml"
cp "$box_flip_bigbox_settings" \
  "$box_flip_bigbox_settings.before-box-flip-smoke"
launchbox_box_flip_screenshot="$test_config_root/launchbox-box-flip.png"
launchbox_box_flip_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$media_root" \
    --launchbox-box-flip-smoke-test \
    --launchbox-box-flip-screenshot \
    "$launchbox_box_flip_screenshot" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$launchbox_box_flip_output" >&2
  exit 1
}
if ! rg -q \
  'LAUNCHBOX_BOX_FLIP_SMOKE_COMPLETE id=fixture-adventure front=Box-Front back=Box-Back flip=1 return=1 controls=1' \
  <<< "$launchbox_box_flip_output"; then
  printf '%s\n' "$launchbox_box_flip_output" >&2
  echo "LaunchBox did not validate the grid box-flip workflow." >&2
  exit 1
fi
if [[ ! -s "$launchbox_box_flip_screenshot" ]] \
  || [[ $(wc -c < "$launchbox_box_flip_screenshot") -lt 1024 ]] \
  || [[ $(od -An -tx1 -N8 "$launchbox_box_flip_screenshot" \
      | tr -d ' \n') != 89504e470d0a1a0a ]]; then
  printf '%s\n' "$launchbox_box_flip_output" >&2
  echo "LaunchBox did not render a valid flipped box-back PNG." >&2
  exit 1
fi
cmp "$media_platform.before-media-smoke" "$media_platform"
cmp "$game_details_settings.before-game-details-smoke" \
  "$game_details_settings"
cmp "$box_flip_bigbox_settings.before-box-flip-smoke" \
  "$box_flip_bigbox_settings"
(
  cd "$media_root"
  sha256sum --check "$media_files_manifest"
) >/dev/null

echo "LaunchBox settings-prioritized front/back box selection, real Flip control, animated return, native-path rendering, and read-only media behavior validated."

bigbox_box_flip_screenshot="$test_config_root/bigbox-box-flip.png"
bigbox_box_flip_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --library "$media_root" \
    --windowed \
    --bigbox-box-flip-smoke-test \
    --bigbox-box-flip-screenshot \
    "$bigbox_box_flip_screenshot" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$bigbox_box_flip_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_BOX_FLIP_SMOKE_COMPLETE id=fixture-adventure front=Box-Front back=Box-Back flip=1 return=1 controls=1' \
  <<< "$bigbox_box_flip_output"; then
  printf '%s\n' "$bigbox_box_flip_output" >&2
  echo "BigBox did not validate the game-wheel box-flip workflow." >&2
  exit 1
fi
if [[ ! -s "$bigbox_box_flip_screenshot" ]] \
  || [[ $(wc -c < "$bigbox_box_flip_screenshot") -lt 1024 ]] \
  || [[ $(od -An -tx1 -N8 "$bigbox_box_flip_screenshot" \
      | tr -d ' \n') != 89504e470d0a1a0a ]]; then
  printf '%s\n' "$bigbox_box_flip_output" >&2
  echo "BigBox did not render a valid full-window box-back PNG." >&2
  exit 1
fi
cmp "$media_platform.before-media-smoke" "$media_platform"
cmp "$game_details_settings.before-game-details-smoke" \
  "$game_details_settings"
cmp "$box_flip_bigbox_settings.before-box-flip-smoke" \
  "$box_flip_bigbox_settings"
(
  cd "$media_root"
  sha256sum --check "$media_files_manifest"
) >/dev/null

echo "BigBox settings-prioritized front/back box selection, real Flip control, F shortcut contract, animated return, native-path rendering, and read-only media behavior validated."

model_viewer_state="$test_config_root/model-viewer-state.json"
launchbox_model_viewer_screenshot="$test_config_root/launchbox-model-viewer.png"
model_viewer_visual_smoke=true
if "$nix_build_sandbox"; then
  # Nix's Linux sandbox intentionally withholds host graphics devices. Xvfb
  # can exercise the Qt Quick 3D scene and controls there, but Qt cannot
  # capture a rendered QQuickWindow. The ordinary developer check below
  # remains the real rendered-pixel gate.
  model_viewer_visual_smoke=false
fi
launchbox_model_viewer_args=(
  --library "$media_root"
  --launchbox-model-viewer-smoke-test
  --model-viewer-state-file "$model_viewer_state"
  --path-mappings-file "$empty_path_mappings"
)
if "$model_viewer_visual_smoke"; then
  launchbox_model_viewer_args+=(
    --launchbox-model-viewer-screenshot
    "$launchbox_model_viewer_screenshot"
  )
fi
launchbox_model_viewer_output=$(
  run_rendered_smoke "$binary_dir/launchbox" \
    "${launchbox_model_viewer_args[@]}" 2>&1
) || {
  printf '%s\n' "$launchbox_model_viewer_output" >&2
  exit 1
}
if ! rg -q \
  'LAUNCHBOX_MODEL_VIEWER_SMOKE_COMPLETE id=fixture-adventure type=jewelCase source=gameOverride fullscan=Box-Full spine=0.143 size=260x230x20 geometry=6 faces=front,back,spine rotate=1 pan=1 zoom=1 lock=horizontal controls=1' \
  <<< "$launchbox_model_viewer_output"; then
  printf '%s\n' "$launchbox_model_viewer_output" >&2
  echo "LaunchBox did not validate its resolved interactive 3D model viewer." >&2
  exit 1
fi
if "$model_viewer_visual_smoke"; then
  if [[ ! -s "$launchbox_model_viewer_screenshot" ]] \
    || [[ $(wc -c < "$launchbox_model_viewer_screenshot") -lt 1024 ]] \
    || [[ $(od -An -tx1 -N8 "$launchbox_model_viewer_screenshot" \
        | tr -d ' \n') != 89504e470d0a1a0a ]]; then
    printf '%s\n' "$launchbox_model_viewer_output" >&2
    echo "LaunchBox did not render a valid interactive 3D model PNG." >&2
    exit 1
  fi
  validate_rendered_model_viewport "$launchbox_model_viewer_screenshot"
fi
if ! rg -q '"rotation_lock": "horizontal"' "$model_viewer_state"; then
  cat "$model_viewer_state" >&2
  echo "LaunchBox did not persist its horizontal model-rotation lock." >&2
  exit 1
fi
cmp "$media_platform.before-media-smoke" "$media_platform"
cmp "$game_details_settings.before-game-details-smoke" \
  "$game_details_settings"
cmp "$box_flip_bigbox_settings.before-box-flip-smoke" \
  "$box_flip_bigbox_settings"
(
  cd "$media_root"
  sha256sum --check "$media_files_manifest"
) >/dev/null

echo "LaunchBox resolved game-over-platform jewel settings, full-scan back/spine/front construction, exact port geometry, six-face Qt Quick 3D textures, actual details entry, rotation, translation, zoom, horizontal lock, focus return, and read-only media behavior validated."

bigbox_model_viewer_screenshot="$test_config_root/bigbox-model-viewer.png"
bigbox_model_viewer_args=(
  --library "$media_root"
  --windowed
  --bigbox-model-viewer-smoke-test
  --model-viewer-state-file "$model_viewer_state"
  --path-mappings-file "$empty_path_mappings"
)
if "$model_viewer_visual_smoke"; then
  bigbox_model_viewer_args+=(
    --bigbox-model-viewer-screenshot
    "$bigbox_model_viewer_screenshot"
  )
fi
bigbox_model_viewer_output=$(
  run_rendered_smoke "$binary_dir/bigbox" \
    "${bigbox_model_viewer_args[@]}" 2>&1
) || {
  printf '%s\n' "$bigbox_model_viewer_output" >&2
  exit 1
}
if ! rg -q \
  'BIGBOX_MODEL_VIEWER_SMOKE_COMPLETE id=fixture-adventure type=jewelCase source=gameOverride fullscan=Box-Full spine=0.143 size=260x230x20 geometry=6 faces=front,back,spine rotate=1 pan=1 zoom=1 restored=horizontal lock=vertical controls=1' \
  <<< "$bigbox_model_viewer_output"; then
  printf '%s\n' "$bigbox_model_viewer_output" >&2
  echo "BigBox did not validate its resolved interactive 3D model viewer." >&2
  exit 1
fi
if "$model_viewer_visual_smoke"; then
  if [[ ! -s "$bigbox_model_viewer_screenshot" ]] \
    || [[ $(wc -c < "$bigbox_model_viewer_screenshot") -lt 1024 ]] \
    || [[ $(od -An -tx1 -N8 "$bigbox_model_viewer_screenshot" \
        | tr -d ' \n') != 89504e470d0a1a0a ]]; then
    printf '%s\n' "$bigbox_model_viewer_output" >&2
    echo "BigBox did not render a valid interactive 3D model PNG." >&2
    exit 1
  fi
  validate_rendered_model_viewport "$bigbox_model_viewer_screenshot"
fi
expected_model_viewer_state='{
  "version": 1,
  "rotation_lock": "vertical"
}'
if [[ $(< "$model_viewer_state") != "$expected_model_viewer_state" ]]; then
  cat "$model_viewer_state" >&2
  echo "BigBox did not restore and atomically replace shared model-viewer state." >&2
  exit 1
fi
cmp "$media_platform.before-media-smoke" "$media_platform"
cmp "$game_details_settings.before-game-details-smoke" \
  "$game_details_settings"
cmp "$box_flip_bigbox_settings.before-box-flip-smoke" \
  "$box_flip_bigbox_settings"
(
  cd "$media_root"
  sha256sum --check "$media_files_manifest"
) >/dev/null

echo "BigBox resolved game-over-platform jewel settings, exact port geometry, six-face Qt Quick 3D textures, game-menu entry, restored horizontal lock, vertical lock replacement, keyboard/pointer control surface, focus return, and read-only media behavior validated."
if ! "$model_viewer_visual_smoke"; then
  echo "Qt Quick 3D rendered-pixel capture is unavailable inside the Nix sandbox; run scripts/check_qml.sh from nix develop for the visual gate."
fi

game_details_ui_state="$test_config_root/game-details-ui-state.json"
expected_game_details_ui_state="$test_config_root/expected-game-details-ui-state.json"
game_details_layout_screenshot="$test_config_root/launchbox-popped-out-game-details.png"
cat > "$expected_game_details_ui_state" <<'EOF'
{
  "version": 1,
  "show_game_details": true,
  "game_details_popped_out": true,
  "game_details_pane_width": 420,
  "game_details_window": {
    "x": 140,
    "y": 100,
    "width": 640,
    "height": 560,
    "maximized": false
  },
  "list_view_column_widths": {
    "Added": 140,
    "Alternate Names": 220,
    "Application Path": 300,
    "Badges": 90,
    "Broken": 90,
    "Community Star Rating": 170,
    "Community Star Rating Count": 190,
    "Completed": 110,
    "Developer": 180,
    "Favorite": 95,
    "Genre": 150,
    "Hide": 80,
    "Installed": 95,
    "Last Played": 140,
    "Launchbox Database ID": 180,
    "Max Players": 110,
    "Modified": 140,
    "Platform": 150,
    "Play Count": 110,
    "Play Mode": 120,
    "Play Time": 120,
    "Portable": 95,
    "Publisher": 180,
    "Rating": 90,
    "Region": 100,
    "Release Date": 120,
    "Release Type": 120,
    "Series": 150,
    "Source": 120,
    "Star Rating": 110,
    "Status": 120,
    "Title": 260,
    "Version": 100,
    "Video URL": 260,
    "Wikipedia URL": 260
  }
}
EOF
game_details_layout_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$media_root" \
    --game-details-layout-smoke-test \
    --game-details-layout-screenshot "$game_details_layout_screenshot" \
    --ui-state-file "$game_details_ui_state" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$game_details_layout_output" >&2
  exit 1
}
if ! rg -q \
  'GAME_DETAILS_LAYOUT_SMOKE_COMPLETE reload=0 pane_width=420 window=140,100,640,560 popped_out=1' \
  <<< "$game_details_layout_output"; then
  printf '%s\n' "$game_details_layout_output" >&2
  echo "LaunchBox did not validate dock, hide, and pop-out transitions." >&2
  exit 1
fi
if ! cmp -s "$expected_game_details_ui_state" "$game_details_ui_state"; then
  printf '%s\n' "$game_details_layout_output" >&2
  diff -u "$expected_game_details_ui_state" "$game_details_ui_state" >&2 || true
  echo "LaunchBox persisted the wrong host-specific details layout." >&2
  exit 1
fi
if [[ ! -s "$game_details_layout_screenshot" ]] \
  || [[ $(wc -c < "$game_details_layout_screenshot") -lt 1024 ]] \
  || [[ $(od -An -tx1 -N8 "$game_details_layout_screenshot" | tr -d ' \n') \
    != 89504e470d0a1a0a ]]; then
  printf '%s\n' "$game_details_layout_output" >&2
  echo "LaunchBox did not render a valid popped-out details PNG." >&2
  exit 1
fi
game_details_layout_reload_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$media_root" \
    --game-details-layout-reload-smoke-test \
    --ui-state-file "$game_details_ui_state" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$game_details_layout_reload_output" >&2
  exit 1
}
if ! rg -q \
  'GAME_DETAILS_LAYOUT_SMOKE_COMPLETE reload=1 pane_width=420 window=140,100,640,560 popped_out=1' \
  <<< "$game_details_layout_reload_output"; then
  printf '%s\n' "$game_details_layout_reload_output" >&2
  echo "LaunchBox did not restore the persisted details window in a new process." >&2
  exit 1
fi
if ! cmp -s "$expected_game_details_ui_state" "$game_details_ui_state"; then
  echo "LaunchBox changed its host-specific details state during reload." >&2
  exit 1
fi
if find "$test_config_root" -maxdepth 1 -type f \
  -name '.game-details-ui-state.json.lbport-*.tmp' -print -quit | rg -q .; then
  echo "LaunchBox left an atomic UI-state temporary behind." >&2
  exit 1
fi
cmp "$media_platform.before-media-smoke" "$media_platform"
cmp "$game_details_settings.before-game-details-smoke" "$game_details_settings"

echo "LaunchBox dock, hide, pop-out, popup geometry, atomic host-state persistence, and new-process restoration validated without shared-library writes."

bigbox_navigation_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/bigbox" \
    --windowed --library "$workspace_root/fixtures/launchbox" \
    --navigation-smoke-test --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$bigbox_navigation_output" >&2
  exit 1
}
if ! rg -q 'BIGBOX_NAVIGATION_SMOKE_COMPLETE entries=3 playlist=1 category=3 platform=3' \
  <<< "$bigbox_navigation_output"; then
  printf '%s\n' "$bigbox_navigation_output" >&2
  echo "BigBox did not validate category, platform, and playlist navigation." >&2
  exit 1
fi

echo "BigBox category/platform/playlist navigation and exact membership filtering validated."

edit_root=$(mktemp -d)
library_filter_root=$(mktemp -d)
launchbox_order_root=$(mktemp -d)
bigbox_order_root=$(mktemp -d)
launchbox_list_root=$(mktemp -d)
launchbox_box_size_root=$(mktemp -d)
launchbox_desktop_tray_root=$(mktemp -d)
crud_root=$(mktemp -d)
additional_application_crud_root=$(mktemp -d)
additional_application_default_root=$(mktemp -d)
game_save_metadata_root=$(mktemp -d)
retroarch_save_scan_root=$(mktemp -d)
dolphin_save_scan_root=$(mktemp -d)
pcsx2_save_scan_root=$(mktemp -d)
game_save_backup_root=$(mktemp -d)
pcsx2_save_backup_root=$(mktemp -d)
pcsx2_save_lifecycle_root=$(mktemp -d)
dolphin_wii_save_lifecycle_root=$(mktemp -d)
game_save_delete_root=$(mktemp -d)
game_save_active_delete_root=$(mktemp -d)
game_save_restore_root=$(mktemp -d)
game_save_saturn_restore_root=$(mktemp -d)
import_root=$(mktemp -d)
import_source_root=$(mktemp -d)
platform_crud_root=$(mktemp -d)
emulator_crud_root=$(mktemp -d)
retroarch_core_editor_root=$(mktemp -d)
emulator_discovery_root=$(mktemp -d)
emulator_bios_root=$(mktemp -d)
emulator_install_root=$(mktemp -d)
emulator_release_fixture_root=$(mktemp -d)
category_crud_root=$(mktemp -d)
playlist_crud_root=$(mktemp -d)
game_grouping_root=$(mktemp -d)
emulator_launch_root=$(mktemp -d)
disabled_lifecycle_root=$(mktemp -d)
short_lifecycle_root=$(mktemp -d)
direct_launch_root=$(mktemp -d)
desktop_command_root=$(mktemp -d)
sequence_launch_root=$(mktemp -d)
archive_launch_root=$(mktemp -d)
m3u_launch_root=$(mktemp -d)
dosbox_launch_root=$(mktemp -d)
scummvm_launch_root=$(mktemp -d)
trap 'rm -rf "$test_config_root" "$media_root" "$edit_root" "$library_filter_root" "$launchbox_order_root" "$bigbox_order_root" "$launchbox_list_root" "$launchbox_box_size_root" "$launchbox_desktop_tray_root" "$crud_root" "$additional_application_crud_root" "$additional_application_default_root" "$game_save_metadata_root" "$retroarch_save_scan_root" "$dolphin_save_scan_root" "$pcsx2_save_scan_root" "$game_save_backup_root" "$pcsx2_save_backup_root" "$pcsx2_save_lifecycle_root" "$dolphin_wii_save_lifecycle_root" "$game_save_delete_root" "$game_save_active_delete_root" "$game_save_restore_root" "$game_save_saturn_restore_root" "$import_root" "$import_source_root" "$platform_crud_root" "$emulator_crud_root" "$retroarch_core_editor_root" "$emulator_discovery_root" "$emulator_bios_root" "$emulator_install_root" "$emulator_release_fixture_root" "$category_crud_root" "$playlist_crud_root" "$game_grouping_root" "$emulator_launch_root" "$disabled_lifecycle_root" "$short_lifecycle_root" "$direct_launch_root" "$desktop_command_root" "$sequence_launch_root" "$archive_launch_root" "$m3u_launch_root" "$dosbox_launch_root" "$scummvm_launch_root"' EXIT

cp -a fixtures/launchbox/. "$library_filter_root/"
library_filter_platform="$library_filter_root/Data/Platforms/Fixture Console.xml"
sed -i \
  '/<ApplicationPath>Games\\Fixture Racer\\racer\.rom<\/ApplicationPath>/,/<ID>fixture-racer<\/ID>/ s|<Broken>false</Broken>|<Broken>true</Broken>|' \
  "$library_filter_platform"
sed -i \
  '/<ID>fixture-racer<\/ID>/a\    <Installed>false</Installed>\n    <MissingVideo>true</MissingVideo>' \
  "$library_filter_platform"
sed -i \
  '/<ApplicationPath>Games\\Fixture Puzzle\\puzzle\.rom<\/ApplicationPath>/,/<ID>fixture-puzzle<\/ID>/ s|<Hide>false</Hide>|<Hide>true</Hide>|' \
  "$library_filter_platform"
cp "$library_filter_platform" "$library_filter_platform.before-filter-smoke"

for shell in launchbox bigbox; do
  arguments=(
    --library "$library_filter_root"
    --library-filter-smoke-test
    --path-mappings-file "$empty_path_mappings"
  )
  if [[ "$shell" == bigbox ]]; then
    arguments+=(--windowed)
  fi
  output=$(QT_QPA_PLATFORM=offscreen \
    "$binary_dir/$shell" "${arguments[@]}" 2>&1) || {
    printf '%s\n' "$output" >&2
    exit 1
  }
  if ! rg -q \
    'LIBRARY_FILTER_SMOKE_COMPLETE games=3 visible=1 resets=' \
    <<< "$output"; then
    printf '%s\n' "$output" >&2
    echo "$shell did not validate combined state and missing-media filtering." >&2
    exit 1
  fi
  cmp "$library_filter_platform.before-filter-smoke" \
    "$library_filter_platform"
done

echo "LaunchBox and BigBox combined state, visibility, and all missing-media filter controls validated without library writes."

cp -a fixtures/launchbox/. "$launchbox_order_root/"
cp -a fixtures/launchbox/. "$bigbox_order_root/"
for shell in launchbox bigbox; do
  if [[ "$shell" == launchbox ]]; then
    order_root=$launchbox_order_root
  else
    order_root=$bigbox_order_root
  fi
  order_platform="$order_root/Data/Platforms/Fixture Console.xml"
  cp "$order_platform" "$order_platform.before-order-smoke"
  arguments=(
    --library "$order_root"
    --library-order-smoke-test
    --path-mappings-file "$empty_path_mappings"
  )
  if [[ "$shell" == bigbox ]]; then
    arguments+=(--windowed)
  fi
  output=$(QT_QPA_PLATFORM=offscreen \
    "$binary_dir/$shell" "${arguments[@]}" 2>&1) || {
    printf '%s\n' "$output" >&2
    exit 1
  }
  if ! rg -q \
    'LIBRARY_ORDER_SMOKE_COMPLETE games=3 sort=PlayCount descending=true random_row=' \
    <<< "$output"; then
    printf '%s\n' "$output" >&2
    echo "$shell did not validate typed sorting and random selection." >&2
    exit 1
  fi
  if ! rg -q '<SortBy>PlayCount</SortBy>' "$order_root/Data/Settings.xml" \
    || ! rg -q '<SortByDesc>true</SortByDesc>' "$order_root/Data/Settings.xml"; then
    echo "$shell did not persist its LaunchBox-compatible Arrange By settings." >&2
    exit 1
  fi
  cmp "$order_platform.before-order-smoke" "$order_platform"
done

echo "LaunchBox and BigBox typed Arrange By, atomic Settings.xml persistence, stable ordering, and random selection validated."

cp -a fixtures/launchbox/. "$launchbox_list_root/"
list_platform="$launchbox_list_root/Data/Platforms/Fixture Console.xml"
list_settings="$launchbox_list_root/Data/Settings.xml"
list_screenshot="$launchbox_list_root/launchbox-list-view.png"
list_ui_state="$launchbox_list_root/list-view-ui-state.json"
expected_list_ui_state="$launchbox_list_root/expected-list-view-ui-state.json"
cat > "$expected_list_ui_state" <<'EOF'
{
  "version": 1,
  "show_game_details": true,
  "game_details_popped_out": false,
  "game_details_pane_width": 360,
  "game_details_window": {
    "x": 120,
    "y": 80,
    "width": 480,
    "height": 640,
    "maximized": false
  },
  "list_view_column_widths": {
    "Added": 140,
    "Alternate Names": 220,
    "Application Path": 300,
    "Badges": 90,
    "Broken": 90,
    "Community Star Rating": 170,
    "Community Star Rating Count": 190,
    "Completed": 110,
    "Developer": 180,
    "Favorite": 95,
    "Genre": 150,
    "Hide": 80,
    "Installed": 95,
    "Last Played": 140,
    "Launchbox Database ID": 180,
    "Max Players": 110,
    "Modified": 140,
    "Platform": 150,
    "Play Count": 110,
    "Play Mode": 120,
    "Play Time": 120,
    "Portable": 95,
    "Publisher": 180,
    "Rating": 90,
    "Region": 100,
    "Release Date": 120,
    "Release Type": 120,
    "Series": 150,
    "Source": 120,
    "Star Rating": 110,
    "Status": 120,
    "Title": 320,
    "Version": 100,
    "Video URL": 260,
    "Wikipedia URL": 260
  }
}
EOF
cp "$list_platform" "$list_platform.before-list-view-smoke"
list_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$launchbox_list_root" \
    --library-list-view-smoke-test \
    --library-list-view-screenshot "$list_screenshot" \
    --ui-state-file "$list_ui_state" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$list_output" >&2
  exit 1
}
if ! rg -q \
  'LIBRARY_LIST_VIEW_SMOKE_COMPLETE reload=0 rows=3 selected=fixture-adventure sort=PlayCount descending=true' \
  <<< "$list_output"; then
  printf '%s\n' "$list_output" >&2
  echo "LaunchBox did not validate its model-backed grid/list transition and sortable list rows." >&2
  exit 1
fi
if [[ ! -s "$list_screenshot" ]] \
  || [[ $(wc -c < "$list_screenshot") -lt 1024 ]] \
  || [[ $(od -An -tx1 -N8 "$list_screenshot" | tr -d ' \n') \
    != 89504e470d0a1a0a ]]; then
  printf '%s\n' "$list_output" >&2
  echo "LaunchBox did not render a valid list-view PNG." >&2
  exit 1
fi
if ! rg -q '<ListView>true</ListView>' "$list_settings" \
  || ! rg -q '<SortBy>PlayCount</SortBy>' "$list_settings" \
  || ! rg -q '<SortByDesc>true</SortByDesc>' "$list_settings" \
  || ! rg -q \
    '<ListViewOrderedColumnPriorities>Badges,Play Count,Title,Platform,Developer,Publisher,Release Date,Rating,Genre,Series,Region,Play Mode,Version,Status,Source,Last Played,Added,Modified,Favorite,Completed,Broken,Portable,Hide,Star Rating,Community Star Rating,Community Star Rating Count,Alternate Names,Wikipedia URL,Max Players,Release Type,Video URL,Installed,Application Path,Launchbox Database ID,Play Time</ListViewOrderedColumnPriorities>' \
    "$list_settings" \
  || ! rg -q \
    '<ListViewVisibleColumnIndexPriorities>33,16,0,1,2,4,5,6,7,8,9,10,11,12,13,14,15,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,34</ListViewVisibleColumnIndexPriorities>' \
    "$list_settings"; then
  echo "LaunchBox did not persist its compatible list view, column order/visibility, and header sort." >&2
  exit 1
fi
if ! cmp -s "$expected_list_ui_state" "$list_ui_state"; then
  printf '%s\n' "$list_output" >&2
  diff -u "$expected_list_ui_state" "$list_ui_state" >&2 || true
  echo "LaunchBox persisted the wrong host-specific list column widths." >&2
  exit 1
fi
cp "$list_settings" "$list_settings.after-list-view-smoke"
cp "$list_ui_state" "$list_ui_state.after-list-view-smoke"
list_reload_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$launchbox_list_root" \
    --library-list-view-reload-smoke-test \
    --ui-state-file "$list_ui_state" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$list_reload_output" >&2
  exit 1
}
if ! rg -q \
  'LIBRARY_LIST_VIEW_SMOKE_COMPLETE reload=1 rows=3 selected=fixture-racer sort=PlayCount descending=true' \
  <<< "$list_reload_output"; then
  printf '%s\n' "$list_reload_output" >&2
  echo "LaunchBox did not restore the list view and list-header sort in a new process." >&2
  exit 1
fi
cmp "$list_settings.after-list-view-smoke" "$list_settings"
cmp "$list_ui_state.after-list-view-smoke" "$list_ui_state"
cmp "$list_platform.before-list-view-smoke" "$list_platform"

echo "LaunchBox model-backed grid/list switching, all 35 original columns, configurable order/visibility/width, stable-ID selection, rendered rows, compatible atomic Settings.xml persistence, platform-native UI state, and new-process restoration validated."

cp -a fixtures/launchbox/. "$launchbox_box_size_root/"
box_size_platform="$launchbox_box_size_root/Data/Platforms/Fixture Console.xml"
box_size_settings="$launchbox_box_size_root/Data/Settings.xml"
box_size_screenshot="$launchbox_box_size_root/launchbox-box-size.png"
cp "$box_size_platform" "$box_size_platform.before-box-size-smoke"
cp "$box_size_settings" "$box_size_settings.before-box-size-smoke"

box_size_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$launchbox_box_size_root" \
    --library-box-size-smoke-test \
    --library-box-size-screenshot "$box_size_screenshot" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$box_size_output" >&2
  exit 1
}
if ! rg -q \
  'LIBRARY_BOX_SIZE_SMOKE_COMPLETE reload=0 rows=3 selected=fixture-adventure size=0.31 cell=397x616' \
  <<< "$box_size_output"; then
  printf '%s\n' "$box_size_output" >&2
  echo "LaunchBox did not validate its real box-size slider and resized grid." >&2
  exit 1
fi
if [[ ! -s "$box_size_screenshot" ]] \
  || [[ $(wc -c < "$box_size_screenshot") -lt 1024 ]] \
  || [[ $(od -An -tx1 -N8 "$box_size_screenshot" | tr -d ' \n') \
    != 89504e470d0a1a0a ]]; then
  printf '%s\n' "$box_size_output" >&2
  echo "LaunchBox did not render a valid resized-grid PNG." >&2
  exit 1
fi
if ! rg -q '<NextBoxSize>0.31</NextBoxSize>' "$box_size_settings"; then
  printf '%s\n' "$box_size_output" >&2
  echo "LaunchBox did not persist the compatible NextBoxSize value." >&2
  exit 1
fi
mapfile -t box_size_backups < <(
  find "$launchbox_box_size_root" -type f \
    -name 'Settings.xml.lbport-transaction-backup-*' -print
)
if [[ ${#box_size_backups[@]} -ne 1 ]] \
  || ! cmp -s "$box_size_settings.before-box-size-smoke" \
       "${box_size_backups[0]}"; then
  printf '%s\n' "$box_size_output" >&2
  echo "LaunchBox did not retain one exact pre-change Settings.xml backup." >&2
  exit 1
fi
cmp "$box_size_platform.before-box-size-smoke" "$box_size_platform"
cp "$box_size_settings" "$box_size_settings.after-box-size-smoke"

box_size_reload_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$launchbox_box_size_root" \
    --library-box-size-reload-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$box_size_reload_output" >&2
  exit 1
}
if ! rg -q \
  'LIBRARY_BOX_SIZE_SMOKE_COMPLETE reload=1 rows=3 selected=fixture-adventure size=0.31 cell=397x616' \
  <<< "$box_size_reload_output"; then
  printf '%s\n' "$box_size_reload_output" >&2
  echo "LaunchBox did not restore box size in a fresh process." >&2
  exit 1
fi
cmp "$box_size_settings.after-box-size-smoke" "$box_size_settings"
cmp "$box_size_platform.before-box-size-smoke" "$box_size_platform"
if [[ $(find "$launchbox_box_size_root" -type f \
  -name 'Settings.xml.lbport-transaction-backup-*' | wc -l) -ne 1 ]]; then
  echo "Box-size reload unexpectedly wrote another Settings.xml backup." >&2
  exit 1
fi

echo "LaunchBox 13.27 box-size range, real slider interaction, responsive grid rendering, exact-backup atomic persistence, stable selection, and new-process restoration validated."

cp -a fixtures/launchbox/. "$launchbox_desktop_tray_root/"
desktop_tray_platform="$launchbox_desktop_tray_root/Data/Platforms/Fixture Console.xml"
desktop_tray_settings="$launchbox_desktop_tray_root/Data/Settings.xml"
desktop_tray_screenshot="$launchbox_desktop_tray_root/launchbox-notifications.png"
cp "$desktop_tray_platform" "$desktop_tray_platform.before-desktop-tray-smoke"
cp "$desktop_tray_settings" "$desktop_tray_settings.before-desktop-tray-smoke"

desktop_tray_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$launchbox_desktop_tray_root" \
    --desktop-tray-smoke-test \
    --desktop-tray-screenshot "$desktop_tray_screenshot" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$desktop_tray_output" >&2
  exit 1
}
if ! rg -q \
  'DESKTOP_TRAY_SMOKE_COMPLETE revision=2 notifications=1' \
  <<< "$desktop_tray_output"; then
  printf '%s\n' "$desktop_tray_output" >&2
  echo "LaunchBox did not validate its system-tray editor and notification center." >&2
  exit 1
fi
if [[ ! -s "$desktop_tray_screenshot" ]] \
  || [[ $(wc -c < "$desktop_tray_screenshot") -lt 1024 ]] \
  || [[ $(od -An -tx1 -N8 "$desktop_tray_screenshot" | tr -d ' \n') \
    != 89504e470d0a1a0a ]]; then
  printf '%s\n' "$desktop_tray_output" >&2
  echo "LaunchBox did not render a valid notification-center PNG." >&2
  exit 1
fi
for expected in \
  '<EnableSystemTray>true</EnableSystemTray>' \
  '<MinimizeToSystemTray>true</MinimizeToSystemTray>' \
  '<CloseToSystemTray>true</CloseToSystemTray>' \
  '<DontSendTrayReminder>false</DontSendTrayReminder>' \
  '<NotificationType>1</NotificationType>' \
  '<Theme>Fixture Theme</Theme>'; do
  if ! rg -Fq "$expected" "$desktop_tray_settings"; then
    printf '%s\n' "$desktop_tray_output" >&2
    echo "LaunchBox did not persist desktop tray setting: $expected" >&2
    exit 1
  fi
done
mapfile -t desktop_tray_backups < <(
  find "$launchbox_desktop_tray_root" -type f \
    -name 'Settings.xml.lbport-transaction-backup-*' -print
)
if [[ ${#desktop_tray_backups[@]} -ne 1 ]] \
  || ! cmp -s "$desktop_tray_settings.before-desktop-tray-smoke" \
       "${desktop_tray_backups[0]}"; then
  printf '%s\n' "$desktop_tray_output" >&2
  echo "LaunchBox did not retain one exact pre-change desktop Settings.xml backup." >&2
  exit 1
fi
cmp "$desktop_tray_platform.before-desktop-tray-smoke" \
  "$desktop_tray_platform"
cp "$desktop_tray_settings" \
  "$desktop_tray_settings.after-desktop-tray-smoke"

desktop_tray_reload_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$launchbox_desktop_tray_root" \
    --desktop-tray-reload-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$desktop_tray_reload_output" >&2
  exit 1
}
if ! rg -q \
  'DESKTOP_TRAY_RELOAD_SMOKE_COMPLETE revision=1 notifications=0' \
  <<< "$desktop_tray_reload_output"; then
  printf '%s\n' "$desktop_tray_reload_output" >&2
  echo "LaunchBox did not restore desktop tray policy in a fresh process." >&2
  exit 1
fi
cmp "$desktop_tray_settings.after-desktop-tray-smoke" \
  "$desktop_tray_settings"
cmp "$desktop_tray_platform.before-desktop-tray-smoke" \
  "$desktop_tray_platform"
if [[ $(find "$launchbox_desktop_tray_root" -type f \
  -name 'Settings.xml.lbport-transaction-backup-*' | wc -l) -ne 1 ]]; then
  echo "Desktop tray reload unexpectedly wrote another Settings.xml backup." >&2
  exit 1
fi

echo "LaunchBox 13.27 system-tray policy, cross-platform Qt tray integration, notification-center interactions, rendered UI, exact-backup persistence, and new-process restoration validated."

mkdir -p "$edit_root/Data/Platforms" "$edit_root/Runtime"
edit_platform="$edit_root/Data/Platforms/Fixture Console.xml"
cp "fixtures/launchbox/Data/Platforms/Fixture Console.xml" "$edit_platform"
install_process_fixture "$edit_root/Runtime/edited-recorder"

edit_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$edit_root" --edit-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$edit_output" >&2
  exit 1
}
if ! rg -q 'EDIT_SMOKE_COMPLETE id=fixture-adventure title="Renamed Adventure" model=box fullscan=1 spine=0.088 size=5x7x1 resets=3 data_changes=1 filtered=0' \
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
  '<CaseColor>-14535868</CaseColor>' \
  '<CoverColor>-11180425</CoverColor>' \
  '<FullImageSpineWidth>0.088</FullImageSpineWidth>' \
  '<ModelSizeString>5;7;1</ModelSizeString>' \
  '<ModelType>box</ModelType>' \
  '<UseFullScanImages>true</UseFullScanImages>' \
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

echo "LaunchBox transactional metadata/launch/alias/custom-field/model edits, lexical Windows-path preservation, persisted Linux launch resolution, metadata search refresh, backup chain, and unknown XML preservation validated."

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

cp -R fixtures/launchbox/Data "$additional_application_crud_root/Data"
additional_application_crud_platform="$additional_application_crud_root/Data/Platforms/Fixture Console.xml"
additional_application_crud_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$additional_application_crud_root" \
    --additional-application-crud-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$additional_application_crud_output" >&2
  exit 1
}
if ! rg -q \
  'ADDITIONAL_APPLICATION_CRUD_SMOKE_COMPLETE writes=3 revision=3 data_changes=3' \
  <<< "$additional_application_crud_output"; then
  printf '%s\n' "$additional_application_crud_output" >&2
  echo "LaunchBox did not validate dialog-driven additional-application CRUD." >&2
  exit 1
fi
for expected in \
  '<ApplicationPath>Games\Fixture Adventure\edited-manual.pdf</ApplicationPath>' \
  '<AutoRunBefore>true</AutoRunBefore>' \
  '<AutoRunAfter>false</AutoRunAfter>' \
  '<CommandLine>--page 3</CommandLine>' \
  '<Developer>Qt Docs</Developer>' \
  '<Disc>2</Disc>' \
  '<EmulatorId />' \
  '<Installed>true</Installed>' \
  '<LastPlayed>2026-07-22T13:14:15.0000000-07:00</LastPlayed>' \
  '<Name>Edited Fixture Manual</Name>' \
  '<PlayCount>5</PlayCount>' \
  '<PlayTime>321</PlayTime>' \
  '<Priority>4</Priority>' \
  '<Publisher>Port Press</Publisher>' \
  '<Region>Europe</Region>' \
  '<ReleaseDate>2005-06-07</ReleaseDate>' \
  '<SideA>true</SideA>' \
  '<SideB>false</SideB>' \
  '<Status>Installed</Status>' \
  '<UseDosBox>false</UseDosBox>' \
  '<UseEmulator>false</UseEmulator>' \
  '<Version>Rev 3</Version>' \
  '<WaitForExit>true</WaitForExit>' \
  '<FutureAdditionalApplicationElement>keep-additional-app-data</FutureAdditionalApplicationElement>'; do
  if ! rg -q -F "$expected" "$additional_application_crud_platform"; then
    echo "Additional-application CRUD did not persist: $expected" >&2
    exit 1
  fi
done
if rg -q -F '<Name>Temporary Fixture Application</Name>' \
  "$additional_application_crud_platform"; then
  echo "Additional-application CRUD retained its temporary application." >&2
  exit 1
fi
if [[ $(rg -c '<AdditionalApplication>' \
  "$additional_application_crud_platform") -ne 1 ]]; then
  echo "Additional-application CRUD changed the final application count." >&2
  exit 1
fi

mapfile -t additional_application_crud_backups < <(
  find "$additional_application_crud_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#additional_application_crud_backups[@]} -ne 3 ]]; then
  echo "Additional-application CRUD did not retain exactly three transaction backups." >&2
  exit 1
fi
additional_application_original_backups=0
additional_application_edited_backups=0
additional_application_temporary_backups=0
for backup in "${additional_application_crud_backups[@]}"; do
  if cmp -s "$backup" \
    'fixtures/launchbox/Data/Platforms/Fixture Console.xml'; then
    ((additional_application_original_backups += 1))
  elif rg -q -F '<Name>Temporary Fixture Application</Name>' "$backup" \
    && rg -q -F '<Name>Edited Fixture Manual</Name>' "$backup"; then
    ((additional_application_temporary_backups += 1))
  elif rg -q -F '<Name>Edited Fixture Manual</Name>' "$backup" \
    && ! rg -q -F '<Name>Temporary Fixture Application</Name>' "$backup"; then
    ((additional_application_edited_backups += 1))
  fi
done
if [[ $additional_application_original_backups -ne 1 \
  || $additional_application_edited_backups -ne 1 \
  || $additional_application_temporary_backups -ne 1 ]]; then
  echo "Additional-application transaction backups do not prove the edit/add/delete chain." >&2
  exit 1
fi
if find "$additional_application_crud_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful additional-application CRUD left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox additional-application dialog editing, lexical paths, add/delete, targeted Qt updates, backup chain, and unknown XML preservation validated."

cp -R fixtures/launchbox/Data "$additional_application_default_root/Data"
additional_application_default_platform="$additional_application_default_root/Data/Platforms/Fixture Console.xml"
additional_application_default_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$additional_application_default_root" \
    --additional-application-default-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$additional_application_default_output" >&2
  exit 1
}
if ! rg -q \
  'ADDITIONAL_APPLICATION_DEFAULT_SMOKE_COMPLETE writes=2 revision=1 resets=2 data_changes=1' \
  <<< "$additional_application_default_output"; then
  printf '%s\n' "$additional_application_default_output" >&2
  echo "LaunchBox did not validate dialog-driven Make Default behavior." >&2
  exit 1
fi
for expected in \
  '<ID>fixture-adventure</ID>' \
  '<Title>Fixture Adventure</Title>' \
  '<CommandLine>--page 3</CommandLine>' \
  '<Emulator>00000000-0000-0000-0000-000000000000</Emulator>' \
  '<UseDosBox>false</UseDosBox>' \
  '<UseScummVM>false</UseScummVM>' \
  '<ScummVMGameType>fixture-scumm-id</ScummVMGameType>' \
  '<Developer>Qt Docs</Developer>' \
  '<Publisher>Port Press</Publisher>' \
  '<Region>Europe</Region>' \
  '<ReleaseDate>2005-06-07</ReleaseDate>' \
  '<Version>Rev 3</Version>' \
  '<Status>Installed</Status>' \
  '<Installed>true</Installed>' \
  '<PlayCount>5</PlayCount>' \
  '<PlayTime>321</PlayTime>' \
  '<LastPlayedDate>2026-07-22T13:14:15.0000000-07:00</LastPlayedDate>' \
  '<Id>fixture-adventure-manual</Id>' \
  '<Name>Edited Fixture Manual</Name>' \
  '<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>' \
  '<FutureAdditionalApplicationElement>keep-additional-app-data</FutureAdditionalApplicationElement>'; do
  if ! rg -q -F "$expected" "$additional_application_default_platform"; then
    echo "Additional-application Make Default did not persist: $expected" >&2
    exit 1
  fi
done
if [[ $(rg -c -F \
  '<ApplicationPath>Games\Fixture Adventure\edited-manual.pdf</ApplicationPath>' \
  "$additional_application_default_platform") -ne 2 ]]; then
  echo "Make Default did not retain the app while copying its lexical path to the game." >&2
  exit 1
fi
if rg -q -F \
  '<ApplicationPath>Games\Fixture Adventure\adventure.rom</ApplicationPath>' \
  "$additional_application_default_platform"; then
  echo "Make Default retained the game's previous default application path." >&2
  exit 1
fi
if [[ $(rg -c '<AdditionalApplication>' \
  "$additional_application_default_platform") -ne 1 ]]; then
  echo "Make Default changed the additional-application record count." >&2
  exit 1
fi

mapfile -t additional_application_default_backups < <(
  find "$additional_application_default_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#additional_application_default_backups[@]} -ne 2 ]]; then
  echo "Additional-application Make Default did not retain exactly two transaction backups." >&2
  exit 1
fi
additional_application_default_original_backups=0
additional_application_default_edited_backups=0
for backup in "${additional_application_default_backups[@]}"; do
  if cmp -s "$backup" \
    'fixtures/launchbox/Data/Platforms/Fixture Console.xml'; then
    ((additional_application_default_original_backups += 1))
  elif rg -q -F \
    '<ApplicationPath>Games\Fixture Adventure\adventure.rom</ApplicationPath>' \
    "$backup" \
    && [[ $(rg -c -F \
      '<ApplicationPath>Games\Fixture Adventure\edited-manual.pdf</ApplicationPath>' \
      "$backup") -eq 1 ]]; then
    ((additional_application_default_edited_backups += 1))
  fi
done
if [[ $additional_application_default_original_backups -ne 1 \
  || $additional_application_default_edited_backups -ne 1 ]]; then
  echo "Make Default backups do not prove the expected edit/default chain." >&2
  exit 1
fi
if find "$additional_application_default_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful Make Default smoke left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox Make Default copying, retained app records, exact backup chain, lexical Windows paths, and unknown XML preservation validated."

cp -R fixtures/launchbox/Data "$game_save_metadata_root/Data"
game_save_metadata_platform="$game_save_metadata_root/Data/Platforms/Fixture Console.xml"
sed -i \
  "\|</GameSave>|r $workspace_root/fixtures/game-save-smoke-records.xml" \
  "$game_save_metadata_platform"
cp "$game_save_metadata_platform" "$game_save_metadata_root/original-platform.xml"
game_save_metadata_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$game_save_metadata_root" \
    --game-save-metadata-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$game_save_metadata_output" >&2
  exit 1
}
if ! rg -q \
  'GAME_SAVE_METADATA_SMOKE_COMPLETE groups=2 writes=4 revision=4 data_changes=4' \
  <<< "$game_save_metadata_output"; then
  printf '%s\n' "$game_save_metadata_output" >&2
  echo "LaunchBox did not validate dialog-driven game-save metadata editing." >&2
  exit 1
fi
if [[ $(rg -c '<GameSave>' "$game_save_metadata_platform") -ne 3 ]] \
  || [[ $(rg -c -F '<SaveGroupName>Renamed Run</SaveGroupName>' \
    "$game_save_metadata_platform") -ne 2 ]] \
  || [[ $(rg -c -F '<SaveGroupName>Split History</SaveGroupName>' \
    "$game_save_metadata_platform") -ne 1 ]]; then
  echo "Game-save metadata smoke produced the wrong final group/version shape." >&2
  exit 1
fi
for expected in \
  '<Title>Renamed Active</Title>' \
  '<FilePath>Saves\Fixture Adventure\slot1.sav</FilePath>' \
  '<FilePath>C:\Users\Ben\RetroArch\saves\fixture-live.srm</FilePath>' \
  '<FilePath>Saves\Fixture Console\fixture-adventure-01.srm</FilePath>' \
  '<ReportedFileSizeBytes>32768</ReportedFileSizeBytes>' \
  '<ReportedLastModifiedUtc>2026-07-22T01:02:03.4567890Z</ReportedLastModifiedUtc>' \
  '<Md5>0123456789abcdef0123456789abcdef</Md5>' \
  '<FutureGameSaveField>keep-live-save-data</FutureGameSaveField>' \
  '<FutureGameSaveField>keep-vault-save-data</FutureGameSaveField>' \
  '<FutureRootElement>preserve-me</FutureRootElement>'; do
  if ! rg -q -F "$expected" "$game_save_metadata_platform"; then
    echo "Game-save metadata smoke did not preserve: $expected" >&2
    exit 1
  fi
done

mapfile -t game_save_metadata_backups < <(
  find "$game_save_metadata_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#game_save_metadata_backups[@]} -ne 4 ]]; then
  echo "Game-save metadata edits did not retain exactly four transaction backups." >&2
  exit 1
fi
game_save_original_backups=0
game_save_renamed_version_backups=0
game_save_renamed_group_backups=0
game_save_combined_backups=0
for backup in "${game_save_metadata_backups[@]}"; do
  if cmp -s "$backup" "$game_save_metadata_root/original-platform.xml"; then
    ((game_save_original_backups += 1))
  elif rg -q -F '<Title>Renamed Active</Title>' "$backup" \
    && ! rg -q -F '<SaveGroupName>Renamed Run</SaveGroupName>' "$backup"; then
    ((game_save_renamed_version_backups += 1))
  elif [[ $(rg -c -F '<SaveGroupName>Renamed Run</SaveGroupName>' \
    "$backup") -eq 1 ]] \
    && [[ $(rg -c -F '<SaveGroupName>Backup Run</SaveGroupName>' \
    "$backup") -eq 2 ]]; then
    ((game_save_renamed_group_backups += 1))
  elif [[ $(rg -c -F '<SaveGroupName>Renamed Run</SaveGroupName>' \
    "$backup") -eq 3 ]] \
    && ! rg -q -F '<SaveGroupName>Split History</SaveGroupName>' "$backup"; then
    ((game_save_combined_backups += 1))
  fi
done
if [[ $game_save_original_backups -ne 1 \
  || $game_save_renamed_version_backups -ne 1 \
  || $game_save_renamed_group_backups -ne 1 \
  || $game_save_combined_backups -ne 1 ]]; then
  echo "Game-save transaction backups do not prove the rename/combine/split chain." >&2
  exit 1
fi
if find "$game_save_metadata_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful game-save metadata edits left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox dialog-driven save grouping/version metadata, active/vault path classification, lexical Windows paths, exact backup chain, and unknown XML preservation validated."

cp -R fixtures/launchbox/Data "$retroarch_save_scan_root/Data"
retroarch_save_scan_platform="$retroarch_save_scan_root/Data/Platforms/Fixture Console.xml"
retroarch_save_scan_emulators="$retroarch_save_scan_root/Data/Emulators.xml"
sed -i \
  -e 's|<Title>Fixture Emulator</Title>|<Title>RetroArch</Title>|' \
  -e 's|<ApplicationPath>Emulators/fixture-emulator</ApplicationPath>|<ApplicationPath>Emulators/RetroArch/retroarch</ApplicationPath>|' \
  -e 's|<CommandLine>--fullscreen</CommandLine>|<CommandLine>-L cores/mesen_libretro.so</CommandLine>|' \
  -e 's|<CommandLine>--platform fixture</CommandLine>|<CommandLine>-L cores/mesen_libretro.so</CommandLine>|' \
  "$retroarch_save_scan_emulators"
mkdir -p \
  "$retroarch_save_scan_root/Emulators/RetroArch/saves" \
  "$retroarch_save_scan_root/Emulators/RetroArch/states" \
  "$retroarch_save_scan_root/Games/Fixture Racer"
printf %s 'retroarch runtime fixture' \
  > "$retroarch_save_scan_root/Emulators/RetroArch/retroarch"
printf '%s\n' \
  'savefile_directory = "saves"' \
  'savestate_directory = "states"' \
  'savefiles_in_content_dir = "false"' \
  'savestates_in_content_dir = "false"' \
  'sort_savefiles_by_content_enable = "false"' \
  'sort_savefiles_enable = "false"' \
  'sort_savestates_by_content_enable = "false"' \
  'sort_savestates_enable = "false"' \
  > "$retroarch_save_scan_root/Emulators/RetroArch/retroarch.cfg"
printf %s 'fixture racer rom' \
  > "$retroarch_save_scan_root/Games/Fixture Racer/racer.rom"
printf %s 'runtime racer save bytes' \
  > "$retroarch_save_scan_root/Emulators/RetroArch/saves/racer.srm"
printf %s 'runtime racer state bytes' \
  > "$retroarch_save_scan_root/Emulators/RetroArch/states/racer.state"
printf %s 'runtime racer auto state bytes' \
  > "$retroarch_save_scan_root/Emulators/RetroArch/states/racer.state.auto"
cp "$retroarch_save_scan_platform" \
  "$retroarch_save_scan_root/original-platform.xml"
retroarch_save_scan_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$retroarch_save_scan_root" \
    --retroarch-save-scan-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$retroarch_save_scan_output" >&2
  exit 1
}
if ! rg -q \
  'RETROARCH_SAVE_SCAN_SMOKE_COMPLETE saves=3 writes=1 revision=1 data_changes=1' \
  <<< "$retroarch_save_scan_output"; then
  printf '%s\n' "$retroarch_save_scan_output" >&2
  echo "LaunchBox did not validate manager-driven RetroArch save discovery." >&2
  exit 1
fi
if [[ $(rg -c '<GameSave>' "$retroarch_save_scan_platform") -ne 4 ]] \
  || [[ $(rg -c -F '<EmulatorCore>mesen_libretro</EmulatorCore>' \
    "$retroarch_save_scan_platform") -ne 3 ]] \
  || [[ $(rg -c -F '<EmulatorFileName>retroarch</EmulatorFileName>' \
    "$retroarch_save_scan_platform") -ne 3 ]] \
  || [[ $(rg -c '<ReportedFileSizeBytes>' \
    "$retroarch_save_scan_platform") -ne 3 ]] \
  || [[ $(rg -c '<ReportedLastModifiedUtc>.*\.[0-9]{7}Z' \
    "$retroarch_save_scan_platform") -ne 3 ]] \
  || [[ $(rg -c '<Md5>[0-9A-F]{32}</Md5>' \
    "$retroarch_save_scan_platform") -ne 3 ]]; then
  echo "RetroArch save discovery wrote incomplete owner or file metadata." >&2
  exit 1
fi
for expected in \
  '<FilePath>Emulators\RetroArch\saves\racer.srm</FilePath>' \
  '<FilePath>Emulators\RetroArch\states\racer.state</FilePath>' \
  '<FilePath>Emulators\RetroArch\states\racer.state.auto</FilePath>' \
  '<OriginalFileName>racer.srm</OriginalFileName>' \
  '<OriginalFileName>racer.state</OriginalFileName>' \
  '<OriginalFileName>racer.state.auto</OriginalFileName>' \
  '<Slot>0</Slot>' \
  '<Slot>-1</Slot>' \
  '<FutureRootElement>preserve-me</FutureRootElement>'; do
  if ! rg -q -F "$expected" "$retroarch_save_scan_platform"; then
    echo "RetroArch save discovery did not persist: $expected" >&2
    exit 1
  fi
done
mapfile -t retroarch_save_scan_backups < <(
  find "$retroarch_save_scan_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#retroarch_save_scan_backups[@]} -ne 1 ]] \
  || ! cmp -s "${retroarch_save_scan_backups[0]}" \
    "$retroarch_save_scan_root/original-platform.xml"; then
  echo "RetroArch save discovery did not retain one exact XML recovery copy." >&2
  exit 1
fi
if find "$retroarch_save_scan_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful RetroArch save discovery left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox manager-driven RetroArch config discovery, regular saves, state slots, portable paths, full metadata, exact XML rollback backup, and cleanup validated."

cp -R fixtures/launchbox/Data "$dolphin_save_scan_root/Data"
dolphin_save_scan_platform="$dolphin_save_scan_root/Data/Platforms/Fixture Console.xml"
dolphin_save_scan_emulators="$dolphin_save_scan_root/Data/Emulators.xml"
sed -i \
  -e 's|racer\.rom|racer.iso|' \
  "$dolphin_save_scan_platform"
sed -i \
  -e 's|<Title>Fixture Emulator</Title>|<Title>Dolphin</Title>|' \
  -e 's|<ApplicationPath>Emulators/fixture-emulator</ApplicationPath>|<ApplicationPath>Emulators/Dolphin/Dolphin.exe</ApplicationPath>|' \
  "$dolphin_save_scan_emulators"
mkdir -p \
  "$dolphin_save_scan_root/Emulators/Dolphin/User/GC/USA/GALE01" \
  "$dolphin_save_scan_root/Emulators/Dolphin/User/GC/USA/Card A" \
  "$dolphin_save_scan_root/Emulators/Dolphin/User/StateSaves" \
  "$dolphin_save_scan_root/Games/Fixture Racer"
printf %s 'dolphin runtime fixture' \
  > "$dolphin_save_scan_root/Emulators/Dolphin/Dolphin.exe"
printf %s 'GALE01 fixture racer disc bytes' \
  > "$dolphin_save_scan_root/Games/Fixture Racer/racer.iso"
printf %s 'runtime folder save bytes' \
  > "$dolphin_save_scan_root/Emulators/Dolphin/User/GC/USA/GALE01/01-GALE-adventure.gci"
printf %s 'runtime card a save bytes' \
  > "$dolphin_save_scan_root/Emulators/Dolphin/User/GC/USA/Card A/01-GALE-card.gci"
printf %s 'runtime state seven bytes' \
  > "$dolphin_save_scan_root/Emulators/Dolphin/User/StateSaves/GALE01.s07"
cp "$dolphin_save_scan_platform" \
  "$dolphin_save_scan_root/original-platform.xml"
dolphin_save_scan_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$dolphin_save_scan_root" \
    --dolphin-save-scan-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$dolphin_save_scan_output" >&2
  exit 1
}
if ! rg -q \
  'DOLPHIN_SAVE_SCAN_SMOKE_COMPLETE saves=3 writes=1 revision=1 data_changes=1' \
  <<< "$dolphin_save_scan_output"; then
  printf '%s\n' "$dolphin_save_scan_output" >&2
  echo "LaunchBox did not validate manager-driven Dolphin save discovery." >&2
  exit 1
fi
if [[ $(rg -c '<GameSave>' "$dolphin_save_scan_platform") -ne 4 ]] \
  || [[ $(rg -c -F '<EmulatorFileName>Dolphin.exe</EmulatorFileName>' \
    "$dolphin_save_scan_platform") -ne 3 ]] \
  || [[ $(rg -c '<ReportedFileSizeBytes>' \
    "$dolphin_save_scan_platform") -ne 3 ]] \
  || [[ $(rg -c '<ReportedLastModifiedUtc>.*\.[0-9]{7}Z' \
    "$dolphin_save_scan_platform") -ne 3 ]] \
  || [[ $(rg -c '<Md5>[0-9A-F]{32}</Md5>' \
    "$dolphin_save_scan_platform") -ne 3 ]]; then
  echo "Dolphin save discovery wrote incomplete owner or file metadata." >&2
  exit 1
fi
for expected in \
  '<FilePath>Emulators\Dolphin\User\GC\USA\GALE01\01-GALE-adventure.gci</FilePath>' \
  '<FilePath>Emulators\Dolphin\User\GC\USA\Card A\01-GALE-card.gci</FilePath>' \
  '<FilePath>Emulators\Dolphin\User\StateSaves\GALE01.s07</FilePath>' \
  '<SaveGroupId>dolphin:gc:fixture-racer:GALE01:Folder:01-GALE-adventure.gci</SaveGroupId>' \
  '<SaveGroupId>dolphin:gc:fixture-racer:GALE01:CardA:01-GALE-card.gci</SaveGroupId>' \
  '<SaveGroupId>fixture-racer-GALE01-State-7</SaveGroupId>' \
  '<DisplayChipText>Card A</DisplayChipText>' \
  '<OriginalFileName>01-GALE-adventure.gci</OriginalFileName>' \
  '<OriginalFileName>01-GALE-card.gci</OriginalFileName>' \
  '<OriginalFileName>GALE01.s07</OriginalFileName>' \
  '<Slot>7</Slot>' \
  '<FutureRootElement>preserve-me</FutureRootElement>'; do
  if ! rg -q -F "$expected" "$dolphin_save_scan_platform"; then
    echo "Dolphin save discovery did not persist: $expected" >&2
    exit 1
  fi
done
mapfile -t dolphin_save_scan_backups < <(
  find "$dolphin_save_scan_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#dolphin_save_scan_backups[@]} -ne 1 ]] \
  || ! cmp -s "${dolphin_save_scan_backups[0]}" \
    "$dolphin_save_scan_root/original-platform.xml"; then
  echo "Dolphin save discovery did not retain one exact XML recovery copy." >&2
  exit 1
fi
if find "$dolphin_save_scan_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful Dolphin save discovery left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox manager-driven Dolphin disc-ID discovery, portable GameCube folder/card saves, state slots, full metadata, exact XML rollback backup, and cleanup validated."

cp -R fixtures/launchbox/Data "$pcsx2_save_scan_root/Data"
pcsx2_save_scan_platform="$pcsx2_save_scan_root/Data/Platforms/Fixture Console.xml"
pcsx2_save_scan_emulators="$pcsx2_save_scan_root/Data/Emulators.xml"
sed -i \
  -e 's|racer\.rom|opaque-content.chd|' \
  "$pcsx2_save_scan_platform"
sed -i \
  -e 's|<Title>Fixture Emulator</Title>|<Title>PCSX2</Title>|' \
  -e 's|<ApplicationPath>Emulators/fixture-emulator</ApplicationPath>|<ApplicationPath>Emulators/PCSX2/pcsx2-qt</ApplicationPath>|' \
  "$pcsx2_save_scan_emulators"
mkdir -p \
  "$pcsx2_save_scan_root/Emulators/PCSX2/memcards/Mcd001.ps2/BASLUS-20312SAVE" \
  "$pcsx2_save_scan_root/Emulators/PCSX2/sstates" \
  "$pcsx2_save_scan_root/Games/Fixture Racer"
printf %s 'pcsx2 runtime fixture' \
  > "$pcsx2_save_scan_root/Emulators/PCSX2/pcsx2-qt"
"$process_fixture" \
  --fixture-mode pcsx2-disc-image \
  --path "$pcsx2_save_scan_root/Games/Fixture Racer/opaque-content.chd" \
  --format chd-cd \
  --serial SLUS_203.12
printf %s 'runtime folder card save bytes' \
  > "$pcsx2_save_scan_root/Emulators/PCSX2/memcards/Mcd001.ps2/BASLUS-20312SAVE/data.bin"
printf %s 'runtime state three bytes' \
  > "$pcsx2_save_scan_root/Emulators/PCSX2/sstates/SLUS-20312 (DEADBEEF).03.p2s"
cp "$pcsx2_save_scan_platform" \
  "$pcsx2_save_scan_root/original-platform.xml"
pcsx2_save_scan_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$pcsx2_save_scan_root" \
    --pcsx2-save-scan-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$pcsx2_save_scan_output" >&2
  exit 1
}
if ! rg -q \
  'PCSX2_SAVE_SCAN_SMOKE_COMPLETE saves=2 writes=1 revision=1 data_changes=1' \
  <<< "$pcsx2_save_scan_output"; then
  printf '%s\n' "$pcsx2_save_scan_output" >&2
  echo "LaunchBox did not validate manager-driven PCSX2 save discovery." >&2
  exit 1
fi
if [[ $(rg -c '<GameSave>' "$pcsx2_save_scan_platform") -ne 3 ]] \
  || [[ $(rg -c -F '<EmulatorFileName>pcsx2-qt</EmulatorFileName>' \
    "$pcsx2_save_scan_platform") -ne 2 ]] \
  || [[ $(rg -c '<ReportedFileSizeBytes>' \
    "$pcsx2_save_scan_platform") -ne 2 ]] \
  || [[ $(rg -c '<ReportedLastModifiedUtc>.*\.[0-9]{7}Z' \
    "$pcsx2_save_scan_platform") -ne 2 ]] \
  || [[ $(rg -c '<Md5>[0-9A-F]{32}</Md5>' \
    "$pcsx2_save_scan_platform") -ne 1 ]]; then
  echo "PCSX2 save discovery wrote incomplete or fabricated metadata." >&2
  exit 1
fi
for expected in \
  '<FilePath>Emulators\PCSX2\memcards\Mcd001.ps2</FilePath>' \
  '<FilePath>Emulators\PCSX2\sstates\SLUS-20312 (DEADBEEF).03.p2s</FilePath>' \
  '<SaveGroupId>pcsx2:Mcd001:BASLUS-20312SAVE</SaveGroupId>' \
  '<SaveGroupId>pcsx2-state:SLUS20312:03</SaveGroupId>' \
  '<OriginalFileName>BASLUS-20312SAVE</OriginalFileName>' \
  '<OriginalFileName>SLUS-20312 (DEADBEEF).03.p2s</OriginalFileName>' \
  '<Slot>3</Slot>' \
  '<FutureRootElement>preserve-me</FutureRootElement>'; do
  if ! rg -q -F "$expected" "$pcsx2_save_scan_platform"; then
    echo "PCSX2 save discovery did not persist: $expected" >&2
    exit 1
  fi
done
mapfile -t pcsx2_save_scan_backups < <(
  find "$pcsx2_save_scan_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#pcsx2_save_scan_backups[@]} -ne 1 ]] \
  || ! cmp -s "${pcsx2_save_scan_backups[0]}" \
    "$pcsx2_save_scan_root/original-platform.xml"; then
  echo "PCSX2 save discovery did not retain one exact XML recovery copy." >&2
  exit 1
fi
if find "$pcsx2_save_scan_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful PCSX2 save discovery left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox manager-driven PCSX2 compressed-CHD filesystem serial extraction, portable folder-card/member discovery, exact state matching, container metadata, regular-file state eligibility, exact XML rollback backup, and cleanup validated."

cp -R fixtures/launchbox/Data "$game_save_backup_root/Data"
game_save_backup_platform="$game_save_backup_root/Data/Platforms/Fixture Console.xml"
sed -i \
  's|<FilePath>Saves\\Fixture Adventure\\slot1.sav</FilePath>|<FilePath>Emulator\\Saves\\slot1.sav</FilePath>|' \
  "$game_save_backup_platform"
mkdir -p "$game_save_backup_root/Emulator/Saves"
game_save_backup_active="$game_save_backup_root/Emulator/Saves/slot1.sav"
game_save_backup_bytes='runtime active save bytes'
printf %s "$game_save_backup_bytes" > "$game_save_backup_active"
cp "$game_save_backup_platform" "$game_save_backup_root/original-platform.xml"
game_save_backup_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$game_save_backup_root" \
    --game-save-backup-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$game_save_backup_output" >&2
  exit 1
}
if ! rg -q \
  'GAME_SAVE_BACKUP_SMOKE_COMPLETE saves=2 writes=1 revision=1 data_changes=1' \
  <<< "$game_save_backup_output"; then
  printf '%s\n' "$game_save_backup_output" >&2
  echo "LaunchBox did not validate the manager-driven manual save backup." >&2
  exit 1
fi
game_save_backup_vault="$game_save_backup_root/Saves/Fixture Console/adventure.sav"
if ! cmp -s "$game_save_backup_active" "$game_save_backup_vault"; then
  echo "Manual save backup did not preserve the exact active bytes." >&2
  exit 1
fi
game_save_backup_md5=$(
  md5sum "$game_save_backup_active" | cut -d ' ' -f 1 | tr '[:lower:]' '[:upper:]'
)
if [[ $(rg -c '<GameSave>' "$game_save_backup_platform") -ne 2 ]] \
  || [[ $(rg -c '<SaveGroupId>' "$game_save_backup_platform") -ne 2 ]] \
  || ! rg -q -F \
    '<FilePath>Saves\Fixture Console\adventure.sav</FilePath>' \
    "$game_save_backup_platform" \
  || ! rg -q -F \
    "<ReportedFileSizeBytes>${#game_save_backup_bytes}</ReportedFileSizeBytes>" \
    "$game_save_backup_platform" \
  || ! rg -q -F "<Md5>$game_save_backup_md5</Md5>" \
    "$game_save_backup_platform" \
  || ! rg -q \
    '<ReportedLastModifiedUtc>[^<]*\.[0-9]{7}Z</ReportedLastModifiedUtc>' \
    "$game_save_backup_platform" \
  || ! rg -q -F '<FutureRootElement>preserve-me</FutureRootElement>' \
    "$game_save_backup_platform"; then
  echo "Manual save backup wrote incomplete or non-lossless XML metadata." >&2
  exit 1
fi
mapfile -t game_save_backup_backups < <(
  find "$game_save_backup_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#game_save_backup_backups[@]} -ne 1 ]] \
  || ! cmp -s "${game_save_backup_backups[0]}" \
    "$game_save_backup_root/original-platform.xml"; then
  echo "Manual save backup did not retain one exact pre-transaction XML backup." >&2
  exit 1
fi
if find "$game_save_backup_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful manual save backup left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox manager-driven save backup, portable vault naming, exact bytes, full metadata, XML rollback backup, and cleanup validated."

cp -R fixtures/launchbox/Data "$pcsx2_save_backup_root/Data"
pcsx2_save_backup_platform="$pcsx2_save_backup_root/Data/Platforms/Fixture Console.xml"
sed -i \
  -e 's|<EmulatorFileName>fixture-emulator</EmulatorFileName>|<EmulatorFileName>pcsx2-qt</EmulatorFileName>|' \
  -e 's|<FilePath>Saves\\Fixture Adventure\\slot1.sav</FilePath>|<FilePath>Emulators\\PCSX2\\memcards\\Mcd001.ps2</FilePath>|' \
  -e '/    <Slot>1<\/Slot>/d' \
  -e '/    <Title>Before the Final Puzzle<\/Title>/a\    <SaveGroupName>My Save File</SaveGroupName>\n    <SaveGroupId>pcsx2:Mcd001:BASLUS-12345SAVE</SaveGroupId>\n    <OriginalFileName>BASLUS-12345SAVE</OriginalFileName>' \
  "$pcsx2_save_backup_platform"
pcsx2_save_backup_member="$pcsx2_save_backup_root/Emulators/PCSX2/memcards/Mcd001.ps2/BASLUS-12345SAVE"
mkdir -p \
  "$pcsx2_save_backup_member" \
  "$pcsx2_save_backup_root/Games/Fixture Adventure"
printf %s 'fixture rom' \
  > "$pcsx2_save_backup_root/Games/Fixture Adventure/adventure.rom"
dd if=/dev/zero of="$pcsx2_save_backup_member/icon.sys" \
  bs=148 count=1 status=none
printf %s 'PS2D' | dd of="$pcsx2_save_backup_member/icon.sys" \
  conv=notrunc status=none
pcsx2_save_backup_bytes='runtime PCSX2 member bytes'
printf %s "$pcsx2_save_backup_bytes" \
  > "$pcsx2_save_backup_member/save.bin"
cp "$pcsx2_save_backup_platform" \
  "$pcsx2_save_backup_root/original-platform.xml"
pcsx2_save_backup_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$pcsx2_save_backup_root" \
    --pcsx2-save-backup-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$pcsx2_save_backup_output" >&2
  exit 1
}
if ! rg -q \
  'PCSX2_SAVE_BACKUP_SMOKE_COMPLETE saves=2 writes=1 revision=1 data_changes=1' \
  <<< "$pcsx2_save_backup_output"; then
  printf '%s\n' "$pcsx2_save_backup_output" >&2
  echo "LaunchBox did not validate manager-driven PCSX2 member backup." >&2
  exit 1
fi
pcsx2_save_backup_archive="$pcsx2_save_backup_root/Saves/Fixture Console/adventure.7z"
pcsx2_save_backup_extract="$pcsx2_save_backup_root/archive-check"
mkdir "$pcsx2_save_backup_extract"
7z x -y -bd -bb0 "-o$pcsx2_save_backup_extract" -- \
  "$pcsx2_save_backup_archive" >/dev/null
if ! cmp -s \
    "$pcsx2_save_backup_member/icon.sys" \
    "$pcsx2_save_backup_extract/icon.sys" \
  || ! cmp -s \
    "$pcsx2_save_backup_member/save.bin" \
    "$pcsx2_save_backup_extract/save.bin"; then
  echo "PCSX2 member backup archive did not preserve exact member bytes." >&2
  exit 1
fi
pcsx2_icon_sha=$(
  sha256sum "$pcsx2_save_backup_member/icon.sys" \
    | cut -d ' ' -f 1 | tr '[:lower:]' '[:upper:]'
)
pcsx2_save_sha=$(
  sha256sum "$pcsx2_save_backup_member/save.bin" \
    | cut -d ' ' -f 1 | tr '[:lower:]' '[:upper:]'
)
pcsx2_manifest_signature=$(
  printf 'icon.sys|%s\nsave.bin|%s\n' \
    "$pcsx2_icon_sha" "$pcsx2_save_sha" \
    | sha256sum | cut -d ' ' -f 1 | tr '[:lower:]' '[:upper:]'
)
pcsx2_logical_size=$((148 + ${#pcsx2_save_backup_bytes}))
if [[ $(rg -c '<GameSave>' "$pcsx2_save_backup_platform") -ne 2 ]] \
  || ! rg -q -F \
    '<FilePath>Saves\Fixture Console\adventure.7z</FilePath>' \
    "$pcsx2_save_backup_platform" \
  || [[ $(rg -c -F \
    '<SaveGroupId>pcsx2:Mcd001:BASLUS-12345SAVE</SaveGroupId>' \
    "$pcsx2_save_backup_platform") -ne 2 ]] \
  || ! rg -q -F \
    '<OriginalFileName>BASLUS-12345SAVE</OriginalFileName>' \
    "$pcsx2_save_backup_platform" \
  || ! rg -q -F \
    "<ReportedFileSizeBytes>$pcsx2_logical_size</ReportedFileSizeBytes>" \
    "$pcsx2_save_backup_platform" \
  || ! rg -q -F "<Md5>$pcsx2_manifest_signature</Md5>" \
    "$pcsx2_save_backup_platform" \
  || ! rg -q \
    '<ReportedLastModifiedUtc>[^<]*\.[0-9]{7}Z</ReportedLastModifiedUtc>' \
    "$pcsx2_save_backup_platform" \
  || ! rg -q -F '<FutureRootElement>preserve-me</FutureRootElement>' \
    "$pcsx2_save_backup_platform"; then
  echo "PCSX2 member backup wrote incomplete or incompatible XML metadata." >&2
  exit 1
fi
mapfile -t pcsx2_save_backup_backups < <(
  find "$pcsx2_save_backup_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#pcsx2_save_backup_backups[@]} -ne 1 ]] \
  || ! cmp -s "${pcsx2_save_backup_backups[0]}" \
    "$pcsx2_save_backup_root/original-platform.xml"; then
  echo "PCSX2 member backup did not retain one exact XML recovery copy." >&2
  exit 1
fi
if find "$pcsx2_save_backup_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful PCSX2 member backup left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox manager-driven PCSX2 folder-card member extraction, verified 7z backup, logical manifest metadata, exact XML recovery, and cleanup validated."

cp -R fixtures/launchbox/Data "$pcsx2_save_lifecycle_root/Data"
pcsx2_save_lifecycle_platform="$pcsx2_save_lifecycle_root/Data/Platforms/Fixture Console.xml"
sed -i \
  -e 's|<EmulatorFileName>fixture-emulator</EmulatorFileName>|<EmulatorFileName>pcsx2-qt</EmulatorFileName>|' \
  -e 's|<FilePath>Saves\\Fixture Adventure\\slot1.sav</FilePath>|<FilePath>Emulators\\PCSX2\\memcards\\Mcd001.ps2</FilePath>|' \
  -e '/    <Slot>1<\/Slot>/d' \
  -e '/    <Title>Before the Final Puzzle<\/Title>/a\    <SaveGroupName>PCSX2 Save</SaveGroupName>\n    <SaveGroupId>pcsx2:Mcd001:BASLUS-12345SAVE</SaveGroupId>\n    <OriginalFileName>BASLUS-12345SAVE</OriginalFileName>' \
  "$pcsx2_save_lifecycle_platform"
sed -i \
  '/<\/GameSave>/a\  <GameSave>\n    <EmulatorCore>fixture-core</EmulatorCore>\n    <EmulatorFileName>pcsx2-qt</EmulatorFileName>\n    <FilePath>Saves\\Fixture Console\\adventure.7z</FilePath>\n    <GameId>fixture-adventure</GameId>\n    <Title>Selected PCSX2 Backup</Title>\n    <SaveGroupName>PCSX2 Save</SaveGroupName>\n    <SaveGroupId>pcsx2:Mcd001:BASLUS-12345SAVE</SaveGroupId>\n    <OriginalFileName>BASLUS-12345SAVE</OriginalFileName>\n  </GameSave>' \
  "$pcsx2_save_lifecycle_platform"
pcsx2_save_lifecycle_card="$pcsx2_save_lifecycle_root/Emulators/PCSX2/memcards/Mcd001.ps2"
pcsx2_save_lifecycle_selected_source="$pcsx2_save_lifecycle_root/selected-member"
pcsx2_save_lifecycle_vault="$pcsx2_save_lifecycle_root/Saves/Fixture Console"
mkdir -p \
  "$(dirname "$pcsx2_save_lifecycle_card")" \
  "$pcsx2_save_lifecycle_vault" \
  "$pcsx2_save_lifecycle_root/Games/Fixture Adventure"
printf %s 'fixture rom' \
  > "$pcsx2_save_lifecycle_root/Games/Fixture Adventure/adventure.rom"
"$process_fixture" \
  --fixture-mode pcsx2-saturated-card \
  --path "$pcsx2_save_lifecycle_card"
"$process_fixture" \
  --fixture-mode pcsx2-restore-source \
  --path "$pcsx2_save_lifecycle_selected_source"
pcsx2_save_lifecycle_initial_member="$pcsx2_save_lifecycle_root/initial-member"
"$process_fixture" \
  --fixture-mode pcsx2-extract \
  --card "$pcsx2_save_lifecycle_card" \
  --member BASLUS-12345SAVE \
  --out "$pcsx2_save_lifecycle_initial_member"
cp "$pcsx2_save_lifecycle_card" \
  "$pcsx2_save_lifecycle_root/original-card.ps2"
(
  cd "$pcsx2_save_lifecycle_selected_source"
  7z a -t7z -mx=9 \
    "$pcsx2_save_lifecycle_vault/adventure.7z" \
    icon.sys save.bin >/dev/null
)
cp "$pcsx2_save_lifecycle_platform" \
  "$pcsx2_save_lifecycle_root/original-platform.xml"
pcsx2_save_lifecycle_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$pcsx2_save_lifecycle_root" \
    --pcsx2-save-lifecycle-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$pcsx2_save_lifecycle_output" >&2
  exit 1
}
if ! rg -q \
  'PCSX2_SAVE_LIFECYCLE_SMOKE_COMPLETE saves=3 writes=2 revision=2 data_changes=2' \
  <<< "$pcsx2_save_lifecycle_output"; then
  printf '%s\n' "$pcsx2_save_lifecycle_output" >&2
  echo "LaunchBox did not validate dialog-confirmed PCSX2 restore and active deletion." >&2
  exit 1
fi
if "$process_fixture" \
  --fixture-mode pcsx2-extract \
  --card "$pcsx2_save_lifecycle_card" \
  --member BASLUS-12345SAVE \
  --out "$pcsx2_save_lifecycle_root/unexpected-live-member" \
  >/dev/null 2>&1; then
  echo "PCSX2 lifecycle did not delete the selected raw-card member." >&2
  exit 1
fi
pcsx2_save_lifecycle_expected=(
  "$pcsx2_save_lifecycle_selected_source/save.bin"
  "$pcsx2_save_lifecycle_initial_member/save.bin"
  "$pcsx2_save_lifecycle_selected_source/save.bin"
)
pcsx2_save_lifecycle_archives=(
  "$pcsx2_save_lifecycle_vault/adventure.7z"
  "$pcsx2_save_lifecycle_vault/adventure-01.7z"
  "$pcsx2_save_lifecycle_vault/adventure-02.7z"
)
for index in 0 1 2; do
  archive="${pcsx2_save_lifecycle_archives[$index]}"
  extracted="$pcsx2_save_lifecycle_root/archive-check-$index"
  mkdir "$extracted"
  7z x -y -bd -bb0 "-o$extracted" -- "$archive" >/dev/null
  if ! cmp -s "$extracted/save.bin" \
    "${pcsx2_save_lifecycle_expected[$index]}"; then
    echo "PCSX2 lifecycle archive $archive has the wrong logical member bytes." >&2
    exit 1
  fi
done
mapfile -t pcsx2_save_lifecycle_card_backups < <(
  find "$(dirname "$pcsx2_save_lifecycle_card")" -maxdepth 1 -type f \
    -name 'Mcd001.ps2.lbport-backup-*' -print
)
if [[ ${#pcsx2_save_lifecycle_card_backups[@]} -ne 2 ]]; then
  echo "PCSX2 lifecycle did not retain exactly two complete raw-card recovery files." >&2
  exit 1
fi
pcsx2_save_lifecycle_original_recovery_count=0
pcsx2_save_lifecycle_selected_recovery_count=0
for index in "${!pcsx2_save_lifecycle_card_backups[@]}"; do
  recovery="${pcsx2_save_lifecycle_card_backups[$index]}"
  extracted="$pcsx2_save_lifecycle_root/recovery-check-$index"
  "$process_fixture" \
    --fixture-mode pcsx2-extract \
    --card "$recovery" \
    --member BASLUS-12345SAVE \
    --out "$extracted"
  if cmp -s "$recovery" \
    "$pcsx2_save_lifecycle_root/original-card.ps2"; then
    pcsx2_save_lifecycle_original_recovery_count=$((pcsx2_save_lifecycle_original_recovery_count + 1))
  fi
  if cmp -s "$extracted/save.bin" \
    "$pcsx2_save_lifecycle_selected_source/save.bin"; then
    pcsx2_save_lifecycle_selected_recovery_count=$((pcsx2_save_lifecycle_selected_recovery_count + 1))
  fi
done
if [[ $pcsx2_save_lifecycle_original_recovery_count -ne 1 ]] \
  || [[ $pcsx2_save_lifecycle_selected_recovery_count -ne 1 ]]; then
  echo "PCSX2 raw-card recovery files do not contain both exact pre-mutation states." >&2
  exit 1
fi
if [[ $(rg -c '<GameSave>' "$pcsx2_save_lifecycle_platform") -ne 3 ]] \
  || [[ $(rg -c -F \
    '<SaveGroupId>pcsx2:Mcd001:BASLUS-12345SAVE</SaveGroupId>' \
    "$pcsx2_save_lifecycle_platform") -ne 3 ]] \
  || rg -q -F \
    '<FilePath>Emulators\PCSX2\memcards\Mcd001.ps2</FilePath>' \
    "$pcsx2_save_lifecycle_platform" \
  || ! rg -q -F '<FutureRootElement>preserve-me</FutureRootElement>' \
    "$pcsx2_save_lifecycle_platform"; then
  echo "PCSX2 lifecycle did not retain exact vault history and unknown XML." >&2
  exit 1
fi
mapfile -t pcsx2_save_lifecycle_xml_backups < <(
  find "$pcsx2_save_lifecycle_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
pcsx2_save_lifecycle_original_found=false
for backup in "${pcsx2_save_lifecycle_xml_backups[@]}"; do
  if cmp -s "$backup" "$pcsx2_save_lifecycle_root/original-platform.xml"; then
    pcsx2_save_lifecycle_original_found=true
  fi
done
if [[ ${#pcsx2_save_lifecycle_xml_backups[@]} -ne 3 ]] \
  || [[ "$pcsx2_save_lifecycle_original_found" != true ]]; then
  echo "PCSX2 lifecycle did not retain all three XML recovery boundaries." >&2
  exit 1
fi
if find "$pcsx2_save_lifecycle_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful PCSX2 lifecycle left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox dialog-confirmed PCSX2 raw-card capacity recovery, restore/deletion, Qt-visible repair result, three exact vault versions, two complete-card recovery files, targeted refresh, and cleanup validated."

cp -R fixtures/launchbox/Data "$dolphin_wii_save_lifecycle_root/Data"
dolphin_wii_save_lifecycle_platform="$dolphin_wii_save_lifecycle_root/Data/Platforms/Fixture Console.xml"
sed -i \
  -e 's|<EmulatorFileName>fixture-emulator</EmulatorFileName>|<EmulatorFileName>Dolphin.exe</EmulatorFileName>|' \
  -e 's|<FilePath>Saves\\Fixture Adventure\\slot1.sav</FilePath>|<FilePath>Emulators\\Dolphin\\User\\Wii\\title\\00010000\\47414d45\\data</FilePath>|' \
  -e '/    <Slot>1<\/Slot>/d' \
  -e '/    <Title>Before the Final Puzzle<\/Title>/a\    <SaveGroupName>My Save File</SaveGroupName>\n    <SaveGroupId>dolphin:wii:fixture-adventure:00010000:47414d45</SaveGroupId>\n    <OriginalFileName>data</OriginalFileName>' \
  "$dolphin_wii_save_lifecycle_platform"
sed -i \
  '/<\/GameSave>/a\  <GameSave>\n    <EmulatorCore>fixture-core</EmulatorCore>\n    <EmulatorFileName>Dolphin.exe</EmulatorFileName>\n    <FilePath>Saves\\Fixture Console\\adventure.7z</FilePath>\n    <GameId>fixture-adventure</GameId>\n    <Title>Selected Wii Backup</Title>\n    <SaveGroupName>My Save File</SaveGroupName>\n    <SaveGroupId>dolphin:wii:fixture-adventure:00010000:47414d45</SaveGroupId>\n    <OriginalFileName>data</OriginalFileName>\n  </GameSave>' \
  "$dolphin_wii_save_lifecycle_platform"
dolphin_wii_save_lifecycle_active="$dolphin_wii_save_lifecycle_root/Emulators/Dolphin/User/Wii/title/00010000/47414d45/data"
dolphin_wii_save_lifecycle_parent="$dolphin_wii_save_lifecycle_root/Emulators/Dolphin/User/Wii/title/00010000/47414d45"
dolphin_wii_save_lifecycle_selected_source="$dolphin_wii_save_lifecycle_root/selected-data"
dolphin_wii_save_lifecycle_vault="$dolphin_wii_save_lifecycle_root/Saves/Fixture Console"
mkdir -p \
  "$dolphin_wii_save_lifecycle_active/nested/empty" \
  "$dolphin_wii_save_lifecycle_selected_source/course/empty" \
  "$dolphin_wii_save_lifecycle_vault" \
  "$dolphin_wii_save_lifecycle_root/Games/Fixture Adventure"
dolphin_wii_save_lifecycle_active_bytes='lifecycle current Wii progress'
dolphin_wii_save_lifecycle_selected_bytes='lifecycle selected Wii progress'
printf %s 'fixture rom' \
  > "$dolphin_wii_save_lifecycle_root/Games/Fixture Adventure/adventure.rom"
printf %s 'current banner' \
  > "$dolphin_wii_save_lifecycle_active/banner.bin"
printf %s "$dolphin_wii_save_lifecycle_active_bytes" \
  > "$dolphin_wii_save_lifecycle_active/nested/progress.dat"
printf %s 'selected banner' \
  > "$dolphin_wii_save_lifecycle_selected_source/banner.bin"
printf %s "$dolphin_wii_save_lifecycle_selected_bytes" \
  > "$dolphin_wii_save_lifecycle_selected_source/course/progress.dat"
(
  cd "$dolphin_wii_save_lifecycle_selected_source"
  7z a -t7z -mx=9 \
    "$dolphin_wii_save_lifecycle_vault/adventure.7z" \
    . >/dev/null
)
cp "$dolphin_wii_save_lifecycle_platform" \
  "$dolphin_wii_save_lifecycle_root/original-platform.xml"
dolphin_wii_save_lifecycle_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$dolphin_wii_save_lifecycle_root" \
    --dolphin-wii-save-lifecycle-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$dolphin_wii_save_lifecycle_output" >&2
  exit 1
}
if ! rg -q \
  'DOLPHIN_WII_SAVE_LIFECYCLE_SMOKE_COMPLETE saves=3 writes=2 revision=2 data_changes=2' \
  <<< "$dolphin_wii_save_lifecycle_output"; then
  printf '%s\n' "$dolphin_wii_save_lifecycle_output" >&2
  echo "LaunchBox did not validate dialog-confirmed Dolphin Wii restore and active deletion." >&2
  exit 1
fi
if [[ -e "$dolphin_wii_save_lifecycle_active" ]]; then
  echo "Dolphin Wii lifecycle did not delete the active title directory." >&2
  exit 1
fi
dolphin_wii_save_lifecycle_expected=(
  "$dolphin_wii_save_lifecycle_selected_bytes"
  "$dolphin_wii_save_lifecycle_active_bytes"
  "$dolphin_wii_save_lifecycle_selected_bytes"
)
dolphin_wii_save_lifecycle_members=(
  'course/progress.dat'
  'nested/progress.dat'
  'course/progress.dat'
)
dolphin_wii_save_lifecycle_empty=(
  'course/empty'
  'nested/empty'
  'course/empty'
)
dolphin_wii_save_lifecycle_archives=(
  "$dolphin_wii_save_lifecycle_vault/adventure.7z"
  "$dolphin_wii_save_lifecycle_vault/adventure-01.7z"
  "$dolphin_wii_save_lifecycle_vault/adventure-02.7z"
)
for index in 0 1 2; do
  archive="${dolphin_wii_save_lifecycle_archives[$index]}"
  extracted="$dolphin_wii_save_lifecycle_root/archive-check-$index"
  mkdir "$extracted"
  7z x -y -bd -bb0 "-o$extracted" -- "$archive" >/dev/null
  member="${dolphin_wii_save_lifecycle_members[$index]}"
  empty="${dolphin_wii_save_lifecycle_empty[$index]}"
  if [[ $(<"$extracted/$member") \
      != "${dolphin_wii_save_lifecycle_expected[$index]}" ]] \
    || [[ ! -d "$extracted/$empty" ]]; then
    echo "Dolphin Wii lifecycle archive $archive has the wrong nested tree." >&2
    exit 1
  fi
done
mapfile -t dolphin_wii_save_lifecycle_recoveries < <(
  find "$dolphin_wii_save_lifecycle_parent" -maxdepth 1 -type d \
    \( -name 'data.lbport-directory-backup-*' \
       -o -name 'data.lbport-directory-delete-backup-*' \) -print
)
if [[ ${#dolphin_wii_save_lifecycle_recoveries[@]} -ne 2 ]]; then
  echo "Dolphin Wii lifecycle did not retain exactly two complete recovery trees." >&2
  exit 1
fi
dolphin_wii_save_lifecycle_recovery_active=false
dolphin_wii_save_lifecycle_recovery_selected=false
for recovery_root in "${dolphin_wii_save_lifecycle_recoveries[@]}"; do
  recovered="$recovery_root/data"
  if [[ -f "$recovered/nested/progress.dat" ]] \
    && [[ $(<"$recovered/nested/progress.dat") \
      == "$dolphin_wii_save_lifecycle_active_bytes" ]] \
    && [[ -d "$recovered/nested/empty" ]]; then
    dolphin_wii_save_lifecycle_recovery_active=true
  fi
  if [[ -f "$recovered/course/progress.dat" ]] \
    && [[ $(<"$recovered/course/progress.dat") \
      == "$dolphin_wii_save_lifecycle_selected_bytes" ]] \
    && [[ -d "$recovered/course/empty" ]]; then
    dolphin_wii_save_lifecycle_recovery_selected=true
  fi
done
if [[ "$dolphin_wii_save_lifecycle_recovery_active" != true ]] \
  || [[ "$dolphin_wii_save_lifecycle_recovery_selected" != true ]]; then
  echo "Dolphin Wii recovery trees do not contain both pre-mutation states." >&2
  exit 1
fi
if [[ $(rg -c '<GameSave>' "$dolphin_wii_save_lifecycle_platform") -ne 3 ]] \
  || [[ $(rg -c -F \
    '<SaveGroupId>dolphin:wii:fixture-adventure:00010000:47414d45</SaveGroupId>' \
    "$dolphin_wii_save_lifecycle_platform") -ne 3 ]] \
  || rg -q -F \
    '<FilePath>Emulators\Dolphin\User\Wii\title\00010000\47414d45\data</FilePath>' \
    "$dolphin_wii_save_lifecycle_platform" \
  || ! rg -q -F '<FutureRootElement>preserve-me</FutureRootElement>' \
    "$dolphin_wii_save_lifecycle_platform"; then
  echo "Dolphin Wii lifecycle did not retain exact vault history and unknown XML." >&2
  exit 1
fi
mapfile -t dolphin_wii_save_lifecycle_xml_backups < <(
  find "$dolphin_wii_save_lifecycle_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
dolphin_wii_save_lifecycle_original_found=false
for backup in "${dolphin_wii_save_lifecycle_xml_backups[@]}"; do
  if cmp -s "$backup" \
      "$dolphin_wii_save_lifecycle_root/original-platform.xml"; then
    dolphin_wii_save_lifecycle_original_found=true
  fi
done
if [[ ${#dolphin_wii_save_lifecycle_xml_backups[@]} -ne 3 ]] \
  || [[ "$dolphin_wii_save_lifecycle_original_found" != true ]]; then
  echo "Dolphin Wii lifecycle did not retain all three XML recovery boundaries." >&2
  exit 1
fi
if find "$dolphin_wii_save_lifecycle_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful Dolphin Wii lifecycle left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox dialog-confirmed Dolphin Wii directory restore/deletion, three verified nested vault archives, two complete recovery trees, targeted refresh, and cleanup validated."

cp -R fixtures/launchbox/Data "$game_save_delete_root/Data"
game_save_delete_platform="$game_save_delete_root/Data/Platforms/Fixture Console.xml"
sed -i \
  's|<FilePath>Saves\\Fixture Adventure\\slot1.sav</FilePath>|<FilePath>Emulator\\Saves\\slot1.sav</FilePath>|' \
  "$game_save_delete_platform"
sed -i \
  '/<Title>Before the Final Puzzle<\/Title>/a\    <SaveGroupName>Delete Smoke</SaveGroupName>\n    <SaveGroupId>delete-smoke-group</SaveGroupId>' \
  "$game_save_delete_platform"
sed -i \
  '/<\/GameSave>/a\  <GameSave>\n    <EmulatorCore>fixture-core</EmulatorCore>\n    <EmulatorFileName>fixture-emulator</EmulatorFileName>\n    <FilePath>Saves\\Fixture Console\\adventure.sav</FilePath>\n    <GameId>fixture-adventure</GameId>\n    <Title>Vault Backup</Title>\n    <SaveGroupName>Delete Smoke</SaveGroupName>\n    <SaveGroupId>delete-smoke-group</SaveGroupId>\n    <OriginalFileName>slot1.sav</OriginalFileName>\n    <ReportedFileSizeBytes>24</ReportedFileSizeBytes>\n    <Md5>00000000000000000000000000000000</Md5>\n  </GameSave>' \
  "$game_save_delete_platform"
mkdir -p \
  "$game_save_delete_root/Emulator/Saves" \
  "$game_save_delete_root/Saves/Fixture Console"
game_save_delete_active="$game_save_delete_root/Emulator/Saves/slot1.sav"
game_save_delete_vault="$game_save_delete_root/Saves/Fixture Console/adventure.sav"
printf %s 'delete smoke active bytes' > "$game_save_delete_active"
printf %s 'delete smoke vault bytes' > "$game_save_delete_vault"
cp "$game_save_delete_platform" "$game_save_delete_root/original-platform.xml"
game_save_delete_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$game_save_delete_root" \
    --game-save-delete-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$game_save_delete_output" >&2
  exit 1
}
if ! rg -q \
  'GAME_SAVE_DELETE_SMOKE_COMPLETE saves=1 writes=1 revision=1 data_changes=1' \
  <<< "$game_save_delete_output"; then
  printf '%s\n' "$game_save_delete_output" >&2
  echo "LaunchBox did not validate dialog-confirmed vault save deletion." >&2
  exit 1
fi
if [[ -e "$game_save_delete_vault" ]]; then
  echo "Vault save deletion retained the selected vault file." >&2
  exit 1
fi
if [[ $(rg -c '<GameSave>' "$game_save_delete_platform") -ne 1 ]] \
  || ! rg -q -F '<FilePath>Emulator\Saves\slot1.sav</FilePath>' \
    "$game_save_delete_platform" \
  || rg -q -F '<Title>Vault Backup</Title>' "$game_save_delete_platform" \
  || ! rg -q -F '<SaveGroupId>delete-smoke-group</SaveGroupId>' \
    "$game_save_delete_platform" \
  || ! rg -q -F '<FutureRootElement>preserve-me</FutureRootElement>' \
    "$game_save_delete_platform"; then
  echo "Vault save deletion removed the wrong row or lost retained XML." >&2
  exit 1
fi
if [[ $(<"$game_save_delete_active") != 'delete smoke active bytes' ]]; then
  echo "Vault save deletion changed the active emulator save." >&2
  exit 1
fi
mapfile -t game_save_delete_xml_backups < <(
  find "$game_save_delete_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#game_save_delete_xml_backups[@]} -ne 1 ]] \
  || ! cmp -s "${game_save_delete_xml_backups[0]}" \
    "$game_save_delete_root/original-platform.xml"; then
  echo "Vault save deletion did not retain one exact XML recovery copy." >&2
  exit 1
fi
mapfile -t game_save_delete_file_backups < <(
  find "$game_save_delete_root/Saves/Fixture Console" -maxdepth 1 -type f \
    -name 'adventure.sav.lbport-transaction-backup-*' -print
)
if [[ ${#game_save_delete_file_backups[@]} -ne 1 ]] \
  || [[ $(<"${game_save_delete_file_backups[0]}") != 'delete smoke vault bytes' ]]; then
  echo "Vault save deletion did not retain one exact file recovery copy." >&2
  exit 1
fi
if find "$game_save_delete_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful vault save deletion left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox dialog-confirmed vault save deletion, exact file/XML recovery copies, active isolation, targeted model refresh, and cleanup validated."

cp -R fixtures/launchbox/Data "$game_save_active_delete_root/Data"
game_save_active_delete_platform="$game_save_active_delete_root/Data/Platforms/Fixture Console.xml"
sed -i \
  -e 's|<EmulatorCore>fixture-core</EmulatorCore>|<EmulatorCore>mednafen_saturn_libretro</EmulatorCore>|' \
  -e 's|<EmulatorFileName>fixture-emulator</EmulatorFileName>|<EmulatorFileName>retroarch</EmulatorFileName>|' \
  -e 's|<FilePath>Saves\\Fixture Adventure\\slot1.sav</FilePath>|<FilePath>Emulator\\Saves\\adventure.bcr</FilePath>|' \
  -e '/    <Slot>1<\/Slot>/d' \
  -e '/<Title>Before the Final Puzzle<\/Title>/a\    <SaveGroupName>My Save File</SaveGroupName>\n    <SaveGroupId>saturn-adventure</SaveGroupId>' \
  "$game_save_active_delete_platform"
mkdir -p "$game_save_active_delete_root/Emulator/Saves"
declare -A game_save_active_delete_bytes=(
  [bcr]='active delete current cartridge bytes'
  [bkr]='active delete current backup ram bytes'
  [smpc]='active delete current clock bytes'
)
for extension in bcr bkr smpc; do
  printf %s "${game_save_active_delete_bytes[$extension]}" \
    > "$game_save_active_delete_root/Emulator/Saves/adventure.$extension"
done
cp "$game_save_active_delete_platform" \
  "$game_save_active_delete_root/original-platform.xml"
game_save_active_delete_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$game_save_active_delete_root" \
    --game-save-active-delete-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$game_save_active_delete_output" >&2
  exit 1
}
if ! rg -q \
  'GAME_SAVE_ACTIVE_DELETE_SMOKE_COMPLETE saves=1 writes=1 revision=1 data_changes=1' \
  <<< "$game_save_active_delete_output"; then
  printf '%s\n' "$game_save_active_delete_output" >&2
  echo "LaunchBox did not validate dialog-confirmed active Saturn save deletion." >&2
  exit 1
fi
for extension in bcr bkr smpc; do
  active="$game_save_active_delete_root/Emulator/Saves/adventure.$extension"
  vault="$game_save_active_delete_root/Saves/Fixture Console/adventure.$extension"
  if [[ -e "$active" ]] \
    || [[ $(<"$vault") != "${game_save_active_delete_bytes[$extension]}" ]]; then
    echo "Active save deletion did not archive and remove the $extension member exactly." >&2
    exit 1
  fi
  mapfile -t active_delete_backups < <(
    find "$game_save_active_delete_root/Emulator/Saves" -maxdepth 1 -type f \
      -name "adventure.$extension.lbport-delete-backup-*" -print
  )
  if [[ ${#active_delete_backups[@]} -ne 1 ]] \
    || [[ $(<"${active_delete_backups[0]}") \
      != "${game_save_active_delete_bytes[$extension]}" ]]; then
    echo "Active save deletion did not retain one exact $extension recovery copy." >&2
    exit 1
  fi
done
game_save_active_delete_size=0
for extension in bcr bkr smpc; do
  ((game_save_active_delete_size += \
    ${#game_save_active_delete_bytes[$extension]}))
done
if [[ $(rg -c '<GameSave>' "$game_save_active_delete_platform") -ne 1 ]] \
  || ! rg -q -F \
    '<FilePath>Saves\Fixture Console\adventure.bcr</FilePath>' \
    "$game_save_active_delete_platform" \
  || rg -q -F '<FilePath>Emulator\Saves\adventure.bcr</FilePath>' \
    "$game_save_active_delete_platform" \
  || ! rg -q -F '<SaveGroupId>saturn-adventure</SaveGroupId>' \
    "$game_save_active_delete_platform" \
  || ! rg -q -F \
    "<ReportedFileSizeBytes>$game_save_active_delete_size</ReportedFileSizeBytes>" \
    "$game_save_active_delete_platform" \
  || ! rg -q '<ReportedLastModifiedUtc>.*\.[0-9]{7}Z' \
    "$game_save_active_delete_platform" \
  || ! rg -q '<Md5>[0-9A-F]{32}</Md5>' \
    "$game_save_active_delete_platform" \
  || ! rg -q -F '<FutureRootElement>preserve-me</FutureRootElement>' \
    "$game_save_active_delete_platform"; then
  echo "Active save deletion did not persist its exact portable recovery version." >&2
  exit 1
fi
mapfile -t game_save_active_delete_xml_backups < <(
  find "$game_save_active_delete_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#game_save_active_delete_xml_backups[@]} -ne 1 ]] \
  || ! cmp -s "${game_save_active_delete_xml_backups[0]}" \
    "$game_save_active_delete_root/original-platform.xml"; then
  echo "Active save deletion did not retain one exact pre-delete XML recovery copy." >&2
  exit 1
fi
if find "$game_save_active_delete_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful active save deletion left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox dialog-confirmed active Saturn save deletion, mandatory portable full-set archive, exact external-capable sibling recovery copies, targeted refresh, and cleanup validated."

cp -R fixtures/launchbox/Data "$game_save_restore_root/Data"
game_save_restore_platform="$game_save_restore_root/Data/Platforms/Fixture Console.xml"
sed -i \
  -e 's|<EmulatorFileName>fixture-emulator</EmulatorFileName>|<EmulatorFileName>Dolphin.exe</EmulatorFileName>|' \
  -e 's|<FilePath>Saves\\Fixture Adventure\\slot1.sav</FilePath>|<FilePath>Emulator\\Saves\\slot1.sav</FilePath>|' \
  -e '/    <Slot>1<\/Slot>/d' \
  "$game_save_restore_platform"
sed -i \
  '/<Title>Before the Final Puzzle<\/Title>/a\    <SaveGroupName>GameCube Save</SaveGroupName>\n    <SaveGroupId>dolphin:gc:fixture-adventure:GAME01:Folder:slot1.sav</SaveGroupId>' \
  "$game_save_restore_platform"
sed -i \
  '/<\/GameSave>/a\  <GameSave>\n    <EmulatorCore>fixture-core</EmulatorCore>\n    <EmulatorFileName>Dolphin.exe</EmulatorFileName>\n    <FilePath>Saves\\Fixture Console\\adventure.sav</FilePath>\n    <GameId>fixture-adventure</GameId>\n    <Title>Older Vault Version</Title>\n    <SaveGroupName>GameCube Save</SaveGroupName>\n    <SaveGroupId>dolphin:gc:fixture-adventure:GAME01:Folder:slot1.sav</SaveGroupId>\n    <OriginalFileName>slot1.sav</OriginalFileName>\n  </GameSave>' \
  "$game_save_restore_platform"
mkdir -p \
  "$game_save_restore_root/Emulator/Saves" \
  "$game_save_restore_root/Saves/Fixture Console"
game_save_restore_active="$game_save_restore_root/Emulator/Saves/slot1.sav"
game_save_restore_selected="$game_save_restore_root/Saves/Fixture Console/adventure.sav"
game_save_restore_new_backup="$game_save_restore_root/Saves/Fixture Console/adventure-01.sav"
game_save_restore_active_bytes='restore smoke current active bytes'
game_save_restore_selected_bytes='restore smoke selected vault bytes'
printf %s "$game_save_restore_active_bytes" > "$game_save_restore_active"
printf %s "$game_save_restore_selected_bytes" > "$game_save_restore_selected"
cp "$game_save_restore_platform" "$game_save_restore_root/original-platform.xml"
game_save_restore_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$game_save_restore_root" \
    --game-save-restore-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$game_save_restore_output" >&2
  exit 1
}
if ! rg -q \
  'GAME_SAVE_RESTORE_SMOKE_COMPLETE saves=3 writes=1 revision=1 data_changes=1' \
  <<< "$game_save_restore_output"; then
  printf '%s\n' "$game_save_restore_output" >&2
  echo "LaunchBox did not validate dialog-confirmed regular-file restore." >&2
  exit 1
fi
if [[ $(<"$game_save_restore_active") != "$game_save_restore_selected_bytes" ]] \
  || [[ $(<"$game_save_restore_selected") != "$game_save_restore_selected_bytes" ]] \
  || [[ $(<"$game_save_restore_new_backup") != "$game_save_restore_active_bytes" ]]; then
  echo "Regular-file restore did not preserve selected/current bytes in the expected destinations." >&2
  exit 1
fi
game_save_restore_md5=$(
  md5sum "$game_save_restore_new_backup" | cut -d ' ' -f 1 | tr '[:lower:]' '[:upper:]'
)
if [[ $(rg -c '<GameSave>' "$game_save_restore_platform") -ne 3 ]] \
  || [[ $(rg -c -F \
    '<SaveGroupId>dolphin:gc:fixture-adventure:GAME01:Folder:slot1.sav</SaveGroupId>' \
    "$game_save_restore_platform") -ne 3 ]] \
  || ! rg -q -F \
    '<FilePath>Saves\Fixture Console\adventure-01.sav</FilePath>' \
    "$game_save_restore_platform" \
  || ! rg -q -F "<Md5>$game_save_restore_md5</Md5>" \
    "$game_save_restore_platform" \
  || ! rg -q -F '<FutureRootElement>preserve-me</FutureRootElement>' \
    "$game_save_restore_platform"; then
  echo "Regular-file restore did not persist the exact pre-restore active version losslessly." >&2
  exit 1
fi
mapfile -t game_save_restore_xml_backups < <(
  find "$game_save_restore_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#game_save_restore_xml_backups[@]} -ne 1 ]] \
  || ! cmp -s "${game_save_restore_xml_backups[0]}" \
    "$game_save_restore_root/original-platform.xml"; then
  echo "Regular-file restore did not retain one exact pre-restore XML recovery copy." >&2
  exit 1
fi
mapfile -t game_save_restore_active_backups < <(
  find "$game_save_restore_root/Emulator/Saves" -maxdepth 1 -type f \
    -name 'slot1.sav.lbport-backup-*' -print
)
if [[ ${#game_save_restore_active_backups[@]} -ne 1 ]] \
  || [[ $(<"${game_save_restore_active_backups[0]}") \
    != "$game_save_restore_active_bytes" ]]; then
  echo "Regular-file restore did not retain one exact active-file recovery copy." >&2
  exit 1
fi
if find "$game_save_restore_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful regular-file restore left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox dialog-confirmed Dolphin regular-file restore, mandatory active backup, atomic replacement, exact recovery copies, targeted refresh, and cleanup validated."

cp -R fixtures/launchbox/Data "$game_save_saturn_restore_root/Data"
game_save_saturn_restore_platform="$game_save_saturn_restore_root/Data/Platforms/Fixture Console.xml"
sed -i \
  -e 's|<EmulatorCore>fixture-core</EmulatorCore>|<EmulatorCore>mednafen_saturn_libretro</EmulatorCore>|' \
  -e 's|<EmulatorFileName>fixture-emulator</EmulatorFileName>|<EmulatorFileName>retroarch</EmulatorFileName>|' \
  -e 's|<FilePath>Saves\\Fixture Adventure\\slot1.sav</FilePath>|<FilePath>Emulator\\Saves\\adventure.bcr</FilePath>|' \
  -e '/    <Slot>1<\/Slot>/d' \
  -e '/<Title>Before the Final Puzzle<\/Title>/a\    <SaveGroupName>My Save File</SaveGroupName>\n    <SaveGroupId>saturn-adventure</SaveGroupId>' \
  -e '/<FutureRootElement>preserve-me<\/FutureRootElement>/i\  <GameSave>\n    <EmulatorCore>mednafen_saturn_libretro</EmulatorCore>\n    <EmulatorFileName>retroarch</EmulatorFileName>\n    <FilePath>Saves\\Fixture Console\\adventure.bcr</FilePath>\n    <GameId>fixture-adventure</GameId>\n    <Title>Selected Saturn Backup</Title>\n    <SaveGroupName>My Save File</SaveGroupName>\n    <SaveGroupId>saturn-adventure</SaveGroupId>\n    <OriginalFileName>adventure.bcr</OriginalFileName>\n  </GameSave>' \
  "$game_save_saturn_restore_platform"
mkdir -p \
  "$game_save_saturn_restore_root/Emulator/Saves" \
  "$game_save_saturn_restore_root/Saves/Fixture Console"
declare -A game_save_saturn_restore_active_bytes=(
  [bcr]='saturn restore current cartridge bytes'
  [bkr]='saturn restore current backup ram bytes'
  [smpc]='saturn restore current clock bytes'
)
declare -A game_save_saturn_restore_selected_bytes=(
  [bcr]='saturn restore selected cartridge bytes'
  [bkr]='saturn restore selected backup ram bytes'
  [smpc]='saturn restore selected clock bytes'
)
for extension in bcr bkr smpc; do
  printf %s "${game_save_saturn_restore_active_bytes[$extension]}" \
    > "$game_save_saturn_restore_root/Emulator/Saves/adventure.$extension"
  printf %s "${game_save_saturn_restore_selected_bytes[$extension]}" \
    > "$game_save_saturn_restore_root/Saves/Fixture Console/adventure.$extension"
done
cp "$game_save_saturn_restore_platform" \
  "$game_save_saturn_restore_root/original-platform.xml"
game_save_saturn_restore_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$game_save_saturn_restore_root" \
    --game-save-saturn-restore-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$game_save_saturn_restore_output" >&2
  exit 1
}
if ! rg -q \
  'GAME_SAVE_SATURN_RESTORE_SMOKE_COMPLETE saves=3 writes=1 revision=1 data_changes=1' \
  <<< "$game_save_saturn_restore_output"; then
  printf '%s\n' "$game_save_saturn_restore_output" >&2
  echo "LaunchBox did not validate dialog-confirmed RetroArch Saturn set restore." >&2
  exit 1
fi
for extension in bcr bkr smpc; do
  active="$game_save_saturn_restore_root/Emulator/Saves/adventure.$extension"
  selected="$game_save_saturn_restore_root/Saves/Fixture Console/adventure.$extension"
  pre_restore="$game_save_saturn_restore_root/Saves/Fixture Console/adventure-01.$extension"
  if [[ $(<"$active") != "${game_save_saturn_restore_selected_bytes[$extension]}" ]] \
    || [[ $(<"$selected") != "${game_save_saturn_restore_selected_bytes[$extension]}" ]] \
    || [[ $(<"$pre_restore") != "${game_save_saturn_restore_active_bytes[$extension]}" ]]; then
    echo "Saturn restore did not preserve selected/current $extension bytes in the expected destinations." >&2
    exit 1
  fi
  mapfile -t active_backups < <(
    find "$game_save_saturn_restore_root/Emulator/Saves" -maxdepth 1 -type f \
      -name "adventure.$extension.lbport-transaction-backup-*" -print
  )
  if [[ ${#active_backups[@]} -ne 1 ]] \
    || [[ $(<"${active_backups[0]}") \
      != "${game_save_saturn_restore_active_bytes[$extension]}" ]]; then
    echo "Saturn restore did not retain one exact $extension recovery copy." >&2
    exit 1
  fi
done
game_save_saturn_restore_size=0
for extension in bcr bkr smpc; do
  ((game_save_saturn_restore_size += \
    ${#game_save_saturn_restore_active_bytes[$extension]}))
done
if [[ $(rg -c '<GameSave>' "$game_save_saturn_restore_platform") -ne 3 ]] \
  || [[ $(rg -c -F '<SaveGroupId>saturn-adventure</SaveGroupId>' \
    "$game_save_saturn_restore_platform") -ne 3 ]] \
  || ! rg -q -F \
    '<FilePath>Saves\Fixture Console\adventure-01.bcr</FilePath>' \
    "$game_save_saturn_restore_platform" \
  || ! rg -q -F \
    "<ReportedFileSizeBytes>$game_save_saturn_restore_size</ReportedFileSizeBytes>" \
    "$game_save_saturn_restore_platform" \
  || ! rg -q '<ReportedLastModifiedUtc>.*\.[0-9]{7}Z' \
    "$game_save_saturn_restore_platform" \
  || ! rg -q '<Md5>[0-9A-F]{32}</Md5>' \
    "$game_save_saturn_restore_platform" \
  || ! rg -q -F '<FutureRootElement>preserve-me</FutureRootElement>' \
    "$game_save_saturn_restore_platform"; then
  echo "Saturn restore did not persist the exact pre-restore set metadata losslessly." >&2
  exit 1
fi
mapfile -t game_save_saturn_restore_xml_backups < <(
  find "$game_save_saturn_restore_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#game_save_saturn_restore_xml_backups[@]} -ne 1 ]] \
  || ! cmp -s "${game_save_saturn_restore_xml_backups[0]}" \
    "$game_save_saturn_restore_root/original-platform.xml"; then
  echo "Saturn restore did not retain one exact pre-restore XML recovery copy." >&2
  exit 1
fi
if find "$game_save_saturn_restore_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful Saturn restore left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox dialog-confirmed RetroArch Saturn set restore, mandatory full-set backup, atomic companion replacement, exact recovery copies, targeted refresh, and cleanup validated."

cp -R fixtures/launchbox/Data "$import_root/Data"
mkdir -p "$import_root/Metadata"
sqlite3 "$import_root/Metadata/LaunchBox.Metadata.db" \
  < fixtures/launchbox/Metadata/fixture.sql
sqlite3 "$import_root/Metadata/LaunchBox.Metadata.db" \
  "INSERT INTO Games VALUES (
    4343, 'Fixture Saga Collector (USA)', 'FIXTURE SAGA COLLECTOR', NULL, 2004,
    'Collector overview', 4, 'Released', 0, NULL, 4.0, 21, NULL,
    'Fixture Console', 'E10+', 'Role-Playing', 'Collector Forge',
    'Collector Press'
  );"
printf 'disc-one-import-bytes' > \
  "$import_source_root/Fixture Sag (USA) - (Disc 1 of 2).rom"
printf 'disc-two-import-bytes' > \
  "$import_source_root/Fixture Sag (USA) - (Disc 2 of 2).rom"
printf 'disc-one-companion-bytes' > \
  "$import_source_root/Fixture Sag (USA) - (Disc 1 of 2).dat"
printf 'disc-two-companion-bytes' > \
  "$import_source_root/Fixture Sag (USA) - (Disc 2 of 2).dat"
printf 'game-manual-pdf-bytes' > \
  "$import_source_root/Fixture Sag (USA) - (Disc 1 of 2).pdf"
mkdir -p "$import_source_root/versions"
printf 'usa-version-import-bytes' > \
  "$import_source_root/versions/Fixture Saga (USA).rom"
printf 'world-version-import-bytes' > \
  "$import_source_root/versions/Fixture Saga (World) (Rev 1).rom"
import_platform="$import_root/Data/Platforms/Fixture Console.xml"
import_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$import_root" --import-smoke-test \
    --import-rom-1 "$import_source_root/Fixture Sag (USA) - (Disc 1 of 2).rom" \
    --import-rom-2 "$import_source_root/Fixture Sag (USA) - (Disc 2 of 2).rom" \
    --import-rom-3 "$import_source_root/versions/Fixture Saga (USA).rom" \
    --import-rom-4 "$import_source_root/versions/Fixture Saga (World) (Rev 1).rom" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$import_output" >&2
  exit 1
}
if ! rg -q 'IMPORT_SMOKE_COMPLETE imported=2 created=7 moved=0 model_games=5' \
  <<< "$import_output"; then
  printf '%s\n' "$import_output" >&2
  echo "LaunchBox did not complete the dialog-driven ROM import preview and copy." >&2
  exit 1
fi
for expected in \
  '<Title>Fixture Saga (USA)</Title>' \
  '<DatabaseID>4242</DatabaseID>' \
  '<Notes>Recovered local metadata overview.</Notes>' \
  '<Developer>Fixture Forge</Developer>' \
  '<Genre>Role-Playing; Strategy</Genre>' \
  '<MaxPlayers>2</MaxPlayers>' \
  '<PlayMode>Cooperative; Multiplayer</PlayMode>' \
  '<Publisher>Fixture Press</Publisher>' \
  '<Rating>E10+</Rating>' \
  '<ReleaseDate>2002-03-04</ReleaseDate>' \
  '<ReleaseType>Released</ReleaseType>' \
  '<WikipediaURL>https://example.org/wiki/Fixture_Saga</WikipediaURL>' \
  '<VideoUrl>https://video.example/fixture-saga</VideoUrl>' \
  '<CommunityStarRating>4.75</CommunityStarRating>' \
  '<Status>Imported ROM</Status>' \
  '<Region>North America</Region>' \
  '<Version>(USA)</Version>' \
  '<ManualPath>Games\Fixture Console\Fixture Saga (USA) (2002)\Fixture Sag (USA) - (Disc 1 of 2).pdf</ManualPath>' \
  '<ApplicationPath>Games\Fixture Console\Fixture Saga (USA) (2002)\Fixture Sag (USA) - (Disc 1 of 2).rom</ApplicationPath>' \
  '<ApplicationPath>Games\Fixture Console\Fixture Saga (USA) (2002)\Fixture Sag (USA) - (Disc 2 of 2).rom</ApplicationPath>' \
  '<ApplicationPath>Games\Fixture Console\Fixture Saga (USA) (2002)\Fixture Saga (USA).rom</ApplicationPath>' \
  '<ApplicationPath>Games\Fixture Console\Fixture Saga (USA) (2002)\Fixture Saga (World) (Rev 1).rom</ApplicationPath>' \
  '<Name>Play (USA) Disc 1...</Name>' \
  '<Name>Play (USA) Disc 2...</Name>' \
  '<Name>Play (USA) Version...</Name>' \
  '<Name>Play (World) (Rev 1) Version...</Name>' \
  '<Region>World</Region>' \
  '<Version>(World) (Rev 1)</Version>' \
  '<Disc>1</Disc>' \
  '<Disc>2</Disc>' \
  '<Priority>1</Priority>' \
  '<Priority>2</Priority>' \
  '<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>'; do
  if ! rg -q -F "$expected" "$import_platform"; then
    echo "ROM import platform XML is missing: $expected" >&2
    exit 1
  fi
done
if [[ $(rg -c -F '<Emulator>fixture-emulator</Emulator>' "$import_platform") -ne 3 ]] \
  || [[ $(rg -c -F '<EmulatorId>fixture-emulator</EmulatorId>' "$import_platform") -ne 4 ]]; then
  echo "ROM import did not pin the selected emulator on both games and all combined ROM records." >&2
  exit 1
fi
for file_name in \
  'Fixture Sag (USA) - (Disc 1 of 2).rom' \
  'Fixture Sag (USA) - (Disc 2 of 2).rom' \
  'Fixture Sag (USA) - (Disc 1 of 2).dat' \
  'Fixture Sag (USA) - (Disc 2 of 2).dat' \
  'Fixture Sag (USA) - (Disc 1 of 2).pdf'; do
  if ! cmp -s "$import_source_root/$file_name" \
    "$import_root/Games/Fixture Console/Fixture Saga (USA) (2002)/$file_name"; then
    echo "ROM import did not preserve exact bytes for $file_name." >&2
    exit 1
  fi
done
for file_name in \
  'Fixture Saga (USA).rom' \
  'Fixture Saga (World) (Rev 1).rom'; do
  if ! cmp -s "$import_source_root/versions/$file_name" \
    "$import_root/Games/Fixture Console/Fixture Saga (USA) (2002)/$file_name"; then
    echo "ROM version import did not preserve exact bytes for $file_name." >&2
    exit 1
  fi
done
mapfile -t import_backups < <(
  find "$import_root/Data/Platforms" -maxdepth 1 -type f \
    -name '*.lbport-transaction-backup-*' -print
)
if [[ ${#import_backups[@]} -ne 1 ]] \
  || ! cmp -s "${import_backups[0]}" \
    'fixtures/launchbox/Data/Platforms/Fixture Console.xml'; then
  echo "ROM import did not retain exactly one exact platform XML backup." >&2
  exit 1
fi
if find "$import_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful ROM import left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox dialog-driven multi-disc and metadata-resolved matching-title ROM import, recovered exact-to-partial local-metadata fallback, ambiguous partial review by stable database ID and typed persistence, filename version/region recovery, selectable version applications, PDF-manual discovery and re-planned portable ManualPath persistence, title/year subfolder planning, same-stem companion copying, validated emulator selection, editable grouped preview, exact streamed bytes, additional-application persistence, shared transaction recovery, and source preservation validated."

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
if [[ ${#platform_catalog_backups[@]} -ne 3 ]]; then
  echo "Platform create/edit/delete did not retain exactly three catalog backups." >&2
  exit 1
fi
platform_original_catalog_backups=0
platform_created_catalog_backups=0
platform_edited_catalog_backups=0
for backup in "${platform_catalog_backups[@]}"; do
  if cmp -s "$backup" fixtures/launchbox/Data/Platforms.xml; then
    ((platform_original_catalog_backups += 1))
  elif rg -q -F '<Name>Dragon 32/64</Name>' "$backup" \
    && rg -q -F '<SortTitle>Dragon, 32/64</SortTitle>' "$backup" \
    && rg -q -F '<Developer>Qt Forge</Developer>' "$backup" \
    && rg -q -F '<Cpu>6809</Cpu>' "$backup" \
    && rg -q -F '<Notes>Edited through the real platform dialog.</Notes>' "$backup" \
    && rg -q -F '<HideInBigBox>true</HideInBigBox>' "$backup" \
    && rg -q -F '<DisableAutoImport>true</DisableAutoImport>' "$backup" \
    && [[ $(rg -c -F '<Platform>Dragon 32/64</Platform>' "$backup") -eq 52 ]] \
    && rg -q -F '<FolderPath>Images\Dragon 32_64\Edited</FolderPath>' "$backup" \
    && rg -q -F '<MediaType>Test Media</MediaType>' "$backup" \
    && rg -q -F '<FolderPath>Portable\Dragon 32_64</FolderPath>' "$backup"; then
    ((platform_edited_catalog_backups += 1))
  elif rg -q -F '<Name>Dragon 32/64</Name>' "$backup" \
    && rg -q -F '<ScrapeAs>Dragon 32/64</ScrapeAs>' "$backup" \
    && [[ $(rg -c -F '<Platform>Dragon 32/64</Platform>' "$backup") -eq 51 ]] \
    && rg -q -F '<FolderPath>Images\Dragon 32_64\Box - Front</FolderPath>' "$backup"; then
    ((platform_created_catalog_backups += 1))
  fi
done
if [[ $platform_original_catalog_backups -ne 1 \
  || $platform_created_catalog_backups -ne 1 \
  || $platform_edited_catalog_backups -ne 1 ]]; then
  echo "Platform catalog backups do not prove the expected portable create/edit/delete chain." >&2
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

echo "LaunchBox dialog-driven platform lifecycle and metadata/folder editing, portable filenames, lexical Windows paths, reference gating, exact backups, and media isolation validated."

cp -R fixtures/launchbox/Data "$emulator_crud_root/Data"
emulator_crud_document="$emulator_crud_root/Data/Emulators.xml"
sed -i \
  '/<Title>Fixture Emulator<\/Title>/a\    <FutureEmulatorField>keep-emulator-data</FutureEmulatorField>' \
  "$emulator_crud_document"
sed -i \
  '/<M3uDiscLoadEnabled>false<\/M3uDiscLoadEnabled>/a\    <FutureMappingField>keep-mapping-data</FutureMappingField>' \
  "$emulator_crud_document"
emulator_crud_original="$emulator_crud_root/original-emulators.xml"
cp "$emulator_crud_document" "$emulator_crud_original"

emulator_crud_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$emulator_crud_root" --emulator-crud-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$emulator_crud_output" >&2
  exit 1
}
if ! rg -q \
  'EMULATOR_CRUD_SMOKE_COMPLETE emulator=.* blocked=1 writes=3 revision=[0-9]+' \
  <<< "$emulator_crud_output"; then
  printf '%s\n' "$emulator_crud_output" >&2
  echo "LaunchBox did not validate dialog-driven emulator management." >&2
  exit 1
fi
for expected in \
  '<Title>Edited Fixture Emulator</Title>' \
  '<ApplicationPath>Emulators\Edited Fixture\fixture.exe</ApplicationPath>' \
  '<AutoHotkeyScript>Smoke launch script</AutoHotkeyScript>' \
  '<UsePauseScreen>true</UsePauseScreen>' \
  '<UseStartupScreen>true</UseStartupScreen>' \
  '<CommandLine>--edited-mapping</CommandLine>' \
  '<Default>true</Default>' \
  '<AutoExtract>false</AutoExtract>' \
  '<M3uDiscLoadEnabled>true</M3uDiscLoadEnabled>' \
  '<FutureEmulatorField>keep-emulator-data</FutureEmulatorField>' \
  '<FutureMappingField>keep-mapping-data</FutureMappingField>'; do
  if ! rg -q -F "$expected" "$emulator_crud_document"; then
    echo "Emulator CRUD did not retain expected XML: $expected" >&2
    exit 1
  fi
done
if rg -q -F 'Temporary Qt Emulator' "$emulator_crud_document" \
  || [[ $(rg -c '^  <Emulator>$' "$emulator_crud_document") -ne 1 ]] \
  || [[ $(rg -c '^  <EmulatorPlatform>$' "$emulator_crud_document") -ne 1 ]]; then
  echo "Emulator CRUD retained the deleted temporary emulator or mapping." >&2
  exit 1
fi
if [[ -e "$emulator_crud_root/Emulators/Edited Fixture" ]] \
  || [[ -e "$emulator_crud_root/C:\Portable\Temporary Qt" ]]; then
  echo "Emulator CRUD interpreted a stored executable path as a host directory." >&2
  exit 1
fi

mapfile -t emulator_crud_backups < <(
  find "$emulator_crud_root/Data" -maxdepth 1 -type f \
    -name 'Emulators.xml.lbport-transaction-backup-*' -print
)
if [[ ${#emulator_crud_backups[@]} -ne 3 ]]; then
  echo "Emulator edit/create/delete did not retain exactly three XML backups." >&2
  exit 1
fi
emulator_original_backups=0
emulator_edited_backups=0
emulator_created_backups=0
for backup in "${emulator_crud_backups[@]}"; do
  if cmp -s "$backup" "$emulator_crud_original"; then
    ((emulator_original_backups += 1))
  elif rg -q -F '<Title>Temporary Qt Emulator</Title>' "$backup" \
    && rg -q -F \
      '<ApplicationPath>C:\Portable\Temporary Qt\temp.exe</ApplicationPath>' \
      "$backup"; then
    ((emulator_created_backups += 1))
  elif rg -q -F '<Title>Edited Fixture Emulator</Title>' "$backup" \
    && ! rg -q -F '<Title>Temporary Qt Emulator</Title>' "$backup"; then
    ((emulator_edited_backups += 1))
  fi
  if ! rg -q -F '<FutureEmulatorField>keep-emulator-data</FutureEmulatorField>' \
    "$backup" \
    || ! rg -q -F '<FutureMappingField>keep-mapping-data</FutureMappingField>' \
      "$backup"; then
    echo "An emulator lifecycle backup lost unknown XML." >&2
    exit 1
  fi
done
if [[ $emulator_original_backups -ne 1 \
  || $emulator_edited_backups -ne 1 \
  || $emulator_created_backups -ne 1 ]]; then
  echo "Emulator backups do not prove the expected edit/create/delete chain." >&2
  exit 1
fi
if find "$emulator_crud_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful emulator CRUD smoke left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox dialog-driven full emulator and platform-mapping editing, generated immutable IDs, default handoff, reference gating, lexical Windows paths, unknown XML, exact backups, and binary-directory isolation validated."

cp -R fixtures/launchbox/Data "$retroarch_core_editor_root/Data"
retroarch_core_editor_document="$retroarch_core_editor_root/Data/Emulators.xml"
sed -i 's/Fixture Console/Super Nintendo Entertainment System/g' \
  "$retroarch_core_editor_root/Data/Platforms.xml" \
  "$retroarch_core_editor_root/Data/Platforms/Fixture Console.xml" \
  "$retroarch_core_editor_document"
sed -i \
  -e 's/<Title>Fixture Emulator<\/Title>/<Title>RetroArch<\/Title>/' \
  -e 's#<ApplicationPath>Emulators/fixture-emulator</ApplicationPath>#<ApplicationPath>Emulators/RetroArch/retroarch</ApplicationPath>#' \
  "$retroarch_core_editor_document"
retroarch_core_directory="$retroarch_core_editor_root/Emulators/RetroArch/cores"
mkdir -p "$retroarch_core_directory"
install_process_fixture \
  "$retroarch_core_editor_root/Emulators/RetroArch/retroarch"
printf 'libretro_directory = "cores"\n' \
  > "$retroarch_core_editor_root/Emulators/RetroArch/retroarch.cfg"
case "$(uname -s)" in
  Darwin) retroarch_core_extension=dylib ;;
  MINGW*|MSYS*|CYGWIN*) retroarch_core_extension=dll ;;
  *) retroarch_core_extension=so ;;
esac
printf 'snes core\n' \
  > "$retroarch_core_directory/snes9x_libretro.$retroarch_core_extension"
printf 'genesis core\n' \
  > "$retroarch_core_directory/genesis_plus_gx_libretro.$retroarch_core_extension"
ln -s "snes9x_libretro.$retroarch_core_extension" \
  "$retroarch_core_directory/unsafe_libretro.$retroarch_core_extension"
cp "$retroarch_core_editor_root/Emulators/RetroArch/retroarch" \
  "$retroarch_core_editor_root/original-retroarch"
cp "$retroarch_core_editor_root/Emulators/RetroArch/retroarch.cfg" \
  "$retroarch_core_editor_root/original-retroarch.cfg"
cp "$retroarch_core_directory/snes9x_libretro.$retroarch_core_extension" \
  "$retroarch_core_editor_root/original-snes-core"
cp "$retroarch_core_directory/genesis_plus_gx_libretro.$retroarch_core_extension" \
  "$retroarch_core_editor_root/original-genesis-core"

retroarch_core_editor_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$retroarch_core_editor_root" \
    --retroarch-core-editor-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$retroarch_core_editor_output" >&2
  exit 1
}
if ! rg -q \
  'RETROARCH_CORE_EDITOR_SMOKE_COMPLETE cores=2 unsafe=1 suggestion=snes9x_libretro revision=[0-9]+' \
  <<< "$retroarch_core_editor_output"; then
  printf '%s\n' "$retroarch_core_editor_output" >&2
  echo "LaunchBox did not validate its RetroArch core mapping editor." >&2
  exit 1
fi
expected_retroarch_command="<CommandLine>-L cores/snes9x_libretro.$retroarch_core_extension --platform fixture</CommandLine>"
if ! rg -q -F "$expected_retroarch_command" \
  "$retroarch_core_editor_document"; then
  echo "RetroArch core editor did not persist the selected native core while retaining unrelated arguments." >&2
  exit 1
fi
if ! cmp -s "$retroarch_core_editor_root/original-retroarch" \
    "$retroarch_core_editor_root/Emulators/RetroArch/retroarch" \
  || ! cmp -s "$retroarch_core_editor_root/original-retroarch.cfg" \
    "$retroarch_core_editor_root/Emulators/RetroArch/retroarch.cfg" \
  || ! cmp -s "$retroarch_core_editor_root/original-snes-core" \
    "$retroarch_core_directory/snes9x_libretro.$retroarch_core_extension" \
  || ! cmp -s "$retroarch_core_editor_root/original-genesis-core" \
    "$retroarch_core_directory/genesis_plus_gx_libretro.$retroarch_core_extension"; then
  echo "Read-only RetroArch core inventory changed an executable, configuration, or core file." >&2
  exit 1
fi
if [[ ! -L "$retroarch_core_directory/unsafe_libretro.$retroarch_core_extension" ]]; then
  echo "RetroArch core inventory changed the refused symbolic-link entry." >&2
  exit 1
fi
mapfile -t retroarch_core_editor_backups < <(
  find "$retroarch_core_editor_root/Data" -maxdepth 1 -type f \
    -name 'Emulators.xml.lbport-transaction-backup-*' -print
)
if [[ ${#retroarch_core_editor_backups[@]} -ne 1 ]]; then
  echo "RetroArch core mapping edit did not retain exactly one XML backup." >&2
  exit 1
fi
if find "$retroarch_core_editor_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful RetroArch core mapping edit left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox RetroArch native core discovery, 13.27 recommendation selection, semantic command-line editing, unsafe-entry refusal, transactional persistence, and read-only core/config behavior validated."

cp -R fixtures/launchbox/Data "$emulator_discovery_root/Data"
find "$emulator_discovery_root/Data" -type f -name '*.xml' \
  ! -name 'Emulators.xml' -exec \
  sed -i 's/Fixture Console/Sony PlayStation 2/g' {} +
emulator_discovery_document="$emulator_discovery_root/Data/Emulators.xml"
sed -i \
  '/<Title>Fixture Emulator<\/Title>/a\    <FutureEmulatorDiscoveryField>keep-discovery-data</FutureEmulatorDiscoveryField>' \
  "$emulator_discovery_document"
emulator_discovery_original="$emulator_discovery_root/original-emulators.xml"
cp "$emulator_discovery_document" "$emulator_discovery_original"
mkdir -p "$emulator_discovery_root/Emulators/PCSX2"
emulator_discovery_candidate="$emulator_discovery_root/Emulators/PCSX2/pcsx2-qt"
install_process_fixture "$emulator_discovery_candidate"
emulator_discovery_candidate_sha=$(
  sha256sum "$emulator_discovery_candidate" | awk '{print $1}'
)
emulator_discovery_candidate_mode=$(stat -c '%a' "$emulator_discovery_candidate")

emulator_discovery_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$emulator_discovery_root" --emulator-discovery-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$emulator_discovery_output" >&2
  exit 1
}
if ! rg -q \
  'EMULATOR_DISCOVERY_SMOKE_COMPLETE emulator=.* candidates=[0-9]+ writes=1 emulator_revision=[0-9]+ discovery_revision=[0-9]+' \
  <<< "$emulator_discovery_output"; then
  printf '%s\n' "$emulator_discovery_output" >&2
  echo "LaunchBox did not validate reviewed installed-emulator discovery and registration." >&2
  exit 1
fi
for expected in \
  '<Title>PCSX2</Title>' \
  '<ApplicationPath>Emulators\PCSX2\pcsx2-qt</ApplicationPath>' \
  '<CommandLine>-fullscreen -nogui</CommandLine>' \
  '<AutoExtract>false</AutoExtract>' \
  '<ForcefulPauseScreenActivation>true</ForcefulPauseScreenActivation>' \
  '<HideConsole>false</HideConsole>' \
  '<HideMouseCursorInGame>true</HideMouseCursorInGame>' \
  '<StartupLoadDelay>5000</StartupLoadDelay>' \
  '<SuspendProcessOnPause>true</SuspendProcessOnPause>' \
  '<UsePauseScreen>true</UsePauseScreen>' \
  '<UseStartupScreen>true</UseStartupScreen>' \
  '<Platform>Sony PlayStation 2</Platform>' \
  '<Default>true</Default>' \
  '<M3uDiscLoadEnabled>false</M3uDiscLoadEnabled>' \
  '<FutureEmulatorDiscoveryField>keep-discovery-data</FutureEmulatorDiscoveryField>'; do
  if ! rg -q -F "$expected" "$emulator_discovery_document"; then
    echo "Emulator discovery registration did not retain expected XML: $expected" >&2
    exit 1
  fi
done
if [[ $(rg -c '^  <Emulator>$' "$emulator_discovery_document") -ne 2 ]] \
  || [[ $(rg -c '^  <EmulatorPlatform>$' "$emulator_discovery_document") -ne 2 ]]; then
  echo "Emulator discovery registration did not append exactly one definition and mapping." >&2
  exit 1
fi
mapfile -t emulator_discovery_backups < <(
  find "$emulator_discovery_root/Data" -maxdepth 1 -type f \
    -name 'Emulators.xml.lbport-transaction-backup-*' -print
)
if [[ ${#emulator_discovery_backups[@]} -ne 1 ]] \
  || ! cmp -s "${emulator_discovery_backups[0]}" "$emulator_discovery_original"; then
  echo "Emulator discovery registration did not retain one exact pre-write backup." >&2
  exit 1
fi
if [[ $(sha256sum "$emulator_discovery_candidate" | awk '{print $1}') \
      != "$emulator_discovery_candidate_sha" ]] \
  || [[ $(stat -c '%a' "$emulator_discovery_candidate") \
      != "$emulator_discovery_candidate_mode" ]]; then
  echo "Emulator discovery executed or modified the candidate executable." >&2
  exit 1
fi
if [[ $(find "$emulator_discovery_root/Emulators" -mindepth 1 -maxdepth 1 \
        -type d | wc -l) -ne 1 ]] \
  || [[ $(find "$emulator_discovery_root/Emulators" -type f | wc -l) -ne 1 ]]; then
  echo "Emulator discovery created an unexpected emulator directory or binary." >&2
  exit 1
fi
if find "$emulator_discovery_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful emulator discovery registration left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox reviewed RetroArch/Dolphin/PCSX2/ScummVM executable discovery, candidate provenance, full-editor registration, portable path storage, recovered defaults, exact backup, and candidate immutability validated."

cp -R fixtures/launchbox/Data "$emulator_bios_root/Data"
find "$emulator_bios_root/Data" -type f -name '*.xml' -exec \
  sed -i 's/fixture-emulator/pcsx2-bios-fixture/g' {} +
emulator_bios_document="$emulator_bios_root/Data/Emulators.xml"
sed -i \
  -e 's#<Title>Fixture Emulator</Title>#<Title>PCSX2</Title>#' \
  -e 's#<ApplicationPath>Emulators/pcsx2-bios-fixture</ApplicationPath>#<ApplicationPath>Emulators\\PCSX2\\pcsx2-qt</ApplicationPath>#' \
  "$emulator_bios_document"
mkdir -p \
  "$emulator_bios_root/Emulators/PCSX2/inis" \
  "$emulator_bios_root/Emulators/PCSX2/custom-bios"
emulator_bios_application="$emulator_bios_root/Emulators/PCSX2/pcsx2-qt"
install_process_fixture "$emulator_bios_application"
printf '' > "$emulator_bios_root/Emulators/PCSX2/portable.ini"
printf '[Folders]\nBios = custom-bios\n' \
  > "$emulator_bios_root/Emulators/PCSX2/inis/PCSX2.ini"
printf 'deliberately not copyrighted firmware\n' \
  > "$emulator_bios_root/Emulators/PCSX2/custom-bios/ps2-0100jd-20000117.bin"
emulator_bios_symlink_target="$emulator_bios_root/Emulators/PCSX2/outside.bin"
printf 'symlink target must not be read as firmware\n' \
  > "$emulator_bios_symlink_target"
ln -s ../outside.bin \
  "$emulator_bios_root/Emulators/PCSX2/custom-bios/ps2-0100j-20000117.bin"
emulator_bios_tree_before=$(
  find "$emulator_bios_root" -mindepth 1 \
    -printf '%P\t%y\t%m\t%l\n' | sort
)
emulator_bios_hashes_before=$(
  while IFS= read -r file; do
    relative=${file#"$emulator_bios_root/"}
    printf '%s\t' "$relative"
    sha256sum "$file" | awk '{print $1}'
  done < <(find "$emulator_bios_root" -type f | sort)
)

emulator_bios_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$emulator_bios_root" --emulator-bios-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$emulator_bios_output" >&2
  exit 1
}
if ! rg -q \
  'EMULATOR_BIOS_SMOKE_COMPLETE emulator=pcsx2-bios-fixture files=73 valid=0 mismatch=1 unsafe=1 missing=71 scans=1 revision=[0-9]+' \
  <<< "$emulator_bios_output"; then
  printf '%s\n' "$emulator_bios_output" >&2
  echo "LaunchBox did not validate the read-only PCSX2 BIOS contract." >&2
  exit 1
fi
emulator_bios_tree_after=$(
  find "$emulator_bios_root" -mindepth 1 \
    -printf '%P\t%y\t%m\t%l\n' | sort
)
emulator_bios_hashes_after=$(
  while IFS= read -r file; do
    relative=${file#"$emulator_bios_root/"}
    printf '%s\t' "$relative"
    sha256sum "$file" | awk '{print $1}'
  done < <(find "$emulator_bios_root" -type f | sort)
)
if [[ "$emulator_bios_tree_after" != "$emulator_bios_tree_before" ]] \
  || [[ "$emulator_bios_hashes_after" != "$emulator_bios_hashes_before" ]]; then
  echo "PCSX2 BIOS audit changed a file, symlink, permission, or directory." >&2
  diff -u \
    <(printf '%s\n%s\n' "$emulator_bios_tree_before" "$emulator_bios_hashes_before") \
    <(printf '%s\n%s\n' "$emulator_bios_tree_after" "$emulator_bios_hashes_after") \
    || true
  exit 1
fi
if find "$emulator_bios_root" -type f \
  \( -name '*.lbport-transaction-backup-*' \
    -o -name '.lbport-transaction-*.json' \) -print -quit | rg -q .; then
  echo "Read-only PCSX2 BIOS audit created a transaction artifact." >&2
  exit 1
fi

echo "LaunchBox complete 73-alternative PCSX2 BIOS group, portable configuration resolution, streamed hash mismatch, symlink refusal, Qt status report, and whole-tree immutability validated."

cp -R fixtures/launchbox/Data "$emulator_install_root/Data"
find "$emulator_install_root/Data" -type f -name '*.xml' -exec \
  sed -i 's/Fixture Console/Sony PlayStation 2/g' {} +
emulator_install_document="$emulator_install_root/Data/Emulators.xml"
sed -i 's#<Default>true</Default>#<Default>false</Default>#' \
  "$emulator_install_document"
emulator_install_original_document="$emulator_install_root/original-emulators.xml"
cp "$emulator_install_document" "$emulator_install_original_document"
emulator_install_asset_name=pcsx2-v2.7.492-linux-appimage-x64-Qt.AppImage
emulator_install_asset="$emulator_release_fixture_root/$emulator_install_asset_name"
emulator_install_execution_marker="$emulator_install_root/installer-executed-artifact"
install_process_fixture "$emulator_install_asset"
emulator_install_asset_size=$(stat -c '%s' "$emulator_install_asset")
emulator_install_asset_sha256=$(sha256sum "$emulator_install_asset" | awk '{print $1}')
printf '%s\n' \
  '[' \
  '  {' \
  '    "tag_name": "v2.7.492",' \
  '    "name": "PCSX2 v2.7.492",' \
  '    "html_url": "https://github.com/PCSX2/pcsx2/releases/tag/v2.7.492",' \
  '    "draft": false,' \
  '    "prerelease": true,' \
  '    "assets": [' \
  '      {' \
  "        \"name\": \"$emulator_install_asset_name\"," \
  "        \"browser_download_url\": \"https://github.com/PCSX2/pcsx2/releases/download/v2.7.492/$emulator_install_asset_name\"," \
  "        \"size\": $emulator_install_asset_size," \
  "        \"digest\": \"sha256:$emulator_install_asset_sha256\"" \
  '      }' \
  '    ]' \
  '  }' \
  ']' > "$emulator_release_fixture_root/releases.json"

emulator_install_output=$(
  LBPORT_UNEXPECTED_EXECUTION_MARKER="$emulator_install_execution_marker" \
    QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$emulator_install_root" --emulator-install-smoke-test \
    --emulator-release-fixture "$emulator_release_fixture_root" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$emulator_install_output" >&2
  exit 1
}
if ! rg -q \
  'EMULATOR_INSTALL_SMOKE_COMPLETE emulator=PCSX2 version=2.7.492 files=3 installs=1 revision=[0-9]+' \
  <<< "$emulator_install_output"; then
  printf '%s\n' "$emulator_install_output" >&2
  echo "LaunchBox did not validate the managed PCSX2 install contract." >&2
  exit 1
fi
emulator_install_directory="$emulator_install_root/Emulators/PCSX2"
emulator_install_executable="$emulator_install_directory/pcsx2-qt.AppImage"
emulator_install_manifest="$emulator_install_directory/.launchbox-port-install.json"
if ! cmp -s "$emulator_install_asset" "$emulator_install_executable" \
  || [[ ! -x "$emulator_install_executable" ]] \
  || [[ ! -f "$emulator_install_directory/portable.ini" ]] \
  || [[ -s "$emulator_install_directory/portable.ini" ]]; then
  echo "Managed PCSX2 install did not retain exact executable bytes, execute permission, and empty portable marker." >&2
  exit 1
fi
for expected in \
  '"schema_version": 3' \
  '"profile_id": "pcsx2"' \
  '"provider": "github:PCSX2/pcsx2"' \
  '"emulator_id": "' \
  '"version": "2.7.492"' \
  "\"asset_byte_len\": $emulator_install_asset_size" \
  "\"asset_sha256\": \"$emulator_install_asset_sha256\"" \
  '"executable_name": "pcsx2-qt.AppImage"' \
  '"installed_files": [' \
  '"relative_path": "pcsx2-qt.AppImage"' \
  '"relative_path": "portable.ini"'; do
  if ! rg -q -F "$expected" "$emulator_install_manifest"; then
    echo "Managed PCSX2 ownership manifest is missing: $expected" >&2
    exit 1
  fi
done
for expected in \
  '<Title>PCSX2</Title>' \
  '<ApplicationPath>Emulators\PCSX2\pcsx2-qt.AppImage</ApplicationPath>' \
  '<Platform>Sony PlayStation 2</Platform>' \
  '<Default>true</Default>'; do
  if ! rg -q -F "$expected" "$emulator_install_document"; then
    echo "Managed PCSX2 install did not persist expected emulator XML: $expected" >&2
    exit 1
  fi
done
if [[ $(find "$emulator_install_directory" -type f | wc -l) -ne 3 ]] \
  || [[ -e "$emulator_install_execution_marker" ]]; then
  echo "Managed PCSX2 install created unexpected files or executed the downloaded artifact." >&2
  exit 1
fi
emulator_install_backup_count=0
while IFS= read -r backup; do
  emulator_install_backup_count=$((emulator_install_backup_count + 1))
  if ! cmp -s "$emulator_install_original_document" "$backup"; then
    echo "Managed PCSX2 install retained an inexact Emulators.xml backup." >&2
    exit 1
  fi
done < <(
  find "$emulator_install_root/Data" -maxdepth 1 -type f \
    -name 'Emulators.xml.lbport-transaction-backup-*' | sort
)
if [[ "$emulator_install_backup_count" -ne 1 ]] \
  || find "$emulator_install_root" -type f \
    -name '.lbport-transaction-*.json' -print -quit | rg -q . \
  || find "$emulator_install_root" -type d \
    -name '.lbport-pcsx2-download-*' -print -quit | rg -q .; then
  echo "Managed PCSX2 install did not retain one exact XML backup or left staging/recovery state behind." >&2
  exit 1
fi

mkdir -p "$emulator_install_directory/inis" \
  "$emulator_install_directory/cheats"
printf '%s\n' '[UI]' 'Theme=SmokeUser' \
  > "$emulator_install_directory/inis/PCSX2.ini"
printf '%s\n' 'user cheat data' \
  > "$emulator_install_directory/cheats/user.pnach"
emulator_installed_document="$emulator_install_root/installed-emulators.xml"
cp "$emulator_install_document" "$emulator_installed_document"

emulator_remove_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$emulator_install_root" --emulator-remove-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$emulator_remove_output" >&2
  exit 1
}
if ! rg -q \
  'EMULATOR_REMOVE_SMOKE_COMPLETE emulator=PCSX2 files=3 settings=1 removals=1 revision=[0-9]+' \
  <<< "$emulator_remove_output"; then
  printf '%s\n' "$emulator_remove_output" >&2
  echo "LaunchBox did not validate the managed PCSX2 removal contract." >&2
  exit 1
fi
if [[ -e "$emulator_install_executable" ]] \
  || [[ -e "$emulator_install_manifest" ]] \
  || [[ -e "$emulator_install_directory/portable.ini" ]] \
  || ! rg -q -F 'Theme=SmokeUser' \
    "$emulator_install_directory/inis/PCSX2.ini" \
  || ! rg -q -F 'user cheat data' \
    "$emulator_install_directory/cheats/user.pnach" \
  || rg -q -F '<ApplicationPath>Emulators\PCSX2\pcsx2-qt.AppImage</ApplicationPath>' \
    "$emulator_install_document"; then
  echo "Managed PCSX2 removal deleted user data, retained an owned path, or retained its emulator definition." >&2
  exit 1
fi
if ! find "$emulator_install_directory" -maxdepth 1 -type f \
  -name 'pcsx2-qt.AppImage.lbport-transaction-backup-*' \
  -exec cmp -s "$emulator_install_asset" {} \; -print -quit | rg -q . \
  || ! find "$emulator_install_directory" -maxdepth 1 -type f \
    -name 'portable.ini.lbport-transaction-backup-*' \
    -size 0 -print -quit | rg -q . \
  || ! find "$emulator_install_directory" -maxdepth 1 -type f \
    -name '.launchbox-port-install.json.lbport-transaction-backup-*' \
    -exec rg -q -F '"schema_version": 3' {} \; -print -quit | rg -q . \
  || ! find "$emulator_install_root/Data" -maxdepth 1 -type f \
    -name 'Emulators.xml.lbport-transaction-backup-*' \
    -exec cmp -s "$emulator_installed_document" {} \; -print -quit | rg -q . \
  || find "$emulator_install_root" -type f \
    -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Managed PCSX2 removal did not retain exact file/XML recovery copies or left pending recovery state." >&2
  exit 1
fi

echo "LaunchBox official-shaped PCSX2 release review, streamed size/SHA-256 verification, non-executing AppImage install, portable path storage, complete ownership manifest, offline removal review, reference/digest safety gates, user-file preservation, exact recovery copies, and transactional cleanup validated."

cp -R fixtures/launchbox/Data "$category_crud_root/Data"
category_crud_catalog="$category_crud_root/Data/Platforms.xml"
category_crud_parents="$category_crud_root/Data/Parents.xml"
sed -i \
  's#<ParentPlatformCategoryName>Fixture Category</ParentPlatformCategoryName>#<ParentPlatformCategoryName>Portable Collections</ParentPlatformCategoryName>#' \
  "$category_crud_parents"
sed -i \
  '/<PlatformName>Fixture Console<\/PlatformName>/a\    <FutureParentElement>keep-parent-data</FutureParentElement>' \
  "$category_crud_parents"
sed -i \
  '/<IsAutogenerated>false<\/IsAutogenerated>/a\    <FutureCategoryElement>keep-category-data</FutureCategoryElement>' \
  "$category_crud_catalog"
category_crud_original_catalog="$category_crud_root/original-platforms.xml"
category_crud_original_parents="$category_crud_root/original-parents.xml"
cp "$category_crud_catalog" "$category_crud_original_catalog"
cp "$category_crud_parents" "$category_crud_original_parents"

category_crud_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$category_crud_root" --category-crud-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$category_crud_output" >&2
  exit 1
}
if ! rg -q 'CATEGORY_CRUD_SMOKE_COMPLETE category="Portable Collections" writes=3 detached=1 navigation_entries=3' \
  <<< "$category_crud_output"; then
  printf '%s\n' "$category_crud_output" >&2
  echo "LaunchBox did not validate dialog-driven nested category creation, editing, and deletion." >&2
  exit 1
fi
if rg -q -F '<Name>Portable Collections</Name>' "$category_crud_catalog"; then
  echo "Category CRUD smoke retained the deleted category record." >&2
  exit 1
fi
for expected in \
  '<Name>Fixture Category</Name>' \
  '<FutureCategoryElement>keep-category-data</FutureCategoryElement>'; do
  if ! rg -q -F "$expected" "$category_crud_catalog"; then
    echo "Category CRUD smoke lost catalog data: $expected" >&2
    exit 1
  fi
done
for expected in \
  '<PlatformName>Fixture Console</PlatformName>' \
  '<FutureParentElement>keep-parent-data</FutureParentElement>' \
  '<PlaylistId>fixture-playlist</PlaylistId>'; do
  if ! rg -q -F "$expected" "$category_crud_parents"; then
    echo "Category CRUD smoke lost hierarchy data: $expected" >&2
    exit 1
  fi
done
if rg -q -F '<ParentPlatformCategoryName>Portable Collections</ParentPlatformCategoryName>' \
  "$category_crud_parents"; then
  echo "Deleted category still owns or parents a hierarchy row." >&2
  exit 1
fi
for media_directory in Images Videos Manuals Music; do
  if [[ -e "$category_crud_root/$media_directory" ]]; then
    echo "Category CRUD unexpectedly created the $media_directory media directory." >&2
    exit 1
  fi
done

mapfile -t category_catalog_backups < <(
  find "$category_crud_root/Data" -maxdepth 1 -type f \
    -name 'Platforms.xml.lbport-transaction-backup-*' -print
)
mapfile -t category_parent_backups < <(
  find "$category_crud_root/Data" -maxdepth 1 -type f \
    -name 'Parents.xml.lbport-transaction-backup-*' -print
)
if [[ ${#category_catalog_backups[@]} -ne 3 \
  || ${#category_parent_backups[@]} -ne 3 ]]; then
  echo "Category create/edit/delete did not retain three paired document backups." >&2
  exit 1
fi
category_original_catalog_backups=0
category_created_catalog_backups=0
category_edited_catalog_backups=0
for backup in "${category_catalog_backups[@]}"; do
  if cmp -s "$backup" "$category_crud_original_catalog"; then
    ((category_original_catalog_backups += 1))
  elif rg -q -F '<Name>Portable Collections</Name>' "$backup" \
    && rg -q -F '<NestedName>Portable</NestedName>' "$backup" \
    && rg -q -F '<SortTitle>Collections, Portable</SortTitle>' "$backup" \
    && rg -q -F '<ImageType>Clear Logo</ImageType>' "$backup" \
    && rg -q -F '<VideoPath>Videos\Portable Collections\theme.mp4</VideoPath>' "$backup" \
    && rg -q -F '<Notes>Edited through the real category dialog.</Notes>' "$backup" \
    && rg -q -F '<HideInBigBox>true</HideInBigBox>' "$backup"; then
    ((category_edited_catalog_backups += 1))
  elif rg -q -F '<Name>Portable Collections</Name>' "$backup"; then
    ((category_created_catalog_backups += 1))
  fi
done
if [[ $category_original_catalog_backups -ne 1 \
  || $category_created_catalog_backups -ne 1 \
  || $category_edited_catalog_backups -ne 1 ]]; then
  echo "Category catalog backups do not prove the expected create/edit/delete chain." >&2
  exit 1
fi
category_original_parent_backups=0
category_created_parent_backups=0
category_edited_parent_backups=0
for backup in "${category_parent_backups[@]}"; do
  portable_children=$(rg -c -F '<PlatformCategoryName>Portable Collections</PlatformCategoryName>' \
    "$backup" || true)
  if cmp -s "$backup" "$category_crud_original_parents"; then
    ((category_original_parent_backups += 1))
  elif [[ $portable_children -eq 2 ]]; then
    ((category_edited_parent_backups += 1))
  elif [[ $portable_children -eq 1 ]]; then
    ((category_created_parent_backups += 1))
  fi
done
if [[ $category_original_parent_backups -ne 1 \
  || $category_created_parent_backups -ne 1 \
  || $category_edited_parent_backups -ne 1 ]]; then
  echo "Category hierarchy backups do not prove retained and added placement ordering." >&2
  exit 1
fi
if find "$category_crud_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful category CRUD smoke left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox dialog-driven nested category lifecycle, descendant counts, paired transactions, child detachment, lexical paths, unknown XML, and media isolation validated."

cp -R fixtures/launchbox/Data "$playlist_crud_root/Data"
playlist_crud_platform="$playlist_crud_root/Data/Platforms/Fixture Console.xml"
playlist_crud_parents="$playlist_crud_root/Data/Parents.xml"
playlist_crud_cache="$playlist_crud_root/Data/ListCache.xml"
playlist_crud_original_platform="$playlist_crud_root/original-platform.xml"
playlist_crud_original_parents="$playlist_crud_root/original-parents.xml"
playlist_crud_original_cache="$playlist_crud_root/original-cache.xml"
cp "$playlist_crud_platform" "$playlist_crud_original_platform"
cp "$playlist_crud_parents" "$playlist_crud_original_parents"
cp "$playlist_crud_cache" "$playlist_crud_original_cache"

playlist_crud_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$playlist_crud_root" --playlist-crud-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$playlist_crud_output" >&2
  exit 1
}
if ! rg -q 'PLAYLIST_CRUD_SMOKE_COMPLETE .* writes=5 detached=1 cache_rows=0 navigation_entries=3' \
  <<< "$playlist_crud_output"; then
  printf '%s\n' "$playlist_crud_output" >&2
  echo "LaunchBox did not validate dialog-driven playlist creation, editing, nesting, filtering, and deletion." >&2
  exit 1
fi
if find "$playlist_crud_root/Data/Playlists" -maxdepth 1 -type f \
  \( -name 'Portable_Queue.xml' -o -name 'Portable Child.xml' \) -print -quit \
  | rg -q .; then
  echo "Playlist CRUD smoke retained a deleted playlist document." >&2
  exit 1
fi
if ! rg -q -F '<PlaylistId>fixture-playlist</PlaylistId>' "$playlist_crud_parents"; then
  echo "Playlist CRUD smoke lost the pre-existing playlist hierarchy row." >&2
  exit 1
fi
if rg -q -F 'Portable/Queue' "$playlist_crud_parents" \
  || rg -q -F 'Portable Child' "$playlist_crud_parents"; then
  echo "Deleted playlist still owns or parents a hierarchy row." >&2
  exit 1
fi
if ! cmp -s "$playlist_crud_platform" "$playlist_crud_original_platform"; then
  echo "Playlist membership editing changed a game platform document." >&2
  exit 1
fi
if ! cmp -s "$playlist_crud_cache" "$playlist_crud_original_cache"; then
  echo "Playlist deletion changed ListCache.xml without a matching cache row." >&2
  exit 1
fi
for media_directory in Images Videos Manuals Music; do
  if [[ -e "$playlist_crud_root/$media_directory" ]]; then
    echo "Playlist CRUD unexpectedly created the $media_directory media directory." >&2
    exit 1
  fi
done

mapfile -t playlist_parent_backups < <(
  find "$playlist_crud_root/Data" -maxdepth 1 -type f \
    -name 'Parents.xml.lbport-transaction-backup-*' -print
)
mapfile -t playlist_document_backups < <(
  find "$playlist_crud_root/Data/Playlists" -maxdepth 1 -type f \
    \( -name 'Portable_Queue.xml.lbport-transaction-backup-*' \
       -o -name 'Portable Child.xml.lbport-transaction-backup-*' \) -print
)
if [[ ${#playlist_parent_backups[@]} -ne 5 \
  || ${#playlist_document_backups[@]} -ne 3 ]]; then
  echo "Playlist lifecycle did not retain five hierarchy and three playlist-document backups." >&2
  exit 1
fi
playlist_manual_backups=0
playlist_auto_backups=0
playlist_child_backups=0
for backup in "${playlist_document_backups[@]}"; do
  if rg -q -F '<Name>Portable/Queue</Name>' "$backup" \
    && rg -q -F '<GameId>fixture-racer</GameId>' "$backup" \
    && rg -q -F '<AutoPopulate>false</AutoPopulate>' "$backup"; then
    ((playlist_manual_backups += 1))
  elif rg -q -F '<Name>Portable/Queue</Name>' "$backup" \
    && rg -q -F '<NestedName>Portable Favorites</NestedName>' "$backup" \
    && rg -q -F '<VideoPath>Videos\Portable Favorites\theme.mp4</VideoPath>' "$backup" \
    && rg -q -F '<FieldKey>Favorite</FieldKey>' "$backup" \
    && rg -q -F '<ComparisonTypeKey>IsTrue</ComparisonTypeKey>' "$backup" \
    && rg -q -F '<AutoPopulate>true</AutoPopulate>' "$backup"; then
    ((playlist_auto_backups += 1))
  elif rg -q -F '<Name>Portable Child</Name>' "$backup"; then
    ((playlist_child_backups += 1))
  fi
done
if [[ $playlist_manual_backups -ne 1 || $playlist_auto_backups -ne 1 \
  || $playlist_child_backups -ne 1 ]]; then
  echo "Playlist backups do not prove the expected manual, auto-filtered, nested, and deleted states." >&2
  exit 1
fi
if find "$playlist_crud_root" -maxdepth 1 -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Successful playlist CRUD smoke left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox dialog-driven playlist lifecycle, stable identity, auto/manual membership, filtering, nesting, child detachment, portable filenames, lexical paths, exact backups, and media isolation validated."

cp -R fixtures/launchbox/Data "$emulator_launch_root/Data"
mkdir -p "$emulator_launch_root/Emulators"
install_process_fixture "$emulator_launch_root/Emulators/fixture-emulator"

cp -R fixtures/launchbox/Data "$disabled_lifecycle_root/Data"
mkdir -p "$disabled_lifecycle_root/Emulators"
install_process_fixture "$disabled_lifecycle_root/Emulators/fixture-emulator"
sed -i \
  's#<UseStartupScreen>true</UseStartupScreen>#<UseStartupScreen>false</UseStartupScreen>#' \
  "$disabled_lifecycle_root/Data/Settings.xml"

cp -R fixtures/launchbox/Data "$short_lifecycle_root/Data"
mkdir -p "$short_lifecycle_root/Emulators"
install_process_fixture "$short_lifecycle_root/Emulators/fixture-emulator"

cp -R fixtures/launchbox-direct/Data "$direct_launch_root/Data"
mkdir -p "$direct_launch_root/LaunchTargets"
install_process_fixture "$direct_launch_root/LaunchTargets/argument-recorder"

cp -R fixtures/launchbox-direct/Data "$desktop_command_root/Data"
mkdir -p "$desktop_command_root/LaunchTargets"
install_process_fixture "$desktop_command_root/LaunchTargets/argument-recorder"
desktop_command_platform="$desktop_command_root/Data/Platforms/Direct Fixture.xml"
cp "$desktop_command_platform" \
  "$desktop_command_platform.before-desktop-command-smoke"

cp -R fixtures/launchbox-archive/Data "$archive_launch_root/Data"
mkdir -p \
  "$archive_launch_root/Emulators" \
  "$archive_launch_root/Games/Archive Fixture" \
  "$archive_launch_root/Runtime" \
  "$archive_launch_root/archive-source"
install_process_fixture "$archive_launch_root/Emulators/archive-recorder"
install_process_fixture "$archive_launch_root/Runtime/archive-after"
printf 'portable archive ROM fixture\n' \
  > "$archive_launch_root/archive-source/Archive Racer.rom"
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
install_process_fixture "$dosbox_launch_root/Runtime/dosbox-recorder"
touch \
  "$dosbox_launch_root/Config/dosbox.conf" \
  "$dosbox_launch_root/Games/DOS Fixture/BIN/PLAY.BAT" \
  "$dosbox_launch_root/Media/Disk One.img" \
  "$dosbox_launch_root/Media/Game.iso"

cp -R fixtures/launchbox-scummvm/Data "$scummvm_launch_root/Data"
mkdir -p \
  "$scummvm_launch_root/Runtime" \
  "$scummvm_launch_root/Games/Monkey Island 2"
install_process_fixture "$scummvm_launch_root/Runtime/scummvm"

cp -R fixtures/launchbox-m3u/Data "$m3u_launch_root/Data"
mkdir -p \
  "$m3u_launch_root/Emulators" \
  "$m3u_launch_root/Games/M3U Fixture" \
  "$m3u_launch_root/Runtime" \
  "$m3u_launch_root/m3u-source"
install_process_fixture "$m3u_launch_root/Emulators/m3u-recorder"
install_process_fixture "$m3u_launch_root/Runtime/m3u-after"
printf 'portable disc one fixture\n' \
  > "$m3u_launch_root/Games/M3U Fixture/Multi Disc Racer (Disc 1).chd"
printf 'portable disc three fixture\n' \
  > "$m3u_launch_root/Games/M3U Fixture/Multi Disc Racer (Disc 3).chd"
printf 'portable disc two fixture\n' \
  > "$m3u_launch_root/m3u-source/Multi Disc Racer (Disc 2).chd"
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

desktop_command_log="$desktop_command_root/desktop-command-arguments.txt"
desktop_command_screenshot="$desktop_command_root/desktop-commands.png"
desktop_command_output=$(
  LBPORT_LAUNCH_SMOKE_LOG="$desktop_command_log" \
    QT_QPA_PLATFORM=offscreen \
    "$binary_dir/launchbox" \
    --library "$desktop_command_root" \
    --desktop-command-smoke-test \
    --desktop-command-screenshot "$desktop_command_screenshot" \
    --map-windows-drive "Z=$desktop_command_root" \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$desktop_command_output" >&2
  exit 1
}
if ! rg -q -F \
  'DESKTOP_COMMAND_SMOKE_COMPLETE focus=Ctrl+F select=Ctrl+Alt+Q id=fixture-direct launches=1 stats_writes=2' \
  <<< "$desktop_command_output"; then
  printf '%s\n' "$desktop_command_output" >&2
  echo "LaunchBox did not validate its focus, random-selection, and random-play desktop commands." >&2
  exit 1
fi
for ((attempt = 0; attempt < 200; ++attempt)); do
  if [[ -f "$desktop_command_log" ]]; then
    break
  fi
  sleep 0.01
done
if ! cmp -s "$desktop_command_log" \
  <(printf '%s\n' --direct 'two words'); then
  printf 'Desktop-command launch arguments were:\n' >&2
  sed 's/^/  /' "$desktop_command_log" >&2 || true
  exit 1
fi
if [[ ! -s "$desktop_command_screenshot" ]] \
  || [[ $(wc -c < "$desktop_command_screenshot") -lt 1024 ]] \
  || [[ $(od -An -tx1 -N8 "$desktop_command_screenshot" | tr -d ' \n') \
    != 89504e470d0a1a0a ]]; then
  printf '%s\n' "$desktop_command_output" >&2
  echo "LaunchBox did not render a valid desktop-command PNG." >&2
  exit 1
fi
mapfile -t desktop_command_backups < <(
  find "$desktop_command_root/Data/Platforms" -maxdepth 1 -type f \
    -name 'Direct Fixture.xml.lbport-transaction-backup-*' -print
)
desktop_command_original_backups=0
desktop_command_session_backups=0
for backup in "${desktop_command_backups[@]}"; do
  if cmp -s "$backup" \
    "$desktop_command_platform.before-desktop-command-smoke"; then
    ((desktop_command_original_backups += 1))
  elif rg -q -F '<PlayCount>1</PlayCount>' "$backup" \
    && rg -q -F '<PlayTime>0</PlayTime>' "$backup" \
    && rg -q -F '<LastPlayedDate>' "$backup"; then
    ((desktop_command_session_backups += 1))
  fi
done
if [[ ${#desktop_command_backups[@]} -ne 2 \
  || $desktop_command_original_backups -ne 1 \
  || $desktop_command_session_backups -ne 1 ]]; then
  echo "Desktop random play did not retain the exact start/end statistics backup chain." >&2
  exit 1
fi
if find "$desktop_command_root" -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Desktop random play left a recovery manifest behind." >&2
  exit 1
fi

echo "LaunchBox standard Find focus, Ctrl+Alt+Q random selection, separate random play, rendered controls, portable mapped launch, exact argv, statistics, and backup validated."

emulator_log="$emulator_launch_root/emulator-arguments.txt"
run_launch_smoke launchbox "$emulator_launch_root" fixture-racer "$emulator_log" \
  --platform fixture \
  "$emulator_launch_root/Games/Fixture Racer/racer.rom"
run_launch_smoke bigbox "$emulator_launch_root" fixture-racer "$emulator_log" \
  --platform fixture \
  "$emulator_launch_root/Games/Fixture Racer/racer.rom"

run_launch_lifecycle_smoke() {
  local shell_name=$1
  local lifecycle_root=$2
  local screens_enabled=$3
  local short_process=${4:-false}
  local lifecycle_log="$lifecycle_root/$shell_name-lifecycle-arguments.txt"
  local lifecycle_screenshot="$lifecycle_root/$shell_name-startup-overlay.png"
  local shutdown_screenshot="$lifecycle_root/$shell_name-shutdown-overlay.png"
  local startup_minimum=600
  local shutdown_minimum=350
  local theme="Fixture Desktop Startup"
  if [[ "$shell_name" == bigbox ]]; then
    startup_minimum=700
    shutdown_minimum=450
    theme="Fixture BigBox Startup"
  fi
  local -a arguments=(
    --library "$lifecycle_root"
    --launch-lifecycle-smoke-test
    --path-mappings-file "$empty_path_mappings"
  )
  if [[ "$screens_enabled" == true ]]; then
    arguments+=(
      --launch-lifecycle-screenshot "$lifecycle_screenshot"
      --launch-lifecycle-shutdown-screenshot "$shutdown_screenshot"
    )
  fi
  local child_sleep=1.05
  if [[ "$short_process" == true ]]; then
    arguments+=(--launch-lifecycle-short-process)
    child_sleep=0.05
  fi
  if [[ "$shell_name" == bigbox ]]; then
    arguments+=(--windowed)
  fi
  rm -f "$lifecycle_log" "$lifecycle_screenshot" "$shutdown_screenshot"
  local output
  output=$(
    LBPORT_LAUNCH_SMOKE_LOG="$lifecycle_log" \
      LBPORT_LAUNCH_SMOKE_SLEEP="$child_sleep" \
      QT_QPA_PLATFORM=offscreen \
      "$binary_dir/$shell_name" "${arguments[@]}" 2>&1
  ) || {
    printf '%s\n' "$output" >&2
    exit 1
  }
  local presentation_count=0
  if [[ "$screens_enabled" == true ]]; then
    presentation_count=1
  fi
  if ! rg -q -F \
    "LAUNCH_LIFECYCLE_SMOKE_COMPLETE id=fixture-racer enabled=$screens_enabled short=$short_process startup_presentations=$presentation_count shutdown_presentations=$presentation_count load_delay_ms=250 startup_minimum_ms=$startup_minimum shutdown_minimum_ms=$shutdown_minimum theme=\"$theme\" source=\"emulator default\"" \
    <<< "$output"; then
    printf '%s\n' "$output" >&2
    echo "$shell_name did not validate its frontend launch-screen lifecycle." >&2
    exit 1
  fi
  if ! cmp -s "$lifecycle_log" \
    <(printf '%s\n' \
      --platform fixture \
      "$lifecycle_root/Games/Fixture Racer/racer.rom"); then
    printf 'Startup lifecycle arguments for %s were:\n' "$shell_name" >&2
    sed 's/^/  /' "$lifecycle_log" >&2 || true
    exit 1
  fi
  if [[ "$screens_enabled" == true ]]; then
    local screenshot
    for screenshot in "$lifecycle_screenshot" "$shutdown_screenshot"; do
      if [[ ! -s "$screenshot" ]] \
        || [[ $(wc -c < "$screenshot") -lt 1024 ]] \
        || [[ $(od -An -tx1 -N8 "$screenshot" | tr -d ' \n') \
          != 89504e470d0a1a0a ]]; then
        printf '%s\n' "$output" >&2
        echo "$shell_name did not render a valid lifecycle-overlay PNG." >&2
        exit 1
      fi
    done
  fi
}

run_launch_lifecycle_smoke launchbox "$emulator_launch_root" true
run_launch_lifecycle_smoke bigbox "$emulator_launch_root" true
run_launch_lifecycle_smoke launchbox "$disabled_lifecycle_root" false
run_launch_lifecycle_smoke launchbox "$short_lifecycle_root" true true
echo "LaunchBox and BigBox frontend-global startup/shutdown policy, exact pre-launch delay, minimum display timing including a short-lived primary, disabled bypass, rendered overlays, exact argv, and supervised session statistics validated."

run_launch_pause_smoke() {
  local shell_name=$1
  local pause_log="$emulator_launch_root/$shell_name-pause-arguments.txt"
  local pause_screenshot="$emulator_launch_root/$shell_name-pause-overlay.png"
  local theme="Fixture Desktop Pause"
  if [[ "$shell_name" == bigbox ]]; then
    theme="Fixture BigBox Pause"
  fi
  local -a arguments=(
    --library "$emulator_launch_root"
    --launch-pause-smoke-test
    --launch-pause-screenshot "$pause_screenshot"
    --path-mappings-file "$empty_path_mappings"
  )
  if [[ "$shell_name" == bigbox ]]; then
    arguments+=(--windowed)
  fi
  rm -f "$pause_log" "$pause_screenshot"
  local output
  output=$(
    LBPORT_LAUNCH_SMOKE_LOG="$pause_log" \
      LBPORT_LAUNCH_SMOKE_SLEEP=1.6 \
      LBPORT_LAUNCH_SMOKE_DELEGATE=1 \
      QT_QPA_PLATFORM=offscreen \
      "$binary_dir/$shell_name" "${arguments[@]}" 2>&1
  ) || {
    printf '%s\n' "$output" >&2
    exit 1
  }
  if ! rg -q -F \
    "LAUNCH_PAUSE_SMOKE_COMPLETE id=fixture-racer presentations=1 suspensions=1 resumptions=1 delegated=1 theme=\"$theme\" source=\"emulator default\"" \
    <<< "$output"; then
    printf '%s\n' "$output" >&2
    echo "$shell_name did not validate its pause/resume process lifecycle." >&2
    exit 1
  fi
  if ! cmp -s "$pause_log" \
    <(printf '%s\n' \
      --platform fixture \
      "$emulator_launch_root/Games/Fixture Racer/racer.rom"); then
    printf 'Pause lifecycle arguments for %s were:\n' "$shell_name" >&2
    sed 's/^/  /' "$pause_log" >&2 || true
    exit 1
  fi
  if [[ ! -s "$pause_screenshot" ]] \
    || [[ $(wc -c < "$pause_screenshot") -lt 1024 ]] \
    || [[ $(od -An -tx1 -N8 "$pause_screenshot" | tr -d ' \n') \
      != 89504e470d0a1a0a ]]; then
    printf '%s\n' "$output" >&2
    echo "$shell_name did not render a valid pause-overlay PNG." >&2
    exit 1
  fi
}

run_launch_pause_smoke launchbox
run_launch_pause_smoke bigbox
echo "LaunchBox and BigBox frontend-global pause policy, emulator-default inheritance, delegated process-group suspension/resumption, shared rendered overlay, exact argv, and session persistence validated."

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
install_process_fixture "$sequence_launch_root/Runtime/sequence-recorder"

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
  Game ID fixture-racer 14 14406 LastPlayedDate
assert_play_stats \
  "$disabled_lifecycle_root/Data/Platforms/Fixture Console.xml" \
  Game ID fixture-racer 9 14401 LastPlayedDate
assert_play_stats \
  "$short_lifecycle_root/Data/Platforms/Fixture Console.xml" \
  Game ID fixture-racer 9 14400 LastPlayedDate
assert_play_stats \
  "$direct_launch_root/Data/Platforms/Direct Fixture.xml" \
  Game ID fixture-direct 2 2 LastPlayedDate
assert_play_stats \
  "$desktop_command_root/Data/Platforms/Direct Fixture.xml" \
  Game ID fixture-direct 1 1 LastPlayedDate
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
  "$disabled_lifecycle_root" \
  "$short_lifecycle_root" \
  "$direct_launch_root" \
  "$desktop_command_root" \
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

cp -R fixtures/launchbox/Data "$game_grouping_root/Data"
grouping_platform="$game_grouping_root/Data/Platforms/Fixture Console.xml"
grouping_catalog="$game_grouping_root/Data/Platforms.xml"
grouping_playlist="$game_grouping_root/Data/Playlists/Fixture Playlist.xml"
grouping_blacklist="$game_grouping_root/Data/ImportBlacklist.xml"
sed -i \
  '/<DisableAutoImport>false<\/DisableAutoImport>/a\    <LastGameId>fixture-racer</LastGameId>' \
  "$grouping_catalog"
sed -i \
  '/<PlaylistId>fixture-playlist<\/PlaylistId>/a\    <LastGameId>fixture-racer</LastGameId>' \
  "$grouping_playlist"
sed -i \
  '/<\/LaunchBox>/i\  <PlaylistGame><GameId>fixture-racer</GameId><GameTitle>Fixture Racer</GameTitle><GamePlatform>Fixture Console</GamePlatform><ManualOrder>2</ManualOrder></PlaylistGame>' \
  "$grouping_playlist"
sed -i \
  '/<\/LaunchBox>/i\  <IgnoredGameId><GameId>fixture-racer</GameId></IgnoredGameId>' \
  "$grouping_blacklist"
mkdir -p "$game_grouping_root/expected"
cp "$grouping_platform" "$game_grouping_root/expected/platform.xml"
cp "$grouping_catalog" "$game_grouping_root/expected/catalog.xml"
cp "$grouping_playlist" "$game_grouping_root/expected/playlist.xml"
cp "$grouping_blacklist" "$game_grouping_root/expected/blacklist.xml"

game_grouping_output=$(
  QT_QPA_PLATFORM=offscreen "$binary_dir/launchbox" \
    --library "$game_grouping_root" --game-grouping-smoke-test \
    --path-mappings-file "$empty_path_mappings" 2>&1
) || {
  printf '%s\n' "$game_grouping_output" >&2
  exit 1
}
if ! rg -q \
  'GAME_GROUPING_SMOKE_COMPLETE revisions=2 games=3 applications=1' \
  <<< "$game_grouping_output"; then
  printf '%s\n' "$game_grouping_output" >&2
  echo "LaunchBox did not validate the real-dialog combine/expand lifecycle." >&2
  exit 1
fi
if [[ $(rg -c '<Game>' "$grouping_platform") != 3 ]] \
  || [[ $(rg -c '<AdditionalApplication>' "$grouping_platform") != 1 ]] \
  || ! rg -q -F \
    '<ApplicationPath>Games\Fixture Racer\racer.rom</ApplicationPath>' \
    "$grouping_platform" \
  || ! rg -q -F \
    '<FutureAdditionalApplicationElement>keep-additional-app-data</FutureAdditionalApplicationElement>' \
    "$grouping_platform" \
  || ! rg -q -F \
    '<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>' \
    "$grouping_platform"; then
  echo "The expanded platform XML did not retain its versions or unknown data." >&2
  exit 1
fi
if [[ $(rg -c '<LastGameId>fixture-adventure</LastGameId>' "$grouping_catalog") != 2 ]] \
  || [[ $(rg -c '<GameId>fixture-adventure</GameId>' "$grouping_playlist") != 1 ]] \
  || ! rg -q -F \
    '<LastGameId>fixture-adventure</LastGameId>' "$grouping_playlist" \
  || rg -q -F 'fixture-racer' "$grouping_blacklist"; then
  echo "The combine transaction did not migrate navigation, playlist, and blacklist references." >&2
  exit 1
fi
for expected in platform catalog playlist blacklist; do
  if ! find "$game_grouping_root/Data" -type f \
    -name '*.lbport-transaction-backup-*' \
    -exec cmp -s "$game_grouping_root/expected/$expected.xml" {} \; \
    -print -quit | rg -q .; then
    echo "No exact pre-combine backup was retained for $expected.xml." >&2
    exit 1
  fi
done
if [[ $(find "$game_grouping_root/Data" -type f \
  -name '*.lbport-transaction-backup-*' | wc -l) != 5 ]]; then
  echo "Combine/expand did not retain the expected five transaction backups." >&2
  exit 1
fi
if find "$game_grouping_root" -type f \
  -name '.lbport-transaction-*.json' -print -quit | rg -q .; then
  echo "Combine/expand left a recovery manifest behind." >&2
  exit 1
fi

echo "The portable Rust fixture validated persisted host mappings, direct/emulator/archive/M3U/DOSBox/ScummVM argv, delegated leased-resource cleanup, launch ordering, selected additional apps, transactional play statistics, and the combine/expand game-grouping lifecycle."
