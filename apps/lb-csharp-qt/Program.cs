using System;
using System.IO;
using Qt.Quick;

namespace LaunchBox.QtPort
{
    public static class Program
    {
        internal static void Main(string[] args)
        {
            string xmlPath = ReadXmlPath(args);
            if (!string.IsNullOrEmpty(xmlPath))
            {
                if (!File.Exists(xmlPath))
                {
                    Console.Error.WriteLine("platform XML does not exist: " + xmlPath);
                    return;
                }

                Environment.SetEnvironmentVariable("LAUNCHBOX_PLATFORM_XML", xmlPath);
            }

            if (args.Length > 0 && string.Equals(args[0], "--smoke", StringComparison.Ordinal))
            {
                Environment.SetEnvironmentVariable("LAUNCHBOX_QT_SMOKE", "1");
            }

            // Qt Bridge owns the native Qt host and exposes public C# types to
            // QML. There is no handwritten native source in this project.
            Qml.LoadFromRootModule("Main");
            Qml.WaitForExit();
        }

        private static string ReadXmlPath(string[] args)
        {
            if (args.Length >= 2 && string.Equals(args[0], "--smoke", StringComparison.Ordinal))
            {
                return args[1];
            }

            return args.Length == 1 && !args[0].StartsWith("--", StringComparison.Ordinal)
                ? args[0]
                : Environment.GetEnvironmentVariable("LAUNCHBOX_PLATFORM_XML") ?? string.Empty;
        }
    }
}
