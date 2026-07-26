# LaunchBox 13.27 BigBox security recovery

This note records the evidence used for the first native `BB-011` PIN/security
vertical. It contains no LaunchBox binaries, copied XAML, artwork, or
proprietary method bodies. Premium entitlement is a separate concern and is
not bypassed or emulated by this work.

## Recovered persisted contract

Three independent fresh 13.27 Wine installations agree on the security values
in `Data/BigBoxSettings.xml`. A fresh installation omits `LockPin`, sets
`ShowGameLockUnlock` to `true`, and contains exactly these 32 permission
settings:

| Setting | Fresh default |
|---|---:|
| `AllowExitWhileUnlocked` | `false` |
| `AllowSettingStarRatingsWhileLocked` | `false` |
| `AllowOpeningGameFoldersWhileLocked` | `false` |
| `AllowOpeningGameImageFoldersWhileLocked` | `false` |
| `AllowOpeningEmulatorsWhileLocked` | `false` |
| `AllowFavoritingGamesWhileLocked` | `false` |
| `AllowHidingGamesWhileLocked` | `false` |
| `AllowMarkingGamesAsBrokenWhileLocked` | `false` |
| `AllowModifyingProgressWhileLocked` | `false` |
| `AllowSleepWhileLocked` | `false` |
| `AllowShutDownWhileLocked` | `false` |
| `AllowRebootWhileLocked` | `false` |
| `AllowChangeViewWhileLocked` | `false` |
| `AllowChangeImageTypeWhileLocked` | `false` |
| `AllowNavigateToGameDiscoveryCenterWhileLocked` | `true` |
| `AllowChangeFilterAllGamesWhileLocked` | `true` |
| `AllowChangeFilterPlatformsWhileLocked` | `true` |
| `AllowChangeFilterPlatformCategoriesWhileLocked` | `true` |
| `AllowViewRetroarchNetplayBrowserWhileLocked` | `true` |
| `AllowChangeFilterPlaylistsWhileLocked` | `true` |
| `AllowChangeFilterGenresWhileLocked` | `true` |
| `AllowChangeFilterDevelopersWhileLocked` | `true` |
| `AllowChangeFilterPublishersWhileLocked` | `true` |
| `AllowChangeFilterSeriesWhileLocked` | `true` |
| `AllowChangeFilterStatusesWhileLocked` | `true` |
| `AllowChangeFilterSourcesWhileLocked` | `true` |
| `AllowChangeFilterRatingsWhileLocked` | `true` |
| `AllowChangeFilterPlayModesWhileLocked` | `true` |
| `AllowChangeFilterRegionsWhileLocked` | `true` |
| `AllowThemesDemoWhileLocked` | `true` |
| `AllowSearchWhileLocked` | `true` |
| `AllowViewAchievementProfileWhileLocked` | `false` |

`AllowExitWhileUnlocked` is the actual serialized 13.27 key even though it is
used as the locked-mode Exit permission. The port preserves that spelling.
Missing or malformed booleans fall back to the observed fresh-install value.

The evidence is independently visible in the recovered
`Unbroken.LaunchBox.Windows...Data.BigBoxSettings` property inventory, the
BigBox `GetSecurityPage` structural entry point, and the value-free real-install
schema. The protected method bodies are not treated as behavioral evidence.

## Recovered PIN interaction

The installed `PinPopupView` resource establishes a four-row, three-column
keypad:

| | | |
|---:|---:|---:|
| 7 | 8 | 9 |
| 4 | 5 | 6 |
| 1 | 2 | 3 |
| 0 | Delete | Done |

The recovered string resources provide Set Pin, Clear Pin, Enter your pin,
Repeat your pin, Pins did not match, Incorrect pin, Show Lock/Unlock, and
Lock/Unlock. The structural view model exposes keyboard/controller directional
navigation, Enter, Escape, pointer buttons, a masked PIN value, and a completion
callback.

The exact protected PIN-length rule was not recoverable. The persisted type is
a string while the visible keypad can enter only digits. The native port
therefore accepts 1 through 32 ASCII digits: the numeric shape is interoperable
and the explicit upper bound prevents malformed data from creating unbounded
UI state. This 32-digit limit is a clean-room safety policy, not an assertion
about an undiscovered LaunchBox limit.

## Native cross-platform boundary

`lb-platform::security` owns the typed settings, PIN validation, redacted
debugging, verification, and permission-to-input/navigation mapping. The PIN
never becomes a CXX-Qt property, model role, status message, or diagnostic;
only a configured flag and explicit verification call cross the bridge.
BigBox starts locked whenever a valid PIN is configured.

The native Qt UI supplies the recovered keypad shape, masked entry,
set-and-repeat flow, mismatch/incorrect feedback, clear-PIN intent, all 32
permission choices, and `ShowGameLockUnlock`. Locked actions pass through one
central dispatcher gate. Exit and window close, view/image changes, attribute
filters, discovery/search, and all implemented navigation kinds use the same
typed policy. Disabled navigation rows are skipped instead of briefly exposing
their contents. Unknown navigation kinds fail closed.

The editor submits one strict version-1 payload containing every permission
exactly once plus an explicit keep/set/clear PIN intent. Rust changes only
those singleton fields in `BigBoxSettings.xml`, retains unknown data, commits
one recoverable root-scoped transaction with an exact sibling backup, rereads
the committed document, and publishes the new policy and revision before
publishing `writing=false`. Clearing a PIN removes `LockPin` rather than
serializing an empty value.

The implementation uses Rust, CXX-Qt, QML, and portable Qt input/window APIs.
It contains no runtime shell, platform command, native path rule, OS credential
store, or Windows-only security API, so the same code path applies to Windows,
Linux, and macOS.

## Verification boundary

Pure tests freeze all 32 keys, their order and defaults, missing/malformed
fallback, valid/invalid PINs, redacted diagnostics, action/navigation mapping,
unknown-kind refusal, and lossless optional-field removal. Controller tests
exercise strict complete payload validation, set and clear transactions, exact
backups, unknown-field retention, committed typed rereads, and conflict-safe
mutation.

A compiled Xvfb scenario starts from a configured PIN and a denied Platforms
permission, proves automatic locked state and two blocked actions, renders and
color-checks the real keypad, rejects an incorrect PIN, unlocks with the
original PIN, renders the complete editor, repeats and saves a replacement
PIN plus two permission values, relocks, rejects the old PIN, accepts the new
PIN, and reports exactly one live-policy write. Rust and the shell gate then
validate the complete 32-field XML contract, exact pre-transaction backup,
absence of PIN values from output, unchanged theme/peer files, and absence of
an interrupted-recovery manifest.

Only permission-gated actions already implemented by the native port can be
connected to live behavior today; the remaining settings are preserved and
editable for later feature verticals. Exact protected timing/selection visuals,
native Windows/macOS Qt interaction, premium-license entitlement, and
permissions for still-unimplemented actions remain explicit parity gates.
