# LaunchBox 13.27 BigBox favorites and star-rating recovery

This note records the evidence used for the first native `BB-017` game-action
vertical. It contains no LaunchBox binaries, copied XAML, artwork, or protected
method bodies. Playlists, discovery/random behavior, related games, and premium
entitlement are separate concerns and are not claimed by this work.

## Recovered persisted contract

Three independent fresh 13.27 Wine installations agree on these values in
`Data/BigBoxSettings.xml`:

| Setting | Fresh default |
|---|---:|
| `ShowStarNextToFavoritedGames` | `true` |
| `ShowFavoritedGamesFirst` | `true` |
| `ShowGameFavorite` | `true` |
| `ShowGameMenuFavorite` | `true` |
| `ShowGameMenuStarRating` | `true` |
| `ShowGameStarRating` | `true` |

Missing or malformed values fall back to those observed defaults. The settings
control the selected-game favorite marker, favorite-first wheel ordering,
details visibility, and the two game-menu actions independently.

The recovered 13.27 game contract contains both integer `StarRating` and
floating-point `StarRatingFloat`. The checked-in full game fixture preserves
the concrete pair `4` and `4.5`, demonstrating that the fractional value is not
derived from the integer companion. Favorite is persisted independently as a
boolean.

## Recovered popup structure

The structural `StarRatingPopupViewModel` inventory exposes a floating-point
`Value`, community rating, vote count, details, popup width, a set action, and
directional, Enter, and Escape handlers. The decoded installed resource is a
centered dark popup approximately 900 by 480 logical pixels. It presents five
faded stars with a clipped bright overlay, allowing the displayed value to end
part-way through a star.

The protected value-stepping and pointer-rounding method bodies are not
available as clean-room behavioral evidence. The native port therefore uses
half-star steps from 0 through 5. This matches the persisted `4.5` evidence,
keeps both XML fields interoperable, and is explicitly port-owned policy rather
than a claim about an unrecovered LaunchBox algorithm. The legacy integer field
is the floor of the accepted floating-point value.

## Native cross-platform boundary

`lb-platform::engagement` owns the six typed settings, half-star validation,
and stable favorite-first projection. `lb-storage` updates Favorite,
`StarRating`, and `StarRatingFloat` together while retaining completion and
unknown XML. The controller commits the containing platform document through
the existing recoverable root-scoped transaction, selects an exact backup,
rereads the committed document, and only then publishes the refreshed wheel
and revision.

The BigBox wheel, details, and game menu honor the recovered visibility
settings. Favorite toggling uses the existing `FavoriteGames` security
permission; opening and saving the rating uses `SetStarRating`. The native Qt
popup accepts keyboard, mapped controller, and pointer input and renders the
fractional value through the recovered faded/bright five-star composition.

The implementation uses Rust, CXX-Qt, QML, portable paths supplied by the
storage boundary, and Qt input/window APIs. It contains no runtime shell,
platform command, Windows path interpretation in QML, or OS-specific UI code,
so the same application code applies to Windows, Linux, and macOS.

## Verification boundary

Pure tests freeze all six defaults and malformed fallback, stable
favorite-first ordering, half-star acceptance/refusal, integer companion
behavior, lossless XML retention, and completion preservation. Security tests
cover both game-action permission mappings.

A compiled Xvfb scenario starts locked with setting-star-ratings allowed and
favoriting denied. It proves the Favorite action is blocked, opens and renders
the real popup at `4.5`, commits `2.5`, unlocks with the configured PIN, removes
Favorite, and observes the final live revision. Rust and the shell gate then
validate Favorite `false`, integer rating `2`, floating rating `2.5`, unchanged
completion and unknown XML, one exact original backup, one exact intermediate
backup, immutable peer files, and no interrupted-recovery manifest.

The exact protected value-step/pointer-rounding behavior, custom-theme binding
surface, playlists, discovery/random behavior, related games, and native
Windows/macOS Qt interaction remain explicit parity gates.
