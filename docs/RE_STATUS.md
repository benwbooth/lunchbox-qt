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
| LaunchBox UI | renders after a repeatable approximately 60-second focus timeout |
| BigBox UI | library loads, but Wine's WPF surface remains black |

The setup payload was intact. The original automatically launched process died
because Wine's built-in UI Automation runtime does not implement
`UiaSetFocus`; a managed first-chance exception hook recovered the exact
`RawUiaSetFocus`/`AutomationElement.SetFocus` stack behind the generic
`0xe0434352` and `0x80004005` report. The isolated prefix now selects a
compatible native UI Automation runtime already present locally, while
retaining both Wine 11.8 DLLs as reversible backups. The full LaunchBox desktop
now paints without the diagnostic hook after its activation handler waits
almost exactly 60 seconds and catches a focus failure.

BigBox has a distinct boundary. With the owner's ignored license and a working
copy of the older real data snapshot, it loads all 35,869 games and creates its
real navigation stack, but its WPF surface is black. The stock theme raises
`0x88980406` at `DUCE.Channel.SyncFlush`; a simplified Old Default text view
avoids that exception but still produces an all-black capture. WPF software
rendering, WineD3D, DXVK, a Wine virtual desktop, a clean Xvfb display, and
theme/media simplification did not fix it. LaunchBox is therefore a usable but
slow Wine visual oracle; BigBox visual and interaction parity still requires a
supported Windows runtime. The exact diagnosis, hashes, reversible prefix
state, and negative experiments are in
`analysis/wine-oracle-13.27.md`.

Wine remains a proven managed-code oracle. A temporary self-contained
.NET 9 `win-x64` reflection host was placed beside the installed first-party
assemblies and run inside the isolated prefix. It loaded and invoked protected
13.27 model-settings code successfully. The probe and copied host artifacts
were removed after recording the non-user-specific contract below; no
proprietary binaries or probe output are checked in.

## Application-data backup evidence

The 13.27 desktop resources expose `_Create Data Backup...`, `_Restore Data
Backup...`, one `AutoBackupLaunchBox` option, and text specifying that the
contents of `LaunchBox\Data` are automatically archived on startup and
shutdown of both frontends. The archives live under `LaunchBox\Backups` and up
to 25 automatic archives may be retained. A fresh ignored 13.27 data tree has
eight core root XML files plus `Platforms/` and `Playlists/`, with
`AutoBackup=true`.

Observed automatic `.7z` files use frontend/event/timestamp names. Their
technical listings place `Data` contents directly at archive root, retain the
two document directories, and carry optional and nested documents without an
enclosing `Data/` folder. No user game/platform names, paths, or XML values are
checked in. The protected create, restore, and options bodies remain stubs in
the structural decompilation, so exact dialog/error sequencing and protected
automatic scheduling remain unavailable.

The clean-room native subset uses those recovered storage facts for manual
create/restore plus automatic startup/shutdown in both frontends. It retains
unknown safe data, rejects links,
traversal/collisions, malformed core XML, and bounded-size violations, calls
7-Zip directly without a shell, re-extracts every new archive for exact
content verification, and restores through a revision-checked atomic directory
replacement with a retained recovery tree. Typed `AutoBackup` persistence,
all four exact archive names, shutdown waiting, shared cross-process locking,
and strict newest-25 retention preserve manual/unrecognized archives. The
evidence, portable scheduling decision, port-owned limits, and compiled
scenarios are recorded in `analysis/data-backup-13.27.md`.

## Game-audit evidence

The structurally recovered 13.27 `AuditViewModel.AuditEntry` exposes one
internal game reference, four date-sort helpers, and exactly 76 visible audit
values. The enclosing model exposes selection, edit, image/video/URL commands,
and distinct selected/duplicate row brushes; the view identifies a themed
`DataGrid`. Its platform-optional menu action establishes all-games and
current-platform scope.

Embedded 13.27 release notes add behavior that protected bodies do not expose:
the audit can span the complete collection, repeated LaunchBox Games Database
IDs identify duplicate rows, copied spreadsheet data includes headers, and a
platform change removes the game from a platform-scoped audit. Those facts are
now frozen in the first native `LIB-015` audit subset. It projects the complete
76-column surface, highlights repeated non-zero database IDs, sorts and
selects rows, exports bounded header-bearing TSV through Qt's clipboard, and
opens the existing full game editor by stable ID. MAME-derived and unavailable
storefront values stay blank rather than becoming fabricated false results.

Pure tests and a compiled rendered workflow require the ordered contract, two
differently titled games sharing one database ID, two-row TSV output, the Edit
transition, a valid screenshot, and byte-identical source XML. Exact protected
column/context behavior, authoritative MAME/storefront sources, and native
Windows/macOS interaction remain open. The full
boundary is recorded in `analysis/game-audit-13.27.md`.

