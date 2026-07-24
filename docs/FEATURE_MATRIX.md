# LaunchBox 13.27 static feature matrix

This is the first mechanically grounded feature census for the port. It is
derived from installed resources, semantic classes, menu actions, view models,
plugin contracts, emulator adapters, and shipped themes. It is intentionally
called a **static** matrix: protected method bodies and the currently failing
Wine launch mean exact workflows, edge cases, and premium gating still need
runtime-oracle scenarios.

Evidence abbreviations:

- **DV**: LaunchBox desktop view/view-model
- **DM**: LaunchBox desktop menu action or command
- **BV**: BigBox view/view-model, option page, or menu action
- **SI**: shared implementation/integration type
- **PA**: public plugin API contract
- **TX**: shipped BigBox theme XAML
- **TP**: bundled third-party runtime/tool

Every row starts at `censused`, not `implemented`.

Port implementation status remains separate from this static census. The
current verified vertical implements the PlayCount, PlayTime, and LastPlayed
storage/session subset of `LIB-012` for main games and selected additional
applications. It also implements a transactional 18-field descriptive editor
subset of `LIB-002`, transactional direct/emulator/DOSBox/ScummVM launch-field
editing subsets of `LIB-001` and `RUN-002`, and a descriptive-metadata search
subset of `LIB-013`, now including 17 typed Arrange By modes, atomic
`SortBy`/`SortByDesc` persistence, and visible-model random selection in both
frontends. The first `DESK-001` subset also switches LaunchBox between its
box-art grid and a virtualized multi-column list over that same model and
stable-ID selection. It loads and atomically persists the original `ListView`
setting, exposes query-backed sortable headers, and renders all 35 recovered
13.27 columns. The original stable column order and visibility indexes persist
transactionally in shared `Settings.xml`; bounded per-host widths persist in
platform-native UI state, and a real Columns dialog edits all three. The grid
also implements the recovered normalized `NextBoxSize` contract with a real
0.05–0.50 Qt slider, 0.001 steps, 0.01 buttons, list-view dimming, responsive
logical-window sizing, atomic exact-backup persistence, and fresh-process
restoration. A source-indexed
transactional editor also implements the
three-field alternate-name and name/value custom-field subset of `LIB-003`.
The dialog-driven portable platform create/edit/delete, recovered platform
metadata, and source-indexed folder subset of `LIB-006` is also implemented
with unknown-XML retention, conservative dependency refusal, and media
isolation. Platform rename remains gated on runtime evidence because the
installed 13.27 `IPlatform.Name` contract is getter-only.
The nested category/sidebar subset of `LIB-007` and `DESK-003` is implemented:
categories can be created, edited, multiply placed, recursively counted and
filtered, and deleted through one paired `Platforms.xml`/`Parents.xml`
transaction. Direct children detach to root without deleting platforms,
playlists, games, or media. Category rename remains gated because the 13.27
`IPlatformCategory.Name` contract is likewise getter-only. Playlist CRUD and
playlist nodes now implement the corresponding `LIB-008` and `DESK-003`
subset: stable-ID nested navigation, manual membership/order, recovered
automatic filter grouping, mutable metadata and BigBox fields, multiple
placements, paired playlist/parent transactions, optional list-cache cleanup,
and detach-to-root deletion all pass real-dialog and storage gates. Unique-name
and ID rename remain gated by the getter-only 13.27 contract. The `BB-002`
subset now provides keyboard-first category/platform/playlist navigation,
exact membership filtering, active-filter state, and `HideInBigBox` handling.
The persisted-status `LIB-014` subset is implemented in both frontends. One
shared typed query composes search and navigation with 13 explicit game-state
predicates, independent hidden/broken inclusion, and every one of the 12
LaunchBox missing-media flags. LaunchBox exposes inline controls; BigBox uses
a focus-navigable full-screen drawer. Unknown filter keys are rejected
atomically, default loading excludes hidden and broken games, and filtering is
read-only.
The filesystem-safe `RUN-001` configuration subset is implemented: a LaunchBox
manager creates, edits, and deletes emulator definitions and source-indexed
per-platform mappings through a typed, lossless `Emulators.xml` transaction.
It covers all 31 recovered emulator fields and all six mapping fields,
generates immutable UUID identities, validates mapping/default uniqueness,
hands off a platform default atomically, blocks deletion while a game or
additional application references the emulator, preserves unknown XML and
stored Windows path syntax, and never installs, updates, or deletes emulator
binaries through the ordinary definition editor. The first `RUN-003` subset
discovers reviewed RetroArch, Dolphin,
PCSX2, ScummVM, and Xemu executables from the portable emulator tree, native
application locations, and `PATH`, shows provenance and registration state,
and routes a selected candidate through the full editor with recovered defaults
and cross-platform reverse path conversion. It never starts or modifies a
candidate. A separate managed PCSX2 provider now selects and verifies official
GitHub artifacts, installs or repairs every portable artifact path, marker,
complete ownership manifest, and XML registration in one recoverable
transaction, updates only audited managed targets, routes the Linux AppImage
through packaged `appimage-run`, and removes only exact owned files after an
offline ownership/reference review while preserving settings and directories.
The macOS provider path safely expands the official bounded tar.xz, normalizes
its versioned app root to `PCSX2.app`, and preserves the main executable mode;
native macOS runtime verification remains open. BigPEmu adds exact official
Windows/Linux ZIP or tar.gz selection, published FNV verification, safe
extraction, and helper-free native registration. Xemu adds exact versioned
Windows/Linux/signed-universal-macOS artifact selection, mandatory GitHub
SHA-256, safe ZIP or direct AppImage preparation, transactional
install/update/repair/removal, and user configuration/BIOS retention. Other
providers, dependencies, cores, and automatic policy remain open. RetroArch
adds its recovered stable-buildbot lifecycle: exact Windows/Linux frontend and
cores 7z pairs or the universal macOS Metal DMG, exact URL/byte-count checks,
clearly labeled local SHA-256 receipts because no upstream digest is
published, safe wrapper normalization, and transactional ownership of the
complete app plus six exact macOS framework links. Core selection and netplay
continue under `RUN-016`; its first subset now inventories installed
native Windows/Linux/macOS cores and applies the frozen 13.27 platform
suggestions through the transactional Qt mapping editor. Individual-core
lifecycle, BigBox selection, and netplay remain open. The first
three `RUN-004` adapters
validate configured PCSX2 against the complete recovered 73-alternative BIOS
group, Xemu against its recovered required boot-ROM/HDD/flash-BIOS groups, and
RetroArch against every applicable row in its mechanically extracted
630-row/103-core 13.27 catalog. They read portable or host-native
configuration, stream MD5 checks, refuse firmware symlinks, and report all
groups through one Qt manager without executing an emulator or changing any
file. Xemu configuration values from a Windows installation use explicit
drive/UNC mappings. RetroArch derives cores from persisted platform mappings,
preserves per-file requirements and `None`/`Any`/`All` group rules, resolves
portable/native/home/mapped-Windows `system_directory` values, and reports
dynamic per-content roots instead of guessing. Other-emulator BIOS adapters,
acquisition, configuration changes, and mutation remain open.
The first `RUN-009` and `RUN-011` subsets resolve immutable startup and shutdown
policy from the actual primary target. A game override wins over the effective
emulator default; non-emulator targets use game settings. LaunchBox and BigBox
load separate global enable, theme, cursor, and millisecond minimum settings.
Both frontends prove the true pre-process delay, startup and shutdown
minimums, rendered overlays, child supervision, exact arguments, and
statistics writes with real processes; a disabled-global path proves complete
presentation and delay bypass. The first `RUN-010` subset independently
resolves pause policy with the same override/emulator/direct precedence,
combines it with each frontend's global enable/theme, and supervises explicit
pause/resume commands without a command shell. Every primary starts in an
isolated Unix process group or Windows Job Object; pause/resume and completion
cover descendants that remain in that launch session. Both shells render the
shared pause overlay and expose a local Ctrl+P/button action.
Theme/media asset selection, global input/focus behavior, scripts, audio/fade
behavior, and deliberately session-escaped processes remain open.
The first `IMP-001` vertical is also implemented: the LaunchBox shell accepts
multiple files or folders, recursively discovers candidates, applies an
optional extension filter, derives editable file- or folder-based titles,
detects already referenced sources and portable destination collisions, and
previews leave/copy/move policies before one revalidated batch commit. Copied
files stream through the same durable transaction as the platform XML; move
deletes each source only after that commit and a byte-revision match. The
emulator page preserves LaunchBox's distinct platform-default, direct-launch,
and explicitly configured-emulator states and revalidates configured IDs on
preview and execution. Complete unambiguous `(Disc N)` and `(Disc N of M)`
filename sets can collapse to one game with every disc represented as an
ordered additional application, including the primary Disc 1 record, matching
the older real installation. The database-driven matching-title option now
groups unique primary/alternate-title metadata matches by stable database ID,
falls back to exact cleaned titles when metadata is absent, retains ambiguous
rows for review, and persists every version—including the primary ROM—as an
ordered `Play {version} Version...` additional application. Filename qualifier
parsing recovers `Version` and normalized `Region` through separator-neutral
lexical handling. Copy/move also implements the recovered
same-filename/different-extension option as an atomic companion-file bundle,
including per-disc companions and collision refusal. Metadata and media
downloads, remaining managed emulator providers and BIOS handling,
descriptor-content dependency copying, broader naming grammars, cancellation,
and runtime-oracle edge-case
parity remain open, so `IMP-001` is not complete.
The implemented local-database subset opens LaunchBox's SQLite file read-only,
canonicalizes platform aliases, applies recovered exact-title and qualifier
rules, falls back only when necessary to recovered substring/all-word matching
with bare-numbered-suffix suppression, exposes compact stable-ID review choices
for ambiguous exact or partial matches, revalidates selections, persists 13
typed metadata fields, and can copy/move the complete file bundle into a
portable final-title/year subfolder.
Playlist generation (`LIB-009`) and theme-specific playlist views remain open.
Persisted Windows-style paths remain lexical until the cross-platform launch
service resolves them. None of those complete their family: bulk editing, all
filter/sort modes, reset commands, rating downloads, and full Windows behavior
parity remain open.

