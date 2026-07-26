# LaunchBox 13.27 search and random-game command contract

This note records the evidence used for the first native `DESK-009` subset. It
separates public names and shortcut behavior from protected method bodies that
decompile as stubs.

## Recovered desktop contract

The installed 13.27 desktop assembly contains these distinct types:

- `MenuActions/FocusSearchMenuAction.cs`
- `MenuActions/SelectRandomGameMenuAction.cs`
- `Commands/PlayRandomGameCommand.cs`

The embedded LaunchBox changelog supplies the observable shortcut and naming
history that the stubbed method bodies cannot:

- the search box is accessible with `Ctrl+F`;
- Select Random Game is accessible with `Ctrl+Alt+Q`;
- the older Launch Random Game feature was changed to Select Random Game for
  more flexibility.

The separate `PlatformFiltersDetailsViewModel` still exposes
`PlayRandomGameCommand`, `RandomGame`, and `RandomPlayableGame`. Therefore the
13.27 evidence describes two operations, not one:

1. select a random game without launching it;
2. select and play a random playable game from a platform/filter detail
   context.

The protected bodies do not reveal candidate construction, entropy, weighting,
or retry rules. No exact original random algorithm is claimed.

## Native implementation boundary

LaunchBox Port implements the recovered distinction as follows:

- Qt's platform-standard Find binding focuses the real search field. This is
  the recovered `Ctrl+F` command on Windows and Linux and follows the native
  Find binding on macOS.
- `Ctrl+Alt+Q` invokes Select Random Game without launching it.
- visible Select Random and Play Random controls expose the two operations
  independently.
- both operations use the shared Rust query projection, so a random candidate
  can only be a currently visible row.
- the selected row is converted back to the game's stable ID before QML updates
  both grid and list selection.
- when multiple candidates exist, the current game is excluded; a single
  candidate remains selectable.
- Play Random refuses to start while loading, writing, launching, or supervising
  an active launch session, then passes the selected row and stable ID through
  the existing shell-free launch boundary.

Visible-row selection, current-game exclusion, and injected entropy are
explicit clean-room port policies because the protected 13.27 algorithm was
not recovered.

BigBox keeps its separately recovered Random Game input action and RANDOM
control. They use the same shared selection function, but this desktop command
milestone does not invent a BigBox random-play shortcut.

## Verification

Pure query tests cover empty, single-row, invalid-index, deterministic
multi-row, and current-game-exclusion behavior.

The compiled LaunchBox workflow loads an isolated one-game library and:

1. moves focus away from search and activates the actual Find shortcut handler;
2. clears both views and activates the actual `Ctrl+Alt+Q` shortcut handler;
3. requires the exact stable game ID in both grid and list;
4. requires both visible native controls and renders them to a PNG;
5. invokes Play Random through a mapped Windows `Z:` path;
6. executes the portable Rust argument recorder with exact argument boundaries;
7. waits for the complete process session and validates both statistics writes;
8. checks the final LaunchBox timestamp/count/time fields, the exact two-step
   backup chain, and absence of a recovery manifest.

Windows and both Darwin core targets compile the shared selection and launch
logic. Native shortcut delivery and visible Qt interaction on Windows and
Intel/Apple Silicon macOS remain real-host release gates.
