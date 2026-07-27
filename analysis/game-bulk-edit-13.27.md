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
hook attached to the real self-contained runtime after the disposable Wine
prefix was repaired, but LaunchBox's activation/data-manager path deadlocked
before the field collection could initialize. Therefore neither this note nor
the port claims that the current native catalog is the complete original list.

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

The Rust catalog currently exposes 26 typed fields:

`Broken`, `Completed`, `Developer`, `Emulator`, `Favorite`, `Genre`, `Hide`,
`Max Players`, `Notes`, `Platform`, `Play Mode`, `Progress`, `Publisher`,
`Rating`, `Region`, `Release Date`, `Release Type`, `Series`, `Sort Title`,
`Source`, `Star Rating`, `Status`, `Version`, `Video Path`, `Wikipedia URL`,
and `Custom Field`.

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

Other `Video Path` and `Emulator` values remain lexical LaunchBox strings/IDs.
Persisted path interpretation and conversion is isolated in the shared
platform service; QML and the storage DOM do not apply Windows path rules. The
product invokes no command interpreter and contains no Windows-, Linux-, or
macOS-specific bulk-editor UI branch.

The native wizard implements all five recovered page positions: Welcome, typed
field selection, conditional Platform media choice, confirmation, and
apply/progress/result. “Make Another Change” returns directly to field
selection, which matches the recovered 13.27 release-note behavior.

## Verification

Pure tests freeze unique catalog keys, named 13.27 additions, editor/value
validation, half-star bounds, and stable multi-value semantics. Storage tests
apply booleans, metadata, custom fields, and a Windows-separated Video Path
while retaining unknown game/custom-field XML and unrelated lexical paths.

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
the real audit dialog, renders the conditional Platform media page and
Publisher confirmation page, applies one Publisher transaction, and verifies
exactly two changed values, a retained unknown element, an unchanged
Windows-separated application path, one byte-exact backup, and clean recovery
state.

## Open parity gates

- recover the complete ordered original field catalog and every combo value;
- model-settings and controller-support bulk surfaces;
- Custom DOSBox Version and the remaining launch/startup/pause fields;
- Emulator propagation to matching additional applications;
- exact three-state meanings and date formatting/time-zone behavior;
- exact original collision policy, apply-time cancellation, restart,
  progress/error wording, and batch size behavior;
- native Windows and both macOS-architecture interaction tests;
- scenario comparison against the supported Windows oracle.