## Library, data, and organization

| ID | Feature family | Static evidence |
|---|---|---|
| LIB-001 | Create, edit, delete, and launch games | DV add/edit game pages; PA `IGame`; DM play/edit/delete |
| LIB-002 | Rich game metadata, notes, sort titles, regions, versions, and status | DV metadata/notes/sort-title pages; PA game/additional-app fields |
| LIB-003 | Alternate names and custom fields | DV alternate-name/custom-field pages; PA `IAlternateName`, `ICustomField` |
| LIB-004 | Additional applications and documents per game | DV additional-app editor and Make Default command; PA `IAdditionalApplication`; BigBox selection actions; port subset has typed/lossless add/edit/delete, launch selection, and an evidence-derived transactional Make Default conversion that retains the selected row |
| LIB-005 | Game controller support metadata | DV controller-support editor; PA controller APIs |
| LIB-006 | Platforms and platform folders | DV platform management; PA `IPlatform`, `IPlatformFolder` |
| LIB-007 | Nested platform categories | DV category editor; PA `IPlatformCategory`, `IParent` |
| LIB-008 | Manual and auto-populated playlists | DV playlist editor; PA manual/automatic playlist and criteria contracts |
| LIB-009 | Child-playlist generation and arcade playlist creation | DV generator; DM generate child/create arcade playlists |
| LIB-010 | Combined games, versions, and expand/collapse | DM combine, show versions, expand games; port importer combines recovered matching-title/version groups; the manual port subset converts same-platform games to selectable version applications, atomically migrates modeled XML references, and expands launchable versions back to standalone games without moving ROM/media files; collapse and remaining presentation parity stay open |
| LIB-011 | Favorites, completed/broken/hidden/installed state | PA game state; search/filter menus; BigBox state resources |
| LIB-012 | Play count, last played, completion, local/community star ratings | PA game/additional-app state; DM reset counters/download ratings; BV rating popup |
| LIB-013 | Search, sort, arrange, filters, random selection, and suggestions | DM arrange/search/random; DV search/suggestion/filter options; the port implements descriptive search plus 17 typed LaunchBox-keyed Arrange By modes with ascending/descending order, missing-last semantics, stable title/ID ties, transactional `Settings.xml` persistence, selection retention across resets, and random visible-game selection that avoids the current game when possible. LaunchBox and BigBox expose shared controller state; suggestions, dynamic/custom-field arrangement, grouped headings, Random-as-an-arrangement, and random play remain open |
| LIB-014 | Missing-media and state filters | DM missing image/video filters, hidden/broken games; the port composes search and platform/category/playlist navigation with favorite, completion, installation tri-state, played, rated, hidden, and broken predicates plus independent hidden/broken visibility and every persisted missing background/banner/3D box/front box/3D cart/cart/clear-logo/manual/marquee/music/screenshot/video flag. LaunchBox has inline controls and BigBox has a focus-navigable drawer; typed stable keys reject unknown input without changing the active filter, default loading excludes hidden/broken records, editor state changes and first-play statistics recompute membership, and both-shell offscreen coverage proves combined predicates and byte-identical XML |
| LIB-015 | Bulk edit and audit | DV bulk-edit wizard and audit view |
| LIB-016 | ROM folder change, consolidation, copying, and media migration | DM path change/consolidate/copy; PA title/media migration |
| LIB-017 | XML-compatible library documents and SQLite local database/cache | PA data manager/platform documents; `LocalDb` EF Core SQLite assembly |
| LIB-018 | Backup and restore application data/settings | DV backup options; DM backup/restore |

