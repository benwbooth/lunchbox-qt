# LaunchBox 13.27 BigBox input recovery

This note records the evidence used for the first native `BB-012` input
vertical. It contains no LaunchBox binaries, copied XAML, proprietary method
bodies, or user-specific controller identifiers.

## Recovered persisted contract

Three independent fresh 13.27 Wine installations agree on the relevant
`Data/BigBoxSettings.xml` and `Data/InputBindings.xml` contracts.

`BigBoxSettings.xml` stores `EnableGamepad` and `UseAllControllers` as booleans.
Each configurable keyboard action normally has four WPF `Key` integer slots:
the base setting, then the same name suffixed `2`, `3`, and `4`. The system
menu, index, and model-rotation settings use numbered `1` through `4` names;
`KeyboardGamePause` has one slot. `BigBoxExitGame`,
`BigBoxFocusInterface`, and `BigBoxScreenshot` have no corresponding keyboard
setting in the recovered contract.

The nonzero defaults agree across the inspected installs:

| BigBox action | Setting | WPF value | Portable Qt sequence |
|---|---|---:|---|
| Navigate left | `KeyboardLeft` | 23 | `Left` |
| Navigate up | `KeyboardUp` | 24 | `Up` |
| Navigate right | `KeyboardRight` | 25 | `Right` |
| Navigate down | `KeyboardDown` | 26 | `Down` |
| Select | `KeyboardSelect` | 6 | `Return` |
| Back | `KeyboardBack` | 13 | `Esc` |
| Play game | `KeyboardPlay` | 59 | `P` |
| Page up | `KeyboardPageUp` | 19 | `PgUp` |
| Page down | `KeyboardPageDown` | 20 | `PgDown` |
| Flip box | `KeyboardFlipBox` | 49 | `F` |
| Play music | `KeyboardPlayMusic` | 56 | `M` |
| Show images | `KeyboardViewImages` | 52 | `I` |
| Exit | `KeyboardExit` | 67 | `X` |
| Volume up | `KeyboardVolumeUp` | 85 | `Num++` |
| Volume down | `KeyboardVolumeDown` | 57 | `N` |
| Zoom in | `KeyboardPdfReaderZoomIn` | 85 | `Num++` |
| Zoom out | `KeyboardPdfReaderZoomOut` | 87 | `Num+-` |

Zero means unbound. The port converts WPF integer values into Qt portable
key-sequence text at the platform boundary. Unknown future values remain
unbound rather than being reinterpreted as host scan codes. Duplicate
sequences are represented by one Qt shortcut with ordered candidate actions,
so the observed shared `Num++` default is not registered ambiguously.

`InputBindings.xml` contains controller rules with `InputAction`,
`ControllerBinding`, and optional `ControllerHoldBinding`. Fresh installations
contain 36 rules, of which these 18 are BigBox defaults:

| Action | Binding |
|---|---|
| Back | `Button2` |
| Navigate down | `LeftStickDown`, `DPadDown` |
| Navigate left | `DPadLeft`, `LeftStickLeft` |
| Navigate right | `LeftStickRight`, `DPadRight` |
| Navigate up | `LeftStickUp`, `DPadUp` |
| Page down | `Button6` |
| Page up | `Button5` |
| Play game | `Button3` |
| Rotate model down | `RightStickDown` |
| Rotate model left | `RightStickLeft` |
| Rotate model right | `RightStickRight` |
| Rotate model up | `RightStickUp` |
| Select | `Button1` |
| Show images | `Button4` |

The recovered `ControllerBinding` enum contains `None`, `Button1` through
`Button32`, the four D-pad directions, four directions for each stick, and
both triggers. Hold rules are chords: a press fires only when its configured
hold binding is already down. Duplicate press events do not repeat an action.

## Recovered action vocabulary

The decompiled structural contract retains 59 BigBox actions in declaration
order:

