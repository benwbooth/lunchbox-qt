using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Xml;
using System.Xml.Linq;
using System.ComponentModel;

namespace LaunchBox.QtPort
{
    // A port-owned persisted game contract. This is intentionally independent
    // of the decompiled WPF Game class and keeps the XML boundary testable on
    // .NET and Mono.
    public sealed class ManagedGame : INotifyPropertyChanged
    {
        private bool favorite;

        public ManagedGame()
            : this(string.Empty, string.Empty, string.Empty, string.Empty, false)
        {
        }

        internal ManagedGame(string id, string title, string platform,
            string applicationPath, bool favorite)
        {
            Id = id;
            Title = title;
            Platform = platform;
            ApplicationPath = applicationPath;
            this.favorite = favorite;
        }

        public event PropertyChangedEventHandler PropertyChanged;

        public string Id { get; private set; }
        public string Title { get; private set; }
        public string Platform { get; private set; }
        public string ApplicationPath { get; private set; }

        public bool Favorite
        {
            get { return favorite; }
            set
            {
                if (favorite == value)
                {
                    return;
                }

                favorite = value;
                PropertyChangedEventHandler handler = PropertyChanged;
                if (handler != null)
                {
                    handler(this, new PropertyChangedEventArgs(nameof(Favorite)));
                }
            }
        }
    }

    internal sealed class ManagedPlatformDocument
    {
        private readonly XDocument document;

        private ManagedPlatformDocument(XDocument document)
        {
            this.document = document;
            Games = document.Root == null
                ? new List<ManagedGame>()
                : document.Root.Elements("Game").Select(ReadGame).ToList();
        }

        internal IList<ManagedGame> Games { get; private set; }

        internal static ManagedPlatformDocument Load(string path)
        {
            if (string.IsNullOrEmpty(path))
            {
                throw new ArgumentException("A platform XML path is required.", nameof(path));
            }

            var settings = new XmlReaderSettings
            {
                DtdProcessing = DtdProcessing.Prohibit,
                XmlResolver = null
            };
            XDocument loaded;
            using (XmlReader reader = XmlReader.Create(path, settings))
            {
                loaded = XDocument.Load(reader, LoadOptions.PreserveWhitespace);
            }

            return new ManagedPlatformDocument(loaded);
        }

        internal int ApplyFavorite(IEnumerable<string> ids, bool value)
        {
            var selected = new HashSet<string>(ids ?? Enumerable.Empty<string>(), StringComparer.Ordinal);
            int changed = 0;
            foreach (XElement gameElement in document.Root == null
                ? Enumerable.Empty<XElement>()
                : document.Root.Elements("Game"))
            {
                XElement idElement = gameElement.Element("ID");
                if (idElement == null || !selected.Contains(idElement.Value))
                {
                    continue;
                }

                XElement favoriteElement = gameElement.Element("Favorite");
                if (favoriteElement == null)
                {
                    favoriteElement = new XElement("Favorite");
                    gameElement.Add(favoriteElement);
                }

                string serialized = value ? "true" : "false";
                if (!string.Equals(favoriteElement.Value, serialized, StringComparison.Ordinal))
                {
                    favoriteElement.Value = serialized;
                    changed++;
                }
            }

            foreach (ManagedGame game in Games.Where(game => selected.Contains(game.Id)))
            {
                game.Favorite = value;
            }
            return changed;
        }

        internal byte[] ToXmlBytes()
        {
            using (var stream = new MemoryStream())
            {
                document.Save(stream, SaveOptions.DisableFormatting);
                return stream.ToArray();
            }
        }

        private static ManagedGame ReadGame(XElement element)
        {
            return new ManagedGame(
                Value(element, "ID"),
                Value(element, "Title"),
                Value(element, "Platform"),
                Value(element, "ApplicationPath"),
                string.Equals(Value(element, "Favorite"), "true", StringComparison.OrdinalIgnoreCase));
        }

        private static string Value(XElement element, string name)
        {
            XElement child = element.Element(name);
            return child == null ? string.Empty : child.Value;
        }
    }
}