## Game bulk-edit evidence

Five distinct 13.27 view models establish Welcome, typed field selection,
conditional Platform/media migration, confirmation, and apply/result pages.
The field view model and BAML prove text, multiline, file, boolean/three-state,
combo/editable-combo, date, half-star, model, controller-support,
multi-value-add/remove, and custom-field editor families. Embedded release
notes independently name star ratings, date confirmation, direct additional
changes, Emulator clear, platform media migration, multi-select add/remove,
Custom DOSBox Version, Broken, emulator propagation to matching additional
applications, Video Path, Custom Fields, and Hide.

A temporary runtime hook attached after the disposable Wine 11.8 prefix's
missing matching `cryptbase.dll` files were restored. LaunchBox then entered
its known activation/data-manager critical-section stall before its protected
field initializer populated the collection, so the complete original ordered
field list is not claimed.

The first native subset freezes a 25-field typed Rust catalog and validates
every field/operation/value combination. It copies stable audit-selected game
IDs, groups them by exact source document, applies lossless DOM mutations, and
stages all documents into one revision-checked recovery transaction. The
compiled wizard renders confirmation and proves a two-game commit, exact
backup, unknown XML and lexical path retention, and clean recovery state.
Platform/media migration, remaining bulk surfaces, emulator propagation, and
exact Windows-oracle behavior remain open. See
`analysis/game-bulk-edit-13.27.md`.

## Older real installation schema

A complete LaunchBox 13.24 installation was located on the owner's read-only
Windows partition. It provides genuine scale and data-shape evidence even
though it is older than the frozen 13.27 binary oracle.

`scripts/analyze_launchbox_schema.py` produced
`analysis/real-install-schema.json`, a value-free census that records element
and attribute names plus aggregate record/media-extension/path-shape counts,
but no filenames, text values, stored paths, accounts, or license data. It
found 37 platform files, 35,869 games, 16,752 additional applications, 20,739
alternate names, 54 playlists, zero custom-field records, and no XML parse
errors.

The same value-free census found 86 global `GameController` records and 7,061
per-game `GameControllerSupport` rows. A temporary managed 13.27 reflection
probe recovered the exact 11 category choices and four support levels; the real
library corroborates optional level-zero encoding and contains the historical
category spelling `Rythm`. The native `LIB-005` vertical now provides
typed/lossless controller catalog CRUD, semicolon platform-name associations,
per-game support editing, immutable generated IDs, cross-document
reference-gated deletion, exact backups, and rendered Qt coverage. The complete
evidence and remaining hardware/BigBox/native-host boundaries are recorded in
`analysis/controller-support-13.27.md`.

The 13.27 resource strings independently establish that deleting a platform
removes its associated games and placements and moves nested categories and
playlists to root. Separate prompts cover game-folder and media deletion, so
those filesystem choices are not implied by platform-record removal. Protected
method bodies remain unavailable. The native `LIB-006` subset therefore exposes
an explicit second review and implements a documented bounded record-only
policy: one recoverable transaction removes the platform and its game-owned XML
records, remediates all modeled emulator, parent, playlist, navigation,
controller, settings, blacklist, clone, and retained-platform references,
retains exact backups, reloads committed models, and never resolves or deletes
ROM, media, manual, music, video, emulator, or save paths. Evidence and
clean-room boundaries are recorded in `analysis/platform-removal-13.27.md`.

The same 13.27 resource boundary distinguishes permanently deleting a game
from the local collection from the separate ROM, additional-application ROM,
and associated-media removal prompts. `AllowDeletingRoms` and
`DeleteAssociatedMediaOnGameDelete` are independent persisted settings.
Protected game-delete bodies remain unavailable, so the native `LIB-001`
subset uses an explicit exact-ID second review and a bounded record-only
policy. One recoverable transaction removes the game and its owned XML rows,
clears modeled clone, playlist/navigation, platform-navigation, blacklist, and
affected list-cache references, keeps exact backups, and reloads the committed
library. It never resolves a stored path or deletes a ROM, additional-app file,
media item, manual, music track, video, save, or directory. Storage,
controller, and rendered Qt tests prove a five-document remediation, seven
exact recovery copies across the complete add/remove/remediate workflow,
unrelated unknown XML retention, and byte-exact representative file retention.
Evidence and clean-room boundaries are recorded in
`analysis/game-removal-13.27.md`.

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

