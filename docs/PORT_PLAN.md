# CXX-Qt/Rust cross-platform port plan

## Product target

Build two native Qt 6 applications over one Rust core:

- **LaunchBox**: mouse/keyboard desktop library management and configuration.
- **BigBox**: controller-first full-screen couch/arcade experience.

Linux, Windows, and macOS are first-class targets. Linux-hosted checks compile
the portable Rust core for Windows and both Darwin architectures; native Qt
runtime behavior still requires real Windows and macOS host gates.

The objective is behavioral and data compatibility, not a mechanical
translation of protected C# syntax. Existing user libraries and media should be
usable without destructive migration. All new implementation code should have
clear provenance and tests derived from observed behavior and public contracts.

## Proposed architecture

```text
 LaunchBox QML shell                 BigBox QML shell
          |                                |
          +---------- CXX-Qt --------------+
                         |
              Rust application facade
                         |
   +----------+----------+----------+----------+
   |          |          |          |          |
 domain    storage    metadata   launcher   extensions
   |          |          |          |          |
   +----------+----------+----------+----------+
                         |
           per-platform service adapters
        Windows          Linux          macOS
```

Suggested Cargo workspace boundaries:

| Crate | Responsibility |
|---|---|
| `lb-domain` | Games, platforms, categories, playlists, emulators, media, saves, settings, stable IDs, validation |
| `lb-storage` | LaunchBox XML read/write, SQLite local DB/cache, migrations, backups, atomic writes, path portability |
| `lb-query` | Search, filter, sort, suggestions, automatic playlists, sidebar hierarchy |
| `lb-metadata` | Games DB/provider traits, matching, regional/media priorities, download queue, cleanup |
| `lb-import` | ROM, folder, drag/drop, storefront, DOS, MAME, image-pack, and Android export workflows |
| `lb-launcher` | Launch plans, emulator mappings, archives, mounts, scripts/hooks, startup/pause, saves, process supervision |
| `lb-integrations` | Achievements, high scores, OBS, LEDBlinky, cloud, emulator adapters, updater |
| `lb-extensions` | Versioned plugin protocol, out-of-process .NET compatibility host where feasible, theme manifest/model |
| `lb-platform` | Trait definitions plus Windows, Linux, and macOS process/window/input/tray/notification/power implementations |
| `lb-ui-bridge` | CXX-Qt `QObject`s, models, properties, signals, async command/result bridges |
| `launchbox` | Desktop QML application and desktop-specific presentation state |
| `bigbox` | Full-screen QML application, navigation state machine, attract/screensaver/marquee behavior |

Rules for the UI boundary:

- Rust owns domain state, persistence, orchestration, and long-running work.
- QML owns composition, animation, focus, themes, and responsive layout.
- CXX-Qt exposes narrow typed view models and list models; it must not become a
  second business-logic layer.
- Provider and OS behavior sits behind traits and is testable without Qt.
- Both executables consume the same commands/events and library snapshots.

## Compatibility policy to decide early

Three compatibility levels must not be conflated:

1. **Data compatibility**: existing `Data`, `Images`, `Videos`, `Manuals`,
   `Music`, and other user folders load without loss and can be round-tripped.
2. **Behavioral compatibility**: feature scenarios produce equivalent visible
   results, files, process arguments, and provider requests.
3. **Binary/theme compatibility**: existing .NET plugins and WPF XAML themes
   run unchanged.

Data and behavioral compatibility are core goals. Full binary/theme
compatibility is not natively possible in Qt: WPF controls cannot render in a
QML scene and arbitrary .NET plugins can call Windows-only APIs. The practical
plan is:

- a new versioned, cross-platform plugin protocol;
- an optional out-of-process .NET host for non-UI legacy plugins whose public
  API calls can be serialized;
- a converter for common BigBox XAML layouts/bindings into a QML theme model;
- explicit reports for unsupported custom WPF controls or theme plugin code;
- a documented QML theme SDK and migration tool.

This preserves useful compatibility without claiming arbitrary WPF/.NET code
can become native Qt automatically.

## Execution phases and gates

### Phase 0: close the evidence gap

Deliverables:

- working licensed Windows/Wine oracle with a disposable library fixture;
- runtime menu/settings census for LaunchBox and BigBox;
- schema snapshots and file-diff harness;
- screenshot/input/process/network capture harnesses;
- post-protection IL capture for selected methods where black-box behavior is
  insufficient;
- feature matrix split into executable scenarios with stable IDs.

Exit gate: every visible menu, option page, wizard, plugin hook, themed view,
and installed adapter is assigned to a scenario, a justified platform-only
bucket, or proven dead/deprecated code. Static source count is not the gate.

### Phase 1: repository and compatibility foundation

Deliverables:

- Rust/CXX-Qt/Qt 6 workspace, Nix dev shell, formatter/linter/test CI;
- domain IDs and models;
- lossless readers for oracle XML/settings and local DB schemas;
- immutable fixture library and golden round-trip tests;
- platform traits and fake adapters;
- event/command protocol shared by both front ends.

Exit gate: an untouched fixture loads and saves with no semantic diff; unknown
fields are preserved; corrupt/partial writes are recoverable.