## Import, export, and discovery

| ID | Feature family | Static evidence |
|---|---|---|
| IMP-001 | ROM file import wizard | DV ROM location/game/options/metadata pages |
| IMP-002 | Bulk ROM folder import | DV bulk import location pages |
| IMP-003 | Scan for added and removed ROMs by platform or globally | DM scan-added/scan-removed actions |
| IMP-004 | Automated import folders and media rules | DV automated-import option pages; SI install/uninstall monitoring |
| IMP-005 | Drag-and-drop import | DV drag/drop mode, selection, and options pages |
| IMP-006 | Windows installed-game import | DV Windows import pages |
| IMP-007 | DOS game import and installation | DV DOS import pages; DM install DOS game; DOSBox integration |
| IMP-008 | MAME full-set import/install/filter/high-score workflow | DV eleven-page MAME wizard; MAME adapter/integration |
| IMP-009 | Steam import | DV Steam account/API/custom URL/game/media pages; SI Steam integration |
| IMP-010 | GOG import | DV GOG credentials/profile/game/media pages; SI GOG integration |
| IMP-011 | Epic Games import | DV Epic parse/game/media pages; SI Epic integration |
| IMP-012 | EA app import | DV EA account/game/media pages; SI EA integration |
| IMP-013 | Ubisoft/Uplay import | DV Ubisoft account/game/media pages; SI Uplay integration |
| IMP-014 | Amazon Games import | DV Amazon account/game/media pages; SI Amazon integration |
| IMP-015 | Xbox/Microsoft library import | DV Xbox account/game/media pages; SI Xbox integration |
| IMP-016 | Image pack import and export | DV image-pack import/create/preview/copy workflow |
| IMP-017 | Export selected platforms, games, and media to LaunchBox for Android | DV eleven-page Android export wizard |
| IMP-018 | Welcome/first-run guided imports | DV welcome/home wizards for storefronts, locations, metadata, and media |

