# C# / Qt port experiment

The `csharp-qt-port` branch is the managed-first experiment. The decompiled
LaunchBox assemblies are evidence for recovering domain contracts, but the
original projects are WPF/WindowsDesktop applications and cannot be reused as
the cross-platform UI.

The checked-in application uses Qt's official C# Bridge: C# owns the managed
model and QML owns the Qt Quick UI. There are no handwritten C++ files, CMake
files, P/Invoke declarations, Win32 handles, WPF controls, or shell-specific
paths in this application. The bridge generator creates its native adapter as
an implementation detail of the build.

Run the current proof of concept from the Nix development shell:

```sh
nix develop
bash scripts/check_csharp_qt.sh
```

The check builds and runs a real Qt Quick event loop in offscreen mode. It
loads `fixtures/launchbox/Data/Platforms/Fixture Console.xml`, exposes a
managed `Library` singleton to QML, renders the three games, edits a favorite,
and verifies that unknown XML and Windows-separated `ApplicationPath` values
survive serialization. It also compiles and runs the portable XML model with
Mono. Mono is a model-only gate for now; Qt's official bridge package targets
.NET 8 on Linux x64 and Windows x64.

## Boundary and next slices

`ManagedPlatformDocument` is a clean-room persistence boundary rather than a
copy of the obfuscated WPF `Game` type. `LibraryViewModel` is the first QML
surface. Future slices should recover one domain contract at a time, add a
managed service, and compare its behavior with the installed LaunchBox oracle
before adding UI.

The official bridge is currently beta and does not yet provide an official
macOS package. macOS and broader Mono UI support therefore remain explicit
follow-up gates rather than claims of completed platform support. The Rust
port is intentionally frozen while this managed/UI experiment is evaluated.