### Phase 2: desktop library vertical slice

Deliverables:

- LaunchBox QML shell, sidebar, grid/list, search/filter/sort;
- game/platform/category/playlist CRUD;
- manual ROM import;
- emulator mapping and basic launch supervision;
- images, game details, play counts, favorites/completion;
- backup/restore and settings.

Exit gate: a user can import, organize, close/reopen, and launch a small fixture
library on Windows, Linux, and macOS, with golden file and UI scenario parity.

### Phase 3: metadata, media, and importer breadth

Deliverables:

- provider traits, matching, queues, retries, cancellation, and cache;
- Games DB/authorized provider integration;
- image/video/manual/music handling and cleanup;
- automated folder, bulk, drag/drop, DOS, MAME, storefront, image-pack, and
  Android export workflows;
- audit, bulk edit, ROM scans, consolidation, and migration tools.

Exit gate: all `IMP-*` and `MED-*` scenarios pass on all three platforms or
carry an explicit external-service blocker with an approved substitute.

### Phase 4: launch orchestration parity

Deliverables:

- robust quoting/path translation/process trees;
- archives, multi-disc playlists, mounts, dependencies, BIOS, and cores;
- startup/pause/shutdown flows and per-game overrides;
- cross-platform action hooks replacing AutoHotkey, plus a Windows AutoHotkey
  compatibility adapter;
- save management and emulator-specific adapters;
- Linux window/process/input implementations for X11 and Wayland;
- macOS application-bundle, process, focus, input, and multi-display
  implementations without depending on a Unix-shell compatibility layer.

Exit gate: a locked emulator/game matrix verifies exact arguments, lifecycle,
cleanup, focus restoration, controller behavior, and saves on Windows, Linux,
and macOS.

### Phase 5: BigBox

Deliverables:

- controller navigation/focus state machine;
- all filter and game view families;
- details, popups, media, startup/pause, screensavers, attract mode, security;
- marquee window and multi-monitor routing;
- QML theme engine, bundled clean-room themes, theme manager, and converter;
- performance work for very large libraries and media-heavy views.

Exit gate: every `BB-*` scenario passes with keyboard and at least two gamepad
families, at 1080p and 4K, in X11/Wayland, Windows, and macOS multi-monitor
runs.

### Phase 6: integrations and extension ecosystem

Deliverables:

- RetroAchievements, authorized MAME/Steam/GOG data, OBS, LEDBlinky, cloud, and
  provider-specific emulator features;
- cross-platform plugin SDK and sample plugins;
- legacy non-UI plugin host and compatibility report;
- QML theme SDK, migration diagnostics, and validation tooling;
- updater, crash recovery, telemetry policy, localization workflow.

Exit gate: all `INT-*`, `DESK-*`, and `OPS-*` scenarios pass, and compatibility
limitations are machine-reported rather than silently ignored.

### Phase 7: parity hardening and release

Deliverables:

- full regression suite against frozen 13.27 fixtures and maintained live
  service contracts;
- accessibility, localization, keyboard-only, gamepad-only, offline, corrupt
  data, and interruption tests;
- package/sign/install/uninstall/update flows for Linux, Windows, and macOS,
  including a universal or separately verified Intel/Apple-silicon app bundle;
- licensing, trademark, privacy, and third-party asset review;
- migration/rollback tooling and user documentation.

Exit gate: the mechanically computed parity ledger is complete and no
uncategorized feature surface remains.

## Parity ledger

Each scenario has independent state; a single percentage must be derived from
these fields, never entered manually:

| Field | Meaning |
|---|---|
| `censused` | Upstream surface has stable evidence and ownership |
| `specified` | Inputs, outputs, errors, persistence, and visible behavior captured |
| `implemented` | New Rust/Qt path exists and has unit/integration tests |
| `windows_verified` | Scenario matches the Windows oracle |
| `linux_verified` | Native Linux scenario passes |
| `macos_verified` | Native macOS scenario passes on Intel and Apple Silicon, or on one architecture plus an explicitly reviewed universal-binary gate |
| `cross_platform_verified` | Data exchanged between OSes remains compatible |

Completion is earned only when all reachable scenarios are verified or placed
in an explicitly reviewed exclusion class. Test coverage, code volume, type
count, and a decompiler successfully emitting files are supporting metrics, not
completion metrics.

## Immediate next milestone

