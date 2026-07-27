# LaunchBox 13.27 game bulk-edit contract

This note records the evidence boundary and first native implementation for the
`LIB-015` bulk editor. The original assemblies, protected method bodies, local
startup hook, Wine prefix, and library data remain ignored.

## Recovered workflow

LaunchBox 13.27 contains these distinct wizard pages:

1. `BulkEditWelcomePage`;
2. `BulkEditFieldSelectPage`;
3. conditional `BulkEditPlatformChangePage`;
4. `BulkEditConfirmPage`;
5. `BulkEditApplyChangePage`.

The field page's view model and BAML establish typed editor families rather
than one unvalidated string box:

- single-line and multiline text;
- file selection;
- two-state and optional three-state booleans;
- fixed or editable combo values;
- a date picker;
- a 0–5 rating control with half-star values;
- model settings and an override choice;
- multi-value add and remove values;
- controller support;
- custom-field name and value handling.

The platform page owns progress state and a choice to migrate associated image
and video media after a platform change. The confirmation page owns the exact
game collection and raises `OnItemsUpdated`; the apply page supports closing or
starting another change without returning through the welcome page.

Protected initialization and apply bodies prevent the static decompiler from
emitting the original complete ordered field list. A temporary .NET startup
hook attached to the real self-contained runtime in a reflinked, disposable
13.27 installation. The normal data-manager constructor stalls at the same
Wine activation boundary, so the probe supplied uninitialized data objects,
empty typed catalogs, and the documented default progress values. It then
invoked the real protected bulk view-model on an STA thread and captured the
live `Fields` collection. No proprietary binary, hook, log, license, or game
value is checked in.

The runtime registered these 51 fixed entries, in order:

1. `3D Model Settings`
2. `Broken`
3. `Controller Support`
4. `Custom DOSBox Version EXE Path`
5. `Developer`
6. `DOSBox Configuration File`
7. `Emulator`
8. `Favorite`
9. `Game Manual Path`
10. `Game Music Path`
11. `Genre`
12. `Hide`
13. `Installed`
14. `Last Played`
15. `Max Players`
16. `Notes`
17. `Pause Screen - Enable`
18. `Pause Screen - Forceful Activation`
19. `Pause Screen - Load State AutoHotkey Script`
20. `Pause Screen - Override Default Settings`
21. `Pause Screen - Pause Game AutoHotkey Script`
22. `Pause Screen - Reset Game AutoHotkey Script`
23. `Pause Screen - Resume Game AutoHotkey Script`
24. `Pause Screen - Save State AutoHotkey Script`
25. `Pause Screen - Suspend Game Process On Pause`
26. `Platform`
27. `Play Mode`
28. `Portable`
29. `Progress`
30. `Publisher`
31. `Rating`
32. `Region`
33. `Release Date`
34. `Release Type`
35. `Series`
36. `Sort Title`
37. `Source`
38. `Star Rating`
39. `Startup Screen - Aggressive Startup Window Hiding`
40. `Startup Screen - Enabled`
41. `Startup Screen - Hide All Non-Exclusive Mode Windows`
42. `Startup Screen - Hide Mouse Cursor During Game`
43. `Startup Screen - Load Delay`
44. `Startup Screen - Override Default Settings`
45. `Startup Screen - Shutdown Enabled`
46. `Status`
47. `Use DOSBox`
48. `Use ScummVM`
49. `Version`
50. `Video URL`
51. `Wikipedia URL`

The bare probe intentionally had no loaded custom-field value catalog, so
dynamic custom-field additions are not included in that fixed list.

## Release-note behavior

The embedded 13.27 release history independently names these bulk-editor
behaviors:

- star-rating changes;
- date values in confirmation;
- skipping Welcome for an additional change;
- clearing Emulator;
- saving after restart, custom-field, and audit-triggered edits;
- offering media migration after Platform changes;
- add/remove operations for multi-select values;
- Custom DOSBox Version;
- Broken;
- propagating an Emulator change to additional applications that use the same
  emulator;
- Video Path, Custom Fields, and Hide.

The same history records crash fixes while applying and finishing changes.
Those fixes make transaction completion and recovery part of the required
behavior, not merely UI polish.

## Current native vertical

The Rust catalog now exposes 45 typed fields. Forty-two follow the recovered
fixed order above. The three retained compatibility surfaces are `Completed`,
`Video Path`, and the generic `Custom Field` editor. The nine recovered fixed
entries not yet exposed are `DOSBox Configuration File`, `Game Manual Path`,
`Game Music Path`, `Installed`, `Last Played`, `Portable`, `Use DOSBox`,
`Use ScummVM`, and `Video URL`.