A separate read-only aggregate media audit recovered the Box Front filename
contract without retaining titles or paths. Across the 37 platforms it found
23,880 unique front-image stems and matched 23,793 games after replacing
Windows-invalid punctuation plus apostrophes with underscores and removing a
trailing numeric `-NN` image ordinal. In the Sega Master System sample all 293
games matched all 293 image stems exactly under that rule. Fifty-eight games
had multiple candidates, confirming that image ordering must be deterministic.
Observed front art used JPG, PNG, GIF, and TIF files beneath media-type and
optional region directories. This is evidence from the older 13.24 library,
not a claim that every 13.27 priority or naming edge case is recovered.

The model-preview boundary now has static and managed-runtime evidence. The
recovered 13.27 desktop `FullscreenModelPreviewViewModel` and BigBox
`ModelPreviewViewModel` expose rotate, translate, zoom-in, and zoom-out
operations; `FlowModelRotationLockMode` has `None`, `LockToY`, and `LockToX`.
The runtime probe recovered the exact persisted `ModelType` key/display pairs:
`box`/Box, `dvd`/DVD Case, `jewelCase`/Jewel Case, and
`longJewelCase`/Long Jewel Case.

A newly constructed `ModelSettings` has nullable colors, image, font, model
type, identity, and size; `FrontSpineIsClear=false`;
`FullImageSpineWidth=0.143`; `FullScanIsLandscape=false`;
`LogoRotation=0,0,0,`; `SpineRotation=0,,0,`; and
`UseFullScanImages=false`. The four editor defaults are:

- Box: black case, full scans enabled, spine width `0.088`, no forced size.
- DVD: black case, white cover, full scans enabled, spine width `0.065`;
  the visible size controls start at `0.7,1,0.065` but forced size is off.
- Jewel and long-jewel: black cover, white text, Segoe UI, full scans off,
  and spine width `0.143`.

The persisted record is a root-level `<ModelSettings>` element. Its recovered
fields are `CaseColor`, `CoverColor`, `FrontSpineImage`,
`FrontSpineIsClear`, `FullImageSpineWidth`, `FullScanIsLandscape`,
`GameId`, `LogoFont`, `LogoRotation`, `ModelSizeString`, `ModelType`,
`PlatformName`, `SpineRotation`, and `UseFullScanImages`. Colors are signed
32-bit ARGB integers and forced sizes are three positive semicolon-separated
numbers such as `5;7.165;1`; rotation strings and `{Resources}\...` values are
opaque LaunchBox data, not host paths.

Resolution is whole-record precedence: a matching per-game record in its
platform document wins over a matching per-platform record in
`Data/Platforms.xml`, followed by the built-in platform name, then its
`ScrapeAs`, then a box fallback. The runtime probe enumerated all 41 built-in
platform names. It distinguishes jewel cases for Amiga CD32, CDTV,
TurboGrafx-CD, CD-i, Dreamcast, Neo Geo CD, and PlayStation; long jewel cases
for Sega CD and Saturn; and DVD cases for the other mapped platforms. It also
recovered platform colors, the forced `5;7.165;1` Genesis/Master System size,
and the clear `{Resources}\{platform}` front-spine value for Dreamcast and
PlayStation. Domain tests require every recovered platform to resolve and
freeze the exceptional values.

The read-only 13.24 installation currently contains 20,658 PNG and 7 JPEG files
under its configured `Box - 3D` directories. Its settings retain
`Box3dImageTypePriorities=Box - 3D`, `RotateModelDefaultsApplied=true`, and
`ShowDetails3dModel=true`; BigBox retains
`ShowGameMenuViewModelFullscreen=true`, `ModelWheelMinimumSpeed=400`,
`Use3dModelCoverFlow=true`, and `Use3dModelImageView=true`. The downloaded
`Box - 3D` flat image family is therefore kept distinct from the interactive
geometry contract and from the separate front/back/spine inputs used by the
first port model vertical.

The same installation also contains real `Box - Full` scans. Inspected
`1090x680` examples use a back-spine-front horizontal layout, with the narrow
spine centered between the two covers. That observation supports the port's
full-scan crop direction; the persisted `FullImageSpineWidth` remains the
authoritative ratio for each resolved record rather than a hard-coded pixel
width.

The port now losslessly reads and writes these records through both platform
readers and the catalog reader, retains unknown future model keys/elements,
validates signed colors/sizes, and resolves
game/platform/built-in/fallback settings once at the Rust controller boundary.
The game and platform dialogs edit the complete recovered property surface and
remove an override to restore inheritance. QML receives a versioned typed
presentation; it never interprets LaunchBox path syntax. The shared viewer
applies recovered case/cover colors and forced aspect/depth, presents distinct
functional box, DVD, jewel, and long-jewel proportions with jewel tray lips
and a DVD hinge band, and splits a selected `Box - Full` scan into
back/spine/front regions when enabled. LaunchBox and BigBox runtime smokes
require the fixture's game-level jewel override to beat its platform DVD
override, require exact port-owned `260x230x20` jewel geometry and the `0.143`
scan ratio, and render the result before accepting the interaction. Separate
transactional smokes prove exact game/platform model edits plus reviewed game
and platform removal cleanup.

