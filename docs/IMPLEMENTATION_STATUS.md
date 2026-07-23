# Implementation status

This document records mechanically verified port work. The 113 feature families
in `FEATURE_MATRIX.md` remain a census; no complete feature family is yet marked
Windows-and-Linux parity verified.

## Working vertical slice

| Area | Current implementation | Verification |
|---|---|---|
| Reproducible toolchain | Pinned Nix flake with Rust, Qt 6.11.1, QML, CXX-Qt 0.8.1, SQLite, 7-Zip, DOSBox Staging, ScummVM, format/lint tools, and package definition | `nix develop` reports Qt 6.11.1 and Rust 1.97.1; `sqlite3`, `7z`, `dosbox`, and `scummvm` are present in the development environment |
| Linux package | Release builds, Qt runtime wrapping, packaged 7-Zip, DOSBox Staging, and ScummVM, and both installed executables | `nix flake check`, explicit release-mode tests, wrapper-content assertions, generated-type QML validation, and offscreen installed-binary smokes |
| Domain | All 107 `Game` fields observed in 13.24, every persisted field recovered from the concrete 13.27 `GameSave` contract, every other platform-file record, plus playlists, emulators/mappings, navigation metadata, parents, controllers, input bindings, list cache, image types, scalar settings, and validation; persisted path strings remain lossless data rather than becoming host paths | The canonical game field inventory is mechanically compared with the value-free real-install schema; a full 16-field 13.27 save fixture is checked through both platform readers; Windows drive/UNC classification and all persisted-path interpretation are confined to the platform service |
| Read index | Streaming platform-file index plus a complete typed index for all other 13.24 XML document groups | Exact counts across the read-only real install match the value-free schema for every record family |
| Lossless editing | DOM-backed platform and auxiliary documents, transactional typed game/additional-application/platform/category/playlist metadata and hierarchy edits, source-indexed alternate-name/custom-field/platform-folder/parent/playlist-filter/playlist-game replacement, metadata-only source-indexed game-save edits, filesystem-backed full game-save append, exact vault-row removal, and active-row-to-vault replacement, platform/category/playlist/game/additional-application/auxiliary-record add/remove, additional-application Make Default conversion, unknown-element retention, serialization, and no-overwrite save; persisted paths remain lexical strings until a platform service boundary | All ten auxiliary families structurally round-trip; retained repeated-metadata, additional-application, game-save, platform-folder, parent, playlist-filter, and playlist-game rows keep their exact unknown children; save metadata edits can change only title/group name/group ID and cannot claim, move, or delete a save file, while manual backup may append a validated full record, vault deletion may remove one exact row with its owned file set, and active deletion may replace one exact active row with its verified vault copy before touching live files; the dialog rename/combine/split chain retains four exact backups, 13.27 lineage/hash/size/timestamp fields, lexical Windows paths, and unknown XML; additional-application edits retain immutable identity/owner/storefront/cloud state, deletion refuses game-save references without deleting targets or media, and Make Default copies the selected row's shared launch/version fields to the game without consuming the row or disturbing game-only/unknown data; category lifecycle atomically updates `Platforms.xml` and `Parents.xml`; playlist lifecycle atomically creates/edits/deletes its document with `Parents.xml`, optionally clears owned `ListCache.xml` rows, detaches direct children to root, and never edits games or media; launch and platform-folder edits preserve Windows separators and unknown XML; platform lifecycle owns exactly 51 default folder rows and never creates media directories; fast/lossless platform readers produce identical modeled fixture data |
| Safe replacement | Typed reparse-before-write validation, exact durable backup, SHA-256 source/target-revision conflict detection, permission preservation, symlink/directory rejection, streamed regular-file staging, same-directory atomic replacement, byte-exact reversible restore, and revision-checked external-capable file-set deletion with all backups prepared before mutation and reverse-order rollback | Every auxiliary family performs a real mutation/save/reload test; changed save source/target, multi-megabyte streamed replacement, injected validation, stale-source, replace, directory-sync, and second-member delete failures prove the documented recovery state; successful three-member deletion retains every exact sibling copy; Windows core compiles through `MoveFileExW` |
| Repository transactions | Root-scoped lock, multi-document replace/create/delete staging, streamed creation or replacement from external files with caller-supplied source and target revisions, exact caller-revision vault deletion, pre/post-stage revision checks, durable version-2 manifest, exact per-file/deletion backups, automatic rollback, conservative crash recovery, and refusal to write past a pending manifest | Two-document replace/create/delete, streamed-ROM-plus-XML, revision-verified active-save-plus-XML creation, exact vault-save-plus-XML deletion, and multi-source set replacement commits; exact golden semantic diff, stale document and changed inspected save source, pending-manifest refusal, injected partial mutation and rollback, simulated process death, recovery, external divergence, unversioned/out-of-root targets, source/target aliasing, filename collision, non-empty deletion, and manifest path escape are tested |
| Emulator save adapters | Platform-neutral `lb-integrations` adapters share configured launch-emulator selection and host-path resolution. RetroArch covers native `retroarch.cfg`, content/core-sorted regular saves and states, cross-platform core-library names, main/additional-application ownership, Saturn companion grouping, and 13.27-compatible composite signatures/replacement rules. Dolphin derives four- or six-character disc IDs from raw ISO/GCM headers, WAD title IDs, or sibling DolphinTool output for RVZ/GCZ/WIA/WBFS; it searches portable and OS-native user roots for region-aware GameCube folder/Card A/Card B files and exact two-digit state slots while leaving Wii directories unclaimed | Seven adapter tests cover RetroArch regular saves, numbered/auto states, sorting, mappings, Windows/Linux/macOS core syntax, Saturn ordering/signatures/symlink refusal, plus Dolphin raw/WAD IDs, DolphinTool-output parsing, portable-root selection, region/card filtering, and state slots. Controller tests prove append-only/idempotent RetroArch and Dolphin XML persistence, including stable Dolphin group matching after a stored path moves, plus transactional Saturn backup/restore/deletion and mapped-external active deletion. Separate real-button offscreen scans prove both adapters with portable paths, owner/core/group/chip/slot/full metadata, targeted revision notification, exact XML recovery copies, unknown-element retention, and manifest cleanup |
| Manual ROM import | Reusable background planner and three-page Qt workflow for multiple native files/folders, recursive discovery, extension filtering, file- or folder-derived editable titles, resolved-path duplicate detection across games and additional applications, portable destination collision checks, leave/copy/move policies, same-stem/different-extension companion copying, non-recursive PDF-manual discovery, portable title/year subfolders, platform-default/direct/explicit emulator selection, conservative complete-disc-set grouping, metadata-resolved matching-title/version grouping, separator-neutral filename title/version/region recovery, and read-only local LaunchBox metadata lookup; PDF matching prefers a case-insensitive ROM stem, accepts a sole folder candidate, and exposes ambiguity without guessing; the SQLite adapter canonicalizes platform aliases, uses exact primary/alternate titles first, falls back only when empty to recovered substring/all-word matching with bare-numbered-suffix suppression, then applies qualifier preference; one result is auto-applied and ambiguous exact or partial results expose stable database-ID choices; matching unique database IDs combine even across alternate source names, exact cleaned titles combine without metadata, ambiguous rows stay separate, deterministic source order selects the primary, and every ROM including the primary persists as an ordered selectable version application with path/emulator/version/region/developer/publisher/date/status; preview and execution revalidate configured and metadata IDs, execution re-plans the selected metadata, edited title, manuals, and destinations before a batch write, streams all copied files and platform XML through one recovery manifest, never overwrites a destination, and removes move sources only after byte verification | Unit tests cover recursive/sorted discovery, filters, duplicate defaults, stale-preview refusal, canonical configured-emulator lookup, unknown/empty-emulator refusal, direct-launch sentinel handling, exact/sole/ambiguous PDF outcomes, portable and reverse-mapped `ManualPath`, exact-first and partial-fallback matching, alternate-title/word-order/qualifier/sequel-suppression rules, unique and ambiguous partial outcomes, explicit candidate selection, removed-candidate refusal, typed database/manual plus filename-derived metadata persistence, complete versus incomplete/ambiguous disc sets, database-resolved version grouping, primary-ROM chooser retention, separator-neutral Windows/Unix qualifier parsing, version-app naming and metadata, portable edited-title subfolders and existing case-variant directory refusal, same-stem discovery and whole-game collision refusal, per-disc companions, typed additional-application insertion, atomic multi-file copy, portable stored paths, and verified multi-file move cleanup; the offscreen real-dialog scenario gives misspelled `Fixture Sag` ROMs two partial SQLite candidates, selects database ID 4242 through the Qt review payload, collapses two disc files to one game, independently combines exact metadata-resolved USA/World revision ROMs into a second game, persists filename region/version/import status and all four selectable applications, copies seven exact files into the corrected metadata-derived title/year folder, pins the configured emulator on both games and all ordered ROM records, refreshes the game/additional-app models, retains an exact XML backup and unknown XML, preserves copy sources, and leaves no recovery manifest |
| Query | Case-insensitive search across title, sort title, notes, and all descriptive editor metadata; platform/category/playlist filters; stable indexed sorting; default hidden/broken exclusion; recovered auto-playlist semantics OR rules sharing a field and AND distinct fields | Unit tested without cloning games or serializing result snapshots; manual/automatic playlist membership and recursive category aggregation have explicit unit and offscreen coverage |
| Process boundary | Shell-free direct/emulator/DOSBox/legacy-ScummVM/additional-app plans, prevalidated priority-ordered before/main/after sequences, a 30-second waited-before ceiling, primary-process gating for after-apps, primary-child start/exit/runtime reporting, an injectable host path resolver, explicit Windows drive/UNC mappings, Windows-native passthrough, portable separator handling, host-independent Windows-path classification, platform-native mapping-config locations, a versioned port-owned mapping document with atomic replacement, Windows-compatible command-line parsing, `%romfile%`/`%romlocation%`/`%platform%`/`%gameid%`/frontend-variable expansion, explicit/default/unassigned emulator selection, per-platform overrides, filename-only/no-space modes, working directories, console hiding, ZIP/7z/RAR auto-extraction, effective-mapping M3U generation from explicit disc records, DOSBox folder/floppy/CD/hard-disk mounts, custom config/executable and `[autoexec]` modes, legacy ScummVM data/save/extras paths and display flags, temporary-resource leasing, OS process spawning, and child reaping | Unit tests cover target/sequence planning, typed/lossless mount parsing, DOSBox host/guest path separation, folder/image command generation, legacy ScummVM native semantic arguments and validation, mutually exclusive legacy modes, drive collisions and root traversal, disc membership and priority, timeout continuation, exact lifecycle order, primary runtime/resource lifetime, native/portable/mapped/unmapped paths, host-independent Windows-path classification, mapping validation/atomic persistence, variables, argument boundaries, cross-platform archive path validation, deterministic launch-file selection, M3U primary-disc validation, and Unix processes; runtime recorders prove persisted mappings after frontend restart, direct/emulator/archive/M3U/DOSBox/ScummVM `argv`, folder/image mounts, archive-named extraction, multi-disc order, generated-playlist and extraction cleanup, mapped Windows paths, before/main/after order, and selected additional-app execution |
| Qt data boundary | Worker-thread indexing/writing/import-preview/import-execution/launching, catalog-backed zero-game platform indexing, flattened category/platform/playlist hierarchy with recursive counts and filters, per-game priority-sorted additional-app and document-order save indexing, ordered alternate-name/custom-field indexing and guarded row accessors, versioned typed game/additional-application/save-manager/platform/category/playlist/import payloads, persisted host-mapping CRUD, stale-request generations, queued CXX-Qt result delivery, and a virtualized `QAbstractListModel` with 37 named identity/state/metadata/launch roles, reset notifications, targeted state/row notifications, save/additional-application/platform/navigation revision signals, and metadata-driven query recomputation | The current packaged release load smoke delivered 35,869 games, 16,752 additional applications, 54 playlists, 11 emulators, and 37 platforms in 2.227 seconds; generated QML metadata plus 37-role/filter, typed-edit/import/save payload, repeated-row, game/additional-application/save/platform/category/playlist CRUD, import, path-mapping, and process-launch runtime smokes prevent silent binding drift |
| LaunchBox shell | Background library loading, nested category/platform/playlist sidebar and dialogs, three-page ROM importer, descriptive search/counts/grid, Play and Launch With, per-game Apps and Saves managers, metadata/hierarchy/membership editors, Host Paths mappings, status/recovery/conflict banners, transactional editing and play statistics, and platform/category/playlist/game/additional-application lifecycle actions | Offscreen role and temporary-library workflows cover state/rich metadata, ROM import, every implemented CRUD family, Make Default, mappings, and real process launches. The Saves manager proves rename/combine/split metadata, real-button RetroArch and Dolphin discovery, dialog-confirmed Dolphin regular-file and complete Saturn-set restore, and backup-first active Saturn-set deletion; controller tests prove regular and complete Saturn-set vault backup and restore, regular or complete Saturn-set vault deletion, mapped-external active-set deletion, and mapped Dolphin GameCube active deletion. Together they verify portable naming, full metadata, exact file/XML recovery copies, targeted refresh, unknown XML retention, and no leftover manifest. Directory/container backup, other-emulator discovery, Dolphin Wii directory/PCSX2 container restore/deletion, automatic policy, repair, and remaining adapters stay open. Additional-application, import, playlist, category, platform, and process smokes retain their documented reference, identity, lexical-path, media-isolation, exact-backup, and lifecycle gates |
| BigBox shell | Separate full-screen QML entry point with a keyboard-first category/platform/playlist filter drawer, controller-owned active filter, `HideInBigBox` projection, horizontal game navigation, Enter/double-click launch, keyboard/button Launch With chooser, and launch status over the shared role model | Offscreen QML role smoke plus an exact hierarchy/manual-or-auto-playlist/category/platform filter sequence; unit coverage proves hidden navigation nodes are removed and visible descendants reparent; real emulator, archive, M3U, DOSBox, legacy ScummVM, and selected additional-app launches run from synthetic fixtures |

