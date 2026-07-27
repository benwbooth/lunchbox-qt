# LaunchBox 13.27 game-controller support contract

This note records the clean-room evidence used for the first native `LIB-005`
vertical. The original application remains an observation oracle; the Rust/Qt
application neither loads nor redistributes proprietary assemblies.

## Persisted records

LaunchBox stores the global controller catalog in
`Data/GameControllers.xml`. Each `<GameController>` has four fields:

| Field | Meaning |
|---|---|
| `Id` | Stable controller identity |
| `Name` | User-visible controller name |
| `Category` | Controller category text |
| `AssociatedPlatforms` | Semicolon-separated LaunchBox platform names |

Per-game compatibility is stored beside the owning game in
`Data/Platforms/*.xml`. Each `<GameControllerSupport>` has:

| Field | Meaning |
|---|---|
| `ControllerId` | Reference to the global controller catalog |
| `GameId` | Reference to a game in the same platform document |
| `SupportLevel` | Optional persisted compatibility integer |

The real older installation contains 86 controller definitions and 7,061
support rows. All support references resolve to 71 used controller IDs, and no
platform document contains a duplicate `(GameId, ControllerId)` pair.

## Exact 13.27 categories and levels

A temporary self-contained managed reflection probe loaded the installed 13.27
first-party assemblies in the ignored Wine oracle and recovered this exact
category collection, in order:

1. `Gamepad`
2. `Joystick`
3. `Keyboard`
4. `Light Gun`
5. `Motion`
6. `Mouse`
7. `Paddle`
8. `Rhythm`
9. `Trackball`
10. `VR`
11. `Wheel/Yoke`

The same probe recovered the exact zero-based support-level choices:

| Integer | Display name | XML form |
|---:|---|---|
| 0 | Not Supported | omit `SupportLevel` |
| 1 | Partial Support | `<SupportLevel>1</SupportLevel>` |
| 2 | Full Support | `<SupportLevel>2</SupportLevel>` |
| 3 | Required | `<SupportLevel>3</SupportLevel>` |

The older real library corroborates the optional zero representation. Its
7,061 rows contain 2,369 absent values, 32 value-1 rows, 107 value-2 rows, and
4,553 value-3 rows.

Historical data is not assumed to use only the current collection. The real
catalog contains the spelling `Rythm`. The port therefore offers the 11
recovered choices for new records but retains and exposes an existing unknown
category unchanged.

## Native implementation boundary

The port:

- validates non-empty immutable IDs, names, and categories;
- generates UUIDs for new definitions;
- rejects new case-insensitive name collisions while allowing an unchanged
  historical duplicate to remain editable;
- validates and de-duplicates associated platform names without interpreting
  them as host paths;
- indexes support rows by stable game ID and exposes all four exact levels;
- writes level zero canonically by omitting `SupportLevel`;
- refuses unknown controller IDs and duplicate support rows;
- replaces only the selected game's support rows, retaining unknown children
  on rows that remain;
- bulk-edits selected games through the recovered independent remove/add
  selectors and applies one exact support level to every addition;
- reloads the controller catalog before a bulk write, canonicalizes stable IDs,
  rejects stale IDs, and commits every affected platform in one transaction;
- blocks catalog deletion while any platform document still references the
  controller and never cascades support deletion;
- uses the shared revision-checked transaction and committed typed reread for
  every catalog or platform mutation.

All stored Windows-style game and media paths remain lexical XML data. The
controller catalog itself contains LaunchBox platform names, not filesystem
paths. CXX-Qt and QML receive typed values and do not branch on Windows, Linux,
or macOS.

## Verification

Domain and storage tests freeze the exact category/level order, optional-zero
encoding, validation, typed/lossless catalog CRUD, support replacement,
unknown XML retention, lexical path retention, duplicate refusal, and
canonical row ordering.

A controller test creates and edits definitions, proves exact backups and
committed rereads, blocks a referenced deletion, replaces the game's support
row with `Required`, and then deletes the unreferenced definition.

Bulk-specific domain, storage, and controller tests prove disjoint
simultaneous add/remove selection, retained unknown XML on level updates,
optional-zero insertion, removal isolation, fresh catalog canonicalization,
stale-ID refusal before platform loading, two-game committed-row refresh, one
exact backup, and clean recovery state.

The compiled rendered Qt scenario drives the real manager, controller editor,
game metadata editor, and delete confirmation. It preserves the historical
`Rythm` category during edit, creates a `Wheel/Yoke` controller with a platform
association, observes one blocked reference, changes `fixture-racer` from Full
Support to Required on the new stable ID, and deletes the old definition. The
runner independently validates the final XML, three exact catalog backups, one
exact platform backup, unknown XML, and clean recovery state.

The bulk-editor scenario selects two games, renders the recovered current and
possible controller lists plus all four support levels, then applies
`Required` to one existing and one missing fixture row in a single recoverable
transaction. The running controller-support map is refreshed before write
completion.

Native Qt interaction on real Windows and macOS hosts remains a release gate;
the portable core is compiled for both targets.
