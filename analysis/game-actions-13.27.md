# LaunchBox 13.27 BigBox favorites, ratings, and playlist-action recovery

This note records the evidence used for the native `BB-017` favorite, rating,
and selected-game playlist-action verticals. It contains no LaunchBox binaries,
copied XAML, artwork, or protected method bodies. Discovery/random behavior,
related games, custom-theme execution, and premium entitlement are separate
concerns and are not claimed by this work.

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
details visibility, and the two favorite/rating menu actions independently.

`ShowGameMenuPlaylistActions` is omitted from all three fresh files, while its
recovered 13.27 property getter defaults to `true`. The port therefore treats a
missing or malformed value as enabled and honors an explicit `false`.

The recovered 13.27 game contract contains both integer `StarRating` and
floating-point `StarRatingFloat`. The checked-in full game fixture preserves
the concrete pair `4` and `4.5`, demonstrating that the fractional value is not
derived from the integer companion. Favorite is persisted independently as a
boolean.

## Recovered popup and action structure

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

The recovered `AddToPlaylistMenuAction` is constructed with the selected game.
`RemoveFromPlaylistMenuAction` is constructed with the selected game and
current playlist and exposes the removed playlist. Release-note evidence says
Add to Playlist was corrected to exclude auto-populated and auto-generated
playlists, matching the desktop restriction to manually populated playlists.
The native target projection therefore offers only manual, non-generated
playlists that do not already contain the selected game. Remove is offered only
while browsing a manual, non-generated playlist that explicitly contains it.

Both actions use the shared recovered list-popup contract: title, ordered
items, current item, Up/Down/Left/Right, Page Up/Down, Enter, Escape, and
double-click. The decoded installed resource is a centered approximately
85%-black surface with a white bold title and blue current-row selection. The
native popup preserves that structure and accepts keyboard, mapped-controller,
and pointer input.

The protected 13.27 append-order assignment is unavailable. A real older
LaunchBox installation on the Windows partition stores its original manual
rows as `0` through `27` and later appended rows as `ManualOrder=-1`. The port
uses that interoperable append sentinel and leaves explicit reordering to the
playlist editor. This is documented clean-room policy, not a claim that every
13.27 code path assigns the same value.

## Native cross-platform boundary

`lb-platform::engagement` owns the seven typed settings, half-star validation,
and stable favorite-first projection. `lb-storage` updates Favorite,
`StarRating`, and `StarRatingFloat` together while retaining completion and
unknown XML. The controller commits the containing platform document through
the existing recoverable root-scoped transaction, selects an exact backup,
rereads the committed document, and only then publishes the refreshed wheel
and revision.

Playlist membership uses a typed `PlaylistGame` row and the same lossless
auxiliary-document and root-scoped transaction boundary. It rejects
auto-populated/generated documents, changes exactly one manual membership,
retains filters and unknown XML, selects the exact backup, rereads and verifies
the committed playlist, refreshes a live playlist projection, and publishes a
separate revision before write completion.

The BigBox wheel, details, and game menu honor the recovered visibility
settings. Favorite toggling uses the existing `FavoriteGames` security
permission; opening and saving the rating uses `SetStarRating`. No recovered
locked-mode permission exists for membership mutation, so the central security
policy deliberately fails the playlist action closed while locked instead of
inventing a permission.

The implementation uses Rust, CXX-Qt, QML, portable paths supplied by the
storage boundary, and Qt input/window APIs. It contains no runtime shell,
platform command, Windows path interpretation in QML, or OS-specific UI code,
so the same application code applies to Windows, Linux, and macOS.

## Verification boundary

Pure tests freeze all seven defaults and malformed fallback, stable
favorite-first ordering, half-star acceptance/refusal, integer companion
behavior, lossless XML retention, and completion preservation. Security tests
cover both recovered favorite/rating permission mappings. Additional storage
and controller tests prove case-insensitive exact membership, manual-only
target projection, deterministic target ordering, derived-playlist refusal,
the `-1` append sentinel, exact backup chaining, committed rereads, and unknown
playlist/game/root XML retention.

One compiled Xvfb scenario starts locked with setting-star-ratings allowed and
favoriting denied. It proves the Favorite action is blocked, opens and renders
the real popup at `4.5`, commits `2.5`, unlocks with the configured PIN, removes
Favorite, and observes the final live revision. Rust and the shell gate then
validate Favorite `false`, integer rating `2`, floating rating `2.5`, unchanged
completion and unknown XML, one exact original backup, one exact intermediate
backup, immutable peer files, and no interrupted-recovery manifest.

A second compiled Xvfb scenario starts locked with one manual, one
auto-populated, and one generated playlist. It proves the unknown action is
denied, unlocks without exposing the PIN, requires only the manual playlist in
the rendered picker, adds the selected game, navigates the real manual
playlist, removes it, and observes the live filtered wheel. Rust and the shell
gate validate two writes/revisions, final puzzle-only membership, one exact
original and one exact added-state backup, `ManualOrder=-1` in the added state,
unknown XML retention, byte-identical automatic/generated/settings/platform/
parent peers, and no recovery manifest.

The exact protected value-step/pointer-rounding and append-order algorithms,
custom-theme binding surface, discovery behavior, related games, and native
Windows/macOS Qt interaction remain explicit parity gates.
