# LaunchBox 13.27 platform-removal contract

This note freezes the clean-room evidence and native-port boundary for deleting
a platform with dependent records. It contains no protected implementation
body and introduces no Wine, .NET, WPF, registry, shell, or Windows-path
dependency into the product.

## Recovered 13.27 evidence

The installed 13.27 desktop contains
`Windows/Desktop/MenuActions/DeletePlatformMenuAction` and
`Windows/Desktop/ViewModels/ManagePlatformsViewModel.DeletePlatform`. Their
protected bodies remain unavailable, so they establish entry points but not a
transaction algorithm.

The unprotected English resources establish the visible behavior:

- `MessageDeletePlatformAreYouSure` says the selected platform and all
  associated games are permanently deleted.
- `MessageDeletePlatformWarning` says every placement is removed, while
  categories and playlists directly beneath the platform are moved to the
  root and are not deleted.
- `MessageWouldYouLikeToDeletePlatformMedia` presents media deletion as a
  separate choice and warns that retained media will need later manual review.
- `MessageDeletePlatformGamesFolder`,
  `MessageDeletePlatformImagesFolder`, and
  `MessageDeletePlatformMediaFolders` establish separate irreversible
  filesystem prompts.

This proves collection-record removal, detach-to-root hierarchy behavior, and
separate optional file/media deletion. It does not prove the protected
mutation order, rollback strategy, every stale-reference rule, or which
filesystem paths 13.27 accepts for deletion.

## Native port contract

The first portable remediation subset deliberately implements only the
recoverable collection operation:

1. The ordinary Delete action freshly scans all ten modeled platform-reference
   families and refuses a non-empty dependency set.
2. A second, explicit review identifies the current platform and dependency
   summary. A stale, dismissed, renamed, or unrelated review cannot authorize
   the operation.
3. The transaction validates that every game in the deleted document belongs
   to the selected platform and that every owned additional application,
   mount, alternate name, custom field, controller-support row, game save, and
   model setting belongs to one of those games.
4. It removes the platform catalog definition, owned folder rows, platform
   model settings, and the complete platform XML document.
5. It removes emulator mappings and clears matching emulator defaults; removes
   platform placements and detaches retained children to root; removes
   matching playlist games and filters; clears navigation selections; removes
   controller associations and matching frontend settings; removes deleted
   game IDs from the import blacklist; and clears retained cross-platform
   clone relationships and orphaned modeled game records.
6. Every changed XML document and the deleted platform document commit under
   one cross-process lock, exact source-revision checks, one durable recovery
   manifest, and exact sibling recovery copies. The Qt model is reloaded from
   the committed documents.

Stored LaunchBox path strings remain lexical XML. This operation never resolves
or opens a game, emulator, media, manual, music, video, save, or folder path.
It never invokes a command interpreter. ROMs, media, manuals, music, videos,
save files, and directories are always retained.

## Deliberately open boundary

The original's separately prompted deletion of game folders or media is not
implemented. A cross-platform port cannot safely infer that a lexical
LaunchBox path denotes an exclusively owned native directory, especially for
mapped Windows drives, UNC paths, shared ROM folders, removable storage, or
case-sensitive hosts. Any future file-removal option must use an explicit
preview, stable ownership and path-mapping evidence, symlink/special-file
refusal, revision checks, recoverable moves, and native Windows, Linux, and
macOS conformance tests.
