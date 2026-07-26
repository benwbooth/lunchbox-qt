# LaunchBox 13.27 application-data backup contract

## Scope

This note records the value-free evidence used for the first `LIB-018`
implementation slice. The original installation, its archives, and decompiled
first-party assemblies remain ignored. No game names, platform names, stored
paths, accounts, license values, or user XML values are copied into this
repository.

This milestone implements manual **Create Data Backup** and **Restore Data
Backup** in LaunchBox desktop mode. Automatic startup/shutdown backup,
cross-process scheduling, and the 25-archive retention policy remain open.

## Recovered 13.27 evidence

Static 13.27 resources expose:

- `_Create Data Backup...` and `_Restore Data Backup...` desktop actions;
- an `AutoBackupLaunchBox` option labeled as automatic LaunchBox XML-data
  backup;
- help text stating that both LaunchBox and Big Box back up the contents of
  `LaunchBox\Data` on startup and shutdown;
- `LaunchBox\Backups` as the automatic archive directory;
- a maximum of 25 retained automatic archives;
- saving, restoring, success, unchanged-on-failure, and unexpected-error
  messages.

The protected bodies of
`BackupDataAndSettingsMenuAction`,
`RestoreDataAndSettingsMenuAction`, and
`OptionsBackupsPageViewModel` remain unavailable in the structural
decompilation. The strings establish the user-facing contract, not the hidden
algorithm.

A fresh 13.27 data tree has these core entries:

```text
BigBoxSettings.xml
Emulators.xml
GameControllers.xml
InputBindings.xml
ListCache.xml
Parents.xml
Platforms/
Platforms.xml
Playlists/
Settings.xml
```

Its exact `Settings.xml` includes `AutoBackup=true`. Observed automatic
archives use `.7z` names such as `Automatic Big Box Startup Data Backup
YYYY-MM-DD HH-mm-ss.7z` and `Automatic LaunchBox Shutdown Data Backup
YYYY-MM-DD HH-mm-ss.7z`.

Technical 7-Zip listings establish that the contents of `Data` are stored at
the archive root; there is no enclosing `Data/` member. Empty `Platforms/` and
`Playlists/` directories are retained. A populated archive also retains
optional/future root documents and every nested platform and playlist
document. Games, ROMs, emulators, images, videos, manuals, music, saves, and
other sibling trees are not part of this application-data archive.

## Native implementation contract

The port treats the entire `Data` tree as an opaque-compatible snapshot with a
small required 13.27 validity floor:

1. Require the ten fresh-install entries above.
2. Parse every required root XML document completely and require a
   `LaunchBox` root.
3. Require the `Settings` and `BigBoxSettings` singleton records.
4. Parse direct XML documents in `Platforms/` and `Playlists/`.
5. Retain every other safe regular file and directory without interpreting or
   dropping it.
6. Reject symlinks, special files, non-Unicode or non-portable names, and
   case-insensitive collisions.
7. Enforce archive, member, aggregate-size, and entry-count limits before and
   after extraction.

Creation hashes the complete source tree, invokes 7-Zip directly with an
argument vector, re-extracts the result into a private directory, and accepts
the archive only when both the source and extracted content revisions still
match. Paths never pass through a command shell.

Restore first validates and extracts the complete archive without touching the
active library. It then acquires the same cross-process lock used by XML
transactions, refuses pending recovery manifests, rechecks both the archive
and active tree revisions, copies to a durable sibling staging tree, and
atomically replaces `Data`. The displaced tree is retained in a unique sibling
recovery directory. A failed replacement is rolled back before the operation
returns. LaunchBox reloads the active library only after the replacement
succeeds.

The bounds are port-owned safety policy because 13.27's protected limits were
not recovered:

| Boundary | Current limit |
|---|---:|
| Archive file | 2 GiB |
| Entries | 200,000 |
| One expanded member | 1 GiB |
| Total expanded regular-file bytes | 8 GiB |

## Mechanically verified scenario

The compiled Qt workflow creates an archive from the checked-in compatible
fixture through the visible LaunchBox control, extracts it independently, and
byte-compares the complete tree. It then changes a game title in the active
platform XML, restores through the visible confirmation path, and requires:

- exact restoration of the original `Data` tree;
- live model reload showing the original title;
- exactly one retained recovery tree containing the mutation;
- byte-identical `Images`, `Metadata`, and `Music` peers;
- the 13.27 archive-root shape with no `Data/` wrapper;
- no command shell in either product operation.

Pure tests additionally reject missing core files, malformed XML, symlinks,
unsafe/colliding archive members, oversized members, and an otherwise valid
`.7z` that lacks the required 13.27 layout.

## Remaining oracle and port work

- Recover exact manual dialog filters, default filename, overwrite behavior,
  and success/error sequencing from a supported Windows runtime.
- Recover or reconstruct automatic startup/shutdown timing and cross-frontend
  coordination.
- Implement the persisted `AutoBackup` option and automatic 25-file retention
  without deleting manual or unrecognized archives.
- Test crash/power-loss points on real Windows and macOS filesystems.
- Run the common Qt controls and direct 7-Zip boundary on native Windows and
  both Intel and Apple Silicon macOS hosts.
