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

The provider assembly exposes its response contract without protected method
bodies:

- the root is `ResponseArray(Response?[]? data)`;
- a row contains signed `Id`, required `Title`, optional `Subtitle`,
  `ListType`, `SortBy`, `SortAsc`, `PriorityRank`, `MinimumItems`, and
  `MaximumItems`, plus nullable `Games` and `Criteria` collections;
- a manual item contains signed Games Database `Id`, `Platform`, and `Title`;
- an automatic criterion contains `Field`, `Comparison`, and nullable `Value`;
- JSON property matching is case-insensitive;
- the response is fetched once for the provider instance, and either the
  populated collection or the empty failure result is reused;
- rows with `PriorityRank` belong to the prioritized provider and the rest to
  the random provider;
- prioritized enumeration orders by rank and randomizes ties;
- nonempty criteria produce an `AutomaticPlaylist`; otherwise non-null games
  produce a `ManualPlaylist`;
- `SortAsc=false` becomes descending arrangement.

The decompiled enumerator appears to dequeue one item and then peek the next.
That would skip the first row and throw on the last row, so the port treats it
as protection/decompiler damage rather than intentional product behavior. It
preserves the independently visible semantic ordering instead of reproducing
an apparent crash.

The native adapter pins the exact HTTPS URL, refuses redirects, applies a
two-MiB document cap and bounded list/game/criterion/text counts, detects
case-ambiguous properties and duplicate IDs, and maps the recovered nullable
record shapes. Those limits and redirect policy are port-owned security
boundaries; they are not claimed as LaunchBox behavior. The current environment
could not resolve `api.gamesdb.launchbox-app.com` on 2026-07-26, so no live
response contents or current service availability are claimed.

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

`lb-integrations` owns the provider transport and bounded parser. `lb-query`
owns the typed six-section local projection, provider-list conversion, time
parsing, stable ordering, bounds, platform grouping, and source markers.
Manual provider rows resolve a positive Games Database ID first and fall back
to case-insensitive exact title/platform matching. Automatic rows use the
shared playlist matcher: OR inside one field and AND across distinct fields.
The matcher covers the represented Boolean, text, multivalue text, numeric,
and date contracts; a row with an unsupported field, comparison, malformed
number/date, or unknown arrangement is rejected instead of silently producing
incorrect membership. Recognized provider arrangement uses the same typed
stable sort engine as the desktop library.

The CXX-Qt controller publishes one strict versioned JSON document and exposes
stable-ID game reveal. It opens immediately with local lists and a `loading`
provider state, fetches on a named Rust worker, then generation-checks the Qt
completion before publishing `ready` or `unavailable`. The first success or
failure is cached for the controller lifetime, matching the recovered
fetch-once behavior. A library replacement cannot receive a stale worker
result. Revealing a game clears incompatible filters and selects that exact
record in the existing wheel; selecting a platform uses the existing typed
platform filter.

One full-screen Qt Quick page validates the version, contract source, provider
state/endpoint/counts, the first six raw section keys/order, bounded dynamic
provider rows, and item shapes before rendering. It omits unavailable or
under-minimum rows from presentation while retaining them in the validated
contract. It shows online-list loading/ready/offline state. Keyboard,
mapped-controller, and pointer input move across lists and items, page lists,
open stable game/platform selections, and return to the wheel. Each horizontal
list tracks its current item, so selection remains visible beyond the first
viewport.

The product implementation contains no shell invocation, `/bin/sh`, Windows
separator interpretation, host command, or OS-specific QML branch. Persisted
paths remain below the existing cross-platform Rust path boundary; QML
receives only guarded native local artwork URLs.

## Verification boundary

Pure integration/query tests freeze all six slots and their order, rating
precedence and stable ties, recently-played ordering, the exact Recently Added
bounds, platform grouping, favorite filtering, hidden/broken exclusion, the
MAME pending marker, and the no-result display rules. Provider tests cover the
case-insensitive and nullable schema, signed IDs, caps, duplicate/ambiguous
input, exact URL pinning, manual ID and fallback resolution, automatic
criteria, nullable bounds and directions, priority-before-random ordering, and
unsupported-semantic rejection. Shared matcher tests cover field grouping plus
Boolean, multivalue text, numeric, and date comparisons.

The compiled BigBox scenario loads a portable fixture through the real
controller, enters Discovery through the central input/security dispatcher,
requires the six local contract sections plus two provider sections, verifies
one manual database-ID row and one automatic-criteria row, traverses rows and
cards, captures and color-checks the visibly selected provider row, selects
the exact stable game ID, and hashes the complete fixture before and after.
The provider fixture is injected only through an explicit smoke-test method;
production library loads use the HTTPS transport and never interpret a fixture
path or environment variable.

The protected Highly Rated, Recently Played, Platforms, and Favorites
algorithms; the MAME high-score adapter; a current live provider response;
custom-theme binding surface; and native Windows/macOS Qt interaction remain
parity gates.
