# LaunchBox 13.27 game-audit contract

This document freezes the evidence boundary and the first native implementation
for `LIB-015`. It covers the game-audit surface only. LaunchBox's separate bulk
edit wizard remains open.

## Recovered evidence

The structural source is
`decompiled/LaunchBox/Unbroken/LaunchBox/Windows/Desktop/ViewModels/AuditViewModel.cs`.
Its nested `AuditEntry` exposes one internal `Game` reference, four date-sort
helpers, and exactly 76 visible values:

- identity and descriptive data: `Id`, `Title`, `Platform`,
  `ApplicationPath`, `AlternateNames`, `CloneOf`, `Progress`, `Developer`,
  `Genres`, `MaxPlayers`, `Notes`, `PlayMode`, `Publisher`, `Region`,
  `ReleaseDate`, `ReleaseType`, `Series`, `Status`, `Version`, `ManualPath`,
  `MusicPath`, `VideoUrl`, `WikipediaUrl`, and `LaunchboxDatabaseId`;
- dates and state: `DateAdded`, `DateModified`, `LastPlayed`, `Broken`,
  `Hidden`, `Installed`, `Duplicate`, and `IsRunnable`;
- related records and media counts: `AdditionalApps`, 16 image-family counts,
  and `VideoCount`;
- storefront values: `IsEpicGames`, `IsGog`, `IsOrigin`, `IsSteam`, and
  `IsUplay`;
- arcade/MAME classification: `Alternate`, `BadDump`, `Fixed`, `IsBootleg`,
  `IsCasino`, `IsFruit`, `IsHack`, `IsMahjong`, `IsMature`, `IsMechanical`,
  `IsNonArcade`, `IsPlayChoice`, `IsPrototype`, `IsQuiz`, `IsRythm`,
  `IsTabletop`, `Overdump`, `Trainer`, `Translation`, `Unlicensed`, and
  `Verified`.

The exact ordered key list is frozen as
`lb_domain::LAUNCHBOX_AUDIT_COLUMNS`; a unit test requires 76 unique keys and
the recovered first, duplicate, and last positions.

The same view model exposes selected-item state, an edit command, image-family
commands, video and URL commands, a selected-row brush, and a distinct
duplicate-row brush. `Views/AuditView.cs` identifies a themed window with a
`DataGrid` and Close button. `MenuActions/AuditMenuAction.cs` accepts an
optional platform, establishing all-games versus current-platform scope.

Embedded 13.27 release notes in
`Unbroken.LaunchBox.Windows.Desktop.Properties.Resources.resx` provide four
additional behavioral facts:

- the audit can cover the entire collection rather than only one platform;
- repeated LaunchBox Games Database IDs mark games as duplicates;
- copied spreadsheet data includes column headings;
- moving a game to another platform removes it from a platform-scoped audit.

The protected method bodies do not expose MAME classification derivation,
storefront lookup rules, exact column widths, selection gestures, sort
comparison details, or every edit-refresh transition. Those details are not
claimed as recovered parity.

## Native implementation

LaunchBox now exposes one **Audit…** menu with the recovered **Audit All
Games** and **Audit Current Platform** actions. The CXX-Qt controller creates a
stable scoped index without copying or modifying library XML. The Qt dialog:

- renders all 76 columns in a horizontally and vertically virtualized table;
- sorts text, numeric, Boolean, and date-shaped values without a shell or
  platform-specific command;
- marks rows sharing the same non-zero LaunchBox Games Database ID as
  duplicates and renders them in red;
- supports row selection, Select All, Clear, `Ctrl+A`, and `Ctrl+C`;
- exports selected rows as tab-separated text with the complete header through
  Qt's native clipboard API, with tabs/newlines sanitized and a 64 MiB bound;
- opens the existing full game editor by stable ID, first making the chosen
  hidden/broken game visible in the ordinary model.

Media values come from the existing bounded, symlink-safe native media index.
Additional applications and alternate names come from the typed loaded
library. Persisted game fields remain lexical LaunchBox data; QML performs no
Windows-path interpretation.

Unknown is not encoded as false. MAME-derived fields and storefront fields for
which no authoritative local source exists are blank. Persisted GOG and Origin
IDs remain visible. This prevents a partially ported metadata provider from
producing a misleading clean audit.

## Verification

Pure domain tests require:

- the exact 76-column contract;
- duplicate grouping only for repeated non-zero database IDs;
- distinct blank/false output;
- selected-row TSV headers and single-line cell sanitization.

The compiled Qt smoke copies a fixture library, assigns the same database ID
to two differently titled games, and then drives the real dialog. It requires
76 columns, three rows, two duplicate rows, sorting, selection, two-row TSV
output, and the real Edit transition. It captures a rendered PNG and
byte-compares the source platform XML afterward.

## Open work

- Recover and implement the separate bulk-edit wizard.
- Recover an authoritative portable source for every MAME classification flag.
- Add the remaining storefront adapters and distinguish unavailable from
  authenticated-empty provider results.
- Verify exact column visibility, width, ordering, context-menu, and edit
  refresh behavior against a usable Windows oracle.
- Run the native Qt interaction and clipboard scenarios on Windows and on both
  Intel and Apple Silicon macOS.
