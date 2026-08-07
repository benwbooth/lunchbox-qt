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

The command is expected to fail today. The current static extraction has no
usable implementations for many protected methods. In the 2026-08-07 census,
the project reached the C# compiler but reported 766 errors:

- 616 calls to anti-tamper callback members that are absent from the emitted
  delegate types;
- 92 `out` parameters left unset by methods whose stored IL decompiles to an
  empty body;
- 52 invalid reconstructions in `DetailedFormatter` (including missing locals,
  invalid `RuntimeFieldHandle` expressions, and fields called as methods);
- additional generic-null, fixed-pointer, and overload errors.

Assigning defaults or adding no-op callback methods would make the project
appear to compile while deleting behavior. That is deliberately not done.
The next implementation step is to recover post-protection IL at the CLR JIT
boundary, place those bodies in a provenance-labelled source tree, and only
then adapt the recovered managed services to a Qt bridge. The existing
`LaunchBox.QtPort` project remains a separate Qt/QML proof of concept; it is
not a claim that this recovered source already runs.
