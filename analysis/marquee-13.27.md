# LaunchBox 13.27 BigBox marquee recovery

This note records the evidence used for the first native `BB-013` secondary
marquee vertical. It contains no LaunchBox binaries, artwork, copied XAML, or
proprietary method bodies.

## Recovered persisted contract

Three independent fresh 13.27 Wine installations agree on these
`Data/BigBoxSettings.xml` values:

| Setting | Observed default |
|---|---:|
| `PrimaryMonitorIndex` | `0` |
| `MarqueeMonitorIndex` | `-1` (disabled) |
| `MarqueeIgnoreThemeViews` | `false` |
| `MarqueeStretchImages` | `false` |
| `MarqueeScreenCompatibilityMode` | `None` |

The recovered compatibility-mode enum has exactly eight values, in declaration
order:

1. `None`
2. `HalfSizeStretched`
3. `ThirdSizeStretched`
4. `BottomHalfCutOff`
5. `TopHalfCutOff`
6. `BottomTwoThirdsCutOff`
7. `TopTwoThirdsCutOff`
8. `TopAndBottomOneThirdCutOff`

The structural inventory also identifies `MarqueeView`,
`MarqueeViewModel`, `GameMarqueeViewModel`, and
`PlatformMarqueeViewModel`. The installed CriticalZone default game view uses
the game marquee image with uniform scaling, and its platform view uses the
platform banner with uniform scaling. The installed Old Default game view
establishes the fallback media vocabulary: silent marquee video, marquee
image, clear logo, box art, and background. Its platform view establishes
banner, clear-logo, and background fallbacks.

## Native cross-platform boundary

`lb-platform::media` owns strict parsing of the five settings and selection
from the existing bounded, symlink-safe media indexes. Game direct-media
priority is silent `Marquee` video, then `Arcade - Marquee`/`Marquee` image.
When theme views are allowed, clear logo, front box art, and fanart background
are available as port-owned fallbacks. Platform media is read only from direct
regular files in `Images/Platforms/<platform>/Banner`, `Clear Logo`, and
`Fanart`; unsafe names, symlinks, case-ambiguous platform directories, excess
files, and unsupported files fail closed.

CXX-Qt publishes guarded native local URLs and typed settings. QML owns a
separate frameless, always-on-top, non-focusable `Window`, silent looping Qt
Multimedia playback, game/platform context changes, and the settings editor.
A small C++ bridge uses only the cross-platform Qt
`QGuiApplication::screens()`, `QScreen`, and `QWindow::setScreen()` APIs. QML
does not contain Windows monitor APIs, Linux display commands, macOS screen
APIs, native path rules, or runtime shell commands. Invalid or disabled
marquee indexes hide the window; current screen topology is rechecked while
BigBox is running.

The editor sends one strict version-1 payload for all five settings. Rust
validates the complete payload, changes only the corresponding elements in
`BigBoxSettings.xml`, retains unknown data, commits one recoverable
root-scoped transaction with one exact sibling backup, rereads the committed
document, and publishes the new policy only after an exact typed round trip.

## Compatibility geometry

The enum names and stored defaults are recovered evidence. The protected
13.27 method bodies do not reveal the exact coordinate transforms. Until
real-host oracle comparison is available, the port uses this deterministic
clean-room interpretation, compressing the complete presentation into the
named visible region:

| Mode | Top | Height |
|---|---:|---:|
| `None` | `0` | full |
| `HalfSizeStretched` | `0` | half |
| `ThirdSizeStretched` | `0` | one third |
| `BottomHalfCutOff` | `0` | half |
| `TopHalfCutOff` | half | half |
| `BottomTwoThirdsCutOff` | `0` | one third |
| `TopTwoThirdsCutOff` | two thirds | one third |
| `TopAndBottomOneThirdCutOff` | one third | one third |

This table is a port policy, not a claim of pixel-exact protected behavior.
Exact compatibility geometry, custom-theme view execution, hot-plug topology
recovery, and real multi-monitor placement/playback on Windows, Linux, and
macOS remain explicit parity gates.

## Verification boundary

Pure tests freeze all settings/defaults/modes, malformed and bounded monitor
handling, game media priority/fallbacks, platform scanning, unsafe-entry
refusal, strict payload validation, lossless XML mutation, exact backup
retention, and transactional live-policy reload.

A compiled Xvfb scenario opens the real display editor, routes both native Qt
windows to the available screen, commits `TopHalfCutOff` with stretching and
theme-view override enabled, waits for genuine H.264 decoder readiness,
renders and color-checks the direct game marquee, navigates through the real
BigBox drawer to a platform, renders and color-checks its banner, validates
the exact safe native files in Rust, and proves that the complete media
manifest remains byte-identical. Xvfb supplies one virtual screen, so this is
not evidence for physical multi-monitor placement or native Windows/macOS Qt
Multimedia behavior.
