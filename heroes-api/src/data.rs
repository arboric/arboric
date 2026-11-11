//! In-memory Star Wars data, using the same ids as the dotnet heroes example.

use std::collections::HashMap;

/// Immutable in-memory database, used as the GraphQL context.
#[derive(Debug, Clone)]
pub struct Database {
    humans: HashMap<&'static str, HumanData>,
    droids: HashMap<&'static str, DroidData>,
}

#[derive(Debug, Clone)]
pub struct HumanData {
    pub id: &'static str,
    pub name: &'static str,
    pub friend_ids: &'static [&'static str],
    pub home_planet: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct DroidData {
    pub id: &'static str,
    pub name: &'static str,
    pub friend_ids: &'static [&'static str],
    pub primary_function: Option<&'static str>,
}

impl juniper::Context for Database {}

impl Database {
    pub fn new() -> Self {
        let humans = [
            HumanData {
                id: "1",
                name: "Luke Skywalker",
                friend_ids: &["3", "4"],
                home_planet: Some("Tatooine"),
            },
            HumanData {
                id: "2",
                name: "Darth Vader",
                friend_ids: &[],
                home_planet: Some("Tatooine"),
            },
        ];
        let droids = [
            DroidData {
                id: "3",
                name: "R2-D2",
                friend_ids: &["1"],
                primary_function: Some("Astromech"),
            },
            DroidData {
                id: "4",
                name: "C-3PO",
                friend_ids: &["1", "3"],
                primary_function: Some("Protocol"),
            },
        ];
        Self {
            humans: humans.into_iter().map(|h| (h.id, h)).collect(),
            droids: droids.into_iter().map(|d| (d.id, d)).collect(),
        }
    }

    pub fn human(&self, id: &str) -> Option<&HumanData> {
        self.humans.get(id)
    }

    pub fn droid(&self, id: &str) -> Option<&DroidData> {
        self.droids.get(id)
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
