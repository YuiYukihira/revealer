//! Locations and the Asset loader for them
use bevy::{reflect::TypeUuid, utils::HashMap};
use serde::Deserialize;

/// A wrapper around a string to represent a location
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
#[serde(transparent)]
pub struct LocationId(String);

/// A location
#[derive(Debug)]
pub struct Location {
    /// The id of the location
    pub id: LocationId,
    /// The name of the location
    pub name: String,
    /// Locations that are a parent of this one.
    pub parent_locations: Vec<LocationId>,
    /// Locations that this location is a parent of.
    pub children_locations: Vec<LocationId>,
    /// A description of the location (intended for public knowledge)
    pub info: Option<String>,
}

/// Same as location, but with some fields missing.
/// We can't derserialize to a [`Location`] directly as the
/// child locations have to be computed.
#[derive(Debug, Deserialize)]
struct LocationDeser {
    /// The id of the location
    pub id: LocationId,
    /// The name of the location
    pub name: String,
    /// Locations that are a parent of this one.
    pub parent_locations: Vec<LocationId>,
    /// A description of the location (intended for public knowledge)
    pub info: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LocationsFile {
    locations: Vec<LocationDeser>,
}

/// A holder for many locations
#[derive(Debug, Default, TypeUuid)]
#[uuid = "9d69dd7b-8fbb-460d-bd7c-10a71f87c2b4"]
pub struct Locations {
    locations: HashMap<LocationId, Location>,
}

impl From<LocationsFile> for Locations {
    fn from(file: LocationsFile) -> Self {
        let mut locations = Self::new();

        // First add all the locations
        for location in file.locations {
            let loc = Location {
                id: location.id.clone(),
                name: location.name,
                parent_locations: location.parent_locations,
                children_locations: Vec::new(),
                info: location.info,
            };

            locations.locations.insert(location.id, loc);
        }

        // Get a list of all the locations and thier children
        let mut children_locations: HashMap<LocationId, Vec<LocationId>> = HashMap::new();

        for (id, location) in &locations.locations {
            for parent_id in &location.parent_locations {
                match children_locations.get_mut(parent_id) {
                    Some(l) => l.push(id.clone()),
                    None => {
                        children_locations.insert(parent_id.clone(), vec![id.clone()]);
                    }
                }
            }
        }

        // Then we can go through and add the children
        for (id, location) in locations.locations.iter_mut() {
            location.children_locations = children_locations.remove(id).unwrap();
        }

        locations
    }
}

impl Locations {
    /// Create a new instance
    pub fn new() -> Self {
        Default::default()
    }

    /// Get a [`Option`]al reference to a [`Location`]
    pub fn get(&self, id: &LocationId) -> Option<&Location> {
        self.locations.get(id)
    }

    /// Get a [`Option`]al mutable reference to a [`Location`]
    pub fn get_mut(&mut self, id: &LocationId) -> Option<&mut Location> {
        self.locations.get_mut(id)
    }

    /// Get an [`Iterator`] over the parents of a [`Location`] from its [`LocationId`]
    pub fn iter_parents(&self, id: &LocationId) -> impl Iterator<Item = &Location> {
        self.locations
            .get(id)
            .into_iter()
            .flat_map(|l| &l.parent_locations)
            .filter_map(|l_id| self.get(l_id))
    }

    /// Get an [`Iterator`] over the children of a [`Location`] from its [`LocationId`]
    pub fn iter_children(&self, id: &LocationId) -> impl Iterator<Item = &Location> {
        self.locations
            .get(id)
            .into_iter()
            .flat_map(|l| &l.children_locations)
            .filter_map(|l_id| self.get(l_id))
    }
}

mod assets {
    use bevy::{
        asset::{AssetLoader, LoadedAsset},
        prelude::{AddAsset, Plugin},
    };

    use super::{Locations, LocationsFile};

    /// Bevy plugin to load a locations file
    pub struct LocationsAssetPlugin;
    impl Plugin for LocationsAssetPlugin {
        fn build(&self, app: &mut bevy::prelude::App) {
            app.add_asset::<Locations>()
                .add_asset_loader(LocationsAssetLoader);
        }
    }

    struct LocationsAssetLoader;
    impl AssetLoader for LocationsAssetLoader {
        fn load<'a>(
            &'a self,
            bytes: &'a [u8],
            load_context: &'a mut bevy::asset::LoadContext,
        ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
            Box::pin(async move {
                let locations_file: LocationsFile = serde_yaml::from_slice(bytes)?;
                let locations: Locations = locations_file.into();
                load_context.set_default_asset(LoadedAsset::new(locations));
                Ok(())
            })
        }

        fn extensions(&self) -> &[&str] {
            &[".locations.yml"]
        }
    }
}
