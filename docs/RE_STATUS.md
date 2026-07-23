# Reverse-engineering status

## Oracle installation

LaunchBox 13.27 was installed unattended with Wine 11.8 Staging into an
isolated workspace-local prefix.

| Item | Verified value |
|---|---|
| Installer | `/home/ben/Downloads/LaunchBox-13.27-Setup.exe` |
| SHA-256 | `19deeee55c135ffb1b720bcfcdecdd9e103ac86a6c47ffdc2b6b5a4af83b6481` |
| Installer format | Inno Setup 6.7.1, 32-bit bootstrapper, 64-bit Windows prefix |
| Install directory | `oracle/LaunchBox` |
| Wine prefix | `oracle/wine-prefix` |
| Installed footprint | approximately 1.5 GiB |
| Runtime | self-contained .NET 9.0.16, `win-x64`, WPF/Windows Desktop |
| Installer result | `Installation process succeeded.` in `oracle/installer.log` |

The setup payload completed before the automatically launched LaunchBox process
threw a managed Wine exception. That launch failure does not invalidate the
installed files, but it means a usable Wine runtime oracle is not yet proven.
A supported Windows VM is the fallback oracle if Wine cannot run the protected
WPF application reliably.

## Older real installation schema

A complete LaunchBox 13.24 installation was located on the owner's read-only
Windows partition. It provides genuine scale and data-shape evidence even
though it is older than the frozen 13.27 binary oracle.

`scripts/analyze_launchbox_schema.py` produced
`analysis/real-install-schema.json`, a value-free census that records element
and attribute names plus aggregate counts, but no filenames, text values,
stored paths, accounts, or license data. It found 37 platform files, 35,869
games, 16,752 additional applications, 20,739 alternate names, 54 playlists,
zero custom-field records, and no XML parse errors.

The census now includes a value-free additional-application shape audit. All
16,752 records have parseable nonnegative priorities ranging from 0 through 26;
12 have an empty application path. Five game-save rows reference additional
applications and all five references resolve, so deletion cannot safely treat
these records as independent. In this particular 13.24 installation every
emulated record has an emulator ID, no direct record retains one, and no
additional application enables DOSBox, automatic before/after execution, or
wait-for-exit. Those zero counts describe this installation only; the recovered
13.27 editor and plugin contracts still require those fields.

The same value-free census now measures the persisted combined-ROM contract.
The 13.24 library contains 6,261 games with at least two `Play … Version...`
additional applications and 15,430 such records. Every group includes an
application whose path equals the primary game's path, proving the chooser
retains the primary ROM instead of only secondary variants. Of those groups,
5,763 still have contiguous one-based priorities and 2,601 use
`Play {Version} Version...` for every entry; nonconforming groups include
MAME short-name choices and subsequently edited priorities. These are aggregate
counts in `analysis/real-install-schema.json`; no IDs, titles, names, versions,
or paths are retained.

The Rust read index independently loaded all 37 platform documents and all
35,869 games. A value-free compatibility audit matched the union of all real
game fields to the canonical 107-field model with no missing or extra names,
then structurally round-tripped all 63 auxiliary documents through their typed
lossless editors. This proves 13.24 XML data compatibility at the observed
field/document boundary; it does not prove 13.27 schema or behavior parity.

A value-free launch-plan audit additionally classified every game without
recording titles or paths. It established that 1,848 games use the all-zero
emulator ID as an explicit direct-launch sentinel, found 4,471 Windows-absolute
game paths, expanded all four configured `%romlocation%` command lines, and
left zero known variables unresolved. With the two currently mounted source
volumes mapped, 35,847 plans resolve; nine games require another drive mapping
and 13 records have no application path. The audit proves resolver coverage,
not file existence or emulator behavior.

