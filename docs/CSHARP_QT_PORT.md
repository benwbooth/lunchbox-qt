# C# / Qt port experiment

The `csharp-qt-port` branch now keeps two deliberately separate pieces:

- `apps/lb-csharp-qt/recovered` links the actual ILSpy-generated C# tree and
  records the Linux compiler census. It is the source-recovery starting point,
  not a replacement implementation.
- `apps/lb-csharp-qt/LaunchBox.QtPort.csproj` is a small Qt/QML bridge
  prototype. It is useful for validating the bridge toolchain, but it does not
  claim LaunchBox feature parity.

The original projects are WPF/WindowsDesktop applications and cannot be used
as the cross-platform UI unchanged. More importantly, the stored IL is
protected: many methods decompile to empty bodies or invalid control-flow
fragments. See [`recovered/README.md`](../apps/lb-csharp-qt/recovered/README.md)
for the reproducible compile boundary and the exact error census.

The prototype uses Qt's official C# Bridge: C# owns a tiny managed model and
QML owns the Qt Quick UI. There are no handwritten C++ files, CMake files,
P/Invoke declarations, Win32 handles, WPF controls, or shell-specific paths in
that prototype. The bridge generator creates its native adapter as an
implementation detail of the build. This does not make the protected
LaunchBox implementation portable.

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

The next real slice is post-protection recovery, not another clean-room DTO.
Recover one semantic service from runtime/JIT evidence, preserve its source
provenance, compile it on Linux, and compare it with the installed LaunchBox
oracle before exposing it to QML.

The official bridge is currently beta and does not yet provide an official
macOS package. macOS and broader Mono UI support therefore remain explicit
follow-up gates rather than claims of completed platform support. The Rust
port is intentionally frozen while this managed/UI experiment is evaluated.
