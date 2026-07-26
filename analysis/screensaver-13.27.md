# LaunchBox 13.27 BigBox screensaver recovery

This note records the evidence used for the first native `BB-009` port subset.
It contains no LaunchBox binaries, artwork, copied XAML, or proprietary method
bodies.

## Recovered persisted contract

The value-free 13.27 schema in `analysis/real-install-schema.json` contains the
following BigBox settings. Their values agree in three fresh 13.27 Wine
installations and the older complete installation on the Windows partition:

| Setting | Observed default |
|---|---:|
| `EnableScreensaver` | `true` |
| `ScreensaverDelay` | `300` seconds |
| `ScreensaverMinimumSwapTime` | `30000` milliseconds |
| `ScreensaverMaximumSwapTime` | `60000` milliseconds |
| `ScreensaverSkipGamesMissingBackground` | `true` |
| `ScreensaverSkipGamesMissingBoxArt` | `true` |
| `ScreensaverSkipGamesMissingVideo` | `false` |
| `ScreensaverView` | empty, selecting the first view |
| `VolumeVideo` | `75` percent |
| `VolumeMaster` | `100` percent |

All inspected installations also contain four `KeyboardStartScreensaver`
slots; their stored values are zero, so the port does not infer a product
default key. The assembly string table and installed resource keys establish
the stored view identifiers `Screensaver1View` through `Screensaver4View`.

## Recovered presentation and behavior

The installed default theme embeds four separate screensaver view resources:

1. a full fanart/background composition with platform/game identity and
   metadata;
2. the same composition transitioning to full-screen game video;
3. a split composition with box art beside video or gameplay screenshot;
4. a centered composition that transitions from box art to video or gameplay
   screenshot with centered title and metadata.

All four bind to one `ScreensaverViewModelBase.Game` and expose a Select
prompt. The base owns `TimeSinceLastSwap`, `TotalTimeBeforeSwap`, a one-shot
timer, random selection, initialize/stop, and Enter/Escape/directional/page
input handlers. Installed diagnostics establish random duration selection
between the configured minimum and maximum and a new game pick on each swap.
The Select prompt plus `OnEnter` establishes exit-to-selected-game behavior;
the remaining handled navigation inputs establish ordinary exit behavior.

The protected 13.27 implementation does not expose its exact random-number
rounding, candidate ordering, transition implementation, or controller-binding
interpretation. The port therefore keeps those points explicit: inclusive
deterministic entropy injection in Rust, stable-ID candidates in library order,
and native Qt transitions. The subsequent `BB-012` vertical routes configured
`BigBoxStartScreensaver`, Select/Play, and Back bindings through the shared
native dispatcher.

## Port boundary

`lb-platform::screensaver` owns strict typed settings, four view identifiers,
inclusive bounded swap timing, guarded candidate projection, and injected
random selection. It projects only native paths already accepted by the shared
symlink-safe media index and never reparses Windows path strings in QML.

The CXX-Qt controller publishes typed policy, candidate metadata, and native
local URLs. QML owns idle/active lifecycle, focus and input capture, one Qt
Multimedia player, the four port-owned presentations, and return/explore
actions. No runtime command shell or OS-specific path literal is used.
