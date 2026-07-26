# LaunchBox / BigBox cross-platform port research

This repository is the evidence and planning workspace for a native,
cross-platform LaunchBox-compatible front end built with Rust, Qt 6, QML, and
[CXX-Qt](https://github.com/KDAB/cxx-qt).

The local LaunchBox 13.27 Windows oracle has been installed with Wine and all
first-party managed assemblies have been structurally decompiled. The original
installation and decompiled proprietary sources are intentionally ignored;
derived inventories and specifications live in this repository. A reversible
native UI Automation replacement now lets the real LaunchBox desktop render
under Wine after a 60-second focus timeout. BigBox loads a licensed real
library but still paints black because of a separate WPF render-thread
incompatibility; the exact boundary is recorded rather than treated as native
port behavior.

## Current artifacts

- [Reverse-engineering status](docs/RE_STATUS.md)
- [LaunchBox 13.27 Wine oracle boundary](analysis/wine-oracle-13.27.md)
- [Static feature matrix](docs/FEATURE_MATRIX.md)
- [Port architecture and execution plan](docs/PORT_PLAN.md)
- [Current implementation and verification status](docs/IMPLEMENTATION_STATUS.md)
- [Machine-readable static inventory](analysis/static-inventory.json)
- [Value-free LaunchBox 13.24 data schema](analysis/real-install-schema.json)

## Develop and run

The flake pins Rust, Qt 6, CXX-Qt, and the native build tools. Both front ends
load the embedded synthetic fixture unless `--library` is supplied.

```bash
nix develop
cargo test --workspace --all-targets
./scripts/check_windows_core.sh
cargo check --target x86_64-apple-darwin -p lb-domain -p lb-query -p lb-platform
cargo check --target aarch64-apple-darwin -p lb-domain -p lb-query -p lb-platform
cargo build -p lb-shell && ./scripts/check_qml.sh
cargo run -p lb-shell --bin launchbox
cargo run -p lb-shell --bin bigbox -- --windowed
```

The Linux-hosted Darwin checks cover the dependency-free portable Rust
boundary for both Intel and Apple Silicon. They do not substitute for native
Qt application-bundle, launch, input, focus, media, and multi-display tests on
real macOS hosts; those are explicit release gates in the port plan.
The flake exposes packages and development shells for both Darwin
architectures. Because nixpkgs unstable no longer evaluates Intel macOS, that
one system uses the last supported `nixpkgs-26.05-darwin` branch; Apple Silicon
and both Linux architectures continue to follow unstable.

Read an existing installation without modifying it:

```bash
cargo run -p lb-shell --bin launchbox -- --library "$LAUNCHBOX_PATH"
cargo run -p lb-storage --example inspect_library -- "$LAUNCHBOX_PATH"
cargo run -p lb-storage --example audit_data_compatibility -- "$LAUNCHBOX_PATH"
cargo run -p lb-storage --example audit_launch_plans -- "$LAUNCHBOX_PATH"
QT_QPA_PLATFORM=offscreen cargo run -p lb-shell --bin launchbox -- \
  --library "$LAUNCHBOX_PATH" --load-smoke-test
```

LaunchBox-relative paths accept either slash style on every host. A Windows
drive or UNC path is native on Windows and must be explicitly mapped elsewhere;
it is never silently treated as a relative Unix filename. Mappings are
shared by both frontends. All persisted-path classification and translation,
including Windows drive/UNC syntax and the platform-native configuration
location, is owned by `lb-platform`; storage retains the original strings and
the Qt layer only supplies native host paths. LaunchBox's **Host Paths…**
dialog persists mappings
in a separate, versioned port-owned JSON file, so the source library XML remains
portable and unchanged. The default is
`$XDG_CONFIG_HOME/launchbox-port/path-mappings.json` on Linux (falling back to
`~/.config`), the user's Application Support directory on macOS, and `%APPDATA%`
on Windows. `--path-mappings-file` selects an explicit file for portable or
isolated configurations.

Repeatable command-line mappings remain available as temporary final overrides
and also apply to the launch-plan auditor:

```bash
cargo run -p lb-shell --bin launchbox -- \
  --library "$LAUNCHBOX_PATH" \
  --path-mappings-file "$PWD/local-path-mappings.json" \
  --map-windows-drive 'D=/mnt/windows-volume' \
  --map-windows-unc 'server/share=/mnt/network-share'
```

LaunchBox now includes the first native system-tray and notification-center
slice. The visible editor reads and losslessly writes the exact five 13.27
`Settings.xml` fields, including the negative reminder flag and original
notification enum integers. Qt supplies one shared tray icon/menu boundary for
Windows, Linux, and macOS with **Show LaunchBox**, **Notifications**, and
**Exit**; ordinary close/minimize is intercepted only when the selected option
is active and the host reports that a tray really exists. The bounded native
notification center supports raised timestamps, info/error state, read/unread,
dismiss, and an unread badge. Compiled coverage drives the real editor,
transaction, rendered notification dialog, exact backup, and fresh-process
reload. A headless Linux runner deliberately cannot prove a desktop panel, so
actual icon/menu and host-notification behavior remains a three-host release
gate. The recovered contract and Wine boundary are documented in
`analysis/system-tray-13.27.md`.

Library parsing runs on a Rust worker thread and returns through CXX-Qt's queued
Qt-thread bridge. QML consumes the controller as a real `QAbstractListModel`
with 51 named identity, state, descriptive-metadata, launch-configuration,
additional-application, game-save, front-image, and play-statistics roles; it
does not receive a whole-library JSON snapshot. `check_qml.sh` validates
generated type metadata, then runs both binaries offscreen and proves all 51
roles survive a model-resetting filter operation. It also edits a
temporary fixture library through the Qt shell and checks the targeted model
notification for a state-only edit, descriptive-metadata search refresh, exact
backup chain, optional-element removal, resulting XML, and unknown-field
preservation. The same real dialog source-indexes and edits repeated alternate
names and custom fields, retaining unknown XML on each kept row, and changes
the application path, command line, emulator selection, and DOSBox/ScummVM
settings in one transaction. A fresh Linux process then
reloads that edit and executes the stored Windows-separated relative path with
the exact expanded argument vector.

LaunchBox and BigBox render each game's selected front image from the library's
own `PlatformFolder` records. The same bounded read-only Rust index retains all
configured image families for both details galleries and indexes Video
Snap, Theme Video, Trailer, Recording, and Marquee media from the recovered
folder contract plus explicit game video paths. Persisted image-type, region,
and video-type priorities, filename normalization, and numeric ordering are
deterministic. Portable backslash paths and explicitly mapped Windows drive/UNC
paths cross the shared host resolver; QML receives only native `file:` URLs and
contains no host path rules. Symlinks, deep nesting, non-regular files,
oversized files, unbounded directories, and unsupported extensions fail
closed. No media or XML is changed.

BigBox also has an independent native Qt marquee window for the selected game
or highlighted platform. It uses the recovered 13.27 primary/marquee monitor,
theme-view, image-stretch, and eight-mode compatibility settings; prioritizes
silent marquee video and direct game/platform marquee art; and exposes a real
display editor backed by one lossless recovery transaction. Screen discovery
and placement use Qt's shared Windows/Linux/macOS `QScreen` and `QWindow`
interfaces. No display shell command, OS-specific monitor API, or host path
rule appears in QML. The exact protected compatibility transforms and custom
theme views remain explicit parity gates; the current clean-room geometry and
evidence boundary are documented in `analysis/marquee-13.27.md`.

BigBox now also implements the recovered PIN/security lock. A configured PIN
starts BigBox locked; the native masked keypad accepts keyboard, controller,
or pointer input; and a visible editor covers Set/Repeat/Clear PIN,
Show Lock/Unlock, and all 32 recovered permission values. Implemented
exit/window, view/image, search/filter, and navigation actions pass through one
typed Rust gate, with unavailable navigation rows skipped. The PIN never
becomes a controller property, model role, status, or diagnostic. Saving uses
one lossless `BigBoxSettings.xml` recovery transaction with an exact backup and
committed reread. The cross-platform implementation contains no runtime shell,
host path rule, credential-store dependency, or Windows-only security API.
Premium entitlement remains separate and is not bypassed. Recovered defaults,
the keypad evidence, clean-room 32-digit safety bound, and remaining parity
gates are documented in `analysis/security-13.27.md`.

BigBox now also has native favorite, half-star rating, and manual-playlist
membership actions for the current `BB-017` slice. The wheel, details, and game
menu independently honor the seven recovered visibility/order settings;
favorite-first ordering is stable; and the centered Qt star and list popups
accept keyboard, mapped-controller, and pointer controls. Favorite and both
rating fields are committed together through a lossless platform transaction.
Add/Remove Playlist targets only manual, non-generated playlists, uses typed
`PlaylistGame` rows, and commits the exact auxiliary document through the same
recoverable transaction boundary. Both paths retain exact backups, reread and
verify committed state, preserve unrelated and unknown XML, and publish live
revisions. Favorite/Rating use the recovered security permissions; playlist
membership fails closed while locked because no matching recovered permission
exists. This shared Rust/CXX-Qt/QML path has no runtime shell, Windows path
rule, or OS-specific UI branch. The evidence, explicit clean-room half-star and
append-order policies, and compiled interaction coverage are recorded in
`analysis/game-actions-13.27.md`.

BigBox now also has a native Related Games popup for the first
`BB-004`/`BB-017` slice. The settings- and security-gated menu action lazily
scores installed games plus the optional local `LaunchBox.Metadata.db` on a
Rust worker, then presents the recovered Recommended Games, Similar Games, and
Possible Ports tabs. Installed rows retain stable game IDs and return to the
real wheel selection; database-only rows are visibly dimmed and informational.
The strict typed profile reader accepts LaunchBox's serialized
`GameSuggesterSaveData`, while malformed profiles fall back as a whole.
Recommended and Similar criteria/weights come from an older complete
installation; Possible Ports and fuzzy matching are explicitly marked
clean-room reconstructions because their protected algorithms were not
recoverable. The shared Rust/CXX-Qt/QML path reads metadata SQLite in
read-only mode, accepts only native media URLs from the existing index, and
contains no runtime shell, Windows path interpretation, or OS-specific UI
branch. Evidence and exact parity boundaries are in
`analysis/related-games-13.27.md`.

BigBox now also has a native Discovery Center for the first
`LIB-013`/`BB-017` slice. The six-slot order is recovered from LaunchBox
13.27's embedded Default-theme view and corroborated by the older complete
13.24 installation: Highly Rated, Recently Played, Recently Added, Platforms,
Favorites, and MAME High Scores. Recently Added implements the exact recovered
360-day, minimum-five, maximum-25 contract. The remaining local rankings are
stable, typed clean-room policies, and the unavailable MAME slot is retained
and source-marked instead of faked. The recovered 13.27 Discovery-list provider
now fetches its exact HTTPS endpoint on a generation-checked Rust worker,
validates a bounded case-insensitive schema, caches the first result, then
appends priority-ranked lists before shuffled non-priority lists. Manual rows
resolve Games Database IDs with title/platform fallback; automatic rows use
the shared LaunchBox OR-within-field/AND-across-fields criterion engine and
reject unsupported semantics rather than misclassifying games. A strict
versioned controller payload drives a full-screen
keyboard/controller/pointer Qt page, remains usable with local rows while the
provider loads or is offline, and returns through stable IDs and typed
filters. The historical endpoint did not resolve during the current live
check, so no current service response is claimed. The product path contains
no runtime shell, OS-specific path parsing, or platform-specific QML. MAME
scores, protected local ranking details, custom themes, live provider-service
validation, and native Windows/macOS interaction remain explicit gates in
`analysis/discovery-center-13.27.md`.

LaunchBox also has a resizable selected-game details pane backed directly by
that role model. Stable game IDs preserve selection through filtering, sorting,
editing, insertion, and removal; when a game is filtered out the first visible
row is selected deterministically. The pane renders a selectable image/video
thumbnail row and large preview, honors `ShowDetailsVideo`,
`AutoPlayDetailsVideo`, and `VideoTypePriorities`, and provides real Qt
Multimedia play/pause and mute controls. It also presents descriptive metadata,
notes, installed/favorite/completed state, play statistics, local/community
ratings, and the existing Play, Edit, Launch With, Apps, Saves, and Wikipedia
actions. **Details** hides or shows it, and **Pop Out/Dock** moves the same live
view into a native Qt window. Dock width, visibility, popup state, normal
geometry, and maximized state are atomically persisted outside the portable
library. Defaults are
`$XDG_CONFIG_HOME/launchbox-port/ui-state.json` on Linux (falling back to
`~/.config`), `~/Library/Application Support/LaunchBox Port/ui-state.json` on
macOS, and `%APPDATA%\LaunchBox Port\ui-state.json` on Windows;
`--ui-state-file` selects an explicit test or portable location. One offscreen
interaction exercises retained selection, filtered-out fallback, restoration,
native artwork decoding, and a real rendered PNG. A separate compiled scenario
uses an authored H.264 test pattern to prove native video URLs, decode,
autoplay, real thumbnail selection, play/pause, and a rendered multi-image
gallery without changing the fixture. A second two-process scenario
exercises hide, dock, pop out, native popup rendering, exact state-file bytes,
shutdown preservation, and fresh-process restoration without writing library
XML or settings.

LaunchBox also exposes the image-only viewer from an image in Details or the
global `I` action. It filters video out, preserves the selected image when
entered from Details, wraps through image families, and shares the explicit
100–400% zoom, fit, bounded pan, wheel, drag, keyboard, and visible-control
contract described below. The viewer is owned by the main window when Details
is docked and by the native details window when popped out; closing it restores
focus to the originating media row. Its compiled interaction drives the real
thumbnail and View Image actions, zooms and pans at 150%, switches to the
gameplay screenshot, resets to fit, closes through Back, renders a visually
inspected PNG, validates native regular files in Rust, and hashes the complete
media tree without writes.

BigBox exposes the same stable-ID media collection through a dedicated
full-screen game-details layer, matching the stock theme's separate image,
image/video, and details presenters. Select **Details** or use mapped Select,
then use mapped left/right to wrap through media, Select to play or pause,
Play Game to launch, and Back to return to the game wheel. The screen includes
selectable thumbnails,
one active Qt Multimedia player, previous/next and mute controls, local and
community ratings, play statistics, metadata, and notes. Playback stops when
the layer closes, and recycled game delegates never own a media player. Its
compiled offscreen scenario opens the real Details button, decodes and
autoplays the authored H.264 fixture, pauses, uses Previous, clicks actual image
and video thumbnails, renders the 1280x720 details screen, resumes playback,
closes through Back, proves the player stops, and verifies every media and
library byte remains unchanged.

BigBox now also exposes the recovered image-only path independently of video.
Select **Images**, use mapped Show Images (whose recovered default is `I`), or
choose **View Image** from an image in Details.
Previous/Next, Page Up/Down, and Enter switch among the indexed image families;
`+`, `-`, and `0` zoom or return to fit, while the mouse wheel, drag, arrows,
and visible controls zoom and pan. Because the protected 13.27 implementation
does not reveal its constants, the port explicitly uses a bounded 100–400%
range in 25% steps and resets zoom/pan when the image changes. Back returns to
Details when opened from there and otherwise returns to the game wheel. The
compiled 1280x720 interaction enters through the real Details thumbnail and
View Image controls, zooms and pans through real buttons, switches from box art
to a gameplay screenshot, resets to fit, checks native regular-file URLs in
Rust, renders a visually inspected PNG, and hashes all media and XML before and
after. The QML and Rust path are platform-neutral; native Windows and
Intel/Apple Silicon macOS Qt interaction remains a real-host release gate.

LaunchBox and BigBox can now flip the selected box between its front and back
without mutating the library. The shared media index selects back art using
LaunchBox's persisted `BackImageTypePriorities`, resolves it through the same
portable/native/mapped-Windows path boundary, and gives Qt only a native local
URL. LaunchBox exposes a per-tile **Flip**/**Front** control; BigBox exposes
**Flip Box**/**Show Front**, honors `ShowGameMenuFlipBox`, and uses the
recovered `F` default. BigBox resets to the front when selection changes,
while LaunchBox keeps the
per-tile state until its Front control is used. Both share an explicit 220 ms
Qt Y-axis transition because the protected original duration is not
recoverable. The offscreen gates drive the actual
controls in both directions, require distinct contained regular front/back
files, render the back state, and hash the full media and XML trees before and
after.

Both frontends now share one typed Qt Quick 3D model preview.
LaunchBox opens it from the settings-gated **3D Model** Details action or `M`;
BigBox uses its settings-gated game-menu action or mapped Show Model. Distinct
safely indexed
front, back, spine, and `Box - Full` files feed six real faces. When the
resolved setting enables full scans, the viewer uses the recovered spine-width
ratio to crop the observed back-spine-front scan layout into the three visible
materials; otherwise it uses the separate image families. The Rust domain and both
XML readers implement the recovered LaunchBox 13.27 root-level
`ModelSettings` schema, exact `box`/`dvd`/`jewelCase`/`longJewelCase` keys,
signed ARGB colors, semicolon-delimited forced sizes, all 41 built-in platform
defaults, and whole-record game-over-platform-over-built-in precedence.
Unknown future model keys and XML remain lossless. QML applies the resolved
case/cover color and proportions, with separate functional box, DVD, jewel,
and long-jewel presentations. The real game and platform editors expose the
recovered model type, colors, full-scan settings, forced size,
front-spine/logo values, and sparse rotation strings. Saving creates, replaces,
or removes the whole scoped override in the same atomic XML transaction while
retaining unknown children; LaunchBox path-like resource strings stay opaque
and never become host paths.

Left-drag or arrows rotate, right-drag or Shift+arrows translate, the wheel or
`+`/`-` zooms, and double-click, Home, or `0` resets. Visible controls expose
the same paths. Free, horizontal, and vertical rotation lock is stored
atomically in the platform-native user configuration directory and shared
between the two applications without changing LaunchBox XML. Sequential
offscreen scenarios require the fixture's game jewel override to beat its
platform DVD override, require the expected jewel dimensions and full-scan
spine ratio, exercise both lock axes, render both model screens, verify native
regular textures and focus return, compare exact persisted state, and hash the
media/XML tree. Separate editor scenarios prove exact model XML and
game/platform inheritance changes. Logo/front-spine/rotation material
rendering, CoverFlow/image-view integration, exact original
meshes/materials/camera/timing, native controllers, and real Windows/macOS Qt
execution remain open.

Both frontends also use one typed, read-only library-filter contract. It
combines text and platform/category/playlist navigation with favorite,
completion, installation tri-state, played/rated, hidden, and broken state;
independent hidden/broken inclusion; and all 12 persisted LaunchBox
missing-media flags. LaunchBox presents inline selectors and toggles, while
BigBox presents a focus-navigable game-filter drawer. Hidden and broken games
remain excluded by default, unknown filter keys cannot change active state,
editor state changes and first-play statistics recompute active membership,
and the offscreen suite proves combined predicates and byte-identical library
XML in both applications.

Validated launch sequences also carry an effective startup-screen policy.
An explicit per-game override wins over the selected emulator's defaults;
direct, DOSBox, and legacy ScummVM targets use the game's own settings. One
compiled Qt Quick startup overlay is shared by LaunchBox and BigBox. Each
frontend selects its own `Settings.xml` or `BigBoxSettings.xml` global enable,
theme name, cursor policy, and minimum startup/shutdown durations.
`StartupLoadDelay` is applied before spawning the primary process on the
existing launch worker; it is not repurposed as a post-start timer. The startup
overlay remains until both primary start and the frontend minimum have passed.
After child exit, the effective `DisableShutdownScreen` policy drives a second
shared modal overlay for the frontend-specific shutdown minimum. The offscreen
runtime suite captures both rendered overlays in both frontends, measures the
pre-launch and minimum-display intervals, proves that a short-lived primary
completes the startup minimum before shutdown, and proves global disable
bypass, exact arguments, continued supervision, and LaunchBox-compatible
statistics writes.
Every spawned primary starts in an isolated Unix process group or Windows Job
Object. Supervision, temporary-resource leases, shutdown presentation,
play-time accounting, and pause/resume continue through descendants that remain
in that launch session after the direct child exits. A portable Rust process
fixture proves both direct and delegated lifecycles without `/bin/sh` or
platform-specific test payloads. Theme/media asset rendering, window/focus
handling, pause scripts, global/controller pause input, and processes that
explicitly escape the supervised session remain open.

The complete read index covers all 107 `Game` fields observed in the 13.24
installation plus every other platform record, playlist, emulator mapping,
platform/category/folder navigation record, parent, setting, controller,
binding, import-blacklist item, and list-cache item. Lossless typed editors for
platform files and all ten auxiliary document families are available through
the Rust storage API. They validate mutations, keep exact durable sibling
backups, and use atomic replacement. `LibraryTransaction` adds exact SHA-256
source revisions, conflict checks, a durable multi-document manifest,
automatic rollback, and conservative crash recovery that refuses to overwrite
externally diverged data. The desktop shell exposes pending recovery and write
conflicts, and its write-enabled editor safely changes favorite, completed,
star-rating, title, sort title, notes, developer, publisher, genre, series,
region, release date/type, version, source, status, content rating, player/mode,
progress, and Wikipedia URL fields through one versioned typed edit payload and
one transaction. The same payload transactionally adds, edits, or removes
ordered alternate names and custom fields and edits the ordinary, DOSBox, and
legacy ScummVM launch fields. Persisted paths remain lexical
LaunchBox strings in QML, the domain, and storage; only `lb-platform` maps them
to native paths at launch. Descriptive metadata participates in search;
metadata changes recompute stable sort and search membership while state-only
changes use targeted role notifications. The shared query layer also implements
the recovered `Settings.xml` `SortBy` and `SortByDesc` contract for title/sort
title, platform, release and library dates, last played, play count/time,
local/community rating, developer, publisher, genre, series, status, and
favorite state. Missing primary values remain last in either direction and
title/ID tie-breaks are deterministic. LaunchBox exposes inline Arrange By,
separate Select Random and Play Random controls, Qt's platform-standard Find
binding, and the recovered `Ctrl+Alt+Q` selection command. Play Random is
guarded against overlapping library and launch operations and enters the same
stable-ID, shell-free process boundary as ordinary Play. BigBox exposes the
same ordering and selection in its keyboard-navigable drawer plus its recovered
Random Game input action and visible button. Changes use the existing
transactional settings writer with an exact backup. Random selection operates
on the visible model and avoids the current game when possible. This is
platform-neutral Rust/Qt behavior shared unchanged by Linux, Windows, Intel
macOS, and Apple Silicon; the exact protected random algorithm is not claimed.
The recovered command evidence and clean-room boundary are recorded in
`analysis/desktop-commands-13.27.md`.
LaunchBox can now switch that same model and stable-ID selection between its
box-art grid and a virtualized, horizontally scrollable list. The list shows
all 35 columns recovered from LaunchBox 13.27; supported headers drive the
shared sorter. A real Columns dialog controls visibility, display order, and
bounded pixel widths. The original `Settings.xml` `ListView`,
`ListViewOrderedColumnPriorities`, and
`ListViewVisibleColumnIndexPriorities` values are loaded and transactionally
persisted with their stable WPF indexes, while machine-specific widths remain
in the port's platform-native UI state. A new process restores both documents
on Linux, Windows, Intel macOS, and Apple Silicon. The same grid now loads and
transactionally persists LaunchBox 13.27's normalized `NextBoxSize` value. Its
real Qt slider uses the recovered 0.05–0.50 range, 0.001 step, and 0.01 button
change, dims in list view, retains stable selection, and computes responsive
box cells from the logical Qt window size rather than host-specific physical
pixels. Compact tiles collapse action chrome safely at the low end. A compiled
two-process scenario drives the real slider, renders the resized grid, proves
one exact settings backup and byte-identical platform XML, and restores the
value without another write.
Existing-platform game additions generate UUIDs and use targeted Qt row
insertion. Deletion freshly scans every modeled platform, playlist, navigation,
clone, save, controller, and blacklist reference and refuses orphaning; an
unreferenced game uses targeted row removal and never deletes media files.
The platform sidebar creates, edits, and deletes platforms through recoverable
transactions. Its scrollable editor covers the recovered descriptive,
hardware, BigBox, media-override, import, and ordered platform-folder fields.
Retained rows keep unknown XML; empty optional values remove their known
elements; folder values remain lexical strings and never trigger host
filesystem operations. Platform names are converted to portable XML
filenames only by `lb-platform` (for example, `Dragon 32/64` becomes
`Dragon 32_64.xml`), while all 51 default media-folder values remain lexical
LaunchBox paths with backslashes. The 13.27 plugin contract exposes
`IPlatform.Name` as getter-only and its protected save body cannot establish
rename semantics, so the editor keeps identity read-only until a runtime
oracle can verify every dependent document and filename update. Platform
deletion scans games, emulator
mappings/defaults, parents, playlists/filters, navigation state, controller
associations, and frontend settings, requires an empty platform document, and
removes only the catalog/folder records and XML document—never media. Empty
catalog platforms remain visible and can receive their first game.
LaunchBox also exposes an Emulators manager backed by the same transaction
boundary. Its typed editor covers all 31 recovered `Emulator` fields and every
per-platform mapping field, creates immutable UUID identities, preserves
source-indexed retained mappings and unknown XML, validates duplicate mappings
and defaults, and transfers the default for a platform atomically. Delete
freshly scans games and additional applications and refuses to orphan an
explicit emulator reference. Application paths remain lexical LaunchBox
strings on every host. The same manager can scan the portable `Emulators`
folder, native application locations, and `PATH` for evidence-reviewed
BigPEmu, RetroArch, Dolphin, PCSX2, ScummVM, and Xemu executable names. Results show their
native path and provenance, never execute a candidate, and require review in
the complete editor before transactional registration. Portable paths are
stored with LaunchBox backslashes while mapped and external paths use the
shared cross-platform host-path boundary. Configuration and discovery never
modify, run, or delete candidate binaries. The first managed binary lifecycle
is available separately for PCSX2: it reads the official `PCSX2/pcsx2` GitHub
release catalog, selects the exact host artifact, requires GitHub's SHA-256
digest and byte count, streams the download with progress and cancellation,
and installs it under `Emulators\PCSX2`. Every native artifact path, the empty
`portable.ini`, a portable relative-path/digest ownership manifest, and the
`Data/Emulators.xml` registration commit under one recoverable transaction.
Existing PCSX2 user files are retained; updates and repairs recheck managed
state, remove only exact obsolete provider paths, and keep recovery copies.
An offline Qt removal review requires the complete current manifest, refuses
modified/missing/unsafe owned files and pinned emulator references, then removes
only exact owned files plus the managed definition in one recoverable
transaction. User settings, unrelated files, and directories remain. Linux uses
the official AppImage, preserves its executable mode, and the Nix package
routes `.AppImage` launches through packaged `appimage-run` without a command
shell. Windows selects the official Qt x64 7z and uses the safe archive
boundary. macOS selects the official Qt tar.xz, bounds and verifies its single
XZ stream, audits the nested tar through the same traversal/link boundary,
normalizes the versioned upstream root to a stable `PCSX2.app`, and preserves
the main executable mode. The upstream bundle is x86-64, so Apple Silicon needs
Rosetta 2; native macOS runtime execution still needs a real-host gate.
BigPEmu has a second managed provider using Rich Whitehouse's official release
page. It selects the exact Windows x64/ARM64 ZIP or Linux x64/ARM64 tar.gz,
checks the published byte count and uppercase 64-bit FNV-1a value, computes a
local SHA-256 receipt, and safely extracts the complete package. Linux tar.gz
handling validates and bounds the single GZip stream before routing every tar
member through the existing traversal/link/special-file boundary. The optional
`make_desktop.sh` helper is explicitly excluded and never invoked; Qt registers
and launches the native executable directly. Install, update, repair, and
removal share the complete ownership, conflict, recovery-copy, user-file
retention, and lossless `Emulators.xml` transaction contract described above.
Xemu is the third managed provider. It reads the official
`xemu-project/xemu` GitHub release, selects only the exact versioned Windows
x64/ARM64 ZIP, Linux x64/ARM64 AppImage, or signed universal macOS ZIP for the
host, and rejects debug, unsigned, and moving-alias artifacts. Downloads require
GitHub's byte count and SHA-256 digest. Linux installs a stable executable
`xemu.AppImage`; Windows archives must contain root `xemu.exe` and
`LICENSE.txt`; macOS archives must contain root `LICENSE.txt` and the exact
`xemu.app/Contents/MacOS/xemu` bundle executable. ZIP members use the shared
safe archive boundary, bundle permissions are retained, and install, update,
repair, stale-owned-file cleanup, registration, and removal use the same
recoverable ownership contract while retaining user configuration and BIOS
files. RetroArch is the fourth managed provider: Windows and Linux use the
exact stable frontend-and-cores pair, while Intel and Apple Silicon use the
official universal Metal app. Its mapping editor also freezes all 56 LaunchBox
13.27 platform/core suggestions and safely inventories only installed native
Windows `.dll`, Linux `.so`, or macOS `.dylib` cores through bounded
`retroarch.cfg` and reviewed portable/host locations. Choosing an installed
core replaces only the semantic `-L`/`--libretro` argument, preserves unrelated
flags, and saves through the existing recoverable `Emulators.xml` transaction.
Discovery is read-only and refuses symlinked configuration/directories and
unsafe or ambiguous entries. Other managed emulators, dependencies,
individual-core acquisition/update/removal, BigBox core selection, netplay,
and automatic update policy remain open. A
configured PCSX2 entry also exposes a read-only BIOS audit using the complete
recovered 13.27 group of 73 filename-and-MD5 alternatives. It resolves
`portable.ini` and `inis/PCSX2.ini` or the host-native PCSX2 configuration root,
streams hashes without loading firmware into memory, requires any one valid alternative
for the required group, and reports missing, mismatched, unreadable, or unsafe
symlink entries without executing PCSX2 or writing any file or directory.
Configured Xemu entries use the same generalized Qt manager for the second
read-only BIOS adapter. It resolves portable `xemu.toml` first, then Xemu's
host-native data location, maps legacy Windows drive/UNC values through the
shared host-path service, and audits the recovered required boot-ROM, HDD, and
flash-BIOS groups. The HDD requires a readable regular file; firmware requires
an exact recovered MD5. It never downloads Xemu's dashboard HDD, creates
`bios`/`saves`, starts Xemu, or rewrites configuration. Configured RetroArch
entries use the third adapter. It parses every one of the
630 core/platform BIOS rows embedded in LaunchBox 13.27, derives the selected
cores from the configured per-platform command lines, and preserves required
files plus `None`/`Any`/`All` group rules. Application-local and host-native
`retroarch.cfg` locations are checked without starting RetroArch;
`system_directory` supports portable, native, home-relative, and explicitly
mapped Windows paths. Nested catalog paths are resolved case-insensitively
without trusting ambiguous names or symlinks. RetroArch's dynamic
`system_directory = "default"` is reported rather than guessed because it
depends on the launched content directory. Other-emulator BIOS adapters and
all firmware acquisition or mutation remain open.
The same sidebar now renders recovered category/platform nesting from
`Parents.xml`, including multiple placements, recursive category game counts,
and descendant filtering. Its category dialog edits the recovered mutable
metadata and root/category/platform/playlist placements while keeping the
getter-only category name fixed after creation. Each save transactionally
updates both `Platforms.xml` and `Parents.xml` with exact backups. Deleting a
category removes its placements and detaches direct child categories,
platforms, and playlists to root; it never deletes their records, games, or
media. All stored video paths remain lexical LaunchBox strings.
Playlist nodes now participate in the same nested sidebar with stable
`PlaylistId` keys, manual or auto-populated counts, recursive category counts,
and exact membership filtering. The playlist dialog covers the recovered
mutable metadata and BigBox fields, manual game order, source-indexed automatic
filter rules, and multiple root/category/platform/playlist placements. The
13.27 contract exposes both `PlaylistId` and the unique `Name` as getter-only,
so an existing playlist edits its `NestedName` display label while retaining
identity. Create/edit writes the playlist document and `Parents.xml` together;
delete removes all instances of the playlist, clears matching `ListCache.xml`
rows when present, and detaches direct children to root. Game XML and media are
never modified or deleted. Playlist filenames use the platform layer's shared
Windows/Linux/macOS-safe component rules, while stored video and game paths
remain lexical LaunchBox strings.
BigBox exposes the same hierarchy through a keyboard-first filter drawer:
Up, Tab, or the **Browse** button opens the category/platform/playlist list,
Enter applies exact
membership, and Right returns to the horizontal game wheel. Stable playlist
IDs stay behind display names, the active filter is controller-owned state,
and nodes marked `HideInBigBox` are omitted while visible descendants are
reparented to the nearest visible level.
Interrupted transactions require an explicit Recover action; conflicts require
a reload and never offer a blind overwrite.

Both frontends now expose read-only manuals and per-game music through the
shared platform boundary. A safe explicit `ManualPath` or `MusicPath` wins;
otherwise the configured per-platform Manual or Music folder is matched by
LaunchBox's title-and-ordinal filename convention. The bounded index accepts
the document and soundtrack extensions observed in the real library, expands
local M3U playlists in document order, deduplicates tracks, and refuses remote
playlist entries, nested playlists, symlinks, oversized files, traversal, and
unbounded lists. Persisted Windows separators remain lexical XML data; Qt sees
only native local-file URLs produced by the existing Linux/Windows/macOS path
resolver.

LaunchBox adds **View Manual** and **Play Music** to docked and popped-out game
details and honors `AutoPlayMusic` and `ShuffleMusic`. BigBox adds the same
actions to its game wheel and full-screen details, gated by
`ShowGameMenuViewManual` and `ShowGameMenuPlayMusic`, and applies the recovered
game-list/details autoplay, music-over-video, repeat, soundtrack-shuffle, and
volume settings. One lifecycle-owned Qt Multimedia player supplies
previous/play-pause/next/stop and volume controls, refreshes on library
replacement, yields to selected video according to policy, and always stops
before a game or additional application launches. Manuals open through Qt's
cross-platform default-application service. Exact Rust tests and compiled
LaunchBox/BigBox smokes use a valid PDF, a real MP3, and a two-track M3U;
exercise the visible actions and audio controls; render the player; validate
safe local URLs; and byte-check the library/media tree.

BigBox now also indexes the recovered `Music/Background` hierarchy, including
the default collection and exact `Platforms`, `Playlists`, and
`Platform Categories` context folders. Folder names pass through the shared
Windows/Linux/macOS-safe component rules, while bounded M3U expansion retains
playlist order and rejects symlinks, traversal, remote URLs, nested lists, and
case-ambiguous contexts. The typed BigBox policy covers enablement, volume,
shuffle, context-specific fallback, on-screen display, and whether video audio
may coexist. A persistent music control opens the real previous, play/pause,
next, stop, and volume surface; navigation changes collections atomically,
missing contexts fall back to default music, and game music, video policy,
launches, library replacement, and window close own explicit pause/stop/resume
transitions. Compiled scenarios decode all four two-track collections, drive
the real navigation and controls, render the OSD, verify both video-audio
coexistence settings, and hash the complete media tree. In-app document
rendering, theme-specific sound integration, broad codec/backend parity,
configurable global media hotkeys/notifications, and native Windows/macOS Qt
Multimedia execution remain open.

BigBox application startup presentation now uses the same cross-platform media
boundary. A small settings/media probe runs before the background library
index, so presentation starts during the load instead of after it. Direct
regular files in `Videos/Startup` form the randomized video collection and
take precedence over exact legacy `Videos/Startup.mp4`. When no video exists,
typed `ShowStartupSplashScreen` controls an original port-owned Qt splash and
typed `PlayStartupSound`, `SoundPack`, `VolumeStartupSound`, and `VolumeMaster`
select and scale either direct WAV files in `Sounds/<pack>/Startup` or exact
legacy `Sounds/<pack>/Startup.wav`. Discovery is sorted and bounded;
unsupported, oversized, nested, symlinked, and non-regular entries are
refused. The lifecycle-owned Qt Multimedia players use native local URLs; a
video runs to its natural end or the shared key/tap skip, while the no-video
splash remains through the library load and its startup WAV decodes
independently. Background and game music stay blocked until handoff.

Compiled scenarios decode genuine H.264 through both video layouts, exercise
skip and natural completion, decode both startup-sound layouts, prove enabled
and disabled splash/sound policy, render and color-check both presentations,
require the startup probe before background loading, validate selected safe
files in Rust, and hash settings plus all media. The proprietary LaunchBox
splash artwork is deliberately not copied. Exact branded artwork/theme
fidelity, custom-theme packaged sound precedence, controller mapping, primary
monitor routing, original VLC/WMP backend parity, video-and-startup-sound
coexistence, and native Windows/macOS Qt Multimedia execution remain open.

The first native BigBox Attract Mode subset reads all nine recovered 13.27
settings through a typed Rust policy and starts from configured idle or an
explicit visible action. It spins the real game wheel with a bounded,
deterministic acceleration/deceleration curve, optionally advances among
non-empty filters, pauses for the configured interval, and owns a full-window
key/pointer/wheel exit layer. Direct WAV files under
`Sounds/<SoundPack>/Move` take precedence over exact legacy `Move.wav`; the
same cross-platform safe-media boundary publishes local URLs to Qt Multimedia
and combines the separate attract navigation/master volumes. Compiled enabled
and disabled scenarios prove automatic and manual entry, wheel/filter changes,
decoded sound, volume, actual return/input controls, rendered UI, and immutable
library data. The protected product's exact curve, filter algorithm, and
view-switch behavior remain explicitly open rather than being inferred.

The first native BigBox screensaver subset recovers the four 13.27 presentation
families and all nine persisted screensaver/video settings into a typed Rust
policy. It projects stable-ID candidates only from the existing symlink-safe
native media index, applies the three missing-media switches, starts from true
idle or an explicit visible action, selects randomly while avoiding the
current game when possible, and swaps within the configured bounded interval.
One lifecycle-owned Qt layer renders fanart/metadata, full video, split
box-and-video/screenshot, or centered media/metadata layouts. Enter explores
the selected game; other key, pointer, wheel, and visible-return input restores
the wheel. Video and master volume are composed without a helper process or
runtime shell. Compiled enabled and disabled scenarios exercise automatic and
manual entry, all four rendered views, genuine H.264 playback, swaps,
return/explore actions, and immutable settings/media. Exact protected random
rounding, transitions, custom-theme fidelity, and native
Windows/macOS Qt Multimedia execution remain explicit parity gates.

The native BigBox input subset reads the recovered 59-action vocabulary,
action-specific keyboard slots, raw WPF key integers, gamepad
enable/all-controller policy, hold chords, and 18 default controller rules
directly from LaunchBox XML. Typed Rust converts persisted Windows key values
into Qt portable sequences and logical Qt key events back into persisted WPF
values, while semantic controller events arrive from native Linux, Windows,
and macOS `gilrs` backends. A visible modal Qt editor covers all recovered
keyboard slots, all 59 controller actions, all 46 binding choices, optional
holds, and the two controller flags. Its changed-only payload commits
`BigBoxSettings.xml` and `InputBindings.xml` as one recovery-backed
transaction, retains exact backups, preserves unknown/future/non-BigBox XML,
and reloads the live policy only from committed files. There are no scan
codes, platform device paths, shell commands, or OS API handles in QML. Pure
tests freeze the lossless contract; compiled scenarios drive the shared
dispatcher and render/interact with the editor through CXX-Qt. Windows and
both macOS architecture targets compile the backend; physical high-button
mapping and native device/application interaction remain real-host release
gates.

LaunchBox also has a three-page manual ROM importer backed by the reusable
`lb-import` crate. It accepts multiple native files and folders, optional
recursive discovery and extension filters, file- or folder-derived editable
titles, duplicate handling, and leave/copy/move policies. The review page is a
real Rust-generated plan, and execution re-plans it before writing. Portable
copies use `Games\<platform>\<file>` regardless of host separator; leave-in-
place paths use the shared reverse path-mapping service. Large files stream
into the same durable transaction as the platform XML rather than being held
in memory. The options page can inherit the platform default emulator, select
LaunchBox's explicit direct-launch sentinel, or pin a configured emulator ID;
preview and execution both revalidate named emulators against
`Data/Emulators.xml`. Copy/move can also include every regular file beside a
ROM with the same filename stem and a different extension, matching the
recovered LaunchBox option without interpreting descriptor-file contents.
The preview reports those companion files, and a collision blocks the entire
game instead of producing a partial set. Copy never overwrites a destination.
An optional read-only local metadata search opens
`Metadata/LaunchBox.Metadata.db`, canonicalizes platform aliases, applies the
recovered title comparison and parenthetical qualifier preference, and
uses exact primary/alternate-title results first. Only when none exist does it
apply the recovered partial fallback: a contiguous normalized substring or all
query words in any order, excluding bare numbered suffixes such as `GAME2`.
Filename/query parenthetical qualifiers are preferred after either search.
A unique result is auto-applied; zero or multiple candidates remain visible
instead of being guessed, and an ambiguous row exposes compact title,
database-ID, year, developer, and publisher choices in the review page.
Execution reruns the same ordered search and rejects a selected database ID
that is no longer a candidate before any write. An automatic or explicitly
selected match persists the database ID, overview, developer, genres,
player/mode data, publisher, content rating, release date/type, URLs, and
community rating as typed game fields.
Copy/move can then place the whole game bundle in a cross-platform-safe
`Title (Year)` subdirectory; edited final titles are re-sanitized and all
destination collisions are rechecked during execution.
The recovered PDF-manual option scans each game source folder without
recursing. A case-insensitive ROM-stem match wins; otherwise a sole PDF is
linked, while multiple non-matching candidates remain visible and unselected.
`ManualPath` uses the same reverse host-path mapping as game paths. If a
same-name PDF is part of a copy/move bundle, the game instead points at its
portable committed destination so move cleanup cannot break the link.
Move commits every ROM, companion, and XML write first, then removes a source
only after both files have matching SHA-256 revisions; a cleanup failure
retains the source and reports a warning.
When requested, complete unambiguous `(Disc N)` or `(Disc N of M)` sets in one
folder and extension collapse to one game. Disc 1 remains the main application
and every disc, including Disc 1, is persisted as a priority-ordered additional
application, matching the older real-install records and the existing M3U
launch contract. Incomplete or colliding sets remain separate preview rows.
The recovered LaunchBox “combine ROMs with matching titles” option runs after
local metadata resolution, so primary or alternate filenames that resolve to
the same database game collapse deterministically even when their source names
differ. Exact cleaned titles provide the no-metadata fallback; ambiguous
metadata rows stay separate for review. Filename qualifiers are parsed without
host-native separator assumptions into LaunchBox `Version` and normalized
`Region` fields. Every grouped ROM, including the primary, is retained as a
priority-ordered selectable application named `Play {version} Version...` with
its path, emulator, version, region, developer, publisher, release date, and
import status. Online media acquisition, MAME-specific options, and the
remaining import families are still open.

Both front ends now expose a shared launch vertical. A launch plan selects
an explicit or single default emulator mapping (or a direct executable), keeps
LaunchBox's explicit unassigned-emulator sentinel distinct from a missing
emulator, resolves paths through a host service, parses persisted Windows
command lines into shell-free semantic arguments, expands LaunchBox command-line
variables, and spawns on a Rust worker. Additional applications are indexed per
game and available through a Launch With chooser in both front ends. Main-game
launches pre-validate and priority-sort automatic before/after applications,
honor the recovered 30-second ceiling for a waited before-app, wait for the
primary process before starting after-apps, and reap every child. Once the
primary child starts, the shared controller transactionally increments its
game or selected additional application's play count and records a seven-digit
local-offset LastPlayed timestamp. When that child returns, its elapsed whole
seconds are added to PlayTime; every write retains an exact backup and reports
conflicts or pending recovery through both front ends.

LaunchBox's per-game `Apps` manager now adds, edits, and deletes additional
application records through a versioned typed payload. The editor covers the
recovered 13.27 application, launch-order, emulator/DOSBox, disc/side,
descriptive metadata, installation, and play-statistics fields. Identity,
ownership, storefront, and cloud fields remain immutable and are retained
losslessly, as are unknown XML children. Stored Windows, UNC, and mixed-
separator paths stay lexical; only the launch service resolves them for the
host OS. Delete removes the XML record only, never its target or media, and is
refused while a game-save row references the application. Make Default retains
the selected additional-application row and transactionally copies its shared
launch, emulator, version metadata, provider/cloud, and play-statistics fields
onto the owning game while preserving game identity and game-only data. Save
records now use the full persisted 13.27 contract in both readers. A per-game
Saves manager displays grouped version history and provides transactional
Rename Version, Rename Group, Combine, and Make New Save metadata operations.
Its native emulator adapters inspect configured RetroArch, Dolphin, and PCSX2 launch
targets for the main game and emulator-owned additional applications and
append only new active rows without deleting history. RetroArch discovery
reads the host-resolved `retroarch.cfg`, applies content/core sorting, and
finds regular saves and numbered/auto states. Saturn `.bcr`, `.bkr`, and
`.smpc` files are one explicit companion set with the 13.27 group-ID and
composite-signature rules. Dolphin discovery reads raw ISO/GCM disc headers,
WAD title IDs, or a sibling DolphinTool for compressed RVZ/GCZ/WIA/WBFS
content, then searches portable and native user roots for GameCube folder/card
files, Wii title `data` directories, and two-digit save-state slots. Disc Wii
saves check both recovered high title IDs; WAD saves use their exact high/low
title ID. Stable Dolphin group IDs preserve idempotency even if a persisted
path moves. PCSX2 discovery checks the recovered
portable/legacy roots and the platform-native data root (`~/.config/PCSX2` by
default on Linux), parses exact `.p2s`/`.p2z` state names, and matches serials
from the disc filesystem before the recovered content-name and game-title
fallbacks. The native, shell-free reader locates ISO9660 `SYSTEM.CNF` through
plain ISO and raw-sector layouts, GZip, CSO, CHD v5, MDF/MDS, and NRG images
with bounded metadata/decompression reads, symlink refusal, and a
path/size/modified-time cache. It also enumerates folder-format and native raw
`.ps2` card members with recovered `icon.sys`, GameIndex, serial, title, size,
timestamp, group-ID, and owner rules. The raw-card reader supports logical-page
images and physical 528-byte pages with spare/ECC data, follows the
indirect/direct FAT and directory chains, and isolates an invalid card instead
of aborting the scan. Ordinary PCSX2 state rows use the regular-file
transaction path; card members retain their card/member boundary. Manual
backup derives collision-free portable
`Saves\<Platform>\<ROM name>[-NN].<ext>` targets through the host-path service,
records exact aggregate size, seven-digit UTC modified time, and MD5, and
commits either one regular file or the complete Saturn set with the full new
`<GameSave>` row under one recovery manifest. A PCSX2 member backup extracts
only the named logical save, creates and re-extracts a flat verified 7z,
records the recovered uppercase SHA-256 folder-manifest signature in the
`<Md5>` field, rechecks the live member for a racing change, and commits the
archive plus XML without writing to the card. A legacy ungrouped source receives
an explicit group ID in that same transaction. Dolphin Wii backup creates a
nested 7z through direct process arguments, re-extracts it through the
traversal/link-safe boundary, compares the complete recursive directory
revision, and commits the verified archive and XML row together. The manager
can also permanently remove one resolved regular-file vault backup or complete
Saturn set and its exact source-indexed history row in one revision-checked
transaction. The active files are excluded, and every vault member plus the XML
receives an exact recovery copy. Restore requires one compatible active row in
the same stable group. It first commits the current regular file or complete
RetroArch Saturn companion set as a new vault version and verifies that copy. A
regular file is then revision-checked and atomically replaced from the selected
vault version with a second exact sibling recovery copy.
RetroArch Saturn restore revision-checks all source and target members and
replaces or creates the selected `.bcr`, `.bkr`, and `.smpc` companions under
one recovery manifest; active companions absent from the selected version are
retained to match 13.27. Evidence-backed regular Dolphin save states, GameCube
save files, and PCSX2 save states use the same backup-first atomic restore path;
PCSX2 card-member restore extracts and validates the selected 7z, commits and
rechecks a new vault version of the live member, builds a complete card working
copy, verifies the restored logical manifest, and replaces the card only if its
whole-file or whole-directory revision is unchanged. Folder cards use a
rollback-capable sibling-directory swap; raw cards use a streamed atomic file
replacement and regenerate physical-page spare/ECC bytes with the recovered
13.27 algorithm. Raw restore deliberately requires an ECC-bearing physical
card, matching the recovered Windows gate. Both forms retain the complete
pre-restore card as a sibling recovery copy and preserve unrelated members.
Dolphin Wii restore commits and verifies the current directory first, rechecks
both the selected archive and live tree, then swaps the complete extracted tree
through the rollback-capable directory boundary while retaining the previous
tree. Other unrecognized directories and ambiguous-active cases remain
adapter-gated.
Confirmed active deletion supports a resolved regular file or complete
RetroArch Saturn companion set, including host-mapped files outside the
library root, regular Dolphin GameCube/state files, a Dolphin Wii title
directory, and a PCSX2 folder/raw card member. It first atomically
replaces the active history row with an exact portable vault copy. Only after
that copy is committed and verified does it revision-check and remove the live
file set or swap a validated complete card working copy, retaining exact sibling
recovery copies and rolling back every attempted deletion if a peer fails. Wii
directory deletion uses an exact recursive revision and one rename into a
retained sibling recovery directory after its vault archive commits.
Stored paths remain lexical; only paths resolved by the host-path service are
classified Active or Vault, while unmapped Windows paths are shown as
Unresolved. Metadata operations never move or delete save files, and backup
never changes active files. Other-emulator scanning, other directory/container
backup, stale-row reconciliation, automatic-backup policy, general repair
commands, and the remaining emulator adapters remain open. Manual game
combine/expand is now a separate transactional library
operation: same-platform games become selectable version applications, modeled
platform/navigation/playlist/clone/save/controller/blacklist references migrate
to the retained root, and launchable versions expand back to standalone games.
Stored paths remain lexical, exact XML backups are retained, and ROM/media
files are never moved or deleted. Collapse and remaining presentation parity
stay open.

Emulator
auto-extraction invokes 7-Zip without a shell for ZIP, 7z, and RAR inputs,
rejects unsafe/encrypted/ambiguous archives, preserves an archive-named
temporary folder, expands ROM variables against the extracted file, and leases
that folder until the consuming process exits. The Nix package supplies 7-Zip;
Windows uses LaunchBox's bundled copy when present. DOSBox games are planned
through a dedicated adapter: custom executable/configuration paths, the C-drive
root, and every folder/image mount are resolved to native host paths by the
same path service, while DOS separators occur only inside guest commands. The
adapter supports folder, floppy-image, CD/ISO, and hard-disk-image records,
LaunchBox's configuration-owned `[autoexec]` mode, and shell-free `argv`. The
Nix package supplies DOSBox Staging as the Unix default; Windows selects the
portable bundled DOSBox unless a game specifies a custom executable. Legacy
`UseScummVM` games use a separate shell-free adapter that resolves their stored
game-data folder through the host path service, supplies it as ScummVM's game,
save, and extras directory, and preserves the stored target ID, fullscreen, and
aspect-correction flags. The Nix package supplies native ScummVM on Unix;
Windows selects LaunchBox's legacy bundled executable. LaunchBox 13.27's newer
ScummVM emulator plugin remains compatible with the ordinary emulator path.
When the effective
emulator-platform mapping enables M3U loading, explicit `Disc` additional-app
records are priority-ordered into a temporary UTF-8 playlist. Every entry is
resolved through the host path service; archived discs are extracted first;
the playlist and all extraction directories remain leased until emulator exit.
The QML smoke suite executes argument, archive, M3U, DOSBox, ScummVM, and sequence recorders,
proves exact argument boundaries, a
mapped Windows drive, persisted mapping CRUD and reuse after a LaunchBox-to-
BigBox restart, extraction lifetime and cleanup, before/main/after order, and a
selected BigBox additional-app launch. It also verifies transactional
PlayCount, PlayTime, and LastPlayed persistence for every launch backend,
backup retention, completed-manifest cleanup, and unknown XML preservation.
Elapsed time follows the isolated launch session, including descendants that
remain in its Unix process group or Windows Job Object after the direct child
exits. Focus-based accounting and processes that explicitly create a new
session or break away from the job remain later lifecycle work.

The current QML library browser and launcher are an early functional vertical slice, not a
claim of LaunchBox/BigBox parity. See the implementation status for the exact
verified boundary.

## Reproduce the local analysis

The scripts are deliberately conservative: they verify the installer hash and
refuse to overwrite an existing oracle or decompile tree.

```bash
./scripts/install_oracle.sh /home/ben/Downloads/LaunchBox-13.27-Setup.exe
./scripts/decompile.sh
uv run python scripts/build_static_inventory.py
uv run python scripts/analyze_launchbox_schema.py "$LAUNCHBOX_PATH"
```

Local inputs and generated proprietary artifacts:

- `oracle/LaunchBox`: installed LaunchBox/BigBox oracle
- `oracle/wine-prefix`: isolated Wine prefix
- `decompiled`: ILSpy project output for first-party assemblies

## Important boundary

The binaries use runtime method-body protection. Static decompilation recovers
valuable type structure, UI/resource names, plugin contracts, data shapes, and
integration boundaries, but many method bodies are empty/default stubs until a
runtime initializer restores or dispatches them. The output is therefore a
feature-discovery source, not a faithful implementation that can be translated
line by line.

Distribution of a compatible product, use of LaunchBox trademarks, premium
entitlements, hosted APIs, and bundled third-party assets all require a separate
rights and licensing review. This repository does not bypass licensing.
