# LaunchBox 13.27 application-data backup contract

## Scope

This note records the value-free evidence used for the first `LIB-018`
implementation slice. The original installation, its archives, and decompiled
first-party assemblies remain ignored. No game names, platform names, stored
paths, accounts, license values, or user XML values are copied into this
repository.

This milestone implements manual **Create Data Backup** and **Restore Data
Backup** in LaunchBox desktop mode plus the shared automatic startup/shutdown
contract in LaunchBox and Big Box.

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

Value-free UTF-16 strings in the real 13.27 frontend assemblies establish all
four names:

```text
Automatic LaunchBox Startup Data Backup
Automatic LaunchBox Shutdown Data Backup
Automatic Big Box Startup Data Backup
Automatic Big Box Shutdown Data Backup
```

The first-party `Unbroken.LaunchBox.Windows.DataBackup` type exposes a static
`AutoBackup(string name)` entry point, although its protected body is
unavailable. LaunchBox calls the startup form from its startup-work
coordinator. Shutdown diagnostics identify the shutdown form, while Big Box
explicitly logs that shutdown waits for its save process before running the
automatic data backup. The separate persisted `MaxAutoBackupsPerGame` value
belongs to game-save version management and is not evidence for application
data retention.

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

`AutoBackup` is represented by a typed domain policy. Exact `true` and `false`
values round-trip; a missing or malformed value defaults to the fresh-13.27
enabled behavior. The visible LaunchBox data-backup dialog changes only that
one field through the ordinary lossless recovery-backed transaction, retains
an exact pre-write backup, rereads the committed document, and publishes the
live policy only after the value verifies.

After a real library finishes loading, each frontend attempts exactly one
verified startup snapshot. Normal application close and frontend handoff stop
owned media, wait for active save/load/launch work, and hold the window open
until the verified shutdown snapshot succeeds or reports a failure. Disabled,
unloaded, conflicted, or pending-recovery libraries skip safely rather than
trapping application exit. The shared cross-process lock serializes the two
processes during a frontend handoff.

Automatic creation accepts only a real `Backups` directory, generates the
exact frontend/event name with a local `YYYY-MM-DD HH-mm-ss` timestamp, and
uses the same source-revision plus bounded re-extraction verification as a
manual backup. Retention recognizes only those four exact names with a
strictly parsed timestamp, sorts them deterministically, and retains the 25
newest across both frontends and both events. Manual files, future or malformed
names, directories, symlinks, and other unrecognized entries are never
retention candidates. A candidate is rechecked immediately before removal.

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

A second compiled workflow starts LaunchBox with 25 recognized seed archives,
requires a verified startup snapshot, toggles the rendered setting
true-false-true through two exact XML transactions, and requests normal
shutdown. It requires:

- exact LaunchBox startup and shutdown names and valid `.7z` payloads;
- the expected true-false-true backup chain;
- exactly the newest 25 recognized automatic archives;
- unchanged manual and unrecognized archives;
- no pending transaction manifest.

An independent Big Box process requires its exact startup and shutdown names,
the same shared retention result, byte-identical `Data`, and two valid
archives. Pure tests additionally freeze all four names, mixed-kind ordering,
manual/unknown/symlink exclusion, real archive creation, and pruning.

## Remaining oracle and port work

- Recover exact manual dialog filters, default filename, overwrite behavior,
  and success/error sequencing from a supported Windows runtime.
- Recover exact protected startup/shutdown scheduling and failure-message
  sequencing from a supported Windows runtime; the current scheduling is the
  evidence-backed portable contract described above.
- Exercise simultaneous LaunchBox/Big Box handoff with large real libraries
  and interruption at every automatic archive and retention boundary.
- Test crash/power-loss points on real Windows and macOS filesystems.
- Run the common Qt controls and direct 7-Zip boundary on native Windows and
  both Intel and Apple Silicon macOS hosts.
