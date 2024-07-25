use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use std::collections::HashSet;

use common_wit::Target;
use http::Uri;

use crate::{ModuleDefinition, ModuleId};

#[derive(Default, Debug, Clone)]
pub enum Frequency {
    #[default]
    Once,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Location {
    #[default]
    Local,
    Remote(Uri),
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Location::Local => match other {
                Location::Local => Ordering::Equal,
                Location::Remote(_) => Ordering::Greater,
            },
            Location::Remote(uri) => match other {
                Location::Local => Ordering::Less,
                Location::Remote(other_uri) => uri
                    .authority()
                    .partial_cmp(&other_uri.authority())
                    .unwrap_or(Ordering::Equal),
            },
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Boundary(HashSet<Location>);

impl Boundary {
    pub fn add(&mut self, location: Location) -> &mut Self {
        self.0.insert(location);
        self
    }

    pub fn intersect(&self, other: &Boundary) -> Boundary {
        Boundary(
            self.0
                .intersection(&other.0)
                .map(|location| location.clone())
                .collect(),
        )
    }

    pub fn get_schedulable(&self) -> Option<Location> {
        self.0.iter().next().cloned()
    }
}

#[derive(Debug, Clone)]
pub struct Schedule {
    pub frequency: Frequency,
    pub boundary: Boundary,
}

#[derive(Default, Debug)]
pub struct ScheduleBuilder {
    frequency: Frequency,
    boundary: Boundary,
}

impl ScheduleBuilder {
    pub fn frequency(&mut self, frequency: Frequency) -> &mut Self {
        self.frequency = frequency;
        self
    }

    pub fn allow_location(&mut self, location: Location) -> &mut Self {
        self.boundary.add(location);
        self
    }

    pub fn build(self) -> Schedule {
        Schedule {
            frequency: self.frequency,
            boundary: self.boundary,
        }
    }
}