The workspace currently has 207 passing Rust tests. Both QML resources are
compiled into the native binaries. Their `--smoke-test` paths verify all 37
model roles before and after filtering from three rows to one under Qt's
offscreen platform. QML is also checked against generated CXX-Qt module
metadata; opening a window alone is not accepted as proof that its controller
or model bindings resolve. Separate launch smokes execute checked-in argument
recorders through direct and default-emulator paths from both front ends and
compare every captured argument. A lifecycle fixture proves waited-before,
main, and after ordering plus manual Launch With selection. The direct fixture
is stored under a synthetic
Windows drive and reaches its Linux executable only through the configured host
mapping.
The mapping smoke creates and removes a UNC entry, retains a drive entry in the
canonical version-1 JSON document, restarts into BigBox without CLI mapping
arguments, and proves that the saved mapping resolves the fixture executable.
An archive fixture is generated with 7-Zip during the smoke rather than stored
as an opaque binary. Both shells prove that auto-extraction preserves the
archive stem as a folder, supplies extracted `%romfile%` and `%romlocation%`
values while the emulator is alive, and cleans the folder after exit.
An M3U fixture deliberately stores its three disc applications out of XML
order, mixes slash styles, includes a non-disc additional app, and archives the
second disc. Both shells prove that the effective mapping generates a stable-
stem playlist in priority order, resolves every entry to a native host path,
keeps both playlist and extracted disc alive for the emulator, and cleans both
after exit.
A DOSBox fixture contains three typed mount records in LaunchBox 13.27's
persisted vocabulary: a CD-ROM folder, a floppy image, and an ISO image. It
mixes slash styles across the executable, configuration, game root, application,
and media fields. Both shells capture identical semantic `argv`: every host
path is native, only the DOS guest `CD` command uses a backslash, mount order is
preserved. The source document receives only the expected transactional session
fields while all unrelated and unknown data is retained.
A legacy ScummVM fixture stores its game-data folder with Windows separators
and leaves the ordinary application path empty. Both shells resolve that folder
to the native host path, capture separate game/save/extras arguments plus the
stored target, fullscreen, and aspect-correction flags, and leave the source
document otherwise intact. The modern 13.27 ScummVM plugin continues through the
ordinary emulator path rather than this legacy adapter.
Across all launch fixtures, both shells now prove that process start increments
the correct main-game or selected-additional-app PlayCount and writes the
seven-digit local-offset LastPlayed field. Fixtures whose child remains alive
for more than a second also prove whole-second PlayTime accumulation. Every
successful statistics transaction retains an exact backup, removes its durable
manifest, and preserves unknown XML.

