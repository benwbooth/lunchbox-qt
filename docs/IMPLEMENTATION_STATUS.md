# Implementation status

This document records mechanically verified port work. The 113 feature families
in `FEATURE_MATRIX.md` remain a census; no complete feature family is yet marked
Windows-and-Linux parity verified.

## Working vertical slice

| Area | Current implementation | Verification |
|---|---|---|
| RetroArch core selection | First `RUN-016` subset. A checked-in catalog freezes the 56 unique platform/core suggestions in LaunchBox 13.27, including all 54 `Recommended` rows. A bounded read-only scanner identifies the current host's native `.dll`, `.so`, or `.dylib` core libraries from configured `libretro_directory`, application-local `cores`, packaged Linux AppImage home, macOS bundle/Application Support, XDG/home, and Windows application-data locations. The shared resolver handles portable `:\`, native, home-relative, and explicitly mapped Windows values. Configuration and core-directory symlinks are refused; entry symlinks/non-Unicode or unsafe names are omitted and counted; case-insensitive duplicate core names fail closed. The complete emulator mapping editor shows inventory/configuration provenance, ranks an installed 13.27 suggestion first, retains missing/custom mappings, and changes only one semantic `-L`, `--libretro`, or `--libretro=` argument while preserving unrelated arguments | Catalog tests prove 56 unique rows and 54 recommendations. Scanner tests cover native extension filtering, AppImage-home discovery, a modeled macOS app bundle with `.dylib` filtering, relative/absolute command paths, unsafe configuration/directory/entry refusal, and Windows-compatible quoting. Controller coverage proves host inventory-to-platform suggestion mapping. The real offscreen editor workflow discovers two cores, refuses a symlink, applies the installed SNES recommendation, preserves an unrelated flag, transactionally writes exactly one mapping with one exact backup, and byte-compares the executable, configuration, and both cores before and after. The Windows core gate compiles the complete portable catalog/scanner; both Darwin targets compile the dependency-free domain/platform boundary, while native macOS Qt and integration-crate execution remain a real-host gate. Individual core acquisition/update/removal, online updater policy, BigBox selection, and netplay remain open |
| Reproducible toolchain | Pinned Nix flake with Rust, Qt 6.11.1, QML, CXX-Qt 0.8.1, SQLite, 7-Zip, DOSBox Staging, ScummVM, Linux `appimage-run`, format/lint tools, package definition, and pinned Windows GNU plus Intel/Apple Silicon Darwin Rust targets | `nix develop` reports Qt 6.11.1 and Rust 1.97.1; `sqlite3`, `7z`, `dosbox`, `scummvm`, and Linux `appimage-run` are present. The Linux-hosted Windows core gate compiles the supported cross-platform core. Pure-Rust domain/platform crates compile for both Darwin architectures; native C/C++ Qt, SQLite, and cryptography dependencies still require an Apple SDK/toolchain and real macOS gate |
| Linux package | Release builds, Qt runtime wrapping, packaged 7-Zip, DOSBox Staging, ScummVM, and `appimage-run`, and both installed executables | `nix flake check`, explicit release-mode tests, wrapper-content assertions for every runtime tool, generated-type QML validation, and offscreen installed-binary smokes |
| Domain | All 107 `Game` fields observed in 13.24, every persisted field recovered from the concrete 13.27 `GameSave` contract, every other platform-file record, plus playlists, emulators/mappings, navigation metadata, parents, controllers, input bindings, list cache, image types, scalar settings, and validation; persisted path strings remain lossless data rather than becoming host paths | The canonical game field inventory is mechanically compared with the value-free real-install schema; a full 16-field 13.27 save fixture is checked through both platform readers; Windows drive/UNC classification and all persisted-path interpretation are confined to the platform service |
| Read index | Streaming platform-file index plus a complete typed index for all other 13.24 XML document groups | Exact counts across the read-only real install match the value-free schema for every record family |
| Lossless editing | DOM-backed platform and auxiliary documents, transactional typed game/additional-application/platform/category/playlist metadata and hierarchy edits, source-indexed alternate-name/custom-field/platform-folder/parent/playlist-filter/playlist-game replacement, metadata-only source-indexed game-save edits, filesystem-backed full game-save append, exact vault-row removal, and active-row-to-vault replacement, platform/category/playlist/game/additional-application/auxiliary-record add/remove, additional-application Make Default conversion, manual game combine/expand, unknown-element retention, serialization, and no-overwrite save; persisted paths remain lexical strings until a platform service boundary | All ten auxiliary families structurally round-trip; retained repeated-metadata, additional-application, game-save, platform-folder, parent, playlist-filter, and playlist-game rows keep their exact unknown children; save metadata edits can change only title/group name/group ID and cannot claim, move, or delete a save file, while manual backup may append a validated full record, vault deletion may remove one exact row with its owned file set, and active deletion may replace one exact active row with its verified vault copy before touching live files; the dialog rename/combine/split chain retains four exact backups, 13.27 lineage/hash/size/timestamp fields, lexical Windows paths, and unknown XML; additional-application edits retain immutable identity/owner/storefront/cloud state, deletion refuses game-save references without deleting targets or media, and Make Default copies the selected row's shared launch/version fields to the game without consuming the row or disturbing game-only/unknown data; manual combine creates a launch representative for every selected game, removes non-root games, migrates every modeled owner/clone/navigation/playlist/blacklist reference in one transaction, and expand consumes representatives while restoring distinct standalone games and application-owned saves; category lifecycle atomically updates `Platforms.xml` and `Parents.xml`; playlist lifecycle atomically creates/edits/deletes its document with `Parents.xml`, optionally clears owned `ListCache.xml` rows, detaches direct children to root, and never edits games or media; launch and platform-folder edits preserve Windows separators and unknown XML; platform lifecycle owns exactly 51 default folder rows and never creates media directories; fast/lossless platform readers produce identical modeled fixture data |
| Emulator configuration | Typed, lossless `Emulators.xml` create/edit/delete for all 31 recovered emulator fields and all six per-platform mapping fields; immutable generated UUIDs; source-indexed retained mappings; nullable mapping extraction mode; duplicate ID, owner/platform pair, and platform-default validation; atomic default handoff; fresh cross-platform-document reference refusal; optimistic revision reporting; stored executable paths remain lexical | Frozen-schema tests mechanically match the field inventories; storage tests cover typed CRUD, source indices, unknown XML, lexical Windows paths, validation, and game/additional-application reference discovery; controller tests cover transactional lifecycle, reference refusal, identity/unknown-field/platform validation, exact backup revisions, and target isolation; the offscreen real-dialog scenario proves edit/create/blocked-delete/delete, generated identity, default handoff, three exact backup states, unknown XML retention, no recovery manifest, and no interpreted emulator directory |
| Emulator executable discovery | Read-only, background discovery for evidence-reviewed BigPEmu, RetroArch, Dolphin, PCSX2, ScummVM, and Xemu executable identities across a bounded portable `Emulators` tree, OS-native application locations, and `PATH`; deterministic canonical-path deduplication; Unix executable-bit checks; explicit KDE `dolphin` collision exclusion; provenance and current-registration state in Qt; reviewed registration through the complete typed editor with recovered 13.27 defaults and supported static platform mappings; shared reverse host-path conversion keeps portable paths lexical and mapped/external paths cross-platform. BigPEmu recognizes the official Windows `BigPEmu.exe` and Linux `bigpemu` names, passively reads its bounded sibling `ReadMe.txt` for the displayed version, registers both Jaguar platforms with `%romfile% -localdata`, and neither invokes nor depends on its bundled desktop shell helper. Xemu registration records `-full-screen -dvd_path` and Microsoft Xbox support but deliberately omits the Windows-only AutoHotkey escape script | Integration tests prove bounded deterministic discovery, portable precedence, canonical-alias deduplication, non-executable refusal, name-collision isolation, BigPEmu and Xemu identities, passive BigPEmu version parsing with symlink/ambiguity refusal, and the reviewed mapping inventory. Controller tests prove recovered defaults, BigPEmu and Xemu platform-neutral registration, portable storage, existing-default preservation, and duplicate-registration refusal. A native launch-plan test proves BigPEmu receives the resolved ROM exactly once plus `-localdata`. The offscreen real-dialog scenario discovers a real executable fixture without starting or changing it, registers through the normal transactional writer, retains unknown XML and one exact backup, refreshes registration state, and proves no emulator binary or directory was created |
| Managed PCSX2 lifecycle | First `RUN-003` install/update/repair/removal provider. It consumes the official GitHub releases API, uses the recovered first-compatible-release behavior, selects the exact x86-64 Linux AppImage, Windows Qt 7z, or macOS Qt tar.xz asset, rejects draft/missing-digest/oversize/untrusted/unsafe entries, streams with bounded memory and cancellation, verifies exact size plus GitHub SHA-256, and never executes an artifact while installing. Linux preserves the AppImage executable mode; Windows extracts through the existing traversal/symlink-safe 7z boundary. macOS verifies the XZ signature and declared unpacked size under a 2 GiB ceiling, extracts exactly one derived tar into a private directory, sends all tar members through the same safe boundary, requires one app root and `Contents/MacOS/PCSX2`, normalizes the upstream versioned root to stable `PCSX2.app`, and restores the main executable mode. The official bundle is x86-64 and therefore requires Rosetta 2 on Apple Silicon. Every portable artifact path, empty port-owned `portable.ini`, portable relative-path/digest ownership manifest, and lossless `Emulators.xml` registration/repoint commit under one recovery manifest. Updates delete only exact obsolete owned paths. The offline removal review requires current complete ownership, exact owned files, a real in-root directory, the original managed emulator path, and no game/additional-application pins; removal transactionally deletes only those files, the manifest, and matching definition while retaining user settings, unrelated files, directories, and exact recovery copies. Legacy manifests remain readable but require repair before removal. Linux AppImage launches remain shell-free and route through packaged `appimage-run` | Provider tests cover exact Linux/Windows/macOS asset selection, official digest requirements, untrusted URLs, size ceilings, streamed progress, cancellation and digest mismatch cleanup, complete/legacy manifest validation, portable Windows-safe relative paths, executable/owned-file modification, missing/unsafe state, and fixture symlink refusal. Platform tests create a real tar.xz, prove the declared-size ceiling precedes extraction, audit both stages, and reject a false XZ signature. Storage coverage proves metadata creation/replacement/deletion and executable permission preservation in one transaction. Controller tests prove Linux initial install/update/removal plus a complete macOS app install/removal with stable path normalization, nested-directory creation, complete ownership, executable mode, retained user settings/directories, and exact recovery copies. The offscreen Linux lifecycle scenarios review an official-shaped release, install byte-exact non-executed content, verify executable mode, portable stored path/default mapping and complete ownership, then perform an offline removal and prove owned-path deletion, definition cleanup, user-file/directory retention, exact file/XML recovery copies, and no pending manifest. Further managed emulators, dependency/core management, automatic update policy, and native macOS Qt/runtime execution remain open |
| Managed BigPEmu lifecycle | Second `RUN-003` provider. It parses Rich Whitehouse's official release page and selects the exact Windows x64/ARM64 ZIP or Linux x64/ARM64 tar.gz for the host; rejects duplicate/malformed/untrusted names, URLs, sizes, and hashes; streams with bounded memory and cancellation; verifies the published byte count and uppercase 64-bit FNV-1a value; and records a locally computed SHA-256. ZIP members use the shared safe archive boundary. Linux GZip must contain exactly the derived tar name, disclose a bounded unpacked size under 512 MiB, and produce one exact stream before every tar member passes the traversal/link/special-file audit. The package must contain one expected executable and a root `ReadMe.txt`. The optional Linux `make_desktop.sh` is excluded and never invoked. All other package files, a provider-specific ownership manifest, and the recovered Jaguar/Jaguar CD `Emulators.xml` registration or repoint commit together. Updates and repairs replace only owned paths, delete only exact obsolete owned files, preserve native executable permissions, and retain unrelated user files. Offline removal applies the same complete ownership, reference, path, digest, transaction, and recovery-copy gates as PCSX2 | Provider tests cover the exact four official page sections, current asset naming, untrusted/duplicate/malformed catalogs, passive bounded version inspection, streamed SHA/FNV verification, cancellation, mismatch cleanup, and provider-specific manifest validation. Platform tests create a real tar.gz and prove signature, stream-name, size ceiling, two-stage extraction, and member safety. The controller lifecycle test installs and updates real official-shaped Linux archives, proves the invalid desktop helper is neither installed nor executed, verifies both hashes, stable native path/mappings, executable mode across replacement, exact stale-file recovery, repair/removal blocking on modifications, lossless XML backups, exact removal recovery copies, retained user settings/directories, and no pending transaction |
| Managed Xemu lifecycle | Third `RUN-003` provider. It consumes the official `xemu-project/xemu` latest release, models all five exact host artifacts (Windows x64/ARM64 ZIP, Linux x64/ARM64 AppImage, and signed universal macOS ZIP), rejects draft, duplicate, debug, unsigned, moving-alias, missing-digest, oversized, and untrusted metadata, and derives the release URL instead of trusting catalog presentation fields. Downloads stream with cancellation and require the exact GitHub byte count and SHA-256. Linux installs stable `xemu.AppImage`; Windows requires root `xemu.exe` plus `LICENSE.txt`; macOS preserves the exact `xemu.app/Contents/MacOS/xemu` hierarchy and bundle permissions. ZIPs use the shared traversal/link/special-file-safe boundary. Every provider file, exact version/tag/artifact ownership manifest, and recovered Microsoft Xbox definition/mapping commit in one transaction. Updates and repairs replace only owned paths, remove only exact stale owned files, and retain user configuration, firmware, unrelated files, and directories. Offline removal applies the complete ownership, reference, definition-path, digest, transaction, and recovery-copy gates | Provider tests cover exact selection for all five architectures, decoy/debug/duplicate/missing-digest/untrusted rejection, streamed byte/digest/progress verification, mismatch cleanup, and exact persisted release metadata. Discovery recognizes the managed Linux AppImage without executing it. Official-shaped Windows and macOS ZIP tests prove exact root and app-bundle contracts plus executable mode. The Linux controller lifecycle test proves install/update/repair classification, stable portable path, recovered command line and default Xbox mapping, executable mode across replacement, exact recovery copies, reference and modification blocking, exact owned-file/definition removal, retained `xemu.toml` and BIOS bytes/directories, and no pending transaction. Native Windows/macOS runtime execution remains a real-host gate |
| Managed RetroArch lifecycle | Fourth `RUN-003` provider. It implements the recovered LaunchBox 13.27 stable-buildbot contract across Windows, Linux, and macOS: exact Windows x64/x86 or Linux x64 frontend plus matching cores 7z pairs, and the official universal Metal DMG for Intel and Apple Silicon. Stable semantic versions, artifact paths, names, and exact HTTP byte counts are allowlisted; moving/nightly/query/foreign assets are rejected. Because the buildbot publishes no SHA-256 sidecars, the UI says so explicitly and the ownership manifest records locally computed download digests without presenting them as upstream verification. Linux and Windows strip only the exact official wrapper root. macOS extracts the exact `RetroArch.app`, requires its executable, `Info.plist`, signature resources, and assets archive, preserves bundle modes, and reconstructs only the six observed safe relative MoltenVK framework symlinks before transactionally owning them. A version-3 manifest records both release artifacts or the single DMG, every file/link, and the emulator definition. Install/update/repair/stale cleanup/removal use the recovery transaction, reject unmanaged collisions and changed ownership, retain unrelated settings/saves/directories, and never execute a downloaded artifact or invoke a shell | Provider tests prove semantic release selection, all four host contracts, mandatory cores pairing, macOS's deliberate lack of a fictitious stable cores archive, exact URL allowlisting, local streaming receipts, and cancellation cleanup. Manifest tests prove the exact macOS link set, safe target resolution, modified-link audit, and v1/v2 readability. Storage tests transactionally create, replace, and delete exact relative symlinks. The official-shaped Linux controller lifecycle performs install/update/repair detection/removal with two archives, exact XML and binary recovery, executable mode, retained unowned configuration/save bytes, and no pending transaction. The macOS preparation test converts all six real-DMG placeholder targets into symlinks and rejects a changed target. The live 1.22.2 universal DMG structure and Windows/Linux wrapper roots were inspected without executing any artifact. Native Windows/macOS runtime execution remains a real-host gate; user-driven core selection/install policy and netplay remain under `RUN-016` |
| Startup/shutdown lifecycle presentation | `RUN-009` plus the first `RUN-011` subset. The platform planner resolves startup and shutdown settings once from the actual primary launch plan: an explicit game override wins, otherwise an effective emulator supplies its defaults, while direct/DOSBox/ScummVM targets use game settings. The immutable sequence carries startup enable, true pre-process load delay, shutdown enable, and provenance. Each shell selects its separate `Settings.xml`/`BigBoxSettings.xml` global enable, theme name, cursor setting, and millisecond startup/shutdown minima. The worker presents startup before sleeping for `StartupLoadDelay` and spawning the primary; the shared Qt overlay dismisses only after both primary start and the frontend minimum. Effective `DisableShutdownScreen` controls a second shared modal overlay after the isolated launch session exits. A short-lived primary queues shutdown until the startup minimum completes. Global disable bypasses both overlays and the load delay. Failure and library replacement clear state without a detached timer thread or shell | Platform tests cover frontend scalar parsing/error refusal plus emulator-default, game-override, direct-game, and selected-additional-application startup/shutdown inheritance. The real-process offscreen suite runs LaunchBox and BigBox against a portable Rust fixture, measures the inherited 250 ms pre-launch delay, distinct 600/700 ms startup minima and 350/450 ms shutdown minima, captures and validates rendered PNGs, proves startup dismissal while the process remains supervised, compares exact argv, and checks play count/time/last-played records. A 50 ms primary separately proves startup-to-shutdown handoff only after the minimum; a disabled-global LaunchBox process proves zero presentation/delay counters with a normal launch. Theme/media asset selection, window/focus hiding, and processes that explicitly escape the supervised session remain open |
| Pause lifecycle presentation and supervised-session control | First `RUN-010` subset. The same actual-primary planning boundary independently resolves `UsePauseScreen`, `SuspendProcessOnPause`, forceful-activation provenance, and game-override/effective-emulator/direct-game precedence. Each shell selects its own global `UsePauseScreen` and `PauseTheme`; the worker owns a typed command receiver while Qt exposes an explicit button/local Ctrl+P action and one shared modal resume overlay. Every primary starts in an isolated Unix process group or Windows Job Object. Unix stops and continues the complete group with `SIGSTOP`/`SIGCONT`; Windows enumerates the job's processes and balances one suspend count per successfully controlled thread. Windows creates the primary suspended, assigns it to the private job, then resumes its initial threads so immediate delegation cannot race supervision. Failed Windows partial suspension is rolled back, duplicate commands are idempotent, and sender loss resumes a paused session before supervision continues. Completion/reload clears Qt and channel state | Policy tests cover frontend scalar parsing/error refusal and all three inheritance sources. Five shell-free integration tests use the same portable Rust fixture on every target to prove exact argument delivery, before/main/after order, the waited-before timeout, direct pause/resume, and delegated descendant supervision plus pause/resume. The delegated test proves the worker remains active after direct-child exit and includes the paused interval in session runtime. The strengthened Windows core gate compiles all fixture and core targets. Separate LaunchBox and BigBox offscreen processes delegate to a child, inherit the emulator policy, record exactly one successful suspension and resumption plus one delegated-session completion, hold a rendered overlay for 300 ms, validate PNG bytes, compare exact argv, and commit play statistics. Archive and M3U smokes also delegate delayed resource reads, proving leased files survive direct-child exit and are removed only after session completion. AutoHotkey pause/resume scripts, global/controller input when the game owns focus, forceful cross-window activation, pause theme/media assets, mute/fade behavior, deliberately session-escaped processes, and pause-excluded play time remain open |
| PCSX2 BIOS audit | Read-only implementation of the recovered LaunchBox 13.27 required `ps2 bios` group with all 73 filename, MD5, and description alternatives; any one valid item satisfies the group. Portable mode requires `portable.ini` and reads `inis/PCSX2.ini`; native mode searches only PCSX2 data roots and falls back to the host-native default. Relative `Bios` values resolve at the owning data root, while foreign Windows-absolute values are rejected on Unix. Regular firmware files are streamed through MD5; missing, mismatched, unreadable, file-symlink, directory-symlink, and other non-regular entries are reported without executing PCSX2 or changing configuration, firmware, or directories | Catalog tests prove all 73 alternatives and uniqueness; integration tests cover every result state, portable and native configuration precedence, foreign-path rejection, file and directory symlink refusal, and whole-tree immutability. Controller coverage proves the complete versioned Qt payload and one guarded notification. The offscreen real-dialog scenario opens the configured PCSX2 BIOS manager, checks the exact 73-row group and mismatch/unsafe counts, verifies configuration provenance, and compares the complete fixture tree, types, modes, symlink targets, and regular-file hashes before and after the scan |
| Xemu BIOS audit | Second read-only `RUN-004` adapter, implementing the recovered required `xemu boot`, `xemu hdd`, and `xemu bios` groups with all seven filenames, six exact MD5s, descriptions, file-required flags, and any-one-valid group semantics. Portable `xemu.toml` beside the executable (or a macOS app's Resources directory) wins over SDL-style host-native Xemu data roots. Existing configured file paths select each group directory; relative paths resolve at the executable directory and foreign Windows drive/UNC paths use explicit host mappings. Missing configured files fall back to recovered application `bios`/`saves` locations. The audit accepts a readable regular HDD without a digest, performs case-insensitive catalog lookup safely on case-sensitive hosts, rejects ambiguous names and symlinks, bounds TOML input, and never starts Xemu, downloads the dashboard HDD, creates directories, or rewrites configuration | Catalog tests prove all three required groups, seven unique alternatives, and the digest-free HDD contract. Integration tests cover portable precedence, group fallbacks, mismatch/missing/presence states, mapped Windows configuration, case-insensitive lookup, ambiguity and symlink refusal, and configuration immutability. Controller tests prove the generalized versioned multi-group Qt payload and Xemu registration defaults; the existing offscreen BIOS dialog verifies the shared payload/rendering boundary with the complete PCSX2 catalog |
| RetroArch BIOS audit | Third read-only `RUN-004` adapter. A checked-in mechanically extracted catalog contains all 630 LaunchBox 13.27 rows across 103 libretro cores, including platform filters, descriptions, nested relative paths, optional MD5s, per-file requirements, group identity, required-group flags, and `None`/`Any`/`All` rules. The audit derives cores from every configured emulator-platform mapping, deduplicates repeated platform/core pairs, and evaluates all selected contexts in one generalized Qt result. Application-local `retroarch.cfg` wins over official host-native configuration candidates; bounded parsing resolves portable `:\`, native absolute, home-relative, and explicitly mapped Windows `system_directory` values. Dynamic per-content `default` is reported instead of mapped to a false global root. Nested lookup is case-insensitive on case-sensitive hosts and refuses ambiguous components, configuration/file/directory symlinks, and non-regular firmware without executing RetroArch or mutating any path | Catalog tests mechanically prove 630 rows, 103 cores, known exact hashes, and all three group rules. Integration tests cover a valid digest-free required BIOS, portable configuration and whole-file immutability, mapped Windows system roots, nested case-insensitive lookup, directory-symlink and case-collision refusal, host-native candidate construction for Unix/macOS/Windows, and explicit dynamic-root refusal. Controller tests prove mapping-command precedence, emulator-command fallback, adapter registration, target/core context, dynamic group IDs/rules, hashes, paths, and versioned Qt payloads. The existing offscreen BIOS dialog compiles and exercises the generalized target/group/file rendering schema |
| Safe replacement | Typed reparse-before-write validation, exact durable backup, SHA-256 source/target-revision conflict detection, permission preservation, symlink/special-entry rejection, streamed regular-file staging, same-directory atomic replacement, byte-exact reversible restore, exact recursive directory revisions, rollback-capable sibling-directory replacement with a retained complete-tree recovery copy, and revision-checked external-capable file-set deletion with all backups prepared before mutation and reverse-order rollback | Every auxiliary family performs a real mutation/save/reload test; changed save source/target, multi-megabyte streamed replacement, changed directory source/target, unsafe directory symlinks, injected validation, stale-source, replace, directory-sync, failed second directory move with successful rollback, and second-member delete failures prove the documented recovery state; successful directory and three-member deletion cases retain exact sibling copies; Windows core compiles through `MoveFileExW` |
| Repository transactions | Root-scoped lock, multi-document replace/create/delete staging, streamed creation or replacement from external files with caller-supplied source and target revisions, opt-in permission preservation for new native executables, small validated byte creation/replacement, exact caller-revision vault deletion, pre/post-stage revision checks, durable version-2 manifest, exact per-file/deletion backups, automatic rollback, conservative crash recovery, and refusal to write past a pending manifest | Two-document replace/create/delete, streamed-ROM-plus-XML, permission-preserving executable-plus-marker-plus-manifest, revision-verified active-save-plus-XML creation, exact vault-save-plus-XML deletion, multi-source set replacement, and five-document game-grouping commits; exact golden semantic diff, stale document and changed inspected save source, pending-manifest refusal, injected partial mutation and rollback, simulated process death, recovery, external divergence, unversioned/out-of-root targets, source/target aliasing, filename collision, non-empty deletion, and manifest path escape are tested |
| Emulator save adapters | Platform-neutral `lb-integrations` adapters share configured launch-emulator selection and host-path resolution. RetroArch covers native `retroarch.cfg`, regular saves/states, cross-platform cores, additional-application ownership, and Saturn companion grouping/signatures. Dolphin derives raw, WAD, or DolphinTool IDs; searches portable and native user roots; discovers region-aware GameCube files, exact state slots, and recovered disc/WAD Wii title directories; and preserves regular-file versus directory-container identity. PCSX2 extracts ISO9660 `SYSTEM.CNF` serials through native bounded readers for plain/raw-sector ISO, GZip, CSO, CHD v5, MDF/MDS, and NRG, discovers ordinary states plus folder/raw-card members, and implements logical/physical-page parsing, FAT/directory traversal, extraction, manifest verification, mutation, ECC regeneration, and the recovered no-space-triggered orphan-FAT capacity repair on private raw working copies | Adapter tests cover RetroArch mappings and set safety; Dolphin raw/WAD/tool IDs, GameCube filtering, both Wii disc high IDs, exact WAD high/low directories, nested metadata, stable groups, and symlink refusal; and PCSX2 filesystem-first disc serials across every reader/layout, authentic `chdman`-verified DVD/CD CHDs, symlink and malformed-input refusal, opaque-content save ownership, reachable-chain preservation, orphan reclamation, retry-only-on-no-space behavior, source immutability, and card mutation boundaries. Controller tests prove append-only/idempotent persistence, compressed-CHD serial ownership, regular/Saturn operations, verified PCSX2 member archives and folder/raw whole-card lifecycle, reclaimed-cluster reporting, mapped external deletion, and verified Dolphin Wii nested archives with revision-checked whole-directory restore/deletion and complete recovery trees. Real-button offscreen scans cover all three adapters, including PCSX2 discovery from an opaque compressed CHD without external tools; the dialog-confirmed PCSX2 raw lifecycle begins with 28 orphan allocations and proves Qt-visible recovery plus three exact vault versions and two exact whole-card recovery files, while the Dolphin Wii lifecycle proves nested bytes, empty directories, and complete recovery trees; both retain owner/core/group/container metadata, targeted notifications, exact XML backups, unknown elements, and no manifest residue |
| Manual ROM import | Reusable background planner and three-page Qt workflow for multiple native files/folders, recursive discovery, extension filtering, file- or folder-derived editable titles, resolved-path duplicate detection across games and additional applications, portable destination collision checks, leave/copy/move policies, same-stem/different-extension companion copying, non-recursive PDF-manual discovery, portable title/year subfolders, platform-default/direct/explicit emulator selection, conservative complete-disc-set grouping, metadata-resolved matching-title/version grouping, separator-neutral filename title/version/region recovery, and read-only local LaunchBox metadata lookup; PDF matching prefers a case-insensitive ROM stem, accepts a sole folder candidate, and exposes ambiguity without guessing; the SQLite adapter canonicalizes platform aliases, uses exact primary/alternate titles first, falls back only when empty to recovered substring/all-word matching with bare-numbered-suffix suppression, then applies qualifier preference; one result is auto-applied and ambiguous exact or partial results expose stable database-ID choices; matching unique database IDs combine even across alternate source names, exact cleaned titles combine without metadata, ambiguous rows stay separate, deterministic source order selects the primary, and every ROM including the primary persists as an ordered selectable version application with path/emulator/version/region/developer/publisher/date/status; preview and execution revalidate configured and metadata IDs, execution re-plans the selected metadata, edited title, manuals, and destinations before a batch write, streams all copied files and platform XML through one recovery manifest, never overwrites a destination, and removes move sources only after byte verification | Unit tests cover recursive/sorted discovery, filters, duplicate defaults, stale-preview refusal, canonical configured-emulator lookup, unknown/empty-emulator refusal, direct-launch sentinel handling, exact/sole/ambiguous PDF outcomes, portable and reverse-mapped `ManualPath`, exact-first and partial-fallback matching, alternate-title/word-order/qualifier/sequel-suppression rules, unique and ambiguous partial outcomes, explicit candidate selection, removed-candidate refusal, typed database/manual plus filename-derived metadata persistence, complete versus incomplete/ambiguous disc sets, database-resolved version grouping, primary-ROM chooser retention, separator-neutral Windows/Unix qualifier parsing, version-app naming and metadata, portable edited-title subfolders and existing case-variant directory refusal, same-stem discovery and whole-game collision refusal, per-disc companions, typed additional-application insertion, atomic multi-file copy, portable stored paths, and verified multi-file move cleanup; the offscreen real-dialog scenario gives misspelled `Fixture Sag` ROMs two partial SQLite candidates, selects database ID 4242 through the Qt review payload, collapses two disc files to one game, independently combines exact metadata-resolved USA/World revision ROMs into a second game, persists filename region/version/import status and all four selectable applications, copies seven exact files into the corrected metadata-derived title/year folder, pins the configured emulator on both games and all ordered ROM records, refreshes the game/additional-app models, retains an exact XML backup and unknown XML, preserves copy sources, and leaves no recovery manifest |
| Query | Case-insensitive search across title, sort title, notes, and all descriptive editor metadata; platform/category/playlist filters; stable indexed sorting; default hidden/broken exclusion; recovered auto-playlist semantics OR rules sharing a field and AND distinct fields | Unit tested without cloning games or serializing result snapshots; manual/automatic playlist membership and recursive category aggregation have explicit unit and offscreen coverage |
| Process boundary | Shell-free direct/emulator/DOSBox/legacy-ScummVM/additional-app plans, Linux AppImage adaptation through `appimage-run` with the artifact as an explicit first argument, prevalidated priority-ordered before/main/after sequences, a 30-second waited-before ceiling, primary-session gating for after-apps, primary start/session-exit/runtime reporting, typed pause/resume commands with disconnect-safe resume, isolated Unix process-group and Windows Job Object supervision, an injectable host path resolver, explicit Windows drive/UNC mappings, Windows-native passthrough, portable separator handling, host-independent Windows-path classification, platform-native mapping-config locations, a versioned port-owned mapping document with atomic replacement, Windows-compatible command-line parsing, `%romfile%`/`%romlocation%`/`%platform%`/`%gameid%`/frontend-variable expansion, explicit/default/unassigned emulator selection, per-platform overrides, filename-only/no-space modes, working directories, console hiding, ZIP/7z/RAR auto-extraction, effective-mapping M3U generation from explicit disc records, DOSBox folder/floppy/CD/hard-disk mounts, custom config/executable and `[autoexec]` modes, legacy ScummVM data/save/extras paths and display flags, session-scoped temporary-resource leasing, OS process spawning, and direct-child reaping | Unit tests cover target/sequence/pause planning, shell-free AppImage program/argument mapping, typed/lossless mount parsing, DOSBox host/guest path separation, folder/image command generation, legacy ScummVM native semantic arguments and validation, mutually exclusive legacy modes, drive collisions and root traversal, disc membership and priority, timeout continuation, native/portable/mapped/unmapped paths, host-independent Windows-path classification, mapping validation/atomic persistence, variables, argument boundaries, cross-platform archive path validation, deterministic launch-file selection, and M3U primary-disc validation. A portable Rust fixture replaces every shell-based runtime payload and proves exact arguments, before/main/after order, timeout continuation, direct and delegated session runtime, group/job pause/resume, and resource lifetime; offscreen runtime coverage proves persisted mappings after frontend restart, direct/emulator/archive/M3U/DOSBox/ScummVM `argv`, folder/image mounts, archive-named extraction, multi-disc order, delegated generated-playlist and extraction cleanup, mapped Windows paths, selected additional-app execution, and both-shell delegated pause/resume |
| Qt data boundary | Worker-thread indexing/writing/import-preview/import-execution/launching/grouping, catalog-backed zero-game platform indexing, flattened category/platform/playlist hierarchy with recursive counts and filters, per-game priority-sorted additional-app and document-order save indexing, ordered alternate-name/custom-field indexing and guarded row accessors, versioned typed game/additional-application/save-manager/platform/category/playlist/import payloads, persisted host-mapping CRUD, stale-request generations, queued CXX-Qt result delivery, and a virtualized `QAbstractListModel` with 37 named identity/state/metadata/launch roles, reset notifications, targeted state/row notifications, save/additional-application/game-grouping/platform/navigation revision signals, and metadata-driven query recomputation | The current packaged release load smoke delivered 35,869 games, 16,752 additional applications, 54 playlists, 11 emulators, and 37 platforms in 2.227 seconds; generated QML metadata plus 37-role/filter, typed-edit/import/save payload, repeated-row, game/additional-application/save/platform/category/playlist CRUD, grouping, import, path-mapping, and process-launch runtime smokes prevent silent binding drift |
| LaunchBox shell | Background library loading, nested category/platform/playlist sidebar and dialogs, three-page ROM importer, descriptive search/counts/grid, Play and Launch With, per-game Apps and Saves managers, metadata/hierarchy/membership editors, combine/expand dialogs, Host Paths mappings, status/recovery/conflict banners, transactional editing and play statistics, and platform/category/playlist/game/additional-application lifecycle actions | Offscreen workflows cover the implemented CRUD, import, grouping, mappings, and real-process families. The Saves manager proves metadata grouping; real-button RetroArch, Dolphin, and PCSX2 discovery, including PCSX2 filesystem serial extraction from an opaque compressed CHD; verified PCSX2 member backup; dialog-confirmed PCSX2 raw-card capacity recovery/whole-card lifecycle and Dolphin Wii whole-directory restore/deletion; Dolphin regular-file and complete Saturn restore; and backup-first active deletion. Together the controller and Qt scenarios verify portable naming, full metadata, container boundaries, exact nested/logical bytes, Qt-visible repair results, exact file/XML/complete-tree recovery copies, targeted refresh, unknown XML retention, and no leftover manifest. Other directory/container adapters, other-emulator discovery, automatic policy, general repair commands, collapse, and remaining adapters stay open |
| BigBox shell | Separate full-screen QML entry point with a keyboard-first category/platform/playlist filter drawer, controller-owned active filter, `HideInBigBox` projection, horizontal game navigation, Enter/double-click launch, keyboard/button Launch With chooser, and launch status over the shared role model | Offscreen QML role smoke plus an exact hierarchy/manual-or-auto-playlist/category/platform filter sequence; unit coverage proves hidden navigation nodes are removed and visible descendants reparent; real emulator, archive, M3U, DOSBox, legacy ScummVM, and selected additional-app launches run from synthetic fixtures |

The LaunchBox shell also has a full emulator-definition and platform-mapping
manager. Its real-dialog scenarios verify generated identity, all recovered
typed fields, atomic default handoff, referenced-delete refusal, lexical paths,
exact backups, unknown XML, target isolation, and transaction cleanup. Its
installed-emulator page discovers six reviewed native identities,
reports candidate provenance, and requires full-editor review before
registration. Discovery remains read-only. Separate PCSX2, BigPEmu, Xemu, and
RetroArch managers now implement provider-backed install/update/repair lifecycles with
official upstream artifact selection, streamed integrity checks, cancellation,
portable ownership state, one recoverable binary/configuration transaction,
packaged Linux AppImage execution, and offline digest/reference-gated removal
that preserves user files and retains recovery copies. BigPEmu additionally
selects all four official Windows/Linux architecture artifacts, verifies the
published FNV-1a value, safely handles bounded tar.gz, and excludes its optional
desktop helper from both installation and execution. Xemu selects the exact
official Windows, Linux, or signed universal macOS artifact, requires GitHub
SHA-256, preserves the native ZIP/app-bundle layout, and retains user firmware
and configuration. RetroArch selects the exact stable Windows/Linux
frontend-and-cores pair or universal macOS Metal DMG, distinguishes published
byte counts from locally computed SHA-256, preserves the signed app layout and
exact framework links, and uses the same recoverable ownership gates. Its first
core-selection subset safely inventories native Windows, Linux, and macOS
cores, applies frozen 13.27 platform suggestions through the complete mapping
editor, and keeps core/configuration discovery read-only. Other emulator
providers, dependency policy, individual-core lifecycle, BigBox selection,
netplay, and automatic update policy remain open.
The first three `RUN-004` adapters provide complete read-only PCSX2, Xemu, and
configured-core RetroArch BIOS validation; other emulator BIOS adapters,
acquisition, configuration changes,
and firmware mutation remain open.

The workspace currently has 340 passing Rust tests. Both QML frontends and the
shared startup and shutdown overlays are compiled into the native binaries. Their
`--smoke-test` paths verify all 37
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
  Metadata/online-media acquisition, remaining managed emulator providers and
  dependencies, other remaining BIOS adapters, and core handling,
  descriptor-content dependency copying,
  broader version/disc-name
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
  composes with archive extraction before writing the playlist. Descendants
  that remain in the isolated launch process group or job keep temporary
  resources leased through their exit; abrupt frontend termination and
  processes that deliberately escape that session still need stale-resource
  handling. Launch
  With selection, additional applications,
  and priority-ordered automatic before/after hooks now work in both shells;
  the recovered LaunchBox 3.1 contract supplies the 30-second before-app wait
  ceiling, but current Wine limitations prevent 13.27 oracle verification.
  PlayCount and LastPlayed now commit when the directly spawned primary child
  starts; PlayTime adds the supervised session's elapsed whole seconds after
  exit for both main games and selected additional applications. This matches
  the recovered historical log ordering while also safely covering descendants
  that stay in the isolated process group or job. Deliberately escaped
  processes and focus-aware exclusion are not yet handled. Dependency/core management,
  BIOS acquisition and mutation, other remaining BIOS adapters,
  session-escape handling, focus, remaining startup/pause theme/media behavior,
  pause scripts/global input/audio/fade/timing exclusion, remaining save workflows, controller
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