These Qt shapes are a functional cross-platform presentation, not a claim of
original WPF mesh or camera parity. Front-spine/logo/rotation material
rendering, CoverFlow and image-view integration, exact
materials/meshes/camera/timing/default input, native controller bindings, and
real Windows/macOS UI execution remain open.

The recovered 13.27 stock `GameDetailsView.xaml` binds `MediaList.SelectedItem`
to `MediaPreview.MediaItem`; the concrete `MediaItem` exposes location, preview,
full-size preview, and an `IsVideo` discriminator. `GameDetailsViewModel`
exposes screenshot/fanart collections, clear-logo/background images, video
visibility, pause/resume, and volume control. The plugin contract separately
exposes explicit video paths and all image details. `VideoTypes.cs` names the
five exact families Recording, Theme Video, Trailer, Video Snap, and Marquee,
and the installed resource list recovers 28 accepted video extensions. The
older real installation's value-free video census found 27,429 MP4 files and
the expected Theme, Trailer, Recordings, and Marquee subfolders. Its settings
enable details video and autoplay with Theme Video, Video Snap, Recording, then
Trailer priority. These facts establish the current read-only selected-game
media contract; they do not establish every codec, theme, platform-video, or
media-management behavior.

Manual and game-music behavior has a separate static boundary. The recovered
13.27 game surface names `GetManualPath`, `GetMusicPath`, `OpenManual`, and
`HasMusic`, while desktop and BigBox resources contain the visible View
Manual, Play Music, and Stop Music actions. The shared static `Music` surface
names play, stop, pause, resume, next, previous, track switching, and volume,
but its protected bodies do not disclose exact queuing or error behavior.
LaunchBox settings expose `AutoPlayMusic` and `ShuffleMusic`. BigBox settings
separately expose `AutoPlayMusicGamesList`, `AutoPlayMusicGameDetails`,
`PrioritizeMusicOverVideoAudio`, `RepeatGameMusic`,
`ShuffleSoundtrackMusic`, `ShowGameMenuPlayMusic`,
`ShowGameMenuViewManual`, and `VolumeMusic`.

The privacy-preserving 13.24 census makes the storage shape concrete without
retaining a title or path. `Manuals` contains 5,547 regular PDFs and two
symlinks; `Music` contains 8,441 MP3 files and 34 M3U documents. Both trees
have exactly a platform-directory plus filename below their root. Game XML
contains 4,240 nonempty manual paths, all relative: 3,182 PDF, 927 TXT, 113
DOC, 14 JPG, two PNG, one HTM, and one RTF. One contains a parent component,
which the census records only as a count. It contains 133 nonempty relative
music paths: 111 MP3, nine MOD, eight OGG, three M3U, one S3M, and one XM.
The older settings snapshot disables LaunchBox autoplay and enables shuffle;
its BigBox snapshot enables list/details autoplay and both menu actions,
disables repeat and soundtrack shuffle, and stores music volume 75. These
aggregate facts establish explicit paths, conventional per-platform fallback,
local playlists, legacy module formats, separate frontend policy, and
symlink-aware indexing. They do not establish protected M3U parser quirks,
every decoder, or original error/notification
behavior.

The 13.27 BigBox settings contract separately exposes
`EnableBackgroundMusic`, `VolumeBackgroundMusic`,
`EnableMusicOnScreenDisplay`, `ShuffleBackgroundMusic`,
`UsePlatformPlaylistCategorySpecificBackgroundMusic`, and
`PlayVideoAudioWithBackgroundMusic`. The release resources name the exact
default `Music\Background` folder plus
`Music\Background\Platforms\[Platform Name]`,
`Music\Background\Playlists\[Playlist Name]`, and
`Music\Background\Platform Categories\[Platform Category Name]`. The same
resources expand automatic background-music recognition with SID, AC3, ALAC,
AMR, DTS, XM, IT, MOD, APE, OPUS, QCP, NSF, and SPC. Two read-only older
settings snapshots agree on disabled background music, volume 75, enabled OSD,
enabled shuffle, and enabled context-specific music; the older contract does
not contain the later video-audio coexistence setting. These facts establish
folder selection and typed policy inputs, but not protected shuffle order,
notification timing, decoder-specific behavior, or custom-theme overrides.