The repository and a narrow library-browsing vertical slice now exist. Exact
multi-document transactions, golden semantic-diff/failure coverage, and the
51-role `QAbstractListModel` now pass. Pending recovery, write-conflict UX,
safe rollback, and the first transactional desktop game-state edit also pass a
temporary-library Qt smoke with a targeted `dataChanged`. Transactional title
editing now composes with the first complete persisted-status `LIB-014`
filtering path: 13 game-state predicates, independent hidden/broken inclusion,
and all 12 missing-media flags live in `lb-query`, while LaunchBox exposes
inline controls and BigBox exposes a focus-navigable drawer. Both real
frontends prove combined state/media filtering, invalid-key refusal, safe
default visibility, stable ordering, and zero library writes. Editor state
changes and the first-play statistics transition also recompute active
membership.
The first `LIB-013` ordering vertical now loads and atomically persists the
recovered `SortBy`/`SortByDesc` settings and shares 17 typed sort modes between
LaunchBox and BigBox. Date values are parsed as timestamps, missing/invalid
primary values remain last in either direction, and deterministic title/ID
ties make incremental insertion match full refreshes. Both shells preserve the
selected game across ordering changes and expose random visible-game selection
that avoids the current game when alternatives exist. Pure tests and real
offscreen processes prove invalid-key refusal, exact PlayCount ordering,
single/multi-result random behavior, Settings.xml backup/persistence, and
byte-identical platform XML. Dynamic/custom-field arrangement, group headings,
suggestions, and random play remain open.
The current `DESK-001` vertical now covers both persisted view modes, every
recovered 13.27 list column, and box sizing. The original `ListView`, column
order, and stable visibility indexes use recoverable shared settings writes;
host-specific widths use platform-native UI state. The grid loads the typed
normalized `NextBoxSize`, exposes the stock 0.05–0.50/0.001 slider contract
with 0.01 buttons, and derives cells from Qt logical window units so the same
setting survives Linux, Windows, standard and Retina macOS scaling. Real
offscreen controls prove stable selection, responsive rendering, exact
settings backups, byte-identical platform XML, and fresh-process restoration.
Native Windows and Intel/Apple Silicon macOS Qt interaction remains an explicit
real-host release gate.
The current read-only media vertical keeps front-art selection in both shells
and adds every configured image family plus per-game Video Snap, Theme Video,
Trailer, Recording, and Marquee media to both details surfaces. The shared
cross-platform resolver is the only persisted-path boundary; Qt receives native
local URLs. Type, region, numeric-image, and recovered video priorities are
deterministic, while directory/file/size/item limits and symlink refusal keep
large or hostile libraries isolated. Pure tests and compiled offscreen
SVG/H.264 decode, selection, autoplay, play/pause, actual-thumbnail, and
rendered-gallery scenarios cover both frontends without media writes. BigBox's
first three `BB-004` subsets follow the stock theme's separate image,
image/video, and game-details presenters: Details or mapped Select opens one
full-screen stable-ID layer outside recycled delegates; mapped left/right or
Previous/Next wraps the collection; Select/play-pause and mute control video;
the distinct Play Game action launches; and mapped Back stops playback and
returns to the game wheel. A metadata/notes column accompanies the media.
Mapped Show Images, the real Images button, or View Image from details opens a
second image-only full-screen surface over the same bounded native-URL index.
Previous/next, Select, and page actions switch image types;
the explicit port policy clamps zoom to 100–400% in 25% increments, resets fit
and pan on type changes, and supports wheel, mouse drag, directional keys, and
visible pan/zoom controls. Back returns to details when nested or to the wheel
when entered directly. LaunchBox now uses the same image-only interaction
contract from the Details View Image action or the global `I` action. The
docked path is owned by the main window, while popped-out Details opens the
modal in its own native window; both stop hidden video, retain the selected
image, and restore focus to the media row on Back. A compiled desktop scenario
drives the shared button activation paths, validates the five-to-six
image/media mapping and native regular files, renders a zoomed PNG, and proves
byte-identical media and library documents. Native Windows and Intel/Apple
Silicon macOS Qt
interaction and Qt Multimedia execution remain explicit real-host gates, and
all media mutation, download, platform-video
management, full-scan/model-detail rendering and model-settings editing,
related/document/application panes, custom BigBox theme layouts/transitions,
cleanup, and migration work remains open.
The current `MED-006`/`DESK-006`/`BB-004` slice adds manuals and game music
without moving stored Windows path syntax into QML. One bounded supplemental
index resolves explicit paths first and configured Manual/Music folders
second, validates regular files at the platform boundary, expands only local
non-nested M3U entries, caps every file/list/track dimension, and emits native
Qt local URLs. LaunchBox details and BigBox wheel/details expose the real
actions. Typed frontend-specific autoplay, shuffle, repeat, volume, visibility,
and music-over-video settings feed one lifecycle-owned Qt Multimedia popup
with previous/play-pause/next/stop/volume controls. Selection changes,
library revisions, video priority, window close, and game/additional-app launch
own explicit state transitions; no helper shell or Windows-only API is
involved. Pure policy/index/symlink tests plus compiled PDF/MP3/M3U action,
decode, control, render, local-URL, and whole-tree immutability smokes cover
both shells. Native Windows and Intel/Apple Silicon macOS playback remain
real-host gates. In-app document rendering, broad codec parity, configurable
global controls/notifications, theme music integration, and media mutation
remain later slices.
The current `BB-014` slice extends that boundary to BigBox background music.
It indexes bounded default, platform, playlist, and platform-category
collections from the recovered `Music/Background` hierarchy using portable
context names and ordered local M3Us. Typed settings own enablement, shuffle,
volume, OSD visibility, context fallback, and video-audio coexistence. One
separate lifecycle player follows navigation atomically, provides persistent
previous/play-pause/next/stop/volume controls, falls back to the default
collection, and coordinates selected video, per-game music, process launches,
library replacement, and window close. Pure policy/path/index/symlink tests and
two compiled eight-track scenarios prove both video coexistence branches,
real navigation, audio decode, control wiring, OSD rendering, local URLs, and
whole-tree immutability without a helper shell. Custom-theme sound integration,
configurable global bindings/notifications, broad codec parity, and native
Windows/macOS playback remain later slices.
The current `BB-007` slice implements the application startup presentation
independently of the per-game startup/shutdown overlays. A bounded, read-only
probe loads only BigBox startup settings and media before the background
library index begins. It prefers direct supported files in the recovered
`Videos/Startup` randomized folder and falls back only to exact legacy
`Videos/Startup.mp4`. With no video, it applies typed
`ShowStartupSplashScreen`, `PlayStartupSound`, `SoundPack`,
`VolumeStartupSound`, and `VolumeMaster`; the selected pack prefers bounded
direct WAV files in `Sounds/<pack>/Startup` over exact legacy
`Sounds/<pack>/Startup.wav`. CXX-Qt publishes guarded native URLs and the
typed policy. BigBox chooses randomly in production, owns separate Qt
Multimedia video and sound players, keeps the port-owned splash over the
background load, blocks background/game music, and hands video off on the
shared key/tap skip, natural end, or fail-open decoder error.

