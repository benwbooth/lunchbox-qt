# LaunchBox 13.27 Wine oracle

This note records the locally verified runtime boundary for the proprietary
LaunchBox 13.27 installation. It is diagnostic evidence for the port, not a
runtime dependency of the native Rust/Qt application. The installation, Wine
prefix, license, user library, managed exception hook, logs, and screenshots
remain ignored and are not redistributed.

## Original failure

The Wine debugger report alone identified a managed exception:

- `0xe0434352` is the CLR's native exception code;
- its embedded `0x80004005` is `E_FAIL`;
- the stack entered CoreCLR and `System.Windows.Forms`.

A temporary .NET 9 startup hook captured the first-chance exception behind that
generic native report:

```text
System.Runtime.InteropServices.SEHException (0x80004005):
External component has thrown an exception.
   at MS.Internal.Automation.UiaCoreApi.RawUiaSetFocus(...)
   at System.Windows.Automation.AutomationElement.SetFocus()
```

Wine's built-in `uiautomationcore.dll` aborts at the unimplemented
`UiaSetFocus` export. This was the fatal LaunchBox startup fault; the installed
payload and its self-contained .NET 9.0.16 runtime were intact.

## Local compatibility replacement

The isolated prefix now uses a compatible older native UI Automation runtime
that was already present in another local Wine prefix. No DLL is checked in.
The original Wine 11.8 files remain beside it as reversible backups.

| Prefix location | Active SHA-256 | Original backup |
|---|---|---|
| `drive_c/windows/system32/uiautomationcore.dll` | `6529733ed8e69e2b5ff7272507825ca4b6662f731373a4a9b7bfe7e45562fc85` | `uiautomationcore.dll.wine-11.8` |
| `drive_c/windows/syswow64/uiautomationcore.dll` | `0001df8d8319524a93ed523382a6cce8de9234211d5f3dc46bb4c530d9385150` | `uiautomationcore.dll.wine-11.8` |

The 64-bit PE has timestamp `2007-10-09 14:56:51`, exports `UiaSetFocus`, and
imports only the expected Windows system libraries. The prefix selects it with
the user registry override
`HKCU\Software\Wine\DllOverrides\uiautomationcore=native,builtin`.

A Windows 11 `UIAutomationCore.dll` was not compatible with Wine 11.8: it
reached Wine's unimplemented `NtQueryWnfStateData`. Reusing a complete older
`.NET Framework 4.8` prefix was also invalid because its native system-DLL
forwarders did not match the current Wine runtime.

To undo the local replacement, restore both `.wine-11.8` backups to their
original names and remove the `uiautomationcore` DLL override. This does not
touch LaunchBox data.

## Verified LaunchBox result

`Core/LaunchBox.exe` now reaches and paints the complete desktop window in the
canonical prefix without the diagnostic startup hook. Menus, toolbar, search,
sidebar, content area, and window chrome were visible in a compositor
screenshot.

Startup still pauses for almost exactly 60 seconds in LaunchBox's activation
handler. A debug run logged:

```text
14:18:02.086  Got Handle, Proceeding to Set Focus
14:19:02.105  Got Element
```

The native runtime changes the fatal SEH exception into a caught
`InvalidOperationException` reporting that the target cannot receive focus.
LaunchBox then continues normally. This makes Wine useful for deliberate
desktop workflow observation, but the fixed delay makes it unsuitable for
fast automated scenarios.

The exception hook also observed caught Wine gaps for
`Windows.Storage.ApplicationData.LocalFolder`, .NET Framework WMI modules, OID
friendly-name lookup, toast-notification COM registration, and cancelled
network operations. None prevented the desktop window from rendering. They
must not be copied into the portable implementation as assumed application
behavior.

## Real library and premium gate

The read-only Windows partition contains the owner's older complete LaunchBox
installation. Its already documented value-free census has 37 platform
documents, 54 playlists, and 35,869 games. A local working copy of its `Data`
directory and the owner's valid license were placed only in the ignored oracle
directory; the source installation was not modified.

This made the BigBox premium gate observable:

1. without the license, `BigBox.exe` redirects to LaunchBox;
2. with the license but no platforms, it reports that games with platforms
   must be added and opens LaunchBox;
3. with the real data snapshot, it loads the 35,869-game library and creates
   the expected full-screen navigation stack.

No license contents, account data, titles, paths, media, or proprietary
application files are committed.

## BigBox post-navigation crash

The generic `0xe0434352` / `0x80004005` debugger report that appeared after
BigBox loaded was a separate Wine compatibility fault. The application log
showed its 60-second theme-demo transition enter a real filters view and then
emit `Attract Mode stopped` on nearly every frame. The managed exception hook
recovered the terminating cause behind the native report:

