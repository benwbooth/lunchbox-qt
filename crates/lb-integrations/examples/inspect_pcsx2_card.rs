use lb_integrations::pcsx2::{extract_pcsx2_memory_card_save, list_pcsx2_memory_card_saves};
use std::env;
use std::path::PathBuf;

fn main() {
    let mut arguments = env::args_os();
    let program = arguments
        .next()
        .map(PathBuf::from)
        .and_then(|path| path.file_name().map(|name| name.to_owned()))
        .unwrap_or_else(|| "inspect_pcsx2_card".into());
    let Some(card) = arguments.next().map(PathBuf::from) else {
        eprintln!("usage: {} CARD.ps2", PathBuf::from(program).display());
        std::process::exit(2);
    };
    let member = arguments.next();
    let destination = arguments.next().map(PathBuf::from);
    if arguments.next().is_some() || member.is_some() != destination.is_some() {
        eprintln!(
            "usage: {} CARD.ps2 [MEMBER DESTINATION]",
            PathBuf::from(program).display()
        );
        std::process::exit(2);
    }
    if let (Some(member), Some(destination)) = (member, destination) {
        let Some(member) = member.to_str() else {
            eprintln!("member name is not valid Unicode");
            std::process::exit(2);
        };
        match extract_pcsx2_memory_card_save(&card, member, &destination) {
            Ok(extracted) => {
                println!("card={}", card.display());
                println!("member={}", extracted.save.directory_name);
                println!("file_count={}", extracted.files.len());
                println!("logical_bytes={}", extracted.save.total_bytes);
                println!("signature={}", extracted.signature);
                return;
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }
    match list_pcsx2_memory_card_saves(&card) {
        Ok(saves) => {
            println!("card={}", card.display());
            println!("save_count={}", saves.len());
            for save in saves {
                println!(
                    "member={}\ttitle={}\ticon_sys={}\tbytes={}",
                    save.directory_name.replace(['\n', '\r', '\t'], " "),
                    save.title.replace(['\n', '\r', '\t'], " "),
                    save.has_icon_sys,
                    save.total_bytes
                );
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
