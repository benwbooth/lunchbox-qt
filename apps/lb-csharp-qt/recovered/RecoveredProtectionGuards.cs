using System;

// ILSpy emitted these anti-tamper callback sites as delegate types, although
// the recovered call sites require static members. The original callback
// bodies are not present in stored IL. Throwing keeps the source compilable
// without silently claiming that the protected initialization succeeded.
internal static class TransformerBuilderVerifier
{
    internal static readonly object ConvertProfilerDecryptor = null;

    internal static void InterruptConfigurationProject(object callback) =>
        throw new MissingMethodException("LaunchBox protected callback body was not recovered: TransformerBuilderVerifier");
}

internal static class RecoveredProtectionGuards
{
    internal static MissingMethodException Missing(string member) =>
        new MissingMethodException($"LaunchBox protected method body was not recovered: {member}");
}

internal static class RecoveredStaticData
{
    // These five blobs are the exact RVA-backed PrivateImplementationDetails
    // fields referenced by DetailedFormatter.GuideVisitorSystem in the oracle.
    internal static readonly byte[] Bytes28 = { 0xd2, 0xf8, 0x04, 0xb0, 0xdf, 0xf8, 0x14, 0xc0, 0xdf, 0xf8, 0x14, 0xd0, 0xdf, 0xf8, 0x14, 0xe0, 0xe3, 0x45, 0x01, 0xd1, 0xdd, 0xf8, 0x00, 0xf0, 0xde, 0xf8, 0x00, 0xf0 };
    internal static readonly byte[] Bytes32A = { 0xe5, 0x92, 0xb0, 0x04, 0xe5, 0x9f, 0xc0, 0x14, 0xe5, 0x9f, 0xd0, 0x14, 0xe5, 0x9f, 0xe0, 0x14, 0xe1, 0x5b, 0x00, 0xc1, 0xa0, 0x00, 0x00, 0x0e, 0x12, 0xff, 0xf1, 0xde, 0x12, 0xff, 0xf1 };
    internal static readonly byte[] Bytes30 = { 0x55, 0x8b, 0xec, 0x8b, 0x45, 0x10, 0x81, 0x78, 0x04, 0x00, 0x00, 0x00, 0x00, 0x74, 0x07, 0xb8, 0x00, 0x00, 0x00, 0x00, 0xeb, 0x05, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x5d, 0xff, 0xe0 };
    internal static readonly byte[] Bytes40 = { 0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x49, 0x39, 0x40, 0x08, 0x74, 0x0c, 0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xe0, 0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xe0 };
    internal static readonly byte[] Bytes32B = { 0x4b, 0x04, 0x40, 0xf9, 0xec, 0x00, 0x00, 0x58, 0x0d, 0x01, 0x00, 0x58, 0x2e, 0x01, 0x00, 0x58, 0x7f, 0x01, 0x00, 0xeb, 0x41, 0x00, 0x00, 0x54, 0xa0, 0x01, 0x1f, 0xd6, 0xc0, 0x01, 0x1f, 0xd6 };

    internal static void Initialize(byte[] target, byte[] source)
    {
        if (target.Length != source.Length)
            throw new InvalidOperationException("Recovered static data length mismatch");
        Buffer.BlockCopy(source, 0, target, 0, source.Length);
    }
}

internal static class TransformerSynchronizerVisitor
{
    internal static readonly object ModifySequentialFieldBuilder = null;

    internal static void InterruptConfigurationProject(object callback) =>
        throw new MissingMethodException("LaunchBox protected callback body was not recovered: TransformerSynchronizerVisitor");
}

internal static class EnumeratorPredictor
{
    internal static readonly object JoinParserConfig = null;

    internal static void InterruptConfigurationProject(int state, object[] token, object direction, object callback) =>
        throw new MissingMethodException("LaunchBox protected callback body was not recovered: EnumeratorPredictor");
}