`Search`, `ShowGameDetails`, `SwitchImageType`, `SwitchView`, `Exit`,
`ShowImages`, `PlayMusic`, `FlipBox`, `PageDown`, `PageUp`, `PlayGame`, `Back`,
`Select`, `NextMusicTrack`, `PreviousMusicTrack`, `ShowDiscoveryCenter`,
`ShowAllGames`, `ShowGenres`, `ShowPlatforms`, `ShowPlaylists`,
`ShowDevelopers`, `ShowPublishers`, `ShowRatings`, `ShowPlayModes`,
`ShowRegions`, `ShowSeries`, `ShowStatuses`, `ShowSources`,
`ShowPlatformCategories`, `Filter`, `StartAttractMode`, `WheelSpin`,
`SwitchTheme`, `ShowAchievements`, `ShowAchievementProfile`, `SetStarRating`,
`ZoomIn`, `ZoomOut`, `LockUnlock`, `ShowPauseScreen`, `ExitGame`,
`FocusInterface`, `VolumeUp`, `VolumeDown`, `NavigateUp`, `NavigateDown`,
`NavigateLeft`, `NavigateRight`, `ShowHighScores`, `Screenshot`,
`ViewSystemMenu`, `OpenIndex`, `RotateModelUp`, `RotateModelDown`,
`RotateModelLeft`, `RotateModelRight`, `ShowModel`, `RandomGame`, and
`StartScreensaver`.

## Native cross-platform boundary

`lb-platform::input` owns the typed settings, WPF-to-Qt conversion, controller
rules, hold/edge state, active-controller policy, and native event backend.
Qt owns application shortcuts and one central BigBox action dispatcher. QML
receives semantic actions only; it never sees Windows scan codes, device
paths, `/dev/input` paths, shell commands, or platform-specific API handles.

The native backend uses `gilrs` on Linux, Windows, and macOS. Its semantic
mapping is:

- south/east/west/north face buttons to `Button1` through `Button4`;
- left/right shoulders to `Button5` and `Button6`;
- select/start/thumb/mode/C/Z to `Button7` through `Button13`;
- D-pad, left/right stick directions, and analog triggers to their named
  bindings.

Stick directions use 0.75 press and 0.65 release thresholds to avoid boundary
chatter. When `UseAllControllers` is false, the first device that presses a
binding owns input until it disconnects. Hot-plug and disconnect are consumed
through the same native event stream.

The implementation intentionally does not claim an exact physical mapping for
`Button14` through `Button32`: `gilrs` exposes normalized semantic buttons, and
the protected 13.27 device-specific mapping behavior remains unavailable.
Native physical-device interaction, focus, and application-bundle validation
on real Windows and Intel/Apple Silicon macOS hosts remain release gates.

## Port-owned editor and persistence contract

The port's editor operates on recovered persisted semantics rather than host
scan codes. Logical Qt keys are converted back to WPF `Key` integers for the
known function, navigation, keypad, alphanumeric, and OEM punctuation
vocabulary. Unknown future integers remain attached to their original slots
and are serialized unchanged unless that exact slot is edited.

Keyboard changes are sparse: only changed action/slot pairs are sent to Rust.
Controller rules are a complete replacement only for the 59 managed BigBox
actions. The lossless XML layer retains desktop rules, unknown/future BigBox
actions, unknown row children, and unknown top-level data verbatim. It rejects
unknown actions/bindings, invalid slot counts, and duplicate
action/binding/hold triples before staging anything.

When a change spans `BigBoxSettings.xml` and `InputBindings.xml`, both files
are staged in one root-scoped repository transaction. A successful commit
retains one exact sibling backup per changed document and the live router is
rebuilt by rereading the committed typed documents. A failed or conflicted
write does not publish partial UI state. An existing empty
`InputBindings.xml` is authoritative, so deliberately clearing every
controller mapping remains empty after later keyboard-only edits and process
restart.