All nine recovered Pause Screen entries and all seven Startup Screen entries
are implemented as individual scalar operations, matching the real field
catalog rather than inventing a grouped record editor. Boolean settings use
typed values. `Startup Screen - Load Delay` accepts only a whole millisecond
value from zero through the persisted `u32` maximum. The five exposed
AutoHotkey scripts use a multiline editor, preserve supplied whitespace in
XML, and support explicit removal. LaunchBox does not expose the game
`SwapDiscsAutoHotkeyScript` through this bulk catalog, so the port leaves that
node untouched.

The positive `Startup Screen - Shutdown Enabled` field deliberately maps to
the inverse of LaunchBox's persisted `DisableShutdownScreen` value. The
remaining labels map one-to-one to the existing 13.27 game XML fields:
`UsePauseScreen`, `ForcefulPauseScreenActivation`,
`OverrideDefaultPauseScreenSettings`, `SuspendProcessOnPause`, the five script
nodes, `AggressiveWindowHiding`, `UseStartupScreen`,
`HideAllNonExclusiveFullscreenWindows`, `HideMouseCursorInGame`,
`StartupLoadDelay`, and `OverrideDefaultStartupScreenSettings`.

The editor kind and clearability come from Rust. The versioned request denies
unknown keys and rejects values from the wrong editor family. Ratings accept
only finite half-star values from 0 through 5. Multi-value Set/Add/Remove/Clear
uses semicolon-separated LaunchBox values, compares entries
case-insensitively, and keeps retained spelling and order. Duplicate
same-name custom fields fail closed instead of being collapsed.

Audit selection is copied as stable internal game IDs when the wizard opens.
Apply resolves those IDs back to their exact source documents, loads each
document once, performs only the chosen typed mutation, stages every changed
platform document, and commits the set through one `LibraryTransaction`.
Optimistic SHA-256 revisions are checked before any replacement. A conflict in
one source therefore leaves every source unchanged. A successful commit leaves
one exact recovery copy per affected platform document and no pending
transaction manifest.

`Platform` is a cross-document operation. It moves the complete `Game`,
`ModelSettings`, `AdditionalApplication`, `Mount`, `AlternateName`,
`CustomField`, `GameControllerSupport`, and `GameSave` elements to the
existing destination platform document, changes only the game's `Platform`
value, and retargets matching manual-playlist `GamePlatform` rows. Complete
elements and unknown root records remain intact.

The conditional page requires an explicit choice to migrate or leave media.
Migration indexes all associated image and video files independent of display
preferences, preserves configured media type plus relative subtype/region
paths, and updates explicit `VideoPath`/`ThemeVideoPath` values when those
files move. It fails closed before XML staging if the scan is truncated, a
source escapes the library, media is shared with an unselected same-title
game, a destination exists or collides under portable case/Unicode rules, or a
destination directory chain contains a symlink or non-directory. The media
copies, exact source deletions, every source/destination platform XML edit, and
playlist edits commit in one recoverable `LibraryTransaction`; directory
preparation can leave only harmless empty directories if a later revision
check refuses the transaction. After commit, the controller re-indexes only
the moved games against their destination platform and publishes a new media
revision, so both migration choices update the running Qt model without a
library reload.

`Custom DOSBox Version EXE Path` and `Video Path` remain lexical LaunchBox
strings. Persisted path interpretation and conversion is isolated in the
shared platform service; QML and the storage DOM do not apply Windows path
rules.

Emulator Set values come from the currently loaded typed emulator catalog.
The background worker reloads that catalog immediately before editing,
case-insensitively resolves the requested ID to its retained canonical
spelling, and rejects a missing/stale ID before loading or staging platform
documents. On a real emulator change, each additional application owned by
that game is retargeted only when `UseEmulator=true` and its prior
`EmulatorId` matches the parent's prior real emulator ID. Direct apps,
different IDs, and apps owned by another game remain unchanged. Clear removes
the matching app's `EmulatorId` while retaining `UseEmulator=true`, the
persisted platform-default state. The transaction result republishes the
committed application groups and a Qt revision so the running editor and
launcher cannot retain the old emulator.

`Controller Support` follows the distinct related-record surface recovered
from the 13.27 view model and BAML. The original type exposes
`GetCurrentControllerValues`, `GetPossibleControllerValues`,
`AllSupportLevels`, and `SupportLevelValue`; its view contains separate
multi-value remove/add controls and the exact resource instruction “Which
support level would you like to set the added controllers at?” One typed
version-3 request therefore carries disjoint controller-ID removal and addition
sets plus one of the exact four recovered levels for additions.

Immediately before any platform document is loaded, the background worker
reloads `GameControllers.xml`, resolves every requested ID case-insensitively,
retains the catalog's canonical spelling, and rejects missing/stale IDs. For
each selected game, removals delete matching support rows; additions insert a
missing row or update an existing row to the chosen level. Retained rows keep
unknown XML children, level zero remains canonically omitted, all affected
platform documents commit together, and the committed per-game support groups
plus revision replace the running Qt state. The native selector shows the
catalog name, category, stable ID, and how many selected games currently carry
each removable row.

