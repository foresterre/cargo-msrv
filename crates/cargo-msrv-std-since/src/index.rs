use crate::Item;
use crate::archive::{ArchivedCollection, ArchivedItemRecord, ArchivedSince};
use rkyv::rancor;
use std::fmt;

/// Identifies an item in a [`StdSinceIndex`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ItemId(u32);

/// The item a public path refers to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathMatch {
    pub item: ItemId,
    /// The version since which the path is usable.
    pub since: semver::Version,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("the collection is not a valid archive: {0}")]
    InvalidArchive(rancor::Error),
}

/// The collection of stabilized items of the standard library.
///
/// The index borrows the bytes of the archive, and reads them in place.
#[derive(Clone, Copy)]
pub struct StdSinceIndex<'a> {
    collection: &'a ArchivedCollection,
}

impl fmt::Debug for StdSinceIndex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StdSinceIndex")
            .field("toolchain", &self.toolchain())
            .field("items", &self.collection.items.len())
            .finish()
    }
}

impl<'a> StdSinceIndex<'a> {
    /// Validates an archive, and reads it in place from then on.
    ///
    /// The bytes must be aligned to 16 bytes, like the bytes written by
    /// [`CollectionBuilder::to_bytes`](crate::CollectionBuilder::to_bytes), and the
    /// archives of [`source`](crate::source). Misaligned bytes are an error.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ParseError> {
        let collection = rkyv::access::<ArchivedCollection, rancor::Error>(bytes)
            .map_err(ParseError::InvalidArchive)?;

        Ok(Self { collection })
    }

    /// The toolchain which generated the collection, e.g. `rustc 1.101.0-nightly (...)`.
    pub fn toolchain(&self) -> Option<&'a str> {
        self.collection.toolchain.as_ref().map(|t| t.as_str())
    }

    /// Looks up a public path, e.g. `std::collections::HashMap`, or a primitive type,
    /// e.g. `u32`.
    pub fn path(&self, path: &str) -> Option<PathMatch> {
        let record = self.collection.paths.get(path)?;

        Some(PathMatch {
            item: ItemId(record.item.to_native()),
            since: record.since.to_version(),
        })
    }

    fn record(&self, item: ItemId) -> Option<&'a ArchivedItemRecord> {
        self.collection.items.get(usize::try_from(item.0).ok()?)
    }

    pub fn item(&self, item: ItemId) -> Option<Item> {
        let record = self.record(item)?;

        Some(Item {
            kind: (&record.kind).into(),
            since: record.since.to_version(),
        })
    }

    /// The canonical path of an item, e.g. `core::option::Option`.
    pub fn canonical(&self, item: ItemId) -> Option<&'a str> {
        self.record(item).map(|record| record.canonical.as_str())
    }

    /// Looks up an associated function or constant of a type or trait.
    pub fn associated_item(&self, parent: ItemId, name: &str) -> Option<Item> {
        let associated = self.record(parent)?.associated_items.get(name)?;

        Some(Item {
            kind: (&associated.kind).into(),
            since: associated.since.to_version(),
        })
    }

    /// The lowest version of the associated functions with the given name, of any type or
    /// trait. This is useful for method calls, where the type of the receiver is unknown.
    pub fn method(&self, name: &str) -> Option<semver::Version> {
        self.collection
            .methods
            .get(name)
            .map(ArchivedSince::to_version)
    }

    /// Returns the version since which a public path is usable, where the path may end
    /// with an associated item, e.g. `std::option::Option::is_some_and`.
    pub fn resolve_path<S: AsRef<str>>(&self, segments: &[S]) -> Option<semver::Version> {
        if let Some(path) = self.path(&join(segments)) {
            return Some(path.since);
        }

        let (name, parent) = segments.split_last()?;
        let parent = self.path(&join(parent))?;

        // The version of the parent path is used instead of the version of the parent
        // item, since an item may be usable at a path before its own version, e.g.
        // `std::ffi::CStr` is usable since 1.0, while `CStr` says 1.64.
        self.associated_item(parent.item, name.as_ref())
            .map(|associated| associated.since.max(parent.since))
    }

    /// Returns the version since which an item, or one of its associated items, is usable.
    /// The `rest` is either empty, or the name of the associated item.
    ///
    /// This is useful for items which are in scope without naming a path, like the items
    /// of the prelude and the primitive types.
    pub fn resolve_from_item<S: AsRef<str>>(
        &self,
        item: ItemId,
        rest: &[S],
    ) -> Option<semver::Version> {
        let since = self.item(item)?.since;

        match rest {
            [] => Some(since),
            [name] => self
                .associated_item(item, name.as_ref())
                .map(|associated| associated.since.max(since)),
            _ => None,
        }
    }
}