Pure precedence/bounds/symlink/policy tests and six compiled scenarios prove
both H.264 layouts, skip and natural completion, both WAV layouts, enabled and
disabled settings, master-adjusted volume, rendered output, early-probe timing,
exact safe source selection, and whole-tree immutability without any runtime
helper shell. The proprietary branded splash is not copied. Exact artwork and
custom-theme parity, packaged-theme sound precedence, controller bindings,
primary-monitor routing, video-and-startup-sound coexistence, original VLC/WMP
backend parity, and native Windows/macOS playback remain later slices.
The current `BB-009` slice recovers all four installed screensaver resources
and their shared game/select contract without copying proprietary XAML or
artwork. A strict Rust policy owns the nine screensaver/video settings,
inclusive bounded swap timing, missing-media gates, stable-ID candidates, and
random selection that avoids the current game when possible. Candidate media
comes only from the existing bounded symlink-safe cross-platform index, and
CXX-Qt publishes guarded native local URLs. One Qt focus layer owns idle/manual
entry, all four fanart/video/box/screenshot/metadata layouts, one-shot swaps,
composed video/master volume, return input, and exact-game exploration.
Launch, load, write, presentation, modal, pause, and Attract lifecycles are
explicit blockers. Six pure tests plus enabled/disabled compiled scenarios
prove real delay and manual entry, all four rendered layouts, H.264 decode,
bounded swaps, disabled-auto behavior, stable selection, and whole-tree
immutability. Exact protected random rounding, transition/controller-binding
behavior, custom themes, and native Windows/macOS multimedia remain later
parity work.
The current `BB-012` slice recovers the complete 59-action BigBox input
vocabulary, every action-specific zero/one/four keyboard-slot contract, the
raw WPF key-number contract, gamepad enable/all-controller settings, all 46
controller binding choices, hold chords, and the exact 18 default controller
rules. A typed `lb-platform` policy converts persisted WPF values into Qt
portable sequences and logical Qt keys back into WPF values, groups duplicate
sequences, validates configured semantic rules, owns press edges and
controller-selection state, and feeds one central QML action dispatcher.
Native `gilrs` backends provide hot-plugged Linux, Windows, and macOS gamepad
events without a runtime helper shell or OS-specific device path in QML. A
visible modal Qt editor covers every recovered keyboard slot, both controller
flags, every action/binding/hold choice, unbinding, row add/remove, and
duplicate validation. Its strict changed-only payload uses one recoverable
transaction for both input XML documents, retains exact backups, preserves
unknown/non-BigBox/future rows, and reloads only committed policy. Pure tests
plus compiled dispatcher and rendered-editor interactions prove the recovered
contract, real route behavior, logical-key/controller edits, exact backups,
and live reload. The Windows and both Darwin cross-target gates compile the
backend. Keyboard automation/mouse options, physical raw high-button mappings,
global input while a game owns focus, and real-device/native Qt validation on
Windows and both macOS architectures remain later parity work. The evidence
and deliberate device-mapping boundary are recorded in
`analysis/input-13.27.md`.
The current `DESK-004`/`BB-004` media vertical adds settings-prioritized box
backs and one shared front/back Qt presenter to the LaunchBox grid and BigBox
wheel. The exact recovered back priority is applied at the native-path index
boundary; missing backs leave the action unavailable. LaunchBox exposes a
visible per-tile Flip/Front control and `F`, while BigBox additionally honors
`ShowGameMenuFlipBox` and assigns the recovered `KeyboardFlipBox=49` default
without colliding with navigation. BigBox wheel selection changes return to
the front; LaunchBox retains each tile's side until its Front control is used.
Separate compiled interactions activate and return through the real controls,
wait for a complete 180-degree transition, render visually checked back-cover
PNGs, validate distinct contained regular files in Rust, and byte-compare the
entire fixture media/XML tree. The protected 13.27 transition duration remains
unknown, so the shared 220 ms animation is an explicit cross-platform Qt
policy. Native Windows and Intel/Apple Silicon macOS interaction remains a
real-host gate.
The current `MED-009` model vertical uses one Qt Quick 3D preview in both
frontends. It builds a true six-face model and maps the safely indexed front,
back, and `Box - Spine` files to distinct faces while keeping top and bottom
as solid materials. A fourth safe `Box - Full` selection can instead construct
the back, spine, and front from the observed back-spine-front scan layout and
the resolved spine-width ratio. LaunchBox honors `ShowDetails3dModel` and enters from
Details or `M`; BigBox honors `ShowGameMenuViewModelFullscreen` and enters
from the game menu or `M`.
Pointer drag, right-drag, wheel, directional/Shift-directional/Page/Home keys,
and visible buttons cover rotation, translation, zoom, and reset. A strict
versioned state document in the native Linux, Windows, or macOS user
configuration location atomically stores free/horizontal/vertical rotation
lock, so one frontend restores changes made by the other without touching
shared LaunchBox XML. Sequential compiled scenarios activate the real entry and
control buttons, prove each lock axis, decode three distinct native regular
textures, render both model surfaces, restore focus, compare the exact
persisted state, and hash the media/XML tree.