```text
System.TypeInitializationException:
The type initializer for 'Microsoft.Data.Sqlite.SqliteConnection' threw an exception.
 ---> System.Reflection.TargetInvocationException
 ---> System.NotImplementedException
   at ABI.Windows.Storage.IApplicationDataMethods.get_LocalFolder(...)
   at Microsoft.Data.Sqlite.SqliteConnection..cctor()
   ...
   at Unbroken.LaunchBox.Windows.Data.Platform.GetPlatformBannerImagePaths()
   at Unbroken.LaunchBox.Windows.BigBox.ViewModels.PlatformFiltersViewModelBase.LoadDetails(...)
```

Microsoft.Data.Sqlite 9.0.14 initializes SQLitePCL and, on Windows, probes
`Windows.Storage.ApplicationData` for SQLite data and temporary directories.
Wine exposes `ApplicationData.Current` but returns `E_NOTIMPL` from
`LocalFolder`. The library catches a failure while obtaining `Current`, not a
failure from the later folder getter, so its static initializer becomes
permanently faulted.

Upstream source:
<https://github.com/dotnet/efcore/blob/v9.0.14/src/Microsoft.Data.Sqlite.Core/SqliteConnection.cs#L61-L107>

The ignored oracle now has a reversible, prefix-local replacement:

| `Core` file | SHA-256 |
|---|---|
| active `Microsoft.Data.Sqlite.dll` | `1bc46f00c262e0af227b575cba2a43782af630338e145b2feab399e7337401be` |
| `Microsoft.Data.Sqlite.dll.wine-oracle-original` | `94111289bef8bcc875cb1ef54fb727f582b4c97936fec1f8615c114095be0a8b` |

The replacement starts from Microsoft's official IL-only 9.0.14 package and
retains the exact assembly name, version `9.0.14.0`, and public-key token
`adb9793829ddae60`. Its one local change preserves SQLitePCL initialization but
skips only the Windows storage-directory probe. No patched or original
assembly is checked in.

An isolated 105-second run then entered
`PlatformWheel1FiltersViewModel`, continued for another 31 seconds, and was
stopped by the diagnostic timeout. The managed and Wine traces contained no
`ApplicationData`, `SqliteConnection`, unhandled-exception, or CLR-crash
record. Restoring the `.wine-oracle-original` file removes this workaround.

## Remaining BigBox blocker

The stock Default theme creates a full-screen WPF window but paints black. At
about the end of the UI Automation delay it reports:

```text
System.Runtime.InteropServices.COMException (0x88980406)
   at System.Windows.Media.Composition.DUCE.Channel.SyncFlush()
```

Microsoft identifies `0x88980406` as `UCEERR_RENDERTHREADFAILURE`, a generic
fatal WPF render-thread failure. The UI-thread `SyncFlush` stack is therefore
not enough to identify a theme-code defect:

<https://learn.microsoft.com/en-us/troubleshoot/developer/dotnet/framework/general/wpf-render-thread-failures>

The following controlled changes did not make BigBox render:

- WPF software rendering through `DisableHWAcceleration=1`;
- WineD3D versus DXVK 2.7.1 on the AMD RX 7900 XTX;
- a Wine virtual desktop;
- the normal mixed-DPI Plasma/XWayland desktop versus an isolated
  1920x1080 Xvfb display;
- the Default theme versus the much simpler Old Default text-list views;
- disabling transitions, game-list music, and random platform videos;
- resolving the observed `LAUNCHBOX_THEME_FOLDER` font path through the host
  filesystem.

The simplified Old Default run loaded `TextFiltersViewModel` and remained
alive for 150 seconds without the `0x88980406` exception, but an isolated
screen capture still contained exactly one color: black. The font error also
remained because WPF treated the value as an assembly pack-resource URI, not a
filesystem path. It is a real caught compatibility fault, but it is not a
proven cause of the black frame.

BigBox is consequently usable under Wine for startup/premium-gate logging,
settings generation, managed probes, and library-load evidence. It is not a
visual oracle on this host. Exact BigBox rendering, focus, transitions, media,
input, and multi-display behavior still require a supported Windows runtime.

## Port boundary

This workaround changes only the ignored proprietary oracle prefix. It adds no
Windows paths, registry access, shell invocation, WPF, or Wine dependency to
the Rust/CXX-Qt product. The native port continues to resolve persisted path
syntax in `lb-platform`, pass host-native paths into Qt, and compile its
portable core for Linux, Windows, Intel macOS, and Apple Silicon macOS.
