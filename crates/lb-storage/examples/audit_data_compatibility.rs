use lb_domain::GAME_XML_FIELDS;
use lb_storage::{AuxiliaryDocument, AuxiliaryDocumentKind, LibraryIndex};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use xmltree::Element;

fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fixtures/launchbox"));
    let data = if root.join("Data").is_dir() {
        root.join("Data")
    } else {
        root.clone()
    };

    let index = LibraryIndex::load(&root)?;
    let indexed_games = index.games().count();
    let platform_files = xml_files(&data.join("Platforms"))?;
    let mut observed_game_fields = BTreeSet::new();
    let mut censused_games = 0usize;
    for path in &platform_files {
        census_game_fields(path, &mut observed_game_fields, &mut censused_games)?;
    }

    if censused_games != indexed_games {
        return Err(format!(
            "typed index returned {indexed_games} games but schema census found {censused_games}"
        )
        .into());
    }
    let modeled_game_fields = GAME_XML_FIELDS
        .iter()
        .map(|field| (*field).to_string())
        .collect::<BTreeSet<_>>();
    let unknown = observed_game_fields
        .difference(&modeled_game_fields)
        .cloned()
        .collect::<Vec<_>>();
    let unobserved = modeled_game_fields
        .difference(&observed_game_fields)
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() || !unobserved.is_empty() {
        return Err(format!(
            "Game field mismatch: unknown={unknown:?}, modeled-but-unobserved={unobserved:?}"
        )
        .into());
    }

    let mut auxiliary_documents = Vec::new();
    let fixed_documents = [
        (AuxiliaryDocumentKind::Emulators, "Emulators.xml"),
        (AuxiliaryDocumentKind::Platforms, "Platforms.xml"),
        (AuxiliaryDocumentKind::Parents, "Parents.xml"),
        (
            AuxiliaryDocumentKind::GameControllers,
            "GameControllers.xml",
        ),
        (AuxiliaryDocumentKind::InputBindings, "InputBindings.xml"),
        (
            AuxiliaryDocumentKind::ImportBlacklist,
            "ImportBlacklist.xml",
        ),
        (AuxiliaryDocumentKind::ListCache, "ListCache.xml"),
        (AuxiliaryDocumentKind::Settings, "Settings.xml"),
        (AuxiliaryDocumentKind::BigBoxSettings, "BigBoxSettings.xml"),
    ];
    for (kind, file_name) in fixed_documents {
        let path = data.join(file_name);
        if path.is_file() {
            auxiliary_documents.push((kind, path));
        }
    }
    auxiliary_documents.extend(
        xml_files(&data.join("Playlists"))?
            .into_iter()
            .map(|path| (AuxiliaryDocumentKind::Playlist, path)),
    );

    for (kind, path) in &auxiliary_documents {
        let original = Element::parse(fs::File::open(path)?)?;
        let document = AuxiliaryDocument::load_as(*kind, path)?;
        let serialized = document.to_xml_bytes()?;
        let reparsed = Element::parse(serialized.as_slice())?;
        if original != reparsed {
            return Err("an auxiliary document changed during lossless round-trip".into());
        }
        AuxiliaryDocument::from_reader(*kind, path, serialized.as_slice())?;
    }

    println!("platform documents audited: {}", platform_files.len());
    println!("game records strictly parsed: {indexed_games}");
    println!(
        "alternate-name records strictly parsed: {}",
        index.alternate_names().count()
    );
    println!(
        "custom-field records strictly parsed: {}",
        index.custom_fields().count()
    );
    println!(
        "observed and modeled Game fields: {}",
        GAME_XML_FIELDS.len()
    );
    println!(
        "auxiliary documents typed/lossless round-tripped: {}",
        auxiliary_documents.len()
    );
    println!("unknown Game fields: 0");
    Ok(())
}

fn xml_files(directory: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("xml"))
        })
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn census_game_fields(
    path: &Path,
    fields: &mut BTreeSet<String>,
    games: &mut usize,
) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_reader(BufReader::new(fs::File::open(path)?));
    let mut buffer = Vec::new();
    let mut depth = 0usize;
    let mut game_depth = None;
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                depth += 1;
                if depth == 2 && element.name().as_ref() == b"Game" {
                    *games += 1;
                    game_depth = Some(depth);
                } else if game_depth.is_some() && depth == 3 {
                    insert_field_name(element.name().as_ref(), fields)?;
                }
            }
            Event::Empty(element) if game_depth.is_some() && depth == 2 => {
                insert_field_name(element.name().as_ref(), fields)?;
            }
            Event::End(_) => {
                if game_depth == Some(depth) {
                    game_depth = None;
                }
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(())
}

fn insert_field_name(
    encoded_name: &[u8],
    fields: &mut BTreeSet<String>,
) -> Result<(), Box<dyn Error>> {
    let name = std::str::from_utf8(encoded_name)?;
    if !name.starts_with("TestOnly") {
        fields.insert(name.to_string());
    }
    Ok(())
}
