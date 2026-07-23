# Synthetic PCSX2 disc fixtures

These files contain no game data. Their 49,152-byte logical source is the tiny
ISO9660 image emitted by `lb-process-fixture --fixture-mode pcsx2-disc-image
--format iso --serial SLUS_203.12`.

They were generated with MAME `chdman` 0.287:

```text
chdman createdvd --input source.iso --output synthetic-lzma.chd \
  --compression lzma --hunksize 2048
chdman createcd --input source.cue --output synthetic-cd.chd
```

`synthetic-lzma.chd` exercises a CHD v5 DVD LZMA hunk stream. Its SHA-256 is
`1493bf53a0d6b4f18251206e8c8deda28b2941569c7a684d226d92511efdd758`.

`synthetic-cd.chd` was created with the default CD codec candidates reported as
CD LZMA/Deflate/FLAC and exercises a 2,448-byte CD hunk stream. Its SHA-256 is
`16982502af576745a58852b6fd6e9b591a8b5f6acde9951fe0f8c0ef2a8eacc1`.

Both passed `chdman verify` before being checked in. Integration tests decode
them through the native Rust reader and require the ISO9660 `SYSTEM.CNF` serial
`SLUS-20312`; no filename or title contains that serial.
