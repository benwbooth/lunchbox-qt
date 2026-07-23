CREATE TABLE Platforms (
    Name TEXT NOT NULL
);

CREATE TABLE PlatformAlternateNames (
    Name TEXT NOT NULL,
    Alternate TEXT NOT NULL
);

CREATE TABLE Games (
    DatabaseID INTEGER PRIMARY KEY,
    Name TEXT NOT NULL,
    CompareName TEXT NOT NULL,
    ReleaseDate TEXT,
    ReleaseYear INTEGER,
    Overview TEXT,
    MaxPlayers INTEGER,
    ReleaseType TEXT,
    Cooperative INTEGER NOT NULL,
    VideoURL TEXT,
    CommunityRating REAL,
    WikipediaURL TEXT,
    Platform TEXT NOT NULL,
    ESRB TEXT,
    Genres TEXT NOT NULL,
    Developer TEXT,
    Publisher TEXT
);

CREATE TABLE GameAlternateTitles (
    AlternateName TEXT NOT NULL,
    DatabaseID INTEGER NOT NULL,
    Region TEXT NOT NULL,
    AltNameCompareValue TEXT NOT NULL
);

INSERT INTO Platforms VALUES ('Fixture Console');
INSERT INTO PlatformAlternateNames VALUES ('Fixture Console', 'Fixture Alias');
INSERT INTO Games VALUES (
    4242,
    'Fixture Saga (USA)',
    'FIXTURE SAGA',
    '2002-03-04 00:00:00',
    2002,
    'Recovered local metadata overview.',
    2,
    'Released',
    1,
    'https://video.example/fixture-saga',
    4.75,
    'https://example.org/wiki/Fixture_Saga',
    'Fixture Console',
    'E10+',
    'Role-Playing; Strategy',
    'Fixture Forge',
    'Fixture Press'
);