Application startup presentation is a separate BigBox path. The recovered
`StartupVideoView` is a dedicated window whose constructor accepts the primary
monitor index, selected playback engine, and video priorities; its control
carries one video path, and `App` retains the startup-video process plus
`GetStartupVideoPath()`. Release resources establish the observable video
storage contract in two steps: the original feature uses exact
`Videos\Startup.mp4` instead of the normal splash, while the later random-video
feature uses direct files placed in `Videos\Startup`. The same release notes
state that a selected video runs for its full length unless skipped with a key
or button. The 13.27 settings contract exposes `VideoPlaybackEngine` and
`VolumeVideo`; the older read-only snapshot stores VLC and volume 75. No
separate video-enable scalar appears, and the inspected older installation
contains neither video location, so the port treats presence of a safe
supported file as enablement and absence as the no-video path.

The no-video path has additional concrete evidence. Recovered
`BigBoxSettings` properties and the value-free 13.27 schema contain
`ShowStartupSplashScreen`, `PlayStartupSound`, `SoundPack`,
`VolumeStartupSound`, and `VolumeMaster`. Fresh 13.27 Wine settings and the
older real installation both store enabled splash and sound, startup/master
volumes 100, and `Sci-Fi Set 3 by Clavius`. Installed sound packs demonstrate
exact single-file `Sounds\<pack>\Startup.wav` and multi-file
`Sounds\<pack>\Startup\*.wav` layouts. Release resources date separate startup
volume to 6.10 and multiple sounds per sound type plus theme-packaged sounds to
11.13. The recovered BigBox resource set also contains a proprietary
transparent startup logo; it is evidence for the branded splash, not an asset
the port redistributes.

These facts establish video location precedence, random selection,
completion/skip, the splash/sound toggles, selected pack, both installed WAV
layouts, and typed volume inputs. They do not establish protected random
generator details, video-priority behavior, whether startup sound coexists
with a startup video, custom-theme sound precedence, exact artwork/animation
timing, error notifications, controller binding, or monitor/focus behavior.
The port therefore starts video alone when one is available and otherwise
uses its own non-proprietary splash plus the selected standalone sound pack;
those unresolved behaviors stay explicitly open.

The implemented cross-platform boundary therefore keeps XML paths lexical,
resolves them once through the existing native/mapped-Windows service, validates
bounded regular files, rejects remote/nested/traversing M3U entries, and gives
Qt only local URLs. Background contexts additionally use the shared
Windows/Linux/macOS-safe component transform and reject case-normalized
ambiguity. A fresh read-only compiled load indexed 5,539 manuals and 8,412
game-music tracks for the 35,869 games in 36.314 seconds. It scanned all
14,022 regular manual/music files across 70 present platform folders, refused
the two symlinks, and reported no oversized or truncated input. That older
library had no background collection. Missing optional background folders are
therefore treated as absent, while missing explicit/configured game targets
stay unresolved rather than being guessed.

The installed CriticalZoneV2 BigBox `TextGamesView.xaml` independently places
an `ImageView`, an `ImageVideoView` explicitly marked as video content, and a
`GameDetailsView` beside the game list. Its game-details resources bind title,
details, notes, rating, favorite, portable, completed, and broken state. The
shared `InputAction` contract includes `BigBoxShowGameDetails`,
`BigBoxShowImages`, `BigBoxSwitchImageType`, `BigBoxZoomIn`,
`BigBoxZoomOut`, `BigBoxPlayGame`, directional and page navigation, Select,
and Back. `MainViewModel.ShowImages(Game, ChildViewModelBase)` enters the
dedicated `ImagesViewModel`; that view model exposes enter, escape, four
directions, page-up, and page-down handlers plus an image path and detail
label. The protected bodies do not recover exact zoom bounds, zoom increments,
pan increments, or action-to-direction mapping. This establishes separate
full-screen media/details presentation, image-type switching, zoom, pan-shaped
navigation, and controller-oriented entry as the first two `BB-004`
boundaries. It does not establish exact protected transition timing, every
configurable binding, theme-specific layout behavior, or
3D/related/document/application panes.