## Metadata and media

| ID | Feature family | Static evidence |
|---|---|---|
| MED-001 | LaunchBox Games Database metadata matching/download | welcome GamesDB and metadata-source views; installed platform metadata |
| MED-002 | EmuMovies authentication and downloads | DV EmuMovies import/options pages |
| MED-003 | Metadata/media update for existing games | DV download-metadata workflow; DM download metadata/media |
| MED-004 | Per-game images and image-type selection | DV images page/image downloader; PA `ImageTypes` catalogue; the port's read-only index retains every configured image family using persisted front-image/region priorities and renders native file URLs in both shells plus a selectable multi-image LaunchBox details gallery. Bounded/symlink-safe scanning and real offscreen SVG decode/render gates cover front, gameplay screenshot, and background fixtures. Image editing, downloads, full-screen viewing, and media mutation stay open |
| MED-005 | Game and platform videos/theme videos | DV videos page/platform-video downloader; video options; the port indexes per-game Video Snap, Theme Video, Trailer, Recording, and Marquee folders plus explicit `VideoPath`/`ThemeVideoPath`, honors recovered details visibility/autoplay/type priorities, and previews the selection through Qt Multimedia. Platform-video download/management and media mutation stay open |
| MED-006 | Manuals, music, and additional media playback | DM view manual/play music; media viewer controls |
| MED-007 | Media manager, cleanup, missing-media refresh, and duplicate-image grouping | DV media manager/cleanup; DM refresh actions; SI image dupe grouper |
| MED-008 | Regional and media-type priorities | DV region and priority option pages; the read-only game-media index applies persisted region, front-image-type, and video-type priority lists with typed bounded fallbacks |
| MED-009 | 3D boxes, DVD/jewel cases, and full-screen model previews | DV model settings/previews; shared cover-flow model controls |
| MED-010 | Screenshots and screen capture | DV screen-capture options; image viewer |
| MED-011 | Background, clear logo, banner, cartridge, disc, arcade, storefront, and screenshot image families | PA exhaustive `ImageTypes` fields |
| MED-012 | Video/audio playback and thumbnails | WPF video controls; bundled VLC, FFmpeg, and Chromium components; the LaunchBox details pane now has real image/video thumbnail selection, Qt Multimedia H.264 decode, autoplay, play/pause, mute, and native-URL preview coverage. Broad codec/backend parity, audio-only playback, and BigBox media views stay open |

## Emulator and launch orchestration

