using System;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.IO;
using Qt.MetaObject;
using Qt.Quick;

namespace LaunchBox.QtPort
{
    // The singleton is the first managed application surface consumed by QML.
    // It is deliberately small: later slices can replace the backing document
    // without changing the QML-facing contract.
    [QObject]
    [QmlElement(Name = "Library", Singleton = true)]
    public sealed class LibraryViewModel : INotifyPropertyChanged
    {
        private ManagedPlatformDocument document;
        private string status = string.Empty;
        private int favoriteEditCount;

        public LibraryViewModel()
        {
            SourcePath = Environment.GetEnvironmentVariable("LAUNCHBOX_PLATFORM_XML") ?? string.Empty;
            SmokeMode = string.Equals(
                Environment.GetEnvironmentVariable("LAUNCHBOX_QT_SMOKE"),
                "1",
                StringComparison.Ordinal);
            Games = new ObservableCollection<ManagedGame>();

            if (!File.Exists(SourcePath))
            {
                status = "Set LAUNCHBOX_PLATFORM_XML to a LaunchBox platform XML file.";
                return;
            }

            try
            {
                document = ManagedPlatformDocument.Load(SourcePath);
                foreach (ManagedGame game in document.Games)
                {
                    Games.Add(game);
                }

                status = "Loaded " + Games.Count + " games";
                if (SmokeMode)
                {
                    RunSmokeChecks();
                }
            }
            catch (Exception exception)
            {
                status = "Load failed: " + exception.Message;
                Console.Error.WriteLine(status);
            }
        }

        public event PropertyChangedEventHandler PropertyChanged;

        public ObservableCollection<ManagedGame> Games { get; private set; }

        public int GameCount
        {
            get { return Games.Count; }
        }

        public int FavoriteEditCount
        {
            get { return favoriteEditCount; }
            private set
            {
                if (favoriteEditCount == value)
                {
                    return;
                }

                favoriteEditCount = value;
                OnPropertyChanged(nameof(FavoriteEditCount));
            }
        }

        public string SourcePath { get; private set; }
        public bool SmokeMode { get; private set; }

        public string Status
        {
            get { return status; }
            private set
            {
                if (string.Equals(status, value, StringComparison.Ordinal))
                {
                    return;
                }

                status = value;
                OnPropertyChanged(nameof(Status));
            }
        }

        public int SetFavorite(string id, bool value)
        {
            if (document == null)
            {
                return 0;
            }

            int changed = document.ApplyFavorite(new[] { id }, value);
            FavoriteEditCount += changed;
            Status = "Updated " + changed + " favorite(s)";
            return changed;
        }

        private void RunSmokeChecks()
        {
            int changed = document.ApplyFavorite(
                new[] { "fixture-adventure", "fixture-racer" }, false);
            string xml = System.Text.Encoding.UTF8.GetString(document.ToXmlBytes());
            bool valid = Games.Count == 3
                && changed == 1
                && !Games[0].Favorite
                && xml.Contains("<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>")
                && xml.Contains("Games\\Fixture Adventure\\adventure.rom");
            if (!valid)
            {
                throw new InvalidOperationException("managed XML compatibility smoke failed");
            }

            FavoriteEditCount = changed;
            Console.WriteLine(
                "CSHARP_QT_MANAGED_SMOKE_COMPLETE games=" + Games.Count
                + " changed=" + changed);
        }

        private void OnPropertyChanged(string name)
        {
            PropertyChangedEventHandler handler = PropertyChanged;
            if (handler != null)
            {
                handler(this, new PropertyChangedEventArgs(name));
            }
        }
    }
}
