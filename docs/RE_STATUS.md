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
reference exists. Save management and Make Default are separate upstream
surfaces and remain unimplemented.

Historical logs from the same read-only installation supply a narrow runtime
oracle for session statistics without exposing game names or paths. Across the
sampled launches, LaunchBox incremented `Game.PlayCount` and saved immediately
after process start, persisted `LastPlayedDate` at the launch instant using a
local UTC offset and seven fractional digits, then added the returned elapsed
whole seconds to `PlayTime` after the launch session ended. The port mirrors
that ordering for the directly spawned primary child; process-tree and
focus-based timing still require a live Windows oracle.

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