`3D Model Settings` follows a second distinct related-record surface. The
13.27 field-page view model exposes `OverrideDefaultModelSettings`,
`OverrideDefaultModelSettingsLabel`, `PossibleModelSettings`,
`SelectedSettings`, and `SelectedSettingsView`; the corresponding BAML proves
the override checkbox, `Model Type:` label, and those bindings. The native
editor presents the complete shared model-settings contract: the exact four
recovered Box, DVD Case, Jewel Case, and Long Jewel Case types; stored model
key; forced case and cover colors; full-scan image/landscape choices and spine
width; clear front-spine choice; and forced model size.

One version-3 request requires an explicit override choice. Enabling it sends
one validated identity-free whole-record template; the worker assigns each
selected game's exact `GameId` and replaces or inserts only that game's
`ModelSettings` record. Disabling it removes only each selected game's record,
so platform and built-in inheritance resumes without copying inherited values
into XML. Existing record updates retain unknown children. The worker reloads
the committed documents and platform catalog, resolves the effective
game/platform/built-in value for every selected ID, and republishes those
values plus the model-settings revision to the live 3D views.

The product invokes no command interpreter and contains no Windows-, Linux-,
or macOS-specific bulk-editor UI branch.

The native wizard implements all five recovered page positions: Welcome, typed
field selection, conditional Platform media choice, confirmation, and
apply/progress/result. “Make Another Change” returns directly to field
selection, which matches the recovered 13.27 release-note behavior.

## Verification

Pure tests freeze unique catalog keys, the exact nine pause and seven startup
labels and relative order, editor/value validation, unsigned delay bounds,
half-star bounds, and stable multi-value semantics. Storage tests apply every
startup/pause mapping, prove the positive-to-negative shutdown inversion,
preserve multiline script whitespace, leave the swap-discs script untouched,
and retain unknown game XML. Existing storage coverage also applies booleans,
metadata, custom fields, a Windows-separated Video Path, and the Custom DOSBox
path while retaining unknown game/custom-field XML and unrelated lexical
paths.

Dedicated emulator tests cover case-insensitive parent/app matching, Set and
Clear, retained `UseEmulator`, owner scoping, direct and different-emulator
isolation, unknown application XML, live refresh rows, catalog-ID
canonicalization, stale-ID rejection before a write, exact recovery, and clean
transaction state.

Dedicated controller-support tests cover simultaneous add/remove validation,
case-insensitive catalog canonicalization, all four typed levels, insertion,
existing-row level replacement, optional-zero encoding, retained unknown row
XML, removal isolation, two-game live refresh, stale-ID refusal before a
write, exact recovery, and clean transaction state.

Dedicated model-settings tests cover identity-free validation, complete
whole-record update and insertion, exact per-game identity assignment, unknown
child retention, override-only removal, platform inheritance after removal,
two-game live refresh, exact recovery, and clean transaction state.

A two-document transaction test proves successful paired commit with exact
backups, then creates a revision conflict in the second document and proves
that the first document remains byte-identical. Platform-transfer storage
coverage moves every modeled owned record and retargets a playlist while
retaining unknown game, owned-record, playlist-row, and root XML.

Media-planning tests cover mixed persisted separators, region/subtype
retention, shared-title refusal, existing-target refusal, and destination
symlink/case-collision refusal. Controller transaction tests move
source/destination XML, playlist XML, an image, and a video together with exact
recovery copies and a clean manifest, and prove the post-commit media index for
both migration choices; a collision test byte-compares both XML documents and
both media files unchanged. The compiled Qt scenario selects two stable IDs in
the real audit dialog, resolves the real fixture emulator through the typed
combo catalog, renders the conditional Platform media page and the Controller
Support remove/add/level surface, then applies Required support to the real
fixture controller. It next renders the complete 3D Model Settings surface and
applies the exact Long Jewel Case override to both games. The scenario verifies
one existing controller level update plus one new row, two complete
model-settings records, two typed 1,250-millisecond startup delays, two
multiline pause scripts, retained unknown elements, an unchanged
Windows-separated application path, the exact four-transaction recovery
chain, live values, and clean recovery state.

## Open parity gates

- implement the nine recovered fixed fields listed above and recover
  value-dependent/dynamic custom-field ordering;
- recover every value-dependent combo choice from a fully initialized
  supported Windows runtime;
- exact three-state meanings and date formatting/time-zone behavior;
- exact original collision policy, apply-time cancellation, restart,
  progress/error wording, and batch size behavior;
- native Windows and both macOS-architecture interaction tests;
- scenario comparison against the supported Windows oracle.