A limited managed 13.27 Wine probe recovered the exact
`box`/`dvd`/`jewelCase`/`longJewelCase` keys, constructor and editor defaults,
root-level XML schema, signed ARGB and semicolon-size representations,
game/platform/built-in precedence, and 41 built-in platform mappings. Both XML
read paths and `Data/Platforms.xml` now retain typed `ModelSettings` plus
unknown future data. A platform-neutral resolver computes a complete setting
for each game before the CXX-Qt boundary. The viewer applies colors and forced
size and uses distinct functional box/DVD/jewel/long-jewel proportions with
jewel lips and a DVD hinge. The game and platform dialogs edit the recovered
whole-record property surface, atomically create/update/remove the corresponding
XML record, and retain unknown future children. Compiled LaunchBox and BigBox
scenarios require a game-level jewel fixture to beat its platform DVD setting,
the expected port geometry, and the full-scan ratio before reporting success;
separate editor and platform-lifecycle smokes prove exact persistence and
cleanup.

Front-spine/logo/rotation material rendering,
`Use3dModelImageView`/CoverFlow paths, exact original meshes/materials/camera
and timing, native controller bindings, and native Windows/Intel/Apple Silicon
macOS interaction remain explicit later gates.
The first three `DESK-002` verticals add a resizable LaunchBox selected-game
details pane over the shared virtualized model plus hide/show and native
pop-out/dock behavior. Its stable-ID selection contract survives filter and
sort resets, edits, insertion, and removal, with a defined first-visible
fallback when the prior game disappears. Artwork, descriptive metadata, notes,
installed/favorite/completed state, play statistics, local and community
ratings, and the existing game actions are live in either host. A versioned
port-owned state document stores dock width, visibility, popup state, normal
geometry, and maximized state in the native Linux, Windows, or macOS
configuration location, never in shared LaunchBox XML. Real offscreen
interactions verify selection transitions, native image paths, dock/hide/pop
transitions, a rendered native popup, atomic exact state bytes, shutdown
preservation, and restoration in a fresh process while fixture
platform/settings documents stay byte-identical. Its third vertical follows
the recovered stock `MediaList`/`MediaPreview` shape with a selectable
thumbnail row, large image/video preview, details-video settings, and playback
controls. The next desktop media gaps are specialized model parity, related and
document/application panes, media mutation/management, and multi-display
geometry recovery; native Windows/macOS Qt interaction also remains open.
Transactional title
editing has expanded into 18 descriptive fields through a versioned typed
payload; it recomputes sort/search membership, removes explicitly cleared
optional elements, and is checked through the real dialog in a two-backup
runtime scenario. Existing-platform UUID additions and conservative
reference-gated removals now use targeted Qt row signals and have their own
backup-chain runtime scenario. Catalog-backed platform creation/deletion now
uses recoverable two-document create/delete transactions, portable host
filenames, 51 lexical LaunchBox folder mappings, a ten-family dependency scan,
and real-dialog runtime coverage that proves empty-platform game insertion,
blocked deletion, exact backups, and media isolation. Existing-platform editing
now covers the recovered metadata surface and source-indexed folder rows in the
real dialog; retained records keep unknown XML, and the create/edit/delete
backup chain is verified without creating media directories. Identity remains
read-only until a Windows runtime oracle resolves the getter-only 13.27 name
contract and cross-document rename behavior. The first `RUN-001` configuration
vertical now manages `Emulators.xml` through a typed LaunchBox dialog and Rust
worker. It covers all 31 recovered emulator fields and all six per-platform
mapping fields, creates immutable UUIDs, validates mapping ownership, rejects
duplicate platforms/defaults, transfers a platform default in the same
lossless transaction, and blocks deletion when any freshly scanned game or
additional application pins the emulator. The Linux offscreen scenario proves
edit/create/blocked-delete/delete, three exact backup states, unknown XML
retention, optimistic revision notification, and that Windows-style executable
paths neither become host paths nor create or delete emulator directories.
The first `RUN-003` subset now discovers evidence-reviewed RetroArch, Dolphin,
PCSX2, ScummVM, and Xemu executable identities from a bounded portable tree,
OS-native application locations, and `PATH` without executing or modifying
them. The Qt manager shows native paths, provenance, and registration state,
then sends a selected candidate through the full typed editor with recovered
13.27 defaults and shared reverse host-path conversion. Deterministic
deduplication, the Unix `dolphin` name collision, existing platform defaults,
candidate immutability, exact XML backup, and binary-directory isolation are
tested. The first managed `RUN-003` provider is PCSX2:
the recovered first-compatible-release policy drives exact official GitHub
asset selection; size, URL, name, and SHA-256 are checked during a bounded
streamed download; cancellation precedes all mutation; and the verified
artifact paths, portable marker, complete portable ownership manifest, and
emulator configuration commit together. Existing managed state is audited for
current/update/repair classification, unsafe or unmanaged executable targets
are blocked, and Linux AppImages keep execute permission and launch through
packaged `appimage-run` without a shell. The official macOS tar.xz is now
handled through bounded single-stream decompression followed by the shared
safe tar boundary, stable app-root normalization, complete file ownership, and
main-executable mode preservation. Offline managed removal is now
implemented with exact digest checks, fresh emulator-reference refusal, one
recoverable file/XML transaction, and explicit preservation of settings,
unrelated files, and directories. BigPEmu is the second provider: the four
official Windows/Linux architecture artifacts are selected from the verified
publisher page; published byte counts and FNV-1a hashes plus local SHA-256 are
checked; ZIP and bounded two-stage tar.gz extraction share the safe member
boundary; the optional desktop helper is excluded; and install, update, repair,
stale-file cleanup, removal, mappings, and executable modes are transactional.
Xemu is the third provider: exact official Windows x64/ARM64 ZIP, Linux
x64/ARM64 AppImage, and signed universal macOS ZIP identities are modeled;
debug, unsigned, moving-alias, duplicate, missing-digest, and untrusted assets
are refused; byte count and SHA-256 are mandatory; ZIP layouts and bundle
permissions are preserved through the shared safe extraction boundary; and
install, update, repair, reference-gated removal, recovery copies, Xbox
registration, executable mode, and user configuration/BIOS retention are
transactional. RetroArch is the fourth provider and follows the recovered
13.27 stable-buildbot contract: Windows and Linux install the exact frontend
and matching cores 7z pair, while Intel and Apple Silicon share the official
universal Metal DMG. The buildbot publishes byte counts but no SHA-256
sidecars, so the UI makes that distinction explicit and records locally
computed SHA-256 receipts without claiming an upstream digest. The macOS app
root, signature resources, permissions, and six exact MoltenVK framework
symlinks are preserved as transaction-owned paths. Install, update, repair,
stale-path cleanup, registration, and reference-gated removal remain
shell-free and never execute a downloaded artifact. Further emulator
providers, dependency policy, automatic update policy, and native
Windows/macOS runtime validation remain open under `RUN-003`. The first
`RUN-016` subset freezes all 56 LaunchBox 13.27 platform/core suggestions,
safely inventories only installed native Windows `.dll`, Linux `.so`, or macOS
`.dylib` cores through configured and reviewed portable/host locations, and
lets the complete Qt emulator editor replace one semantic core argument without
discarding unrelated flags. Individual-core acquisition/update/removal,
BigBox selection, and netplay remain open. The first three `RUN-004` adapters audit the complete recovered
PCSX2 BIOS group of 73 alternatives, Xemu's three required boot-ROM/HDD/
flash-BIOS groups, and RetroArch's complete 630-row/103-core requirement
resource without executing an emulator or mutating configuration or firmware.
PCSX2 honors its portable marker and
portable/native `PCSX2.ini` roots. Xemu prefers portable `xemu.toml`, then its
host-native data root; values from an older Windows installation cross the
explicit drive/UNC mapping boundary. RetroArch selects rows from every
configured platform/core mapping, preserves `None`/`Any`/`All` groups, checks
application-local and host-native configuration candidates, and resolves
portable/native/home/mapped-Windows `system_directory` values while refusing
dynamic content roots, symlink traversal, and case ambiguity. All three
adapters stream hashes and expose every result in one generalized read-only Qt
manager. Remaining emulator adapters, firmware acquisition, configuration
changes, and mutation remain open under `RUN-004`. The ROM importer now resolves
primary and alternate filenames through the local database, combines matching
stable IDs into one deterministic game, recovers separator-neutral
version/region qualifiers, and retains every ROM including the primary as an
ordered LaunchBox-compatible chooser entry; the real-dialog smoke covers this
alongside multi-disc import. The first shell-free direct/default-emulator launch
plan now also runs through both Qt front ends on Linux; checked-in
argument recorders prove its exact `argv`, command-line variable expansion,
explicit unassigned-emulator semantics, mapped Windows-drive handling, and
transactional play-statistics writes. Generated archive and M3U fixtures now
also prove
ZIP/7z/RAR preparation semantics, extracted-variable expansion, explicit-disc
priority order, path-resolved playlist generation, supervised temporary-
resource lifetime, and cleanup in both shells. Typed DOSBox mount records now
round-trip losslessly and feed a dedicated launch adapter; a mixed-separator
fixture proves native host paths, DOS guest paths, folder/image mount commands,
and play-statistics persistence through both shells. A legacy ScummVM fixture
likewise proves
native game/save/extras paths, target and display flags, and transactional
session updates through both shells. The launch boundary now increments
PlayCount and records LastPlayed at primary spawn, adds the isolated launch
session's observed whole-second runtime to PlayTime on exit, retains exact backups, and
surfaces transaction failures. The `RUN-009` slice and first `RUN-011` subset
now resolve effective startup/shutdown settings from the actual primary target,
then combine them with the selected frontend's separate global settings.
`StartupLoadDelay` precedes process spawn on the launch worker; it is no longer
approximated as a post-start display timer. Shared startup and shutdown
overlays enforce the frontend-specific minimums, apply the startup-screen
cursor setting, and honor effective `DisableShutdownScreen`; disabling startup
screens globally bypasses both presentation and delay. LaunchBox and BigBox
real-process smokes measure the timing boundaries, render both overlays,
compare exact arguments, and verify statistics. The first `RUN-010` subset now
resolves game-override/emulator-default/direct-game pause policy, combines it
with separate LaunchBox/BigBox global enable/theme settings, and sends typed
pause/resume commands to the supervising worker. Unix controls the exact
process group; Windows controls all processes in a private Job Object. The
direct child can delegate while supervision, pause/resume, temporary-resource
leases, and session accounting continue. Both shells render and verify the
shared pause overlay with the same portable Rust fixture used by platform
integration tests. Theme/media asset selection, pause scripts, global
input/focus behavior, mute/fade behavior, processes that deliberately escape
the supervised session, and pause-excluded play time remain open.
Source-indexed alternate names and custom fields
now edit through the same real Qt dialog and transaction; retained rows keep
unknown XML, new rows survive reload, and the custom-field fixture is explicitly
13.27-contract-derived because the older real library contains none. A per-game
additional-application manager now covers the recovered editable field set
through a versioned payload and transactional worker. Its real-dialog smoke
proves edit/add/delete ordering, targeted model updates, three exact backup
states, lexical Windows paths, immutable provider/cloud state, unknown XML,
game-save reference refusal, and target/media isolation. Its Make Default action
retains the selected row and transactionally copies the evidence-derived shared
launch/version fields onto the owning game; a second real-dialog smoke proves
the edit/default backup chain, direct-launch sentinel, model refresh, lexical
Windows path, and game-only/unknown-data retention. The first save-management
subset now models the full persisted 13.27 save record and provides lossless
group/version inventory plus rename, combine, and split metadata operations. A
new platform-neutral `lb-integrations` boundary owns explicit RetroArch,
Dolphin, and PCSX2 save adapters. All reuse configured launch-emulator selection for main
and emulator-owned additional applications and append only new rows without
deleting history. RetroArch discovery reads the native host-resolved
configuration, follows content/core sort settings, and finds regular saves,
numbered/auto states, or one grouped Saturn companion set. Dolphin derives
disc IDs from raw ISO/GCM headers, WAD title IDs, or a sibling DolphinTool for
compressed images, searches portable and OS-native user roots, and records
region-aware GameCube folder/card files plus exact two-digit state slots with
stable group IDs. It also derives the recovered disc and WAD high/low title IDs
and discovers Wii `data` directories behind an explicit directory-container
result.
PCSX2 checks portable/legacy and OS-native data roots, parses the recovered
state filename/slot/group contract, extracts the content serial from ISO9660
`SYSTEM.CNF`, and discovers both folder-format and raw `.ps2` card members with
recovered serial, GameIndex, title, `icon.sys`, grouping, and single-context
fallback rules. The disc reader handles plain ISO and raw-sector layouts,
GZip, CSO, CHD v5, MDF/MDS, and NRG through native Rust with bounded reads,
symlink refusal, and no subprocess. The card reader handles logical pages or
physical 528-byte pages with spare/ECC data and traverses the indirect/direct
FAT and directory chains. An explicit integration type preserves the
card/member boundary, and invalid cards are isolated from the rest of
discovery. Ordinary state rows use the regular-file path.
Manual backup now covers one resolved regular active
file or all present `.bcr`/`.bkr`/`.smpc` members with collision-free portable
vault naming, aggregate size/time and 13.27-compatible MD5 metadata,
source-revision checking, and one recoverable file-set-plus-XML transaction; it
never changes active files. It also covers one PCSX2 logical card member by
extracting it to a flat 7z, verifying the archive by re-extraction and the
recovered uppercase SHA-256 folder manifest, re-extracting the live member to
detect a race, then committing the archive and XML together without modifying
the card. Dolphin Wii backup creates a nested 7z with direct process arguments,
re-extracts it through the safe archive boundary, compares the full recursive
revision, and commits the verified archive and XML together. A separate
confirmed action deletes one resolved regular-file vault backup or complete
Saturn set and its exact history row while retaining exact file/XML recovery
copies. Regular-file restore requires one compatible resolved active row in the
same stable group, commits and verifies a new vault version of its current
bytes, then revision-checks and
atomically replaces the active file from the selected vault version while
retaining an exact sibling recovery copy. RetroArch Saturn restore first
commits and verifies the complete current companion set, then revision-checks
and atomically replaces or creates all selected companions while retaining
active members absent from the selected version like 13.27. Other-emulator
scanning and other directory/container backup remain open. Active deletion now
first atomically replaces one regular or Saturn active row with its verified
portable vault set, then revision-checks and deletes even host-mapped external
live files with exact sibling recovery copies and all-or-rollback behavior; the
PCSX2 path described below applies the same backup-first invariant at the
logical-member and complete-card boundaries. Dolphin Wii restore and deletion
apply it at the whole-title-directory boundary with exact revision checks and
retained complete-tree recovery directories.
Recovered Dolphin and PCSX2 13.27 regular-file semantics allow the same path
for discovered Dolphin GameCube/state files and PCSX2 state rows. PCSX2
card-member restore/deletion now follows the recovered complete-working-copy
shape for both folder and raw cards: it commits and verifies the current
logical member as a flat 7z, mutates a private whole-card copy, re-extracts the
result for manifest verification, and revision-checks the selected archive and
live card before a recoverable swap. Folder cards use a rollback-capable
sibling-directory replacement; raw cards use atomic file replacement and
regenerate physical spare/ECC pages. Raw restore is deliberately limited to
ECC-bearing physical cards, matching the recovered 13.27 gate. If that private
raw-card import first fails for lack of space, the port follows the recovered
13.27 capacity-recovery path: it walks every directory reachable from the root,
marks each referenced FAT chain, clears only allocated unreachable clusters,
durably rewrites the working-copy FAT, and retries the import. The source/live
card is never repaired in place, a zero-cluster repair still retries like
13.27, and the Qt result reports the exact reclaimed count only when recovery
occurred. Active deletion
commits the vault archive and removes the active XML row before swapping the
validated deletion copy, so a final conflict remains recoverable through Find
Active Saves. Stale-row reconciliation, automatic policy, general repair
commands, and the remaining adapters remain open. The manual
`LIB-010` subset now
combines same-platform games into launchable version applications and expands
them back transactionally; collapse and remaining presentation parity stay
open.
The next milestone is
to close the remaining
Phase 0/1 evidence and product-safety gates:

