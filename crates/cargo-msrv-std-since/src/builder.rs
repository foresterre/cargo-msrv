use crate::archive::{ALIGNMENT, AssociatedRecord, Collection, ItemRecord, PathRecord, Since};
use crate::{Item, ItemKind};
use rkyv::rancor;
use rkyv::util::AlignedVec;
use std::collections::{BTreeMap, HashMap};

/// A public path of an item, e.g. `std::collections::HashMap`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathEntry {
    /// The canonical path of the item, which identifies the item.
    pub canonical: String,
    /// The version since which the path is usable. For a re-export, this is the version
    /// of the re-export, which may be older than the version of the item.
    pub since: semver::Version,
}

/// Builds a collection, which is written as an rkyv archive.
///
/// When an item, path, or associated item is inserted more than once, the lowest version
/// is kept.
#[derive(Debug, Default)]
pub struct CollectionBuilder {
    toolchain: Option<String>,
    items: BTreeMap<String, Item>,
    paths: BTreeMap<String, PathEntry>,
    /// Keyed by the canonical path of the type or trait, and the name of the item.
    associated_items: BTreeMap<(String, String), Item>,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("the path '{path}' refers to the unknown item '{canonical}'")]
    UnknownItem { path: String, canonical: String },

    #[error("the version '{0}' does not fit in the archive")]
    VersionOutOfRange(semver::Version),

    #[error("the collection has more items than fit in the archive")]
    TooManyItems,

    #[error("unable to write the archive: {0}")]
    Archive(rancor::Error),
}

impl CollectionBuilder {
    pub fn set_toolchain(&mut self, toolchain: impl Into<String>) {
        self.toolchain = Some(toolchain.into());
    }

    pub fn insert_item(&mut self, canonical: impl Into<String>, item: Item) {
        insert_lowest(&mut self.items, canonical.into(), item, |i| &i.since);
    }

    /// The item of the path must be inserted too, before the collection is written.
    pub fn insert_path(&mut self, path: impl Into<String>, entry: PathEntry) {
        insert_lowest(&mut self.paths, path.into(), entry, |e| &e.since);
    }

    /// Adds an associated item of the type or trait with the given canonical path.
    /// Associated items of a parent which is not inserted, are not written.
    pub fn insert_associated_item(
        &mut self,
        parent: impl Into<String>,
        name: impl Into<String>,
        item: Item,
    ) {
        let key = (parent.into(), name.into());
        insert_lowest(&mut self.associated_items, key, item, |i| &i.since);
    }

    /// Writes the collection as an rkyv archive.
    pub fn to_bytes(&self) -> Result<AlignedVec<ALIGNMENT>, BuildError> {
        let collection = self.collection()?;

        rkyv::to_bytes::<rancor::Error>(&collection).map_err(BuildError::Archive)
    }

    fn collection(&self) -> Result<Collection, BuildError> {
        let ids: HashMap<&str, u32> = self
            .items
            .keys()
            .enumerate()
            .map(|(id, canonical)| Ok((canonical.as_str(), u32::try_from(id)?)))
            .collect::<Result<_, std::num::TryFromIntError>>()
            .map_err(|_| BuildError::TooManyItems)?;

        let mut items = self
            .items
            .iter()
            .map(|(canonical, item)| {
                Ok(ItemRecord {
                    canonical: canonical.clone(),
                    kind: item.kind,
                    since: since(&item.since)?,
                    associated_items: BTreeMap::new(),
                })
            })
            .collect::<Result<Vec<_>, BuildError>>()?;

        let mut methods: BTreeMap<String, Since> = BTreeMap::new();

        for ((parent, name), item) in &self.associated_items {
            let Some(&id) = ids.get(parent.as_str()) else {
                continue;
            };

            let since = since(&item.since)?;

            if item.kind == ItemKind::AssociatedFunction {
                methods
                    .entry(name.clone())
                    .and_modify(|lowest| *lowest = (*lowest).min(since))
                    .or_insert(since);
            }

            let record = AssociatedRecord {
                kind: item.kind,
                since,
            };
            items[id as usize]
                .associated_items
                .insert(name.clone(), record);
        }

        let paths = self
            .paths
            .iter()
            .map(|(path, entry)| {
                let &item =
                    ids.get(entry.canonical.as_str())
                        .ok_or_else(|| BuildError::UnknownItem {
                            path: path.clone(),
                            canonical: entry.canonical.clone(),
                        })?;

                let record = PathRecord {
                    item,
                    since: since(&entry.since)?,
                };

                Ok((path.clone(), record))
            })
            .collect::<Result<BTreeMap<_, _>, BuildError>>()?;

        Ok(Collection {
            toolchain: self.toolchain.clone(),
            items,
            paths,
            methods,
        })
    }
}

fn since(version: &semver::Version) -> Result<Since, BuildError> {
    let component =
        |c: u64| u16::try_from(c).map_err(|_| BuildError::VersionOutOfRange(version.clone()));

    Ok(Since {
        major: component(version.major)?,
        minor: component(version.minor)?,
        patch: component(version.patch)?,
    })
}

fn insert_lowest<K: Ord, V>(
    map: &mut BTreeMap<K, V>,
    key: K,
    value: V,
    since: impl Fn(&V) -> &semver::Version,
) {
    match map.get_mut(&key) {
        Some(existing) if since(existing) <= since(&value) => {}
        Some(existing) => *existing = value,
        None => {
            map.insert(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StdSinceIndex;

    fn item(since: &str) -> Item {
        Item {
            kind: ItemKind::Type,
            since: since.parse().unwrap(),
        }
    }

    #[test]
    fn keeps_the_lowest_version() {
        let mut builder = CollectionBuilder::default();

        builder.insert_item("a", item("1.2.0"));
        builder.insert_item("a", item("1.1.0"));
        builder.insert_item("a", item("1.3.0"));

        assert_eq!(builder.items["a"], item("1.1.0"));
    }

    #[test]
    fn path_to_unknown_item() {
        let mut builder = CollectionBuilder::default();
        let entry = PathEntry {
            canonical: "unknown".to_string(),
            since: semver::Version::new(1, 0, 0),
        };
        builder.insert_path("std::unknown", entry);

        assert!(matches!(
            builder.to_bytes(),
            Err(BuildError::UnknownItem { .. })
        ));
    }

    #[test]
    fn version_out_of_range() {
        let mut builder = CollectionBuilder::default();
        builder.insert_item("a", item("1.70000.0"));

        assert!(matches!(
            builder.to_bytes(),
            Err(BuildError::VersionOutOfRange(_))
        ));
    }

    #[test]
    fn associated_items_of_unknown_parents_are_not_written() {
        let mut builder = CollectionBuilder::default();
        builder.insert_associated_item("unknown", "f", item("1.70.0"));

        let bytes = builder.to_bytes().unwrap();
        let index = StdSinceIndex::from_bytes(&bytes).unwrap();

        assert_eq!(index.method("f"), None);
    }
}
