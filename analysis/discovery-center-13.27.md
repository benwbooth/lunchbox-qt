# LaunchBox 13.27 BigBox Discovery Center recovery

This note records the evidence used for the first native `LIB-013`/`BB-017`
Discovery Center vertical. It contains no LaunchBox binaries, copied XAML,
artwork, library records, remote-service responses, or protected method
bodies.

## Recovered product structure

The structurally decompiled 13.27 `DiscoveryPageViewModel` exposes
`HighlyRatedList`, `RecentlyPlayedList`, `PlatformsList`, `FavoritesList`,
`MameHighScoresList`, and `ActiveItem`, together with item, list, page, Enter,
and Back navigation. The embedded Default-theme `DiscoveryPageView` supplies
the missing inline Recently Added list and fixes the complete order:

1. Highly Rated;
2. Recently Played;
3. Recently Added;
4. Platforms;
5. Favorites;
6. MAME High Scores.

The inline Recently Added contract is exact: `Sort="DateAdded"`,
`MaximumItems="25"`, `MinimumItems="5"`, and one `DateAdded RecentDays 360`
criterion. The theme also identifies the special `Platforms`,
`RecentlyPlayed`, `Recommended`, and `MameHighScores` list types and binds
active item media. The native payload retains all six ordered slots even when
one is unavailable, so later adapters can fill the recovered position without
changing the controller/QML contract.

The 13.27 evidence is in:

- `decompiled/BigBox/Unbroken/LaunchBox/Windows/BigBox/ViewModels/DiscoveryPageViewModel.cs`;
- the `DiscoveryPageView` value in
  `decompiled/BigBox/Unbroken.LaunchBox.Windows.BigBox.Properties.ThemedViews.resx`;
- `decompiled/Unbroken.LaunchBox.Plugins/Unbroken/LaunchBox/Plugins/PlaylistBase.cs`;
- `decompiled/Unbroken.LaunchBox.Plugins/Unbroken/LaunchBox/Plugins/PlaylistProviderPlugin.cs`;
- `decompiled/Unbroken.LaunchBox.Windows.PlaylistProvider/Unbroken/LaunchBox/Windows/PlaylistProvider/ProviderPlugin.cs`.

The user's older complete installation at
`/mnt/Windows/Users/benwb/LaunchBox` identifies itself as 13.24 in
`Data/Settings.xml`. Its
`Themes/Default/Views/DiscoveryPageView.xaml` independently contains the same
six-list block, order, and exact Recently Added contract. That installation was
used only as corroborating structural evidence; its library data and theme
file are not copied into the port.

## Recovered provider contract

The public plugin surface models each discovery playlist with stable ID,
list type, title, subtitle, arrangement, direction, minimum, and maximum item
fields. `ProvidePrioritizedPlaylists()` contributes before
`ProvideRandomPlaylists()`. The concrete 13.27 provider fetches
`https://api.gamesdb.launchbox-app.com/api/discovery-lists`, prioritizes
`PriorityRank`, randomizes ties, and projects manual or automatic playlists.

The native page deliberately does not contact that service yet. Remote schema
versioning, cache/failure behavior, consent and privacy policy, deterministic
testing, and exact merge behavior need their own evidence-backed adapter. No
test substitutes a fabricated response and no static built-in row is claimed
to implement that provider.

## Protected behavior boundary

The list properties and theme contract survive static extraction, but the
protected view-model methods do not expose the original ranking and MAME
adapter implementations. The native local projection therefore separates
recovered structure from explicit port policy:

- hidden and broken games are excluded from this initial safe projection;
- Highly Rated orders community rating, then local float rating, then legacy
  integer rating, with vote count and stable identity ties;
- Recently Played orders parsed `LastPlayedDate` newest first;
- Recently Added uses the recovered inclusive 360-day window, newest first,
  with the recovered minimum and maximum;
- Platforms use stable case-insensitive title order, game counts, and a stable
  representative game for artwork;
- Favorites use stable title and ID order;
- every game list is bounded to 25 items;
- the MAME High Scores slot is retained but marked unavailable and
  undisplayable until its separate high-score adapter exists.

Payload `source` values distinguish `recoveredThemeContract` from
`recoveredViewModelPortRanking`, `recoveredViewModelPortProjection`, and
`recoveredViewModelAdapterPending`. These names prevent the deterministic
native ranking from being mistaken for recovered protected behavior.

## Native cross-platform implementation

`lb-query` owns the typed six-section projection, time parsing, stable
ordering, bounds, platform grouping, and source markers. The CXX-Qt controller
publishes one strict versioned JSON document and exposes stable-ID game reveal.
Revealing a game clears incompatible filters and selects that exact record in
the existing wheel; selecting a platform uses the existing typed platform
filter.

One full-screen Qt Quick page validates the version, contract source, six raw
section keys, order, and item shapes before rendering. It omits unavailable or
under-minimum rows from presentation while retaining them in the validated
contract. Keyboard, mapped-controller, and pointer input move across lists and
items, page lists, open stable game/platform selections, and return to the
wheel. Each horizontal list tracks its current item, so selection remains
visible beyond the first viewport.

The product implementation contains no shell invocation, `/bin/sh`, Windows
separator interpretation, host command, or OS-specific QML branch. Persisted
paths remain below the existing cross-platform Rust path boundary; QML
receives only guarded native local artwork URLs.

## Verification boundary

Pure query tests freeze all six slots and their order, rating precedence and
stable ties, recently-played ordering, the exact Recently Added bounds,
platform grouping, favorite filtering, hidden/broken exclusion, the MAME
pending marker, and the no-result display rules.

The compiled BigBox scenario loads a portable fixture through the real
controller, enters Discovery through the central input/security dispatcher,
requires six contract sections and four visible fixture sections, traverses
rows and cards, captures and color-checks the rendered page, selects the exact
stable game ID, and hashes the complete fixture before and after.

The protected Highly Rated, Recently Played, Platforms, and Favorites
algorithms; the MAME high-score adapter; prioritized/random remote provider
lists; provider cache/offline behavior; custom-theme binding surface; and
native Windows/macOS Qt interaction remain parity gates.