1. Make the 13.27 oracle run in a supported Windows VM if Wine remains blocked.
2. Capture first-run and edit/import/launch diffs for the fixture.
3. Convert the first ten critical runtime scenarios into structured fixtures
   and map them to feature-matrix IDs.
4. Expand desktop editing beyond the now-working emulator-configuration,
   additional-application, platform, nested-category, playlist, save-group
   metadata, and manual
   combine/expand workflows into the remaining filesystem-safe save
   backup/restore/delete adapters and reviewed cascade/remediation choices for
   dependencies. Category and playlist deletion now follow
   the recovered detach-to-root behavior without deleting child records or
   media. Alternate names and custom fields now pass typed storage and real-dialog runtime gates. The
   launch-configuration subset now edits direct/emulator/DOSBox/ScummVM fields
   transactionally, preserves stored Windows path syntax, and proves a fresh
   Linux process can execute the edited path; rich descriptive metadata,
   existing-platform add/remove, and conservative cross-document validation
   also pass.
5. Extend the launcher beyond the now-working direct/default-emulator, Launch
   With, automatic additional-app, persisted host-mapping, archive
   auto-extraction, multi-disc/M3U, DOSBox mount, legacy ScummVM, and inherited
   frontend-specific startup/shutdown-overlay and supervised process-group/job
   pause/resume paths into scripts, theme/media selection, global/controller
   pause input, focus/window handling, deliberate session-escape handling, and
   focus-aware pause-excluded play-time parity.
6. Run the native Qt shell and transaction scenarios on Windows as well as
   Linux; the current Windows gate covers the non-Qt core crates.

The initial QML shells must remain evidence clients for these contracts, not a
substitute for earning behavior and data parity.

## Known risks and external decisions

| Risk | Required treatment |
|---|---|
| Closed-source code, trademarks, premium assets/features | Obtain legal/rights guidance before publishing or distributing; do not bypass entitlement checks |
| LaunchBox Games DB, cloud, theme/plugin downloads, and other hosted services | Use documented/authorized APIs or replace with configurable providers |
| Protected implementation bodies | Prefer behavior specs; dump runtime IL only for owned/licensed analysis where necessary and keep provenance |
| WPF theme and .NET plugin compatibility | Use conversion/out-of-process bridges with explicit compatibility reports; do not promise arbitrary binary compatibility |
| AutoHotkey and Windows shell/process assumptions | Define portable action and OS service APIs; retain Windows compatibility adapters |
| Wayland focus/global-input restrictions | Design around portal/compositor constraints and test supported compositors explicitly |
| Emulator/provider drift | Version adapter contracts and maintain live conformance fixtures separately from the frozen 13.27 oracle |
