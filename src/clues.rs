//! Clues and the Asset loader for them
use bevy::{prelude::Component, reflect::TypeUuid, utils::HashMap};
use serde::Deserialize;

pub use assets::CluesAssetPlugin;

use crate::locations::LocationId;

/// A wrapper around a string to represent a person
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
#[serde(transparent)]
pub struct PersonId(String);

/// A wrapper around a string to represent a clue
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
#[serde(transparent)]
pub struct ClueId(String);

/// A clue, is a specfic peice of information that is linked
/// to several locations and persons.
#[derive(Debug, Deserialize)]
pub struct Clue {
    /// The id of the clue
    pub id: ClueId,
    /// The locations relevant to the clue
    pub locations: Vec<LocationId>,
    /// The persons relevant to the clue
    pub persons: Vec<PersonId>,
    /// The actual information of the clue
    pub information: String,
}

#[derive(Debug, Deserialize)]
struct CluesFile {
    clues: Vec<Clue>,
}

/// A holder for many clues, allows you to lookup clues
/// by a common location or place
#[derive(Debug, Default, TypeUuid)]
#[uuid = "764bc9f6-8f08-4184-a3de-805bd2eeecab"]
pub struct Clues {
    clues: HashMap<ClueId, Clue>,
    by_person: HashMap<PersonId, Vec<ClueId>>,
    by_location: HashMap<LocationId, Vec<ClueId>>,
}

impl From<CluesFile> for Clues {
    fn from(clues_file: CluesFile) -> Self {
        let mut clues = Self::new();
        for clue in clues_file.clues {
            clues.insert(clue)
        }
        clues
    }
}

impl Clues {
    /// Construct a new empty instance
    pub fn new() -> Self {
        Default::default()
    }

    /// Insert a new clue
    pub fn insert(&mut self, clue: Clue) {
        for person in &clue.persons {
            match self.by_person.get_mut(person) {
                Some(peeps) => {
                    peeps.push(clue.id.clone());
                }
                None => {
                    self.by_person.insert(person.clone(), vec![clue.id.clone()]);
                }
            }
        }
        for location in &clue.locations {
            match self.by_location.get_mut(location) {
                Some(locs) => {
                    locs.push(clue.id.clone());
                }
                None => {
                    self.by_location
                        .insert(location.clone(), vec![clue.id.clone()]);
                }
            }
        }
        self.clues.insert(clue.id.clone(), clue);
    }

    /// Get a reference to a clue by it's [`ClueId`].
    pub fn get(&self, clue: &ClueId) -> Option<&Clue> {
        self.clues.get(clue)
    }

    /// Get a mutable reference to a clue by it's [`ClueId`].
    pub fn get_mut(&mut self, clue: &ClueId) -> Option<&mut Clue> {
        self.clues.get_mut(clue)
    }

    /// Get all clues by a [`LocationId`]
    pub fn get_by_location(&self, location: &LocationId) -> impl Iterator<Item = &Clue> {
        self.by_location
            .get(location)
            .into_iter()
            .flatten()
            .filter_map(|id| self.clues.get(id))
    }

    /// Get all clues by a [`PersonId`], also takes an option
    /// that if set to [`Some`] decides whether to only get known
    /// or unknown clues
    pub fn get_by_person(&self, person: &PersonId) -> impl Iterator<Item = &Clue> {
        self.by_person
            .get(person)
            .into_iter()
            .flatten()
            .filter_map(|id| self.clues.get(id))
    }

    /// Get all clues by a [`PersonId`] and a [`LocationId`], also
    /// takes an option that if set to [`Some`] decides whether to
    /// only get known or unknown clues
    pub fn get_by_person_and_location(
        &self,
        person: &PersonId,
        location: &LocationId,
    ) -> impl Iterator<Item = &Clue> {
        let people = self.by_person.get(person).into_iter().flatten();
        let locations: Vec<_> = self
            .by_location
            .get(location)
            .into_iter()
            .flatten()
            .collect();
        people
            .filter(move |clue| locations.contains(clue))
            .filter_map(|c| self.clues.get(c))
    }
}

mod assets {
    use bevy::{
        asset::{AssetLoader, LoadedAsset},
        prelude::{AddAsset, Plugin},
    };

    use super::{Clues, CluesFile};

    /// Bevy plugin to load a clues file
    pub struct CluesAssetPlugin;
    impl Plugin for CluesAssetPlugin {
        fn build(&self, app: &mut bevy::prelude::App) {
            app.add_asset::<Clues>().add_asset_loader(CluesAssetLoader);
        }
    }

    struct CluesAssetLoader;
    impl AssetLoader for CluesAssetLoader {
        fn load<'a>(
            &'a self,
            bytes: &'a [u8],
            load_context: &'a mut bevy::asset::LoadContext,
        ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
            Box::pin(async move {
                let clues_file: CluesFile = serde_yaml::from_slice(bytes)?;
                let clues: Clues = clues_file.into();
                load_context.set_default_asset(LoadedAsset::new(clues));
                Ok(())
            })
        }

        fn extensions(&self) -> &[&str] {
            &[".clues.yml"]
        }
    }
}
