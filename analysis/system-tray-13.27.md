# LaunchBox 13.27 system tray and notification contract

This note records the evidence used for the first native `DESK-008` subset. It
does not treat protected method bodies that decompile as stubs as behavioral
proof.

## Static contract

The installed 13.27 assemblies expose these singleton `Settings.xml` fields on
`Unbroken.LaunchBox.Windows.Data.Settings`:

| Field | Type | Native meaning |
|---|---|---|
| `EnableSystemTray` | Boolean | Create/show the desktop tray item |
| `MinimizeToSystemTray` | Boolean | Hide instead of remaining minimized |
| `CloseToSystemTray` | Boolean | Hide instead of exiting on ordinary close |
| `DontSendTrayReminder` | Boolean | Negative storage form of the visible reminder option |
| `NotificationType` | Integer | Notification presentation enum |

The 13.27 `OptionsSystemTrayPageViewModel` surface has
`EnableSystemTray`, `CloseToSystemTray`, `MinimizeToSystemTray`, and
`ShowSendToTrayReminder`. The last property confirms that the user-facing
positive option maps to the negative persisted field. The separate
`OptionsNotificationsPageViewModel` exposes a selected integer and a collection
of choices.

`NotificationCenter.NotificationTypes` declares this exact zero-based order:

1. `LaunchBoxNotifications`
2. `WindowsNotifications`
3. `MessageBoxes`

The notification center also exposes a notification collection, raised/read/
unread/dismissed events, passive/error/info/input entry points, and an explicit
sent-to-tray notification. The recovered notification model carries raised/read
dates, an error flag, message, icon, lifespan, buttons, and read/dismiss
operations. The first port subset implements the message, raised date, error,
read/unread, and dismiss contract; action buttons, icons, lifespans, and the
broader producer set remain open.

The original desktop uses the WPF `Hardcodet.Wpf.TaskbarNotification`
implementation. That implementation is evidence for product behavior, not a
portable dependency: the port uses Qt's `SystemTrayIcon`, `Menu`, and
host-notification APIs on Windows, Linux, and macOS.

## Fresh-instance defaults

A temporary self-contained managed reflection probe loaded the real 13.27
`Unbroken.LaunchBox.Windows.dll` under the repository's Wine prefix and
constructed the settings type without opening the LaunchBox UI. It reported:

```text
EnableSystemTray=False
AlwaysShowSystemTrayIcon=<absent>
MinimizeToSystemTray=False
CloseToSystemTray=False
DontSendTrayReminder=False
NotificationType=0
```

Therefore the port's fresh policy is tray disabled, no close/minimize
interception, reminder enabled, and LaunchBox notifications selected. The
older 13.24 library is not used to override these defaults; its tray settings
remain useful only as historical persistence evidence.

## Native implementation boundary

The port:

- parses invalid or missing values to the measured fresh defaults;
- keeps saved close/minimize choices while the master tray switch is disabled,
  but does not intercept a window operation in that state;
- intercepts close/minimize only when Qt reports a real system tray as
  available, preventing an inaccessible hidden window on unsupported desktop
  sessions;
- writes only the five singleton fields through the lossless library
  transaction, retains an exact pre-change backup, rereads the committed typed
  policy, and refuses a non-matching round trip;
- uses the original enum integers in `Settings.xml`, while presenting
  `WindowsNotifications` as “System Notifications” on non-Windows hosts;
- bounds the in-memory notification center to 128 entries and each message to
  4096 UTF-8 bytes.

The entry cap and message cap are explicit port-owned safety policies because
no protected limit was recovered.

## Verification

Pure Rust coverage freezes the defaults, negative reminder inversion, enum
keys/labels/integers, malformed fallback, strict versioned payload, unknown
field rejection, all-five-field persistence, exact backup, unknown XML
retention, and committed typed reread.

The compiled LaunchBox scenario opens and submits the real editor, creates info
and error notifications, marks one read, dismisses one, renders the native
notification center, validates all five XML values and one exact backup, proves
the platform document is byte-identical, and starts a second process to prove
policy restoration without another write.

The offscreen Linux runner intentionally reports no system tray, so it proves
the safe unavailable-host path rather than pretending to exercise a desktop
panel. Actual icon/menu activation, close/minimize interception, and host
notifications remain Windows, Linux-desktop, and macOS real-host gates.

## Wine boundary

The original `0xe0434352`/`0x80004005` report is now resolved. A temporary
first-chance exception hook traced it to Wine's unimplemented
`uiautomationcore!UiaSetFocus`, called through
`AutomationElement.SetFocus()`. The ignored oracle prefix now uses a
reversible compatible native UI Automation runtime and the complete LaunchBox
desktop paints after a repeatable approximately 60-second focus timeout.

This does not prove the real tray surface on Linux, Windows, or macOS, and
BigBox still has a separate WPF render blocker. The hashes, registry override,
observed exceptions, and negative rendering experiments are recorded in
`analysis/wine-oracle-13.27.md`.