| ID | Feature family | Static evidence |
|---|---|---|
| RUN-001 | Add/edit/remove emulators and per-platform mappings | DV emulator editor/platform page; PA `IEmulator`, `IEmulatorPlatform`; port configuration subset implements all 31 recovered emulator fields and all six mapping fields with immutable generated IDs, source-indexed lossless XML edits, default handoff, game/additional-app reference gating, exact backups, unknown-field retention, lexical paths, and real-dialog Linux coverage; emulator binary lifecycle stays under `RUN-003` |
| RUN-002 | Command lines, quoting/spacing, console hiding, and launch-with overrides | PA emulator fields; DM/BV launch-with actions |
| RUN-003 | Emulator/core discovery, install, update, removal, and dependency handling | DV install-emulator/dependencies pages; PA emulator plugin install/update contracts; port discovery implements read-only, deterministic reviewed registration for native BigPEmu, RetroArch, Dolphin, PCSX2, ScummVM, and Xemu identities across a bounded portable tree, OS-native locations, and `PATH`, with provenance, canonical deduplication, Unix collision isolation, shared reverse path conversion, recovered 13.27 defaults, and real-dialog candidate-immutability coverage. Four managed providers now share exact portable ownership, streamed/cancellable verification, unmanaged-target refusal, install/update/repair classification, stale-owned-path cleanup, lossless `Emulators.xml` registration, reference-gated exact removal, recovery copies, and unrelated-user-file retention. PCSX2 selects exact official Linux AppImage, Windows Qt 7z, or macOS Qt tar.xz artifacts using GitHub byte count/SHA-256; it safely normalizes the complete macOS bundle to stable `PCSX2.app`, and Linux launches through packaged shell-free `appimage-run`. BigPEmu selects the exact four official Windows/Linux architecture artifacts, verifies published byte count and uppercase FNV-1a plus local SHA-256, safely extracts ZIP or bounded tar.gz, preserves native mode, and excludes and never invokes `make_desktop.sh`. Xemu selects the exact five versioned stable Windows x64/ARM64 ZIP, Linux x64/ARM64 AppImage, or signed universal macOS ZIP artifacts; rejects debug, unsigned, moving-alias, duplicate, missing-digest, and untrusted metadata; requires GitHub byte count/SHA-256; validates root Windows files or the exact macOS app hierarchy; preserves executable/bundle mode; and retains user configuration and BIOS data. RetroArch selects the exact stable Windows/Linux frontend-and-cores 7z pair or universal macOS Metal DMG, allowlists URLs and published byte counts, explicitly records only a locally computed SHA-256 because the buildbot publishes no digest, strips exact wrapper roots, and transactionally preserves the signed app layout, modes, and six exact framework symlinks. Official-shaped controller/archive tests prove all four ownership lifecycles without executing downloaded artifacts or invoking a shell. Further managed emulator providers, dependency policy, user-selected core management, netplay, automatic policy, and native Windows/macOS runtime execution remain open |
| RUN-004 | BIOS discovery and validation | DV BIOS wizard; PA BIOS groups/files; the first three port adapters implement the complete recovered PCSX2 required group with 73 filename/MD5/description alternatives, Xemu's three required boot-ROM/HDD/flash-BIOS groups with seven exact entries, and RetroArch's complete 630-row/103-core resource selected by configured platform/core mappings. All provide portable/native configuration resolution, streamed hashing, symlink refusal, and one generalized versioned read-only Qt manager. Xemu additionally bounds TOML input, resolves legacy Windows drive/UNC values through explicit mappings, accepts its digest-free HDD by safe presence, and never performs the original plugin's download/configuration mutation. RetroArch preserves platform filters, optional hashes, per-file requirements, group IDs/descriptions/required flags, and `None`/`Any`/`All` rules; bounds `retroarch.cfg`; checks official host-native candidates; resolves portable, home-relative, native, and explicitly mapped Windows `system_directory` values; performs safe case-insensitive nested lookup; and reports the content-dependent `default` value rather than inventing one BIOS root. PCSX2 has whole-tree-immutable real-dialog coverage; Xemu and RetroArch have catalog, mapping, fallback/path, ambiguity/symlink, immutability, controller-payload, and registration tests through the shared compiled dialog schema. Other-emulator adapters, acquisition, configuration changes, and firmware mutation remain open |
| RUN-005 | Archive auto-extraction and cleanup | PA auto-extract fields; bundled 7-Zip |
| RUN-006 | Multi-disc/M3U handling | PA emulator-platform `M3uDiscLoadEnabled` |
| RUN-007 | DOSBox folder/image drive mappings | DV mounts page; PA `IMount`; recovered 13.27 `MOUNT`/`IMGMOUNT` vocabulary and flags; native Linux synthetic runtime fixture implemented |
| RUN-008 | AutoHotkey launch, pause, resume, reset, save/load state, swap-disc, and exit scripts | emulator editor; PA script fields; bundled AutoHotkey |
| RUN-009 | Startup screens and per-emulator/game overrides | DV startup settings/edit pages; shared startup views; the port resolves immutable startup/shutdown policy with recovered game-override/emulator-default precedence and direct-game fallback. LaunchBox and BigBox independently load their global enable, theme name, cursor behavior, and minimum display durations. One shared startup overlay is presented before the inherited `StartupLoadDelay`; the existing worker then spawns the primary in an isolated Unix process group or Windows Job Object, and Qt dismisses only after both primary start and the frontend minimum. Both-shell process smokes use one portable Rust fixture to measure the pre-launch and minimum intervals, capture valid rendered PNGs, enforce dismissal before normal session exit, compare exact argv, and verify session persistence; a 50 ms primary proves minimum-preserving shutdown handoff, and a disabled-global smoke proves presentation and delay bypass. Supervision and shutdown presentation wait for in-session descendants. Theme/media asset selection, window/focus hiding, and processes that deliberately escape the supervised session remain open |
| RUN-010 | Pause screens and per-emulator/game overrides | DV pause settings/edit pages; shared pause views; the first port subset resolves immutable `UsePauseScreen`, `SuspendProcessOnPause`, and forceful-activation provenance with game override, effective-emulator default, and direct-game precedence. LaunchBox and BigBox independently load global enable and theme from their settings, offer one shared modal overlay plus local Ctrl+P/button/resume actions, and send typed commands to the supervising worker. Unix uses `SIGSTOP`/`SIGCONT` on the isolated process group; Windows starts the primary suspended, assigns it to a private Job Object, resumes it, then enumerates all job processes and balances one suspend count per successfully controlled thread with partial-failure rollback. Sender loss resumes a paused session before supervision continues. Five portable-fixture integration tests cover direct and delegated completion/pause plus ordering, timeout, and exact arguments; both-shell offscreen processes delegate to a child and prove emulator inheritance, one suspension/resumption, one delegated-session completion, rendered PNGs, exact argv, and session persistence. Archive and M3U smokes prove delegated descendants retain temporary resources through their delayed reads. The strengthened Windows core gate compiles all fixture/core targets. AutoHotkey pause/resume scripts, theme/media assets, global/controller input while another application owns focus, forceful cross-window activation, mute/fade behavior, deliberately session-escaped processes, and pause-excluded play time remain open |
| RUN-011 | Window hiding, mouse hiding, startup delay, and shutdown-screen policy | PA emulator launch fields; first port subset applies `StartupLoadDelay` before process spawn, honors each frontend's global startup/shutdown minimum and startup-screen cursor setting, resolves effective `DisableShutdownScreen`, and renders a shared post-session shutdown overlay. Both frontends have measured real-process and PNG evidence; window hiding/focus restoration and processes that explicitly escape the supervised session remain open |
| RUN-012 | Game save management, version history, backup, restore, and deletion | DV saves/history; desktop save commands; PA save contracts; port subset models every persisted 13.27 `GameSave` field, provides a lossless Qt group/version inventory with transactional rename/combine/split metadata operations, and uses native RetroArch, Dolphin, and PCSX2 adapters to discover and append active rows without deleting history; RetroArch covers regular saves, numbered/auto states, and grouped Saturn companion sets; Dolphin derives raw/WAD/compressed disc IDs and discovers portable/native GameCube folder/Card A/Card B files, exact two-digit states, and disc/WAD Wii title directories with stable recovered group IDs; PCSX2 extracts an ISO9660 `SYSTEM.CNF` serial natively from plain/raw-sector ISO, GZip, CSO, CHD v5, MDF/MDS, and NRG content before discovering exact ordinary state rows plus folder-format and raw-card members with recovered serial/GameIndex/title/`icon.sys` matching and explicit container-member metadata; backup covers regular files, complete Saturn sets, PCSX2 logical card members, and complete Dolphin Wii directories; Wii trees are archived with direct process arguments, safely re-extracted, compared by exact recursive revision, and committed with XML, while PCSX2 members are extracted to a flat verified 7z, recorded with the recovered SHA-256 folder manifest, rechecked against the live card, and committed without modifying it; restore requires a compatible active row and first commits/verifies the current version; vault deletion removes one resolved regular file or complete Saturn set plus its history row with exact recovery copies; Saturn restore revision-checks and atomically replaces/creates selected companions while retaining active members absent from the selected version like 13.27; evidence-backed Dolphin state/GameCube and PCSX2 state files share the backup-first ordinary-file restore and active-delete path; Dolphin Wii restore/deletion use exact whole-tree revisions, rollback-capable replacement or rename-based deletion, and retained complete recovery trees; PCSX2 folder/raw card-member restore and deletion commit and verify a logical-member archive, mutate and validate a complete working card, then use an exact whole-card revision and retained recovery copy at a rollback-capable directory or atomic file replacement boundary; physical raw restore regenerates spare/ECC bytes and deliberately rejects logical-page cards like recovered 13.27; after a raw working-copy import reports no space, the recovered 13.27 repair trigger walks root-reachable directories and FAT chains, frees only allocated unreachable clusters, durably rewrites the private copy, retries even when zero clusters were freed, and reports any reclaimed count through Qt before the validated whole-card swap; active deletion first commits a verified portable vault version and detaches the active history row before touching live data, with external regular files using all-or-rollback sibling copies, Wii saves using complete-tree recovery, and PCSX2 using the complete-card swap; other-emulator scanning, other directory/container backup, stale-row reconciliation, automatic policy, general repair commands, and remaining adapters remain open |
| RUN-013 | Additional apps before/after or alternate launches | DV additional-app editor; PA launch/effective command line |
| RUN-014 | Controller configuration actions exposed by emulator adapters | PA controller action/option/version contracts |
| RUN-015 | RetroAchievements credential injection and hardcore mode | emulator RetroAchievements page; PA credential/support contracts |
| RUN-016 | RetroArch core selection, install, and netplay | RetroArch adapter; DV/BV netplay; install RetroArch view. The `RUN-003` lifecycle installs the exact matching stable cores archive with Windows/Linux RetroArch and records it separately; macOS has no corresponding stable cores archive and deliberately does not invent one. The first selection subset freezes all 56 LaunchBox 13.27 platform/core suggestions (54 marked recommended), reads bounded application-local and official host-native `retroarch.cfg` candidates, resolves portable/native/home/mapped-Windows `libretro_directory` values, and safely inventories only native `.dll`, `.so`, or `.dylib` regular core files from portable, AppImage-home, macOS bundle/Application Support, and host configuration locations. The Qt emulator mapping editor exposes installed choices, installed 13.27 suggestions, missing/custom state, and read-only provenance; choosing one replaces only one semantic `-L`/`--libretro` argument and retains unrelated flags before the existing transactional `Emulators.xml` save. Symlinked configuration/directories are refused; unsafe core entries and case-insensitive duplicate names cannot become choices. Individual core download/update/removal, online updater policy, BigBox selection, and netplay remain open |
| RUN-017 | Emulator-specific adapters | separate BigPEmu, Dolphin, MAME, PCSX2, RetroArch, ScummVM, and Xemu assemblies; the platform-neutral BigPEmu adapter implements reviewed Windows/Linux identity, bounded readme version inspection, exact Jaguar/Jaguar CD registration defaults, native shell-free `%romfile% -localdata` launch planning, official Windows x64/ARM64 and Linux x64/ARM64 release selection, FNV/SHA verification, safe ZIP/tar.gz extraction, helper-free transactional install/update/repair/removal, and exact portable ownership; legacy built-in ScummVM host-path/argument adapter implemented with a native Linux runtime fixture; platform-neutral `lb-integrations` adapters now implement RetroArch config-driven save/state discovery, Windows/Linux/macOS core parsing, mapped paths, Saturn companion grouping/signatures/backup/restore/deletion, and complete configured-core BIOS auditing from the 630-row 13.27 resource; Dolphin raw ISO/GCM and WAD disc IDs, sibling DolphinTool compressed-image IDs, native/portable user roots, region-aware GameCube folder/card discovery, exact state slots, recovered Wii disc/WAD title-directory discovery, stable group IDs, ordinary-file operations, verified nested Wii archives, and whole-directory restore/deletion with complete recovery trees; PCSX2 portable/native roots, native bounded ISO9660 `SYSTEM.CNF` extraction through plain/raw-sector ISO, GZip, CSO, CHD v5, MDF/MDS, and NRG readers, exact state parsing and serial ownership, GameIndex/title/`icon.sys` folder/raw-card member discovery, logical and 528-byte physical-page raw-card parsing, indirect/direct FAT traversal, logical member extraction and 13.27 folder manifests, verified member backup, folder/raw complete-card restore/deletion working copies, raw FAT/directory/file writes with ECC regeneration, no-space-triggered orphan-FAT capacity recovery on private raw working copies, controller-level revision conflicts and complete-card recovery copies, Qt-visible reclaimed-cluster reporting, and ordinary-state operations; and Xemu recovered Xbox registration, complete read-only portable/native BIOS-group validation with explicit cross-host path mapping, exact official five-artifact release selection, mandatory SHA-256 verification, safe ZIP/direct-AppImage preparation, and transactional install/update/repair/removal with exact portable ownership. Authentic `chdman`-verified synthetic DVD/CD CHDs and the offscreen compressed-CHD scan prove filesystem serial ownership without filename/title hints or an external reader |

