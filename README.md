# LaunchBox / BigBox cross-platform port research

This repository is the evidence and planning workspace for a native,
cross-platform LaunchBox-compatible front end built with Rust, Qt 6, QML, and
[CXX-Qt](https://github.com/KDAB/cxx-qt).

The local LaunchBox 13.27 Windows oracle has been installed with Wine and all
first-party managed assemblies have been structurally decompiled. The original
installation and decompiled proprietary sources are intentionally ignored;
derived inventories and specifications live in this repository.

## Current artifacts

- [Reverse-engineering status](docs/RE_STATUS.md)
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
cargo build -p lb-shell && ./scripts/check_qml.sh
cargo run -p lb-shell --bin launchbox
cargo run -p lb-shell --bin bigbox -- --windowed
```

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

Library parsing runs on a Rust worker thread and returns through CXX-Qt's queued
Qt-thread bridge. QML consumes the controller as a real `QAbstractListModel`
with 37 named identity, state, descriptive-metadata, launch-configuration, and
additional-application, and game-save roles; it does not receive a whole-library
JSON snapshot. `check_qml.sh` validates generated type metadata, then runs both
binaries offscreen and proves all 36 roles survive a model-resetting filter
operation. It also edits a
temporary fixture library through the Qt shell and checks the targeted model
notification for a state-only edit, descriptive-metadata search refresh, exact
backup chain, optional-element removal, resulting XML, and unknown-field
preservation. The same real dialog source-indexes and edits repeated alternate
names and custom fields, retaining unknown XML on each kept row, and changes
the application path, command line, emulator selection, and DOSBox/ScummVM
settings in one transaction. A fresh Linux process then
reloads that edit and executes the stored Windows-separated relative path with
the exact expanded argument vector.

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
to native paths at launch. Descriptive metadata participates in search; metadata changes
recompute stable sort and search membership while state-only changes use
targeted role notifications.
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
Up, Tab, or F opens the category/platform/playlist list, Enter applies exact
membership, and Right returns to the horizontal game wheel. Stable playlist
IDs stay behind display names, the active filter is controller-owned state,
and nodes marked `HideInBigBox` are omitted while visible descendants are
reparented to the nearest visible level.
Interrupted transactions require an explicit Recover action; conflicts require
a reload and never offer a blind overwrite.

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
Rename Version, Rename Group, Combine, and Make New Save metadata operations,
plus manual backup for a resolved regular active-save file. Manual backup
derives the portable `Saves\<Platform>\<ROM name>[-NN].<ext>` target through
the host-path service, records the exact size, seven-digit UTC modified time,
and MD5, and commits the streamed vault copy with the full new `<GameSave>` row
under one recovery manifest. A legacy ungrouped source receives an explicit
group ID in that same transaction. The manager can also permanently remove one
resolved regular-file vault backup and its exact source-indexed history row in
one revision-checked transaction. The active file is excluded, both the vault
file and XML receive exact recovery copies, and RetroArch Saturn companion sets
are refused until their adapter is implemented. Restore requires one compatible
regular active row in the same stable group. It first commits the current active
bytes as a new vault version, verifies that copy, then revision-checks and
atomically replaces the active file from the selected vault version while
retaining a second exact sibling recovery copy. Dolphin, PCSX2, RetroArch Saturn,
directory, and ambiguous-active cases remain adapter-gated.
Stored paths remain lexical; only paths resolved by the host-path service are
classified Active or Vault, while unmapped Windows paths are shown as
Unresolved. Metadata operations never move or delete save files, and backup
never changes the active file. Save scanning, directory/container and companion
file backup, emulator-specific restore, active/container/companion-set deletion,
automatic-backup policy, repair, and emulator-specific adapters remain open, as
does manual game combine/expand.

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
Elapsed time currently follows the directly spawned primary child; detached
descendant and focus-based accounting remain later lifecycle work.

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
