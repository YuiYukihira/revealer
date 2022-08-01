//! This is a Work In Progress and represents the desired final state for the game. Most
//! features mentioned are not actually implemented yet.
//!
//! Revealer is a "game" bon out of a need and desire for an easy way to manage intriuge and
//! investigations in my D&D campaigns. It displays one of several maps at a time and
//! allows you to mark information and certain locations and people.
//! You can select a specific character and get a display of related locations and/or movements
//! related to the them.
#![deny(missing_docs)]

use bevy::{prelude::*, utils::HashSet};
use clues::ClueId;

pub mod clues;
pub mod locations;

/// One the different modes the game runs in
pub enum Mode {
    /// In server mode, there is no display, it acts only to interface with the client and server.
    Server,
    /// In DM mode, all [`clues_asset::Clue`]s are available and the `known` setting can be changed.
    DM,
    /// In player mode only the clues `known` can be seen.
    Player,
}

/// Stores the currently known clues
#[derive(Debug, Component)]
pub struct CluesComponent {
    clues: HashSet<ClueId>,
}

fn main() {
    App::new().add_plugin(clues::CluesAssetPlugin).run();
}
