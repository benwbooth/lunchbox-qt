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

## First native vertical

The Rust catalog currently exposes 25 typed fields:

`Broken`, `Completed`, `Developer`, `Emulator`, `Favorite`, `Genre`, `Hide`,
`Max Players`, `Notes`, `Play Mode`, `Progress`, `Publisher`, `Rating`,
`Region`, `Release Date`, `Release Type`, `Series`, `Sort Title`, `Source`,
`Star Rating`, `Status`, `Version`, `Video Path`, `Wikipedia URL`, and
`Custom Field`.

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

`Video Path` and `Emulator` are stored as lexical LaunchBox strings/IDs.
The bulk editor does not classify, resolve, open, or normalize a persisted
path. It invokes no command interpreter and contains no Windows-, Linux-, or
macOS-specific UI branch.

The native wizard implements Welcome, typed field selection, confirmation, and
apply/progress/result pages. The recovered Platform page is conditional and is
not shown because Platform mutation and media migration are not implemented in
this vertical. “Make Another Change” returns directly to field selection, which
matches the recovered 13.27 release-note behavior.

## Verification

Pure tests freeze unique catalog keys, named 13.27 additions, editor/value
validation, half-star bounds, and stable multi-value semantics. Storage tests
apply booleans, metadata, custom fields, and a Windows-separated Video Path
while retaining unknown game/custom-field XML and unrelated lexical paths.

A two-document transaction test proves successful paired commit with exact
backups, then creates a revision conflict in the second document and proves
that the first document remains byte-identical. The compiled Qt scenario
selects two stable IDs in the real audit dialog, opens the wizard, renders its
Publisher confirmation page, applies one transaction, and verifies exactly two
changed publisher values, a retained unknown element, an unchanged
Windows-separated application path, one byte-exact backup, and clean recovery
state.

## Open parity gates

- recover the complete ordered original field catalog and every combo value;
- Platform mutation, the conditional migration choice, and exact media move
  collision/cancellation behavior;
- model-settings and controller-support bulk surfaces;
- Custom DOSBox Version and the remaining launch/startup/pause fields;
- Emulator propagation to matching additional applications;
- exact three-state meanings and date formatting/time-zone behavior;
- restart/cancel/progress/error wording and batch size behavior;
- native Windows and both macOS-architecture interaction tests;
- scenario comparison against the supported Windows oracle.