The same recovered `InputAction` contract separately names
`LaunchBoxShowImages`, `LaunchBoxZoomIn`, `LaunchBoxZoomOut`,
`LaunchBoxPageUp`, `LaunchBoxPageDown`, and `LaunchBoxFlipBox`, while
`Settings.AlwaysShowImagesFullscreen` exposes the desktop presentation policy.
The static 13.27 extraction also retains desktop
`BoxItemViewModel.FlipBox`, BigBox `GamesViewModelBase.FlipBox`, and matching
menu-action entry points, although their protected bodies do not recover the
transition. The owner's read-only 13.24 installation adds concrete settings
evidence: `BackImageTypePriorities` is exactly `Box - Back,Box - Back -
Reconstructed,Advertisement Flyer - Back,Fanart - Box - Back`;
`ShowGameMenuFlipBox` is enabled; and `KeyboardFlipBox` is WPF key code 49,
the `F` key. Its media tree contains the corresponding back-art folder
families. This establishes front/back selection, visible BigBox menu policy,
and the default keyboard action, but not exact protected transition timing,
controller binding, 3D-model behavior, or every theme-specific presentation.
The port therefore uses the recovered priority and visibility settings with an
explicit platform-neutral 220 ms Qt Y-axis transition rather than presenting
that duration as original behavior.

The compiled port subsequently loaded that same read-only installation through
the normal Qt worker transaction and selected 24,920 front images across the
full 47-entry persisted fallback list. It scanned 161,968 supported files in
1,144 media folders without unsafe, oversized, or truncated entries. The
debug Linux run over the mounted Windows filesystem completed in 27.301
seconds and delivered all 35,869 games to the model. The loader also had to
preserve two distinct `Arcade` mappings that the real library retains as
default; unpinned launch selection remains deliberately ambiguous rather than
silently choosing one.

After the selected-game index was generalized, a new read-only Qt load retained
the same 35,869 games and 24,920 front selections plus 211,120 detail media
items. It scanned 211,085 supported files across 1,802 folders in 34.852 seconds
with zero unsafe, oversized, or truncated entries. The 104 unresolved entries
are isolated configured or explicit media paths rather than a load failure.
This benchmark was run on Linux over the mounted Windows filesystem; native
Windows and Intel/Apple Silicon macOS media backend behavior remains a separate
release gate.

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
13.27 runtime parity remains unverified because these launch scenarios have not
yet been exercised against the repaired, slow-starting Wine desktop oracle.

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
directory replacement/deletion only for Wii save folders. The port implements
both shapes in its second native save adapter. It derives
disc IDs from raw ISO/GCM headers, the recovered WAD ticket/title offset, or a
sibling DolphinTool for RVZ/GCZ/WIA/WBFS images; checks portable and
platform-native user roots; applies the recovered disc-region mapping; and
discovers preferred GameCube folder files, matching Card A/Card B GCI files,
exact two-digit `StateSaves` slots, and Wii `data` directories. Disc saves
search recovered high IDs `00010000` and `00010004` with the first four disc-ID
bytes encoded as lowercase hex; WAD saves use the exact eight-byte title ID.
The adapter emits recovered group IDs, names, and chips, and its stable identity
prevents duplicates after a stored path moves. Regular rows use the existing
revision-checked backup-first restore and active-delete paths. Wii rows use
shell-free nested 7z creation, safe re-extraction, exact recursive revision
comparison, mandatory verified pre-mutation vault copies, rollback-capable
whole-directory replacement, and rename-based deletion with retained complete
recovery trees. Controller and real-button Qt scenarios prove discovery plus
dialog-confirmed restore/deletion, three vault versions, nested and empty
directories, both pre-mutation recovery trees, exact XML backups, and cleanup.
Other unrecognized directories are never claimed by this adapter.

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

The recovered BigPEmu plugin exposes Windows x64 and ARM64 plus Linux x64 and
ARM64 downloads from Rich Whitehouse's official page. That page publishes a
byte count and 64-bit FNV-1a value for each artifact. The Windows packages are
ZIP files with `BigPEmu.exe` at the root; Linux packages are one-wrapper
tar.gz files containing `bigpemu`, `ReadMe.txt`, runtime data, and an optional
`make_desktop.sh`. The port models those four identities explicitly, verifies
the page contract and downloaded FNV before extraction, computes SHA-256 for
durable ownership, and sends ZIP/tar members through the shared safe archive
boundary. It strips only the Linux wrapper, requires the executable and
readme, preserves execute mode, excludes the desktop helper, and directly
registers/launches `%romfile% -localdata` without a command shell. Managed
install, update, repair, stale-owned-file cleanup, and exact removal use the
same recoverable transaction and reference gates as PCSX2.