## Real-install evidence

A read-only LaunchBox 13.24 installation supplied by the project owner was
profiled without recording filenames, element values, stored paths, accounts,
or license data. The derived schema reports:

- 37 platform XML files totaling 196,571,938 bytes;
- 35,869 games;
- 16,752 additional applications;
- 20,739 alternate names;
- zero custom-field records in this older library;
- 7,061 controller-support records;
- 33 game-save records;
- 54 playlist files;
- 104 playlist filters and 955 playlist-game records;
- 11 emulators with 63 per-platform mappings;
- 37 platform definitions, 4 categories, and 1,890 media folders;
- 159 parent relationships, 86 controller records, and 36 input bindings;
- 362 LaunchBox settings, 79 image-type settings, and 555 BigBox settings;
- 2 import-blacklist records and 122 list-cache records;
- zero XML parse errors in the schema census.

The alternate-name implementation is grounded in all 20,739 real rows and
their exact `GameID`, `Name`, and `Region` shape. That older installation has
no custom-field rows. Custom fields are therefore tested against a synthetic
fixture derived from the installed 13.27 `ICustomField` contract and concrete
serializer metadata (`GameID`, `Name`, and `Value`), not presented as
real-library runtime evidence.

The first whole-library lossless DOM test loaded the correct 35,869 games but
took 116.810 seconds and grew to roughly 2.4 GiB RSS. This led to an explicit
two-path design:

