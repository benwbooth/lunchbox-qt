# LaunchBox 13.27 Attract Mode recovery

This note records the evidence used for the first native `BB-010` port subset.
It contains no LaunchBox binaries, assets, or copied proprietary method bodies.

## Recovered persisted contract

The value-free 13.27 schema in `analysis/real-install-schema.json` contains the
following BigBox settings:

| Setting | Observed default |
|---|---:|
| `EnableAttractMode` | `false` |
| `AttractModeSwitchFilters` | `true` |
| `AttractModeDelay` | `120` seconds |
| `AttractModeTimePerMovement` | `5` seconds |
| `AttractModeMaximumSpeed` | `20` milliseconds |
| `AttractModeMinimumSpeed` | `200` milliseconds |
| `PlayMoveInAttractMode` | `false` |
| `VolumeAttractModeNavigationSound` | `15` percent |
| `VolumeAttractModeMaster` | `100` percent |

The defaults agreed between a fresh 13.27 Wine installation and the older
complete installation on the Windows partition. Both installations also
contained `KeyboardStartAttractMode`, its three additional keyboard slots,
and `ControllerStartAttractMode`; their stored values were zero in the
inspected installations, so no product-specific default binding is inferred.

## Recovered behavior

Static inspection identifies the BigBox `AttractMode` type with timer-owned
initialization, stop, elapsed, wheel-spin, and view/filter-switch state. The
BigBox input action and options page establish both explicit start and
idle-driven entry. Installed resources and release notes establish repeated
wheel movement, optional filter switching, move-sound policy, separate
navigation/master volume, and exit on user input.

The protected 13.27 method bodies do not expose the exact wheel step count,
acceleration function, filter-selection algorithm, or controller binding
interpretation. The port therefore does not claim those details as recovered
parity. It uses a documented 16-step symmetric native curve bounded by the
persisted minimum/maximum intervals, switches only among non-empty navigation
rows, supplies an explicit `A` shortcut until the general input-mapping family
is implemented, and treats key, pointer-button, wheel, and the visible return
control as exit input.

## Port boundary

`lb-platform::attract` owns strict parsing and safe defaults. The CXX-Qt
controller exposes only typed values, bounded interval queries, and
symlink-safe native local URLs for direct WAV files under
`Sounds/<SoundPack>/Move`, with exact legacy `Move.wav` fallback. QML owns the
idle timer, active focus/input layer, movement lifecycle, visible status, and
Qt Multimedia player/output pair. No command shell or OS-specific path literal
is used by the runtime feature.

Pure tests freeze the settings contract and curve. Compiled offscreen scenarios
prove delayed automatic entry, explicit manual entry with automatic mode both
enabled and disabled, movement and filter switching, decoded WAV selection,
the effective `40 * 50 / 10000 = 0.20` volume, exit through actual controls,
rendering, and byte-identical settings/media after use.