## Online, automation, and extension integrations

| ID | Feature family | Static evidence |
|---|---|---|
| INT-001 | RetroAchievements login, achievements, profile, progress, recent activity, and leaderboards | DV/BV achievement views; SI/PA RetroAchievements APIs |
| INT-002 | MAME high scores, player profiles, versions, and leaderboards | DV/BV MAME views; SI MAME high-score integration |
| INT-003 | Steam achievements and stats | SI Steam achievement models/client |
| INT-004 | GOG achievements | SI GOG achievement models/client |
| INT-005 | Bezel Project integration | DV bezel wizard; SI Bezel Project client/models |
| INT-006 | OBS Studio integration | DV OBS options; SI `ObsStudio` |
| INT-007 | LEDBlinky integration | DV LEDBlinky options |
| INT-008 | Cloud connect, browse, sync, and disconnect | DV cloud connect; DM cloud actions |
| INT-009 | Plugin discovery, management, updates, game/system menus, launch hooks, events, badges, and theme elements | DV plugin manager; DM update scan; PA plugin interfaces |
| INT-010 | Playlist-provider plugins | PA provider contracts; separate playlist-provider assembly |
| INT-011 | Web content and embedded browser views | desktop/BigBox web views; bundled Chromium/CefSharp |

## LaunchBox desktop experience