- browsing/search uses the read-optimized index on a worker thread (15.796
  seconds for the complete data index in the current unoptimized development
  build), then delivers the result through Qt's queued event loop;
- editing opens only the affected document as a lossless DOM and uses the
  transaction/backup pipeline; the QML editor exposes favorite, completed,
  integer star-rating, 18 descriptive metadata fields, launch configuration,
  alternate names, and custom fields through one versioned typed payload and
  transaction.

The reusable value-free compatibility auditor strictly parsed all 35,869 real
games, matched the union of their fields to the 107-field canonical model with
zero unknown or unobserved fields, and structurally round-tripped all 63
auxiliary documents (54 playlists plus nine fixed data files). It never writes
to the source installation.

A second value-free auditor builds a shell-free launch plan for every game. It
identified the all-zero emulator ID as LaunchBox's explicit “unassigned
emulator” sentinel: 1,848 games using it must bypass a platform default and
launch directly. With the two source volumes currently mounted on Linux mapped
to their stored drive letters, 35,847 games produce plans; nine more reference
an unmapped drive, and the remaining 13 have no application path. When every
observed drive is given a syntactic mapping, 35,856 plans resolve and only those
13 pathless records remain. Four emulator command lines use `%romlocation%`;
all expand and zero known variables remain in resulting argument vectors. This
is structural plan coverage, not proof that every referenced file exists or
that every Windows executable has a native Linux adapter.

