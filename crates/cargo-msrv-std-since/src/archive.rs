//! The format of the archive, which is written by the [`CollectionBuilder`], and read in
//! place by the [`StdSinceIndex`].
//!
//! [`CollectionBuilder`]: crate::CollectionBuilder
//! [`StdSinceIndex`]: crate::StdSinceIndex

use crate::ItemKind;
use std::collections::BTreeMap;

/// The alignment of the archive, which rkyv requires to access it in place.
pub const ALIGNMENT: usize = 16;

/// A version, which is small in the archive compared to a [`semver::Version`].
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct Since {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl ArchivedSince {
    pub fn to_version(&self) -> semver::Version {
        semver::Version::new(
            self.major.to_native().into(),
            self.minor.to_native().into(),
            self.patch.to_native().into(),
        )
    }
}

// The maps are ordered, so the same input gives the same archive, which keeps the diffs of
// the bundled archive small. The order of a `HashMap` differs from run to run.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Collection {
    pub toolchain: Option<String>,
    /// Indexed by [`ItemId`](crate::ItemId).
    pub items: Vec<ItemRecord>,
    pub paths: BTreeMap<String, PathRecord>,
    /// The lowest version of the associated functions, by name.
    pub methods: BTreeMap<String, Since>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ItemRecord {
    pub canonical: String,
    pub kind: ItemKind,
    pub since: Since,
    pub associated_items: BTreeMap<String, AssociatedRecord>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct AssociatedRecord {
    pub kind: ItemKind,
    pub since: Since,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct PathRecord {
    pub item: u32,
    pub since: Since,
}