The official Xemu download page points Windows and macOS users to ZIP packages
and Linux users to AppImages. The official GitHub latest release exposes five
stable versioned targets: Windows x64 and ARM64 ZIPs, Linux x64 and ARM64
AppImages, and one signed universal macOS ZIP. It also exposes debug, unsigned,
and moving-alias artifacts that are not interchangeable release contracts. The
port therefore matches only the exact versioned stable filename for the current
host, requires the GitHub byte count and SHA-256 digest, and persists the exact
tag/name/URL relationship in its ownership manifest. Real current artifacts
establish the archive layouts: Windows has root `xemu.exe` and `LICENSE.txt`;
macOS has root `LICENSE.txt` plus `xemu.app`, whose executable is
`Contents/MacOS/xemu`. Linux installs the AppImage directly and routes launch
through the existing direct-argument `appimage-run` adapter. The provider never
starts Xemu during inspection or installation. Install, update, repair,
reference-gated removal, exact recovery copies, and preservation of
user-owned configuration/BIOS data share the same transaction boundary as the
other managed providers. Native Windows and macOS runtime behavior still needs
real-host verification.

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
grouping, and single-context rules. The recovered content-header path first
searches an ISO9660 primary volume and root directory for `SYSTEM.CNF`; plain
ISO and 2,336/2,352/2,448-byte raw sectors, GZip, CSO, CHD v5, MDF/MDS, and NRG
readers supply the logical sectors. The port implements those readers natively
with bounded scans, metadata and decompression limits, symlink refusal, and a
path/size/modified-time cache. Known disc formats do not accept the old fuzzy
prefix fallback after a failed filesystem parse. Unknown formats retain the
recovered bounded prefix fallback.

The native memory-card filesystem supports logical pages and 528-byte physical
pages with spare/ECC data, traverses indirect/direct FAT and directory chains,
and rejects corrupt or unsafe paths without aborting other-card discovery. Its
mutation path allocates and frees FAT chains, extends the root directory,
serializes member/file directory entries, writes file clusters, and regenerates
each modified physical page's spare/ECC bytes with the recovered 13.27
algorithm. It was also checked read-only against an authentic 8.65 MB
ECC-bearing card, where it found both logical members, while an
unformatted/invalid sibling card was isolated. Container rows retain the card
path and internal directory name without inventing a whole-card digest;
ordinary state rows use the existing regular-file transactions. PCSX2 manual
backup extracts the named member, creates and re-extracts a flat 7z, verifies
the recovered uppercase SHA-256 logical-folder manifest, rechecks the live card
member for a racing change, and transactionally stores the archive plus XML
without writing to the card. Restore/deletion then use that archive boundary to
build a private complete folder/raw card copy, validate the mutated logical
member, recheck archive and live-card revisions, and replace the entire card
with a retained recovery copy. Folder-card replacement uses two sibling renames
with explicit rollback; raw-card replacement is atomic and physical raw restore
requires spare/ECC pages, matching recovered 13.27.

The recovered `Pcsx2MemoryCardHelper.RepairFilesystem` call site is narrower
than a general repair command. `AddSaveFile` invokes it only after a raw-card
import reports `No free space`, then retries the import. The helper seeds
reachability with the root FAT chain, breadth-first traverses existing
directory entries while de-duplicating first clusters, marks file and directory
chains, clears every allocated but unreachable FAT entry, and flushes only when
it reclaimed clusters. The port implements that trigger and orphan-prune
algorithm only on the already-private raw-card working copy. Its stronger
existing boundary then validates the restored logical member and swaps the
whole card only after archive and live revisions still match. A zero-cluster
repair still retries and fails safely if capacity remains insufficient.

A real-button offscreen raw-card lifecycle begins with 28 allocated orphan
clusters, proves the initial import cannot fit, observes the exact recovery
count through the Qt status, completes selected restore, commits mandatory
pre-restore and pre-delete vault versions, deletes the active member, and
retains both exact complete-card recovery files plus exact XML history with no
transaction residue. Controller coverage separately retains the folder-card
whole-tree path. Synthetic CHD v5 DVD and 2,448-byte CD images generated and
verified with `chdman`, plus ISO, raw-sector, MDF/MDS, NRG, GZip, and compressed
CSO fixtures, require the `SYSTEM.CNF` serial even when the content filename
and game title are opaque. The real-button offscreen PCSX2 scan uses that
compressed CHD path and persists the matching state and card member.
The 13.27 RetroArch plugin does not override `EmulatorPlugin.RemoveSave`, so
the Windows implementation calls the base `File.Delete` on only the persisted
primary path. The port deliberately deletes the already-established Saturn
ownership set instead of orphaning `.bkr`/`.smpc` companions; the mandatory
vault copy and per-member recovery copies make that safety extension explicit.
Stale-row reconciliation, automatic policy, general repair commands, and the
remaining adapters remain open.

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