The same auditor now plans all 16,752 additional applications against their
parent game and emulator context. With the mounted E: and F: volumes mapped,
16,736 produce plans; four reference the unavailable H: drive and 12 have no
application path. A syntactic H: mapping raises coverage to 16,740 and leaves
only the 12 pathless records. This is structural coverage and does not imply
that Windows-only targets execute natively on Linux.
It also reports 633 explicit disc application records across 239 games,
including four archive-backed disc records. Those aggregate counts drove the
M3U implementation; names and paths remain absent from the audit output.
The older installation contains zero `Mount` records and zero games marked
`UseDosBox`, so DOSBox behavior is verified against the recovered 13.27
contract and a synthetic fixture rather than falsely attributed to that
real-library census.
It likewise contains zero games marked `UseScummVM`; its ScummVM-related
booleans are serialized defaults on ordinary games, not evidence that the
legacy mode is active. Legacy behavior is therefore verified against the
recovered 13.27 contract and a synthetic fixture.

## Deliberate limitations

- `Game` field coverage is complete for the observed 13.24 installation, not
  yet for the newer 13.27 oracle. Free-form LaunchBox strings such as dates,
  paths, enum-like labels, scripts, and URLs deliberately retain their exact
  lexical values rather than being normalized during persistence.
- Settings retain all 917 observed scalar keys/values and offer strict typed
  accessors, but most keys do not yet have dedicated semantic domain fields.
