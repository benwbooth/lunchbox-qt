//! Native integrations that sit above LaunchBox's persisted data model.
//!
//! Adapters receive native host paths only after `lb-platform` has resolved
//! LaunchBox's lexical paths. They return discovered filesystem facts without
//! writing XML or mutating emulator data; the application/storage layers own
//! those transactions.

pub mod dolphin;
pub mod retroarch;

use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmulatorSaveKind {
    Game,
    State { slot: i32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredEmulatorSave {
    pub game_id: String,
    pub additional_application_id: Option<String>,
    pub emulator_file_name: String,
    pub emulator_core: String,
    pub kind: EmulatorSaveKind,
    pub primary_path: PathBuf,
    pub companion_paths: Vec<PathBuf>,
    pub save_group_id: Option<String>,
    pub save_group_name: String,
    pub display_chip_text: Option<String>,
}

impl DiscoveredEmulatorSave {
    pub fn slot(&self) -> Option<i32> {
        match self.kind {
            EmulatorSaveKind::Game => None,
            EmulatorSaveKind::State { slot } => Some(slot),
        }
    }

    pub fn all_paths(&self) -> impl Iterator<Item = &PathBuf> {
        std::iter::once(&self.primary_path).chain(&self.companion_paths)
    }
}