| ID | Feature family | Static evidence |
|---|---|---|
| DESK-001 | Box/grid and list library views | PA main-view modes; DV content view models |
| DESK-002 | Resizable sidebar, game details, controls bar, and pop-out details | PA dimensions/view state; DM toggles/pop-out; the LaunchBox details pane is resizable, hideable, dockable/pop-out, platform-state persisted, and includes the recovered selectable image/video media preview contract |
| DESK-003 | Platform/category/playlist sidebar management | DM sidebar add/edit/delete; PA root hierarchy |
| DESK-004 | Image zoom, type switching, box flip, full-screen images/models | DM zoom/image/model/flip actions |
| DESK-005 | Backgrounds, colors, fonts, spacing, and dialog theme | DV visual option pages and compiled styles |
| DESK-006 | Video/music autoplay and shuffle | DM autoplay/shuffle actions; video/audio controls |
| DESK-007 | Keyboard/game-controller mappings and controller database | DV mapping/controller options and management |
| DESK-008 | Notifications and system tray behavior | DV notifications/tray options and notification views |
| DESK-009 | Search focus, random play, and random selection | desktop commands/menu actions |
| DESK-010 | Automatic updater, changelog, beta center, and local DB update | updater/beta views; DM update actions |
| DESK-011 | Localization | 18 LaunchBox/Unbroken satellite resource cultures in runtime dependencies |
| DESK-012 | Premium registration and BigBox mode entry | DM license/premium/BigBox actions; entitlement state in PA |