- The JSON snapshot and numeric game-row facade are gone. Filtering and full
  library replacement use correct model-reset notifications; the game-state
  editor emits a targeted `dataChanged` for its three roles. Title edits
  correctly recompute sorting and filter membership with a model reset. Visible
  game additions/removals use `beginInsertRows`/`beginRemoveRows`; filtered-out
  mutations correctly leave the visible row model unchanged.
- The QML editor writes 18 descriptive fields, favorite/completed/integer
  star-rating state, launch configuration, alternate names, and custom fields.
  It blocks edits while recovery is pending, exposes an explicit safe-rollback
  action, and requires reload after a write conflict.
  Its root lock coordinates this port's processes; the original LaunchBox
  application does not honor it, so exact revision checks remain the
  protection against external writers.
- Add Game targets any catalog-backed platform document, including a newly
  created empty one, and writes a validated minimal record with a generated
  UUID; exact Windows-created game default-field parity is not yet
  oracle-verified. Add Platform creates a portable filename and the 51 default
  folder records observed across the real installation without interpreting
  or creating those stored media paths. Existing-platform editing covers the
  recovered metadata fields and source-indexed folder rows, preserves unknown
  XML, and creates no media directories. Platform identity remains read-only:
  the installed 13.27 `IPlatform.Name` contract is getter-only and the
  protected runtime body has not established a safe cross-document/filename
  rename cascade. Game and platform deletion are
  deliberately conservative: they block on modeled cross-document references
  instead of cascading and never delete media. The dependency scans are fresh
  and backgrounded, but an uncooperative external LaunchBox process can still
  race between scanning other documents and the revision-checked commit.
- The manual ROM importer covers the recovered location, platform, emulator,
  leave/copy/move, extension, folder-title, duplicate, and editable game-list
  steps. Default, explicit direct launch, and configured-emulator selection
  remain distinct; configured IDs are canonicalized and revalidated during
  preview and execution. It deliberately refuses stale previews and
  destination overwrites. The recovered same-name option copies or moves
  regular sibling files with the same stem and a different extension as one
  collision-checked transaction bundle; this applies independently to each
  grouped disc. Complete, contiguous, collision-free `(Disc N)` and
  `(Disc N of M)` sets within one folder and extension can be grouped; Disc 1
  remains the game path and every disc becomes an ordered additional
  application. Incomplete or ambiguous sets remain separate. Copy/move
  targets are portable `Games\<platform>` paths on every host and can use a
  sanitized `Title (Year)` child derived from the final edited title and a
  unique local metadata match; leave-in-place paths become library-relative
  where possible, reverse explicit Windows
  drive/UNC mappings where applicable, and otherwise remain native absolute
  host paths. Move is loss-averse: if post-commit source deletion fails or the
  source no longer matches its committed copy, the imported game remains valid
  and the source is retained with a warning. The first local-database slice
  canonicalizes platform aliases, reproduces exact primary/alternate-title
  comparison followed only when empty by the recovered substring/all-word
  fallback and bare-numbered-suffix suppression, applies parenthetical
  qualifier preference after either search, auto-applies a unique match,
  exposes compact stable-ID choices for multiple exact or partial matches,
  revalidates the chosen ID at execution, and persists the selected match's
  typed text/player/date/URL/rating fields. The recovered PDF option scans the
  main game folder without recursing, prefers a case-insensitive imported-file
  stem, accepts a sole
  candidate, and leaves multiple non-matching PDFs explicit and unlinked.
  External manuals use reversible host-path mappings; a same-name PDF included
  in a copy/move bundle instead receives its portable committed path.
  Metadata/online-media acquisition, emulator installation/editing and BIOS/core
  handling, descriptor-content dependency copying, broader version/disc-name
  grammars, MAME/FBNeo branches, cancellation/partial retention, and exact 13.27
  duplicate/error rules remain open. Database-resolved primary/alternate-title
  combining and exact cleaned-title fallback are implemented, including the
  primary ROM's selectable version record.