The audit now also plans all 16,752 additional applications against their
parent games. With mounted E: and F: mappings, 16,736 plans resolve; four more
need the unavailable H: mapping and 12 records have no application path. With a
syntactic H: mapping, 16,740 resolve. The recovered plugin contract establishes
the meanings of automatic before/after and wait-for-exit flags, while the
installed 3.1 changelog supplies the 30-second before-app wait ceiling. These
sources and deterministic native fixtures specify the current implementation;
13.27 runtime parity remains unverified while the Wine oracle cannot run.

The recovered 13.27 editor contract exposes name/path/command line,
before/after/wait behavior, emulator and DOSBox choice, priority, disc and side,
developer/publisher/region/date/version/status, installed state, play count,
play time, and last played. ID and owning game are getter-only, while storefront
and cloud state are outside the editor. The port therefore edits exactly that
typed subset, retains identity/owner/provider/cloud values and unknown XML,
allows the 12 observed empty paths, and refuses deletion when a game-save
reference exists.

Make Default is a distinct recovered upstream command. The
[official help page](https://feedback.launchbox-app.com/help/articles/2413817-additional-apps)
states that the selected additional application becomes the default under the
Launching section, and LaunchBox's founder described the operation as replacing
the game's application path
[along with other fields](https://forums.launchbox-app.com/topic/35543-77-beta-6-released/).
The protected 13.27 view model exposes the command, label, and
`OnAdditionalAppMadeDefault` event, while the surviving data contracts expose
conversion helpers in both directions between a game and an additional
application. The real-install combined-version census also proves that the
primary/default path remains represented by an additional-application row, so
the selected row must not be consumed.

The implemented conversion copies every launch/version field representable on
both record types: path, command line, emulator selection, DOSBox mode,
developer/publisher/region/date/version/status, installed state, play
statistics, storefront identifiers, install path, and cloud state. Direct
launch maps to LaunchBox's explicit all-zero unassigned-emulator sentinel.
Because an additional application cannot express legacy ScummVM mode, the game
mode flag is cleared while its latent game-only ScummVM settings remain
untouched. Game identity, title, platform, presentation/media/input settings,
and unknown XML remain on the game, and the selected additional-application
record remains unchanged. A read-only comparison of 25 historical real-library
backup snapshots found no observed Make Default transition, so this field map
is static-contract- and documentation-derived rather than a claimed live 13.27
runtime oracle.

Manual combine/expand has a separate recovered contract. The
[official help page](https://feedback.launchbox-app.com/help/articles/2413817-additional-apps)
states that Combine Selected Games asks which selected game should be the
default and converts the others to additional applications, while Expand
Selected Games creates individual entries from those applications. Surviving
13.27 semantic signatures expose
`AdditionalApplication.GetFromGame(newGameId, game, priority, region,
version)`, `Game.GetFromAdditionalApplication(app, title, region, version,
platform, originalGame)`, `Combining.Combine(games, rootGame)`, and
`Combining.Expand(games)`. The value-free real-install census independently
shows that every observed combined group retains an additional-application
representative for the primary/default launch path.

The implemented subset therefore creates or reuses one launch representative
for every selected same-platform game, retains the chosen root game, moves
every modeled owned record to it, and atomically remaps clone relationships,
platform/category/playlist last-game state, manual playlist membership, and
import-blacklist rows. Expansion consumes only launchable version rows:
automatic helpers and document extensions remain attached, the representative
equal to the root launch is consumed without creating a duplicate, and every
other representative becomes a new game cloned from the root's otherwise
unrepresentable presentation data. Application-owned saves follow their new
game. Stored paths remain lexical values, unknown XML is retained wherever the
source record survives or is cloned, and neither operation moves or deletes
ROM/media files. This is static-contract-, documentation-, real-census-, and
fixture-derived evidence; a supported Windows runtime oracle is still needed
for field-by-field 13.27 parity and collapse/presentation behavior.

The concrete 13.27 `GameSave` contract recovers 16 persisted fields: owning
game/additional-application IDs, emulator filename/core, title, group name and
ID, display chip, match lineage, migration family, lexical file path, original
filename, slot, reported byte size and modified timestamp, and MD5. Runtime-only
base properties such as `IsDirectory` are not persisted by that concrete
record. The port now models all 16 in both readers and retains them losslessly.
Its Qt Saves manager treats an existing group ID as authoritative and gives
each ungrouped legacy row a separate in-memory key rather than guessing a 13.27
migration. Rename, combine, and split edit only title/group fields through an
indexed XML transaction. File ownership and emulator operations remain outside
that metadata transaction. Static 13.27 RetroArch code recovers the exact
configuration keys, `fileName*.*` and `.state*` scans, `.srm`/`.mcr`
preference, state-slot parsing, Saturn extension ranking/group ID, companion
copy, and CRLF-manifest signature rules. A platform-neutral RetroArch adapter
implements those semantics against native or mapped host paths and persists
only newly discovered active rows. The first filesystem-backed operation backs
up a resolved regular active file or complete present Saturn
`.bcr`/`.bkr`/`.smpc` set: it uses collision-free
`Saves\<Platform>\<ROM name>[-NN].<ext>` targets, captures exact aggregate
size/seven-digit UTC modified time/MD5, and commits revision-checked streamed
copies plus the full new row as one recoverable transaction. Active files are
read-only during backup. The port can also delete one resolved regular-file
vault copy or complete Saturn set and its source-indexed row in a single
revision-checked transaction with exact file and XML recovery copies. Plain
regular-file restore requires one compatible active row in the same stable
group, first commits and
verifies a new vault version of the active bytes, then revision-checks and
atomically replaces the active path from the selected vault file while
retaining an exact sibling recovery copy. RetroArch Saturn restore first
commits and verifies the complete current companion set, then revision-checks
and atomically replaces or creates all selected companions while retaining
active members absent from the selected version like 13.27. Other-emulator
scanning and other directory/container backup remain open. Active deletion now
first
atomically replaces one regular or Saturn active row with its verified portable
vault set, then revision-checks and deletes even host-mapped external live files
with exact sibling recovery copies and all-or-rollback behavior.
The 13.27 Dolphin plugin restores state and GameCube rows with ordinary
overwrite-copy and removes ordinary paths with `File.Delete`; it uses recursive
directory replacement/deletion only for Wii save folders. The port therefore
implements its second native save adapter for regular Dolphin rows. It derives
disc IDs from raw ISO/GCM headers, the recovered WAD ticket/title offset, or a
sibling DolphinTool for RVZ/GCZ/WIA/WBFS images; checks portable and
platform-native user roots; applies the recovered disc-region mapping; and
discovers preferred GameCube folder files, matching Card A/Card B GCI files,
and exact two-digit `StateSaves` slots. The adapter emits recovered group IDs,
names, and card chips, and its stable GameCube identity prevents duplicates
after a stored path moves. Separate controller and real-button Qt scenarios
prove append-only persistence, full owner/hash/time/size metadata, exact XML
recovery copies, and cleanup. Those regular rows then use the existing
revision-checked backup-first restore and active-delete paths. The adapter
recognizes `dolphin:wii:` as the explicit directory boundary and never claims
Wii directories or other unrecognized directories as ordinary files.

The recovered 13.27 PCSX2 installer implements `GetCurrentVersion`,
`GetInstallableVersions`, `InstallEmulator`, and update checking through the
emulator-plugin contract. It queries GitHub releases, chooses the first
compatible Windows Qt 7z asset, downloads with progress/cancellation, extracts
into an existing emulator directory or `Emulators/PCSX2`, creates the portable
marker, runs the first-time wizard, and returns either the repointed existing
definition or a new definition with recovered defaults and platform mappings.
The port keeps that first-compatible-release policy but moves provider I/O and
host artifact selection behind a platform-neutral Rust boundary. The official
PCSX2 documentation identifies AppImage/Flatpak as Linux distribution paths,
`portable.ini` or `portable.txt` as portable-mode markers, and `-version` as a
supported command-line option
([running](https://pcsx2.net/docs/setup/running/),
[configuration](https://pcsx2.net/docs/configuration/general/),
[CLI](https://pcsx2.net/docs/advanced/cli/)). The official GitHub release API
supplies an asset `digest` used as a required SHA-256 oracle. The port selects
the exact x86-64 Linux AppImage or Windows Qt 7z suffix, checks the official
release URL/name/size/digest, and commits the artifact, marker, port-owned
manifest, and emulator XML together without executing the downloaded program.
The Linux Nix runtime supplies `appimage-run`. The official macOS Qt tar.xz is
a single XZ stream containing one versioned `.app`; the port bounds that stream,
audits the derived tar with the existing traversal/link checks, requires the
expected bundle executable, normalizes the install to stable `PCSX2.app`, and
records every regular file in the same ownership transaction. The official
bundle is x86-64 and requires Rosetta 2 on Apple Silicon. Native macOS runtime
execution and the recovered first-time wizard remain open.

The recovered 13.27 PCSX2 plugin uses the exact
`SERIAL (CRC).NN.p2s`/`.p2z` contract for ordinary state files, but treats each
logical memory-card save as a named member inside a `.ps2` card. The port's
third native save adapter preserves that boundary explicitly. It checks the
recovered executable-relative and legacy roots plus current PCSX2's
platform-native data root; current upstream PCSX2 defines Linux data under
`$XDG_CONFIG_HOME/PCSX2` or `~/.config/PCSX2`, with `memcards` and `sstates`
below it ([upstream source](https://github.com/PCSX2/pcsx2/blob/master/pcsx2/Pcsx2Config.cpp)).
It matches exact state serials and folder/raw-card members using the recovered
content/header serial, GameIndex title, alternate-title, ROM-name, `icon.sys`,
grouping, and single-context rules. The native filesystem supports logical
pages and 528-byte physical pages with spare/ECC data, traverses indirect/direct
FAT and directory chains, and rejects corrupt or unsafe paths without aborting
other-card discovery. Its mutation path allocates and frees FAT chains, extends
the root directory, serializes member/file directory entries, writes file
clusters, and regenerates each modified physical page's spare/ECC bytes with
the recovered 13.27 algorithm. It was also checked read-only against an
authentic 8.65 MB ECC-bearing card, where it found both logical members, while
an unformatted/invalid sibling card was isolated. Container rows retain the
card path and internal directory name without inventing a whole-card digest;
ordinary state rows use the existing regular-file transactions. PCSX2 manual
backup extracts the named member, creates and re-extracts a flat 7z, verifies
the recovered uppercase SHA-256 logical-folder manifest, rechecks the live card
member for a racing change, and transactionally stores the archive plus XML
without writing to the card. Restore/deletion then use that archive boundary to
build a private complete folder/raw card copy, validate the mutated logical
member, recheck archive and live-card revisions, and replace the entire card
with a retained recovery copy. Folder-card replacement uses two sibling renames
with explicit rollback; raw-card replacement is atomic and physical raw restore
requires spare/ECC pages, matching recovered 13.27. A real-button offscreen
lifecycle proves selected restore, mandatory pre-restore and pre-delete vault
versions, active deletion, two complete-card recovery trees, unrelated-member
retention, exact XML history, and transaction cleanup. Compressed/full
disc-image serial extraction and PCSX2 filesystem repair/capacity recovery
remain explicit gates.
The 13.27 RetroArch plugin does not override `EmulatorPlugin.RemoveSave`, so
the Windows implementation calls the base `File.Delete` on only the persisted
primary path. The port deliberately deletes the already-established Saturn
ownership set instead of orphaning `.bkr`/`.smpc` companions; the mandatory
vault copy and per-member recovery copies make that safety extension explicit.
Dolphin Wii directory handling, PCSX2 filesystem repair/capacity recovery, full
PCSX2 disc-image serial extraction, stale-row reconciliation, automatic policy,
repair, and the remaining adapters remain open.

Historical logs from the same read-only installation supply a narrow runtime
oracle for session statistics without exposing game names or paths. Across the
sampled launches, LaunchBox incremented `Game.PlayCount` and saved immediately
after process start, persisted `LastPlayedDate` at the launch instant using a
local UTC offset and seven fractional digits, then added the returned elapsed
whole seconds to `PlayTime` after the launch session ended. The port mirrors
that ordering for an isolated launch session. Descendants that remain in its
Unix process group or Windows Job Object extend the observed runtime after the
direct child exits. That supervision is an explicit cross-platform safety
extension, not a recovered 13.27 parity claim; deliberately escaped processes
and focus-based timing still require a live Windows oracle.

The recovered `StartupViewModelBase` constructor independently carries
`minimumStartupScreenDisplayTime` and `loadDelay`, while
`MainStartupViewModel` carries `minimumShutdownScreenDisplayTime` and
`disableShutdownScreen`. The option labels describe the minima in seconds, but
a privacy-preserving read of the older installation's two settings documents
shows the scalar values stored in milliseconds (`1000` for one second).
LaunchBox and BigBox also persist separate global enable, theme, and cursor
settings. The port therefore treats load delay as a pre-process delay and the
two minima as frontend presentation policy rather than conflating these three
values.

Pause recovery has a similarly explicit boundary. `MainPauseViewModel`
receives separate pause and resume scripts plus
`suspendProcessOnPause` and `forcefulActivation`; the shared
`MainPauseView` exposes `Display`, `CloseAndDispose`, and the static
`PauseInProcess` guard while importing foreground/window APIs. LaunchBox and
BigBox each persist `UsePauseScreen`, `PauseTheme`, `PauseScreenMuting`, and
`PauseScreenFading`, while emulator plugin defaults independently set
`UsePauseScreen`, `SuspendProcessOnPause`, and
`ForcefulPauseScreenActivation`. The port therefore resolves pause policy
separately from startup/shutdown policy. The recovered
`ProcessSuspender` imports `NtSuspendProcess`, `NtResumeProcess`, and
`OpenProcess`, but its protected method bodies do not establish target
selection or process-tree behavior. The port's isolated process-group/Job
Object suspension is therefore an explicit safety design rather than a claim
that LaunchBox 13.27 uses the same boundary. Protected bodies still prevent a
claim about exact AutoHotkey script order, cross-window forceful activation,
mute/fade timing, or controller/global-key behavior; those remain oracle tasks.

The port keeps these stored Windows paths lexical in LaunchBox XML. A separate
versioned host-mapping document translates drive and UNC roots only at the
platform/process boundary. Windows path classification, separator handling,
mapping resolution, and the OS-specific mapping-document location all live in
`lb-platform`; the storage and Qt layers do not reinterpret persisted strings
with host-native path rules. Deterministic Qt coverage creates drive and UNC
mappings, removes one, restarts from LaunchBox into BigBox, and successfully
launches a Windows-path fixture through the retained mapping without rewriting
the fixture library.

## Decompiled scope

ILSpy 9.1 decompiled 16 first-party assemblies as nested C# projects with the
installed `Core` directory used for reference resolution.

Entity counts below come directly from assembly metadata. `C# files` counts the
generated project output. Delegates are unusually numerous because of compiler
generation and protection machinery.

| Assembly | Bytes | Classes | Interfaces | Structs | Delegates | Enums | C# files |
|---|---:|---:|---:|---:|---:|---:|---:|
| `LaunchBox.dll` | 20,932,096 | 2,267 | 10 | 47 | 6,278 | 33 | 7,165 |
| `BigBox.dll` | 10,198,528 | 698 | 2 | 24 | 1,785 | 9 | 2,029 |
| `Unbroken.dll` | 516,608 | 78 | 0 | 34 | 196 | 6 | 197 |
| `Unbroken.LaunchBox.dll` | 3,042,304 | 214 | 1 | 31 | 1,351 | 18 | 1,415 |
| `Unbroken.LaunchBox.LocalDb.dll` | 303,104 | 47 | 0 | 33 | 0 | 0 | 20 |
| `Unbroken.LaunchBox.Plugins.dll` | 102,400 | 56 | 31 | 1 | 0 | 2 | 90 |
| `Unbroken.LaunchBox.SourceGenerators.dll` | 45,056 | 9 | 0 | 0 | 0 | 0 | 2 |
| `Unbroken.LaunchBox.Windows.dll` | 64,007,680 | 1,355 | 15 | 110 | 8,742 | 78 | 9,355 |
| `Unbroken.LaunchBox.Windows.BigPEmu.dll` | 19,456 | 9 | 0 | 0 | 0 | 0 | 3 |
| `Unbroken.LaunchBox.Windows.Dolphin.dll` | 64,000 | 37 | 0 | 1 | 0 | 2 | 12 |
| `Unbroken.LaunchBox.Windows.Mame.dll` | 28,672 | 20 | 0 | 1 | 0 | 0 | 9 |
| `Unbroken.LaunchBox.Windows.Pcsx2.dll` | 163,840 | 83 | 6 | 11 | 0 | 1 | 38 |
| `Unbroken.LaunchBox.Windows.PlaylistProvider.dll` | 17,920 | 9 | 0 | 0 | 0 | 0 | 6 |
| `Unbroken.LaunchBox.Windows.RetroArch.dll` | 135,168 | 38 | 0 | 2 | 0 | 0 | 10 |
| `Unbroken.LaunchBox.Windows.ScummVm.dll` | 21,504 | 11 | 0 | 0 | 0 | 0 | 5 |
| `Unbroken.LaunchBox.Windows.Xemu.dll` | 28,672 | 14 | 0 | 0 | 0 | 0 | 5 |
| **Total** |  | **4,945** | **65** | **295** | **18,352** | **149** | **20,361** |

Other recovered structural evidence:

- 329 LaunchBox BAML resources, including 257 desktop views
- 31 BigBox BAML resources
- 29 shared BAML resources
- 125 readable XAML files across the two installed BigBox themes
- 129 semantic LaunchBox desktop menu-action classes
- 289 semantic LaunchBox desktop view-model classes
- 23 semantic BigBox menu-action classes
- 102 semantic BigBox view-model classes
- 89 readable plugin API contract files
- no ILSpy failure markers in the generated C# tree

The full paths and names are captured in
[`analysis/static-inventory.json`](../analysis/static-inventory.json).

The real 13.24 rows establish the exact alternate-name shape as `GameID`,
`Name`, and optional `Region`. Because that installation contains no
`CustomField` rows, custom-field persistence is not attributed to the older
library. Installed 13.27 metadata instead exposes `ICustomField` with game ID,
name, and value, while the concrete serializer maps `GameID`, `Name`, and
`Value`. The port's synthetic fixture and lossless editor use that exact
contract; live 13.27 creation/edit behavior remains an oracle task.

Platform editing has a similarly explicit static boundary. The 13.27
`IPlatform` contract makes `Name` getter-only while exposing the recovered
metadata and folder APIs, and `AddEditPlatformViewModel` contains the expected
metadata/folder/document edit surface. The port now edits those mutable fields
and source-indexed folder rows losslessly, but deliberately keeps platform
identity read-only because the protected save body and failing Wine runtime do
not establish the required game/emulator/playlist/parent/controller/settings
and filename rename behavior.

Playlist editing now has the same explicit evidence boundary. `IPlaylist`
exposes getter-only `PlaylistId` and unique `Name`, mutable nested display name,
sort/notes/video/image/category/last-game/BigBox metadata, inclusion and
auto-populate flags, ordered games, filters, and children. The add/edit view
model exposes the corresponding metadata, hierarchy, manual-game, and
auto-filter surfaces. Installed resource text says every playlist needs at
least one hierarchy location and that Delete permanently removes all instances
while a single location is removed through the Parents tab. The value-free
13.24 census contains 51 auto-populated documents, 3 manual documents, 104
filters, and 955 playlist-game rows. Repeated filters sharing a field occur
alongside filters for distinct fields; the implemented membership evaluator
uses the observed LaunchBox grouping of OR within one field and AND across
fields. Deletion removes the playlist document and owned placement/cache rows,
detaches direct children to root, and never treats membership as ownership of
game XML or media. Live 13.27 runtime comparison remains pending.

The 13.27 installation also yielded a concrete DOSBox mount contract despite
the protected method bodies. `IMount` exposes game ID, drive letter,
filesystem, mount type, path, and media type. The installed strings identify
`Folder` and `File` mount types; `Floppy`, `CD-ROM/ISO`, and `Hard Disk` media;
and the `MOUNT`/`IMGMOUNT` command templates and `-t`/`-fs` flags. Resource text
confirms that these mounts are DOSBox-only, while custom DOSBox executable,
configuration, C-drive root, and configuration-owned `[autoexec]` paths are
separate concerns. This evidence drives the native adapter and synthetic
Linux runtime fixture; live Windows-oracle parity is still pending.

LaunchBox 13.27 contains two distinct ScummVM paths. Its modern
`Unbroken.LaunchBox.Windows.ScummVm.dll` emulator plugin creates a normal
emulator entry whose recovered default command line is
`-p %romfile% --auto-detect --fullscreen`. The platform XML contract also
retains the legacy built-in `UseScummVM`, game-data-folder, target-ID,
fullscreen, and aspect-correction fields. Installed 13.27 strings recover the
legacy `--no-console`, `--savepath`, `--extrapath`, `-p`, `-f`, and
`--aspect-ratio` vocabulary plus the bundled
`ThirdParty\ScummVM\ScummVM.exe` location. Resource text establishes that the
game-data folder is required and that legacy ScummVM is mutually exclusive
with DOSBox. The native adapter follows those semantic argument boundaries and
resolves the persisted folder only at the host-platform boundary; exact live
Windows-oracle argument ordering remains pending because the method bodies are
protected.

## Protection boundary

The assemblies are not ordinary unobfuscated .NET applications:

- 5,633 of 7,165 LaunchBox source filenames, 1,588 of 2,029 BigBox
  filenames, and 7,881 of 9,355 shared Windows filenames contain generated
  `Transformer`-style names.
- At least 1,638 first-party source files contain a simple public `NoInlining` method that
  decompiles to an empty body; default-return accessors add more protected
  methods beyond that proxy count.
- Semantic classes survive, but implementations do not. For example,
  `ImportSteamGamesMenuAction.OnSelect()` is empty and `AttractMode`
  properties return constants in static output.
- Protector helpers resolve `getJit`, patch executable memory with
  `VirtualProtect`/`mprotect`, prepare method handles, and redirect JIT
  compilation. This is direct evidence that runtime code differs from the
  stored IL.

Therefore:

1. The decompile is complete as a structural extraction.
2. It is not complete as recovered behavior.
3. No porting percentage should be derived from the 20,361 generated files.

## Next reverse-engineering work

1. Establish a working runtime oracle in Wine or a disposable Windows VM.
2. Create a tiny deterministic library fixture and capture first-run files,
   XML/SQLite schemas, settings, screenshots, process launches, and file diffs.
3. Turn every feature-matrix row into one or more repeatable oracle scenarios.
4. If UI observation is insufficient, instrument the CLR JIT boundary and dump
   the post-protection IL supplied for selected semantic methods before it is
   compiled. Keep token/MVID mappings so dumps remain attributable.
5. Decompile runtime-recovered IL into a separate provenance-labeled tree; do
   not overwrite the static extraction.
6. Map external network behavior only with owned accounts and valid licenses;
   redact credentials and tokens from captures.
