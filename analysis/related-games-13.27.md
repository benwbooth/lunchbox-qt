# LaunchBox 13.27 BigBox Related Games recovery

This note records the evidence used for the first native `BB-004`/`BB-017`
Related Games vertical. It contains no LaunchBox binaries, copied XAML,
artwork, metadata-database rows, or protected method bodies.

## Recovered product structure

The structurally decompiled 13.27 `RelatedGamesPopupViewModel` owns three
separate `GameSuggester` instances and three result collections:

1. Recommended Games;
2. Similar Games;
3. Possible Ports.

It also exposes current-section and current-game state plus left, right, up,
down, page, Enter, and Escape handlers. The installed Default theme resource
uses a centered, approximately 85%-opaque black popup with a three-pixel white
border, a white title bar, three tab labels, and 200-logical-pixel result rows.
Each row contains 150-logical-pixel artwork, title, score, year, platform, and
notes. Nonlocal database rows are dimmed and carry a cloud marker; the current
row uses the stock `#FF3399FF` blue.

Four inspected installations persist
`ShowGameMenuViewRelatedGames=true`. The native menu action therefore defaults
to visible when the setting is absent, honors an explicit false value, and
uses the recovered `BigBoxShowDiscoveryCenter` locked-mode permission.

## Recovered suggester profiles

The older complete installation contains serialized
`GameSuggesterSaveData` values for Recommended and Similar Games in
`Settings.xml`. These establish `AllowDbGames`, `MinimumScore`, required
criteria, weighted criteria, per-criterion local/database filters, and the
following defaults:

| Profile | Required criteria | Weighted criteria |
|---|---|---|
| Similar | database Notes is not empty; database ReleaseType is Released; Title differs from the selected game | Title similar 2; AlternateName similar 2; Series similar 2; Genre equal 3; PlayMode equal 2; MaxPlayers equal 1; Platform equal 2; Rating equal 2; Developer equal 1; Publisher equal 1 |
| Recommended | database ReleaseType is Released; Title differs; StarRating is greater than 3.5; local Series is not similar to the selected game | Genre equal 3; PlayMode equal 2; MaxPlayers equal 1; Platform equal 1; StarRating greater than 4.1 scores 3; Rating equal 2; Developer equal 1; Publisher equal 1 |

Both profiles allow local LaunchBox games and rows from the local
`LaunchBox.Metadata.db`. The port parses a bounded 256 KiB profile, rejects an
unknown field/comparison/filter or malformed profile as a whole, and falls
back to the recovered default rather than evaluating a partial contract.

No inspected installation contains a serialized Possible Ports profile. An
official LaunchBox forum answer describes possible ports as entries with the
exact same game name on another platform. The native profile therefore
requires an equal title, a different platform, and Released status for
database rows. Its payload source is explicitly `portReconstruction`; it is
not presented as a recovered hidden default.

## Protected behavior boundary

The semantic `GameSuggester` surface survives static extraction, but its
matching and score implementation is protected by the same runtime method-body
system documented in `RE_STATUS.md`. The native port can apply the recovered
criteria, filters, and weights, but cannot claim the original fuzzy-match or
percentage-rounding algorithm.

The explicit port policy is:

- text equality is trimmed and case-insensitive;
- fuzzy similarity first accepts equal normalized token sets, otherwise
  requires at least two shared distinct tokens and at least half overlap in
  both directions after dropping `a`, `an`, `the`, and `and`;
- weighted matches contribute their recovered integer weight;
- percentages use deterministic nearest-integer division;
- profiles with only required criteria report 100% once all requirements pass;
- ties use title, platform, database ID, and stable local ID;
- each section is bounded to ten results.

These choices are isolated in `lb-query`, named as port-owned behavior, and
covered by unit tests so a future Windows oracle can replace them without
changing the Qt or metadata boundaries.

## Native cross-platform implementation

`lb-query` owns typed profiles, strict XML parsing, required/weighted
evaluation, scoring, and stable result ordering. Local games retain their
stable LaunchBox IDs and alternate names. `lb-metadata` opens the optional
local SQLite metadata database read-only, projects only the suggestion fields,
and includes alternate names without copying the database into application
state. Database rows already represented by a local database ID or
title/platform pair are deduplicated.

The CXX-Qt controller loads candidates lazily on a named Rust worker, publishes
a versioned three-section JSON document, ignores stale generations after a
library replacement, and clears the result on failure. Selecting an installed
row clears incompatible filters, reveals the same stable game ID in the
existing wheel model, and returns to the game view. Database-only rows remain
informational.

One QML popup owns the recovered three-tab presentation and keyboard,
mapped-controller, and pointer behavior. It accepts only the strict versioned
payload, displays native local artwork URLs already resolved by the shared
media boundary, dims database rows, and never interprets a persisted path.
There is no runtime shell, host command, Windows separator rule, or
OS-specific UI branch in this feature.

## Verification boundary

Unit coverage freezes both recovered profiles, the reconstructed Possible
Ports source marker, strict malformed-profile fallback, local/database
filtering, score breakdowns, deterministic ties, similarity behavior,
metadata-corpus projection, alternate names, and stable installed IDs. A
controller test proves the versioned three-section payload and installed-port
identity.

The compiled BigBox scenario creates a real local SQLite metadata database
with one database-only exact-title port, opens Related Games through the
settings/security-gated action, waits for the lazy worker, and requires three
ordered sections. It verifies one installed Recommended result and one
database-only Possible Port, captures and color-checks the rendered popup,
switches tabs through the real input surface, selects the installed stable ID,
and byte-hashes `Data`, `Images`, `Metadata`, and `Music` before and after.

The exact protected similarity, percentage rounding, result cap, metadata
refresh cadence, custom-theme binding surface, remote database update behavior,
and native Windows/macOS Qt interaction remain parity gates.