## BigBox full-screen experience

| ID | Feature family | Static evidence |
|---|---|---|
| BB-001 | Controller/keyboard-first full-screen shell | BV main/options/input models |
| BB-002 | Platform/category/playlist filter navigation | BV filter and platform wheel/hybrid view models |
| BB-003 | Text, box, thumbnail, wall, wheel, horizontal-wheel, and cover-flow game views | BV/TX view families |
| BB-004 | Game details, images, video, 3D model, related games, and additional documents/apps | BV details/media/popups/actions |
| BB-005 | Themed menus, popups, option controls, and bindable theme elements | TX view set; PA BigBox theme plugin API |
| BB-006 | Theme manager, screenshots, details, download/update, demo notifications, and per-theme settings | BV theme manager/download/update views and option pages |
| BB-007 | Startup video and startup/shutdown screens | BV startup video; shared startup view |
| BB-008 | Pause screen | BigBox pause view model; shared pause view; first shared rendered/process-control subset is implemented under `RUN-010` |
| BB-009 | Four screensaver modes | BV `Screensaver1` through `Screensaver4` and options |
| BB-010 | Attract mode with wheel spinning/view switching | BigBox `AttractMode`; attract option page |
| BB-011 | PIN/security lock and premium gating | BV PIN popup/security options; PA lock/premium state |
| BB-012 | Gamepad, keyboard, keyboard-automation, and mouse options | BV option pages and binding controls |
| BB-013 | Secondary marquee display for games/platforms | BV marquee window/models/options; TX marquee views |
| BB-014 | Images, videos, sound, music notifications, transitions, and image cache | BV option pages/media models |
| BB-015 | RetroAchievements and MAME leaderboards in full-screen UI | BV achievement/MAME popup models |
| BB-016 | RetroArch netplay and launch/core selection menus | BV netplay and selection actions |
| BB-017 | Favorites, star ratings, playlists, random/discovery, and related games | BV popups/menu actions/discovery model |
| BB-018 | Desktop-mode handoff and application exit | BV desktop/exit menu actions |

## Platform and operational behavior

| ID | Feature family | Static evidence |
|---|---|---|
| OPS-001 | Portable install-relative paths and external media folders | installed layout; PA platform documents/path fields |
| OPS-002 | File/process monitoring for game installs/uninstalls | shared monitoring types |
| OPS-003 | Low-level keyboard and mouse hooks | shared low-level-hook namespaces |
| OPS-004 | Native process/window/controller/audio/video services | shared Native/Processes/GameControllers/Audio namespaces |
| OPS-005 | Update, backup, notification, and error-handling lifecycle | desktop options/actions and shared services |
| OPS-006 | Windows desktop integration | WPF, registry, shell APIs, tray, toast, service/process dependencies |
| OPS-007 | Linux equivalents | not present upstream; must be implemented and verified as a port requirement |

## What remains undiscovered

This matrix is broad but not yet a closed census. Before calling it exhaustive,
runtime work must identify:

- every desktop and BigBox menu item under free and premium entitlements;
- settings defaults, dependencies, and platform/theme overrides;
- exact importer matching, merge, duplicate, retry, cancellation, and error rules;
- all data schemas, migrations, caches, backup semantics, and concurrency rules;
- command-line quoting, process lifecycle, archive, mount, and cleanup behavior;
- network protocols, rate limits, account states, and offline behavior;
- theme binding semantics and plugin lifecycle/compatibility behavior;
- hidden/beta/deprecated features and version-gated migrations.

Rows are split or added as oracle scenarios reveal independent behaviors. No
row is marked verified merely because a similarly named class exists.