fn join<S: AsRef<str>>(segments: &[S]) -> String {
    segments
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CollectionBuilder, ItemKind, PathEntry};
    use rkyv::util::AlignedVec;
    use std::sync::LazyLock;

    static ARCHIVE: LazyLock<AlignedVec<16>> = LazyLock::new(archive);

    fn version(v: &str) -> semver::Version {
        v.parse().unwrap()
    }

    fn archive() -> AlignedVec<16> {
        let mut builder = CollectionBuilder::default();
        builder.set_toolchain("rustc 1.101.0-nightly");

        let item = |kind, since| Item {
            kind,
            since: version(since),
        };
        let path = |canonical: &str, since| PathEntry {
            canonical: canonical.to_string(),
            since: version(since),
        };

        builder.insert_item("core::option::Option", item(ItemKind::Type, "1.0.0"));
        builder.insert_item("core::hint::black_box", item(ItemKind::Function, "1.66.0"));
        builder.insert_item("core::ffi::c_str::CStr", item(ItemKind::Type, "1.64.0"));
        builder.insert_item("u32", item(ItemKind::Type, "1.0.0"));

        builder.insert_path("std::option::Option", path("core::option::Option", "1.0.0"));
        builder.insert_path(
            "std::prelude::rust_2021::Option",
            path("core::option::Option", "1.55.0"),
        );
        builder.insert_path(
            "std::hint::black_box",
            path("core::hint::black_box", "1.66.0"),
        );
        builder.insert_path("std::ffi::CStr", path("core::ffi::c_str::CStr", "1.0.0"));
        builder.insert_path("core::ffi::CStr", path("core::ffi::c_str::CStr", "1.64.0"));
        builder.insert_path("u32", path("u32", "1.0.0"));

        let function = ItemKind::AssociatedFunction;
        builder.insert_associated_item(
            "core::option::Option",
            "is_some_and",
            item(function, "1.70.0"),
        );
        builder.insert_associated_item("core::option::Option", "map", item(function, "1.0.0"));
        builder.insert_associated_item(
            "core::ffi::c_str::CStr",
            "from_ptr",
            item(function, "1.0.0"),
        );
        builder.insert_associated_item("u32", "div_ceil", item(function, "1.73.0"));
        builder.insert_associated_item("u32", "is_some_and", item(function, "1.80.0"));

        builder.to_bytes().unwrap()
    }

    fn index() -> StdSinceIndex<'static> {
        StdSinceIndex::from_bytes(&ARCHIVE).unwrap()
    }

    #[yare::parameterized(
        item = { &["std", "hint", "black_box"], Some("1.66.0") },
        reexport_newer_than_item = { &["core", "ffi", "CStr"], Some("1.64.0") },
        reexport_older_than_item = { &["std", "ffi", "CStr"], Some("1.0.0") },
        associated_item_of_reexport = { &["std", "ffi", "CStr", "from_ptr"], Some("1.0.0") },
        associated_item = { &["std", "option", "Option", "is_some_and"], Some("1.70.0") },
        associated_item_through_newer_path = { &["std", "prelude", "rust_2021", "Option", "map"], Some("1.55.0") },
        unknown_associated_item = { &["std", "option", "Option", "unknown"], None },
        unknown_path = { &["std", "unknown"], None },
        empty = { &[], None },
    )]
    fn resolve_path(segments: &[&str], expected: Option<&str>) {
        assert_eq!(index().resolve_path(segments), expected.map(version));
    }

    #[yare::parameterized(
        item = { "std::option::Option", &[], Some("1.0.0") },
        prelude_item_uses_the_version_of_the_item = { "std::prelude::rust_2021::Option", &[], Some("1.0.0") },
        associated_item = { "std::option::Option", &["is_some_and"], Some("1.70.0") },
        primitive = { "u32", &["div_ceil"], Some("1.73.0") },
        too_many_segments = { "std::option::Option", &["a", "b"], None },
    )]
    fn resolve_from_item(path: &str, rest: &[&str], expected: Option<&str>) {
        let index = index();
        let item = index.path(path).unwrap().item;

        assert_eq!(index.resolve_from_item(item, rest), expected.map(version));
    }

    #[test]
    fn item_and_canonical() {
        let index = index();
        let item = index.path("std::ffi::CStr").unwrap().item;

        assert_eq!(index.canonical(item), Some("core::ffi::c_str::CStr"));
        assert_eq!(
            index.item(item),
            Some(Item {
                kind: ItemKind::Type,
                since: version("1.64.0")
            })
        );
    }

    #[test]
    fn method_takes_the_lowest_version() {
        assert_eq!(index().method("is_some_and"), Some(version("1.70.0")));
        assert_eq!(index().method("unknown"), None);
    }

    #[test]
    fn toolchain() {
        assert_eq!(index().toolchain(), Some("rustc 1.101.0-nightly"));
    }

    #[test]
    fn invalid_archive() {
        let mut bytes = AlignedVec::<16>::new();
        bytes.extend_from_slice(b"not an archive");

        assert!(matches!(
            StdSinceIndex::from_bytes(&bytes),
            Err(ParseError::InvalidArchive(_))
        ));
    }

    #[test]
    fn misaligned_archive() {
        let mut bytes = AlignedVec::<16>::new();
        bytes.push(0);
        bytes.extend_from_slice(&ARCHIVE);

        assert!(matches!(
            StdSinceIndex::from_bytes(&bytes[1..]),
            Err(ParseError::InvalidArchive(_))
        ));
    }
}
