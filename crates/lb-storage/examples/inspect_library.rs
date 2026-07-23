use lb_storage::LaunchBoxDataIndex;
use std::collections::BTreeMap;
use std::env;
use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    let Some(path) = env::args_os().nth(1) else {
        eprintln!("usage: cargo run -p lb-storage --example inspect_library -- <LaunchBox path>");
        return ExitCode::from(2);
    };

    let started = Instant::now();
    let data = match LaunchBoxDataIndex::load(&path) {
        Ok(library) => library,
        Err(error) => {
            eprintln!("library inspection failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let library = data.platforms();

    let mut counts = BTreeMap::<&str, usize>::new();
    let mut game_count = 0usize;
    for game in library.games() {
        game_count += 1;
        *counts.entry(&game.platform).or_default() += 1;
    }

    println!("platform documents: {}", library.platforms().len());
    println!("games: {game_count}");
    println!(
        "additional applications: {}",
        library.additional_applications().count()
    );
    println!("alternate names: {}", library.alternate_names().count());
    println!("custom fields: {}", library.custom_fields().count());
    println!(
        "controller support records: {}",
        library.controller_support().count()
    );
    println!("game saves: {}", library.game_saves().count());
    println!("distinct platform names: {}", counts.len());
    println!("playlist documents: {}", data.playlists().len());
    println!("playlist filters: {}", data.playlist_filters().count());
    println!("playlist games: {}", data.playlist_games().count());
    let emulator_configuration = data.emulator_configuration();
    println!(
        "emulators: {}",
        emulator_configuration
            .map(|configuration| configuration.emulators.len())
            .unwrap_or_default()
    );
    println!(
        "emulator platforms: {}",
        emulator_configuration
            .map(|configuration| configuration.platforms.len())
            .unwrap_or_default()
    );
    let catalog = data.platform_catalog();
    println!(
        "platform definitions: {}",
        catalog
            .map(|catalog| catalog.platforms.len())
            .unwrap_or_default()
    );
    println!(
        "platform categories: {}",
        catalog
            .map(|catalog| catalog.categories.len())
            .unwrap_or_default()
    );
    println!(
        "platform folders: {}",
        catalog
            .map(|catalog| catalog.folders.len())
            .unwrap_or_default()
    );
    println!("parent relationships: {}", data.parents().len());
    println!("game controllers: {}", data.game_controllers().len());
    println!("input bindings: {}", data.input_bindings().len());
    println!("ignored game IDs: {}", data.ignored_game_ids().len());
    println!("list cache items: {}", data.list_cache().len());
    println!(
        "LaunchBox settings: {}",
        data.settings()
            .map(|settings| settings.entries.len())
            .unwrap_or_default()
    );
    println!(
        "image type settings: {}",
        data.settings()
            .map(|settings| settings.image_type_settings.len())
            .unwrap_or_default()
    );
    println!(
        "BigBox settings: {}",
        data.big_box_settings()
            .map(|settings| settings.entries.len())
            .unwrap_or_default()
    );
    println!("elapsed: {:.3}s", started.elapsed().as_secs_f64());
    ExitCode::SUCCESS
}
