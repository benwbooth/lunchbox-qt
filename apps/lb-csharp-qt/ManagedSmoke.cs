using System;
using System.IO;
using System.Linq;
using System.Text;

namespace LaunchBox.QtPort
{
    // Mono exercises the portable persistence layer only. Qt's official C#
    // Bridge currently targets .NET 8 on Linux/Windows, so this keeps the
    // managed contract independently verifiable on the existing Mono runtime.
    internal static class ManagedSmoke
    {
        private static int Main(string[] args)
        {
            if (args.Length != 1 || !File.Exists(args[0]))
            {
                Console.Error.WriteLine("usage: ManagedSmoke <platform-xml>");
                return 2;
            }

            ManagedPlatformDocument document = ManagedPlatformDocument.Load(args[0]);
            int changed = document.ApplyFavorite(
                new[] { "fixture-adventure", "fixture-racer" }, false);
            string xml = Encoding.UTF8.GetString(document.ToXmlBytes());
            bool valid = document.Games.Count == 3
                && changed == 1
                && !document.Games.First(game => game.Id == "fixture-adventure").Favorite
                && xml.Contains("<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>")
                && xml.Contains("Games\\Fixture Adventure\\adventure.rom");
            if (!valid)
            {
                Console.Error.WriteLine("managed XML compatibility smoke failed");
                return 3;
            }

            Console.WriteLine(
                "CSHARP_QT_MANAGED_MONO_SMOKE_COMPLETE games=" + document.Games.Count
                + " changed=" + changed);
            return 0;
        }
    }
}
