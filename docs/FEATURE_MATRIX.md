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
subset of `LIB-013`. A source-indexed transactional editor also implements the
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
the older real installation. Copy/move also implements the recovered
same-filename/different-extension option as an atomic companion-file bundle,
including per-disc companions and collision refusal. Metadata and media
downloads, database-driven version combining, emulator installation/BIOS
handling, descriptor-content dependency copying, cancellation, and
runtime-oracle edge-case parity remain open, so `IMP-001` is not complete.
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
| LIB-004 | Additional applications and documents per game | DV additional-app editor; PA `IAdditionalApplication`; BigBox selection actions |
| LIB-005 | Game controller support metadata | DV controller-support editor; PA controller APIs |
| LIB-006 | Platforms and platform folders | DV platform management; PA `IPlatform`, `IPlatformFolder` |
| LIB-007 | Nested platform categories | DV category editor; PA `IPlatformCategory`, `IParent` |
| LIB-008 | Manual and auto-populated playlists | DV playlist editor; PA manual/automatic playlist and criteria contracts |
| LIB-009 | Child-playlist generation and arcade playlist creation | DV generator; DM generate child/create arcade playlists |
| LIB-010 | Combined games, versions, and expand/collapse | DM combine, show versions, expand games |
| LIB-011 | Favorites, completed/broken/hidden/installed state | PA game state; search/filter menus; BigBox state resources |
| LIB-012 | Play count, last played, completion, local/community star ratings | PA game/additional-app state; DM reset counters/download ratings; BV rating popup |
| LIB-013 | Search, sort, arrange, filters, random selection, and suggestions | DM arrange/search/random; DV search/suggestion/filter options |
| LIB-014 | Missing-media and state filters | DM missing image/video filters, hidden/broken games |
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
| MED-004 | Per-game images and image-type selection | DV images page/image downloader; PA `ImageTypes` catalogue |
| MED-005 | Game and platform videos/theme videos | DV videos page/platform-video downloader; video options |
| MED-006 | Manuals, music, and additional media playback | DM view manual/play music; media viewer controls |
| MED-007 | Media manager, cleanup, missing-media refresh, and duplicate-image grouping | DV media manager/cleanup; DM refresh actions; SI image dupe grouper |
| MED-008 | Regional and media-type priorities | DV region and priority option pages |
| MED-009 | 3D boxes, DVD/jewel cases, and full-screen model previews | DV model settings/previews; shared cover-flow model controls |
| MED-010 | Screenshots and screen capture | DV screen-capture options; image viewer |
| MED-011 | Background, clear logo, banner, cartridge, disc, arcade, storefront, and screenshot image families | PA exhaustive `ImageTypes` fields |
| MED-012 | Video/audio playback and thumbnails | WPF video controls; bundled VLC, FFmpeg, and Chromium components |

## Emulator and launch orchestration

| ID | Feature family | Static evidence |
|---|---|---|
| RUN-001 | Add/edit/remove emulators and per-platform mappings | DV emulator editor/platform page; PA `IEmulator`, `IEmulatorPlatform` |
| RUN-002 | Command lines, quoting/spacing, console hiding, and launch-with overrides | PA emulator fields; DM/BV launch-with actions |
| RUN-003 | Emulator/core discovery, install, update, and dependency handling | DV install-emulator/dependencies pages; PA emulator plugin install/update contracts |
| RUN-004 | BIOS discovery and validation | DV BIOS wizard; PA BIOS groups/files |
| RUN-005 | Archive auto-extraction and cleanup | PA auto-extract fields; bundled 7-Zip |
| RUN-006 | Multi-disc/M3U handling | PA emulator-platform `M3uDiscLoadEnabled` |
| RUN-007 | DOSBox folder/image drive mappings | DV mounts page; PA `IMount`; recovered 13.27 `MOUNT`/`IMGMOUNT` vocabulary and flags; native Linux synthetic runtime fixture implemented |
| RUN-008 | AutoHotkey launch, pause, resume, reset, save/load state, swap-disc, and exit scripts | emulator editor; PA script fields; bundled AutoHotkey |
| RUN-009 | Startup screens and per-emulator/game overrides | DV startup settings/edit pages; shared startup views |
| RUN-010 | Pause screens and per-emulator/game overrides | DV pause settings/edit pages; shared pause views |
| RUN-011 | Window hiding, mouse hiding, startup delay, and shutdown-screen policy | PA emulator launch fields |
| RUN-012 | Game save management, version history, backup, restore, and deletion | DV saves/history; desktop save commands; PA save contracts |
| RUN-013 | Additional apps before/after or alternate launches | DV additional-app editor; PA launch/effective command line |
| RUN-014 | Controller configuration actions exposed by emulator adapters | PA controller action/option/version contracts |
| RUN-015 | RetroAchievements credential injection and hardcore mode | emulator RetroAchievements page; PA credential/support contracts |
| RUN-016 | RetroArch core selection, install, and netplay | RetroArch adapter; DV/BV netplay; install RetroArch view |
| RUN-017 | Emulator-specific adapters | separate BigPEmu, Dolphin, MAME, PCSX2, RetroArch, ScummVM, and Xemu assemblies; legacy built-in ScummVM host-path/argument adapter implemented with a native Linux runtime fixture |

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
| DESK-002 | Resizable sidebar, game details, controls bar, and pop-out details | PA dimensions/view state; DM toggles/pop-out |
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
| BB-008 | Pause screen | BigBox pause view model; shared pause view |
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
