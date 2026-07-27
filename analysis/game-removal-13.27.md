# LaunchBox 13.27 game-removal contract

This note freezes the clean-room evidence and native-port boundary for removing
an individual game from a LaunchBox collection. It contains no protected
implementation body and introduces no Wine, .NET, WPF, registry, shell, or
Windows-path dependency into the product.

## Recovered 13.27 evidence

The installed desktop contains
`Windows/Desktop/Commands/DeleteSelectedGamesCommand` and
`Windows/Desktop/ViewModels/GameDetailsViewModel.DeleteGame`. Their protected
bodies remain unavailable, so they establish entry points but not a mutation
algorithm.

The unprotected 13.27 resources establish the user-visible separation:

- `MessageDeleteGamePermanent` asks whether to permanently delete one named
  game.
- `MessageDeleteSelectedGames` says selected games are permanently deleted
  from the local collection.
- `MessageDeleteRomFile` and `MessageDeleteRomFiles` separately ask whether to
  remove ROM files from the hard drive and warn that this cannot be undone.
- `MessageDeleteAdditionalAppRoms` separately covers additional-application
  ROMs.
- `MessageDeleteGameMediaFiles` separately asks whether to remove associated
  media.
- `AllowDeletingRoms` and `DeleteAssociatedMediaOnGameDelete` are distinct
  persisted settings in the 13.27 settings contract.

This proves that collection-record removal does not itself authorize ROM,
additional-application ROM, or media-file deletion. The protected bodies do
not expose mutation order, rollback behavior, every stale-reference rule, or
the exact media/file ownership algorithm.

## Native port contract

The first portable remediation subset deliberately implements the recoverable
collection operation only:

1. The ordinary Delete action freshly scans every modeled reference and
   refuses a non-empty dependency set.
2. A separate review is tied to the exact current game ID and dependency
   summary. A stale, dismissed, or unrelated review cannot authorize removal.
3. The transaction removes the selected `<Game>` and its owned additional
   applications, DOSBox mounts, alternate names, custom fields, controller
   support rows, game-save metadata, and game model settings.
4. Retained games are never removed. Clone relationships targeting the deleted
   ID are cleared. Playlist membership and last-game navigation references are
   removed, affected playlist cache rows are invalidated, and import-blacklist
   rows for the ID are removed.
5. The source platform, every changed peer platform or playlist, navigation
   catalog, optional list cache, and import blacklist commit under one
   cross-process lock, exact source-revision checks, one durable recovery
   manifest, and exact sibling recovery copies.
6. The Qt library reloads from the committed documents so LaunchBox and BigBox
   projections do not retain stale game, playlist, navigation, save, or model
   state.

Stored LaunchBox paths remain lexical XML. This operation never resolves or
opens a game, additional-application, emulator, media, manual, music, video,
save, or folder path and never invokes a command interpreter. All ROMs, media,
manuals, music, videos, save files, and directories are retained.

## Deliberately open boundary

ROM, additional-application ROM, and associated-media deletion are not
implemented. A cross-platform port cannot infer exclusive ownership from a
lexical LaunchBox path, especially for mapped Windows drives, UNC paths, shared
ROM folders, removable storage, archives, multi-disc sets, or case-sensitive
hosts. Any future file-removal option must use an explicit per-path preview,
stable ownership and path-mapping evidence, link and special-file refusal,
revision checks, recoverable moves, and native Windows, Linux, and macOS
conformance tests.
