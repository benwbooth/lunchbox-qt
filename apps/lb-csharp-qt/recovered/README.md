# Recovered C# compile boundary

This directory is a compatibility project over the actual ILSpy output. It
does not contain a clean-room LaunchBox model and it does not replace missing
method bodies with defaults. The source glob in
`Unbroken.LaunchBox.Recovered.csproj` points at the local, ignored
`decompiled/Unbroken.LaunchBox` tree produced by `scripts/decompile.sh`.

The original project cannot be used unchanged on Linux: it is a Windows
desktop project and its references target .NET 9. The compatibility project
only changes the target/runtime plumbing and links the recovered C# to the
installed oracle assemblies. `compatibility-shape.patch` records the limited
compiler-shape repairs made to the decompiler output (for example, invalid
static-class return types and generated-regex declarations). Those repairs do
not recover behavior.

Run the census after the local oracle and decompiled tree are present:

```text
scripts/check_recovered_csharp.sh
```

The project now reaches a clean C# build under the .NET 9 Linux SDK. The
2026-08-07 build reports 0 errors and 929 warnings after the compiler-shape
repairs described below.

The repairs are intentionally bounded:

- delegate fields in `DetailedFormatter` use the delegate types shown by the
  IL rather than `object`;
- the five `PrivateImplementationDetails` blobs used by
  `GuideVisitorSystem` are copied from their oracle PE RVAs and recorded in
  `RecoveredProtectionGuards.cs`;
- generated-regex, static-class, accessibility, and fixed-buffer artifacts are
  normalized by `compatibility-shape.patch`;
- methods whose stored IL is empty still throw a named
  `MissingMethodException`, and anti-tamper callback sites use the same
  explicit guard. They do not return fabricated values.

This is a compile boundary, not a claim of runtime feature parity. Any path
that reaches a protected method fails explicitly until post-protection IL is
recovered at the CLR JIT boundary. The existing `LaunchBox.QtPort` project
remains a separate Qt/QML proof of concept.