The desktop box-size contract is recoverable despite protected implementation
bodies. `ILaunchBoxMainViewModel.BoxSize` is a mutable `double`; concrete
settings retain legacy `BoxSize` and modern single-precision `NextBoxSize`.
The untouched 13.27 installation stores `NextBoxSize` as `0.17214286`, while
the older real installation stores a different in-range user value, proving
that it is persisted state rather than a compile-time constant. The 13.27
stock `LBThemes/Default/Views/ControlsView.xaml` binds its slider two-way to
`BoxSize` with minimum 0.05, maximum 0.50, `SmallChange` 0.001, and
`LargeChange` 0.01; it dims/disables the control in list view and wires value,
drag-start, and drag-complete events. The stock boxes view passes
`BoxChildSize` to its virtualizing tile panel, and settings retain
`NextBoxAspectRatio` 0.645. The protected `BoxChildSize` body itself is a
default-return stub in stored IL, so the port explicitly treats the normalized
value, recovered range/steps/aspect, and interaction state as authoritative
while keeping the exact internal WPF pixel formula an oracle boundary.

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
identity read-only because the protected save body and current runtime captures
do not establish the required game/emulator/playlist/parent/controller/settings
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

The 13.27 BigBox resources and installed settings establish four separate
screensaver views and one shared model. `Screensaver1View` is a
fanart/metadata composition, `Screensaver2View` transitions to full video,
`Screensaver3View` places box art beside video or a gameplay screenshot, and
`Screensaver4View` centers media plus title/metadata. The model exposes the
selected `Game`, time-since-swap, total swap time, a one-shot timer, random
selection, and Enter/navigation handlers; every view exposes a Select prompt.
Fresh 13.27 prefixes and the older complete installation agree on enabled,
300-second delay, 30–60 second swaps, required background/box but optional
video, first-view, 75% video, and 100% master defaults. The protected method
bodies still conceal exact random rounding/order, transition details, and
input-binding interpretation. The evidence and port boundary are recorded in
`analysis/screensaver-13.27.md`.

Three fresh 13.27 installations also agree on the complete persisted BigBox
input contract. `BigBoxSettings.xml` stores `EnableGamepad`,
`UseAllControllers`, and normally four WPF `Key` integer slots per configurable
action. The nonzero defaults cover four directions, Select, Back, Play,
PageUp/PageDown, Flip, Music, Images, Exit, volume, and PDF zoom. Zero is
unbound. The structural extraction retains 59 BigBox `InputAction` values and
`ControllerBinding` values for `Button1` through `Button32`, D-pad, both
sticks, and both triggers. Each `InputBindings.xml` contains 36 rules, exactly
18 of which are the shared fresh-install BigBox defaults; rules also have an
optional hold binding. These independent files establish action names,
keyboard slots, semantic controller rules, and chords without relying on
protected method bodies. They do not establish device-specific mapping for
the high numbered buttons or the exact original active-controller algorithm.
The complete evidence and native port boundary are recorded in
`analysis/input-13.27.md`.

The 13.27 `RelatedGamesPopupViewModel` structurally exposes three independent
`GameSuggester` instances and result collections for Recommended Games,
Similar Games, and Possible Ports, plus tab, row, page, Enter, and Escape
navigation. The decoded installed Default resource establishes a centered dark
three-tab popup, white title/border, blue current row, artwork, score, year,
platform, notes, and a dimmed cloud treatment for nonlocal results. Four
inspected installations agree that `ShowGameMenuViewRelatedGames` is enabled.
An older complete `Settings.xml` preserves exact serialized
`GameSuggesterSaveData` criteria, filters, and weights for Recommended and
Similar profiles. No inspected installation preserves a Possible Ports
profile; public product guidance establishes only exact title on another
platform. The protected `GameSuggester` method bodies do not establish its
fuzzy comparison, rounding, cap, or refresh cadence, so those remain explicit
port-owned behavior and parity gates. The complete evidence and native
boundary are recorded in `analysis/related-games-13.27.md`.

Frontend switching has a narrow structural contract and a protected behavioral
boundary. LaunchBox retains `BigBoxModeMenuAction.OnSelect()` and static `Go()`,
but both bodies are unavailable in stored IL. BigBox retains
`DesktopModeMenuAction`, whose constructor accepts the current `Game`, while
its Enter handler is likewise protected. Installed resources independently
name Big Box mode, Switch to Desktop Mode, and refusal to switch while another
operation is in progress. This supports visible bidirectional actions,
operation exclusion, and stable selected-game carryover from BigBox; it does
not establish the original process lifetime, argument protocol, or premium
validation. The native port therefore documents its direct sibling-process
launch and strict durable-argument allowlist as clean-room policy. It does not
copy, infer, or bypass proprietary entitlement state.

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

1. Establish the BigBox WPF runtime oracle in a disposable supported Windows
   VM. Use the repaired Wine prefix for deliberate LaunchBox desktop
   observation and managed probes, while retaining only independently
   checkable, non-user-specific evidence.
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