- The process-launch vertical supports direct executables and one
  explicit or default emulator mapping, including mapping-level command-line
  overrides and the five observed/documented LaunchBox command-line variables.
  Stored paths resolve behind a host service; Windows drives and UNC shares are
  native on Windows and require explicit mappings on Linux/macOS. The Host Paths
  screen persists validated mappings outside LaunchBox XML using platform-native
  config locations; CLI mappings are non-persistent final overrides. Mapping a
  path does not make a Windows-only executable runnable on Linux. Legacy
  `UseScummVM` games use the dedicated native adapter; modern ScummVM plugin
  mappings continue through the ordinary emulator path.
  DOSBox games use the portable bundled executable on Windows or packaged
  DOSBox Staging on Unix, unless a custom executable is configured. Custom and
  bundled configuration paths, configuration-owned `[autoexec]`, C-drive roots,
  and typed folder/floppy/CD/hard-disk mounts are supported without privileged
  OS mounts. All persisted paths go through the host resolver; DOS separators
  are generated only for guest-relative commands. Values that cannot be safely
  represented in a DOSBox `-c` command, duplicate/reserved drive letters,
  unknown mount vocabulary, and applications outside the C-drive root fail
  explicitly. Exact command parity with a live 13.27 Windows oracle and broader
  DOSBox configuration/import UI remain open. ZIP, 7z, and RAR auto-extraction now follows emulator and
  per-platform settings, uses the bundled 7-Zip on Windows or the packaged
  command on Unix, validates member paths before extraction, rejects symbolic
  links and encrypted or ambiguous contents, preserves the archive stem inside
  its private temporary directory, and retains that directory through process
  exit. M3U loading follows only the effective emulator-platform mapping. It
  uses the modeled `Disc` field rather than localized display-name parsing,
  retains LaunchBox priority order, requires the game path to identify the
  first disc record, resolves every line through the same path service, and
  composes with archive extraction before writing the playlist. Abrupt frontend
  termination and emulators that immediately delegate to a detached process
  still need stale-directory/process-tree handling. Launch
  With selection, additional applications,
  and priority-ordered automatic before/after hooks now work in both shells;
  the recovered LaunchBox 3.1 contract supplies the 30-second before-app wait
  ceiling, but current Wine limitations prevent 13.27 oracle verification.
  PlayCount and LastPlayed now commit when the directly spawned primary child
  starts; PlayTime adds its elapsed whole seconds after exit for both main games
  and selected additional applications. This matches the recovered historical
  log ordering, but detached launchers, descendant-process supervision, and
  focus-aware exclusion are not yet handled. Dependency/BIOS/core management,
  full process-tree lifecycle, focus,
  startup/pause/shutdown screens, remaining save workflows, controller
  behavior, and the remaining emulator-specific adapters remain unimplemented.
- Runtime visual/behavior parity against licensed LaunchBox and premium BigBox
  has not been established. Wine installed 13.27 but does not yet provide a
  reliable WPF oracle.

## Next implementation gate

The transaction, golden semantic-diff, role-model, recovery/conflict UX,
desktop rich metadata/state, manual ROM import foundation,
existing-platform metadata/folder editing,
transactional nested-category lifecycle, reference-gated game/platform add/remove, and first native Linux
direct/emulator/additional-app launch plus persisted path-mapping gates now pass
at the storage and QML boundaries. Phase 1 remains open until the editor covers
reviewed dependency-remediation choices, the remaining launch lifecycle breadth is implemented,
and the Windows oracle produces executable import/launch scenarios for
comparison.
