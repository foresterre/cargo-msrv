//! Walks the public module tree of `std`, `core`, and `alloc` from the crate roots, and
//! follows re-exports into other crates, like rustdoc does.
//!
//! Every public path of an item is recorded, with the version since which the path is
//! usable: the highest version of the modules along the path, and of the item or the
//! re-export at the end of it.

use crate::json::{Crate, Id, Inner, JsonItem, Level, Primitive, Stability};
use cargo_msrv_std_since::{CollectionBuilder, Item, ItemKind, PathEntry};
use std::collections::{HashMap, HashSet};

/// The crates which are walked from their root.
const ROOTS: [&str; 3] = ["std", "core", "alloc"];

/// The crates which are loaded, since the roots re-export their items.
pub const CRATES: [&str; 4] = ["std", "core", "alloc", "std_detect"];

/// Guards against deep module trees, in addition to the check for cycles.
const MAX_DEPTH: usize = 12;

/// A crate of the rustdoc JSON, with a lookup of its items by their canonical path.
pub struct Loaded {
    krate: Crate,
    /// The ids of the items defined by this crate, by their canonical path and kind.
    /// The kind is part of the key, since items of different kinds may share a path,
    /// e.g. the `core::panic` module and the `core::panic` macro.
    by_path: HashMap<(Vec<String>, String), Id>,
}

impl Loaded {
    pub fn new(krate: Crate) -> Self {
        let by_path = krate
            .paths
            .iter()
            .filter(|(_, summary)| summary.crate_id == 0)
            .map(|(id, summary)| ((summary.path.clone(), summary.kind.clone()), *id))
            .collect();

        Self { krate, by_path }
    }
}

/// The outcome of a walk.
pub struct Walked {
    pub builder: CollectionBuilder,
    /// The versions which could not be parsed. The items with these versions are skipped.
    pub unknown_versions: HashSet<String>,
}

/// An item in one of the loaded crates.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct Node<'c> {
    krate: &'c str,
    id: Id,
}

enum Stable {
    Since(semver::Version),
    Unstable,
    Unknown,
}

pub struct Walker<'c> {
    crates: &'c HashMap<&'static str, Loaded>,
    /// The version of `since = "CURRENT_RUSTC_VERSION"`.
    current: semver::Version,
    builder: CollectionBuilder,
    visited_parents: HashSet<Node<'c>>,
    unknown_versions: HashSet<String>,
}

impl<'c> Walker<'c> {
    /// The `crates` must contain the [`CRATES`].
    pub fn new(crates: &'c HashMap<&'static str, Loaded>, current: semver::Version) -> Self {
        Self {
            crates,
            current,
            builder: CollectionBuilder::default(),
            visited_parents: HashSet::new(),
            unknown_versions: HashSet::new(),
        }
    }

    pub fn walk(mut self) -> Walked {
        for root in ROOTS {
            let node = Node {
                krate: root,
                id: self.crates[root].krate.root,
            };
            let since = semver::Version::new(1, 0, 0);
            self.visit(node, root.to_string(), vec![], &since, None, &mut vec![]);
        }

        self.primitives();

        Walked {
            builder: self.builder,
            unknown_versions: self.unknown_versions,
        }
    }

    fn item(&self, node: Node<'c>) -> Option<&'c JsonItem> {
        self.crates[node.krate].krate.index.get(&node.id)
    }

    /// Follows the id of an item in another crate, to the crate which defines it.
    fn resolve(&self, krate: &'c str, id: Id) -> Option<Node<'c>> {
        let loaded = &self.crates[krate];

        if loaded.krate.index.contains_key(&id) {
            return Some(Node { krate, id });
        }

        let summary = loaded.krate.paths.get(&id)?;
        let name = &loaded.krate.external_crates.get(&summary.crate_id)?.name;
        let (&other, other_loaded) = self.crates.get_key_value(name.as_str())?;
        let key = (summary.path.clone(), summary.kind.clone());
        let id = *other_loaded.by_path.get(&key)?;

        Some(Node { krate: other, id })
    }

    fn canonical(&self, node: Node<'c>, fallback: &str) -> String {
        self.crates[node.krate]
            .krate
            .paths
            .get(&node.id)
            .map(|summary| summary.path.join("::"))
            .unwrap_or_else(|| fallback.to_string())
    }

    fn stability(&mut self, item: &JsonItem) -> Stable {
        match &item.stability {
            None => Stable::Unknown,
            Some(Stability {
                level: Level::Unstable(_),
            }) => Stable::Unstable,
            Some(Stability {
                level: Level::Stable { stable },
            }) => match stable.since.as_str() {
                "CURRENT_RUSTC_VERSION" => Stable::Since(self.current.clone()),
                since => match since.parse() {
                    Ok(version) => Stable::Since(version),
                    Err(_) => {
                        self.unknown_versions.insert(since.to_string());
                        Stable::Unknown
                    }
                },
            },
        }
    }

    /// Visits an item at a public path, and records it.
    ///
    /// The `since` is the version since which the parent path is usable. If the item is
    /// visited through a re-export, the `reexport_since` is the stability of the re-export.
    fn visit(
        &mut self,
        node: Node<'c>,
        name: String,
        mut path: Vec<String>,
        since: &semver::Version,
        reexport_since: Option<&semver::Version>,
        ancestors: &mut Vec<Node<'c>>,
    ) {
        let Some(item) = self.item(node) else {
            return;
        };

        // Items without stability information are skipped, since no evidence is better
        // than wrong evidence.
        let Stable::Since(item_since) = self.stability(item) else {
            return;
        };

        let Some(kind) = kind(&item.inner) else {
            return;
        };

        path.push(name);
        let joined = path.join("::");
        // The stability of a re-export says since when the item is usable at its path.
        // This may be older than the stability of the item, e.g. `std::ffi::CStr` is
        // usable since 1.0, while `CStr` itself says 1.64, when it moved to `core`.
        let at_path = reexport_since.unwrap_or(&item_since);
        let path_since = since.clone().max(at_path.clone());
        let canonical = self.canonical(node, &joined);

        self.builder.insert_item(
            canonical.clone(),
            Item {
                kind,
                since: item_since,
            },
        );
        self.builder.insert_path(
            joined,
            PathEntry {
                canonical: canonical.clone(),
                since: path_since.clone(),
            },
        );

        if let Some(module) = &item.inner.module
            && !ancestors.contains(&node)
            && ancestors.len() < MAX_DEPTH
        {
            ancestors.push(node);
            self.module_children(node, &module.items, &path, &path_since, ancestors);
            ancestors.pop();
        }

        self.associated_items_of(node, item, &canonical);
    }

    fn module_children(
        &mut self,
        module: Node<'c>,
        children: &[Id],
        path: &[String],
        since: &semver::Version,
        ancestors: &mut Vec<Node<'c>>,
    ) {
        for &id in children {
            let child = Node {
                krate: module.krate,
                id,
            };
            let Some(item) = self.item(child) else {
                continue;
            };

            if !item.visibility.is_public() {
                continue;
            }

            let Some(reexport) = &item.inner.use_ else {
                if let Some(name) = &item.name {
                    self.visit(child, name.clone(), path.to_vec(), since, None, ancestors);
                }
                continue;
            };

            // A re-export may have been stabilized later than the item it re-exports.
            let reexport_since = match self.stability(item) {
                Stable::Since(version) => Some(version),
                Stable::Unstable => continue,
                Stable::Unknown => None,
            };

            let Some(target) = reexport.id.and_then(|id| self.resolve(module.krate, id)) else {
                continue;
            };

            if !reexport.is_glob {
                let name = reexport.name.clone();
                let reexport_since = reexport_since.as_ref();
                self.visit(
                    target,
                    name,
                    path.to_vec(),
                    since,
                    reexport_since,
                    ancestors,
                );
            } else if let Some(glob) = self.item(target).and_then(|i| i.inner.module.as_ref())
                && !ancestors.contains(&target)
            {
                ancestors.push(target);
                let glob_since = reexport_since.map_or(since.clone(), |v| since.clone().max(v));
                self.module_children(target, &glob.items, path, &glob_since, ancestors);
                ancestors.pop();
            }
        }
    }

    fn associated_items_of(&mut self, node: Node<'c>, item: &'c JsonItem, canonical: &str) {
        if !self.visited_parents.insert(node) {
            return;
        }

        let inner = &item.inner;

        if let Some(with_items) = &inner.trait_ {
            self.associated_items(node.krate, &with_items.items, canonical);
        }

        let impls: Vec<Id> = [&inner.struct_, &inner.enum_, &inner.union]
            .into_iter()
            .flatten()
            .flat_map(|with_impls| with_impls.impls.iter().copied())
            .collect();

        self.inherent_impls(node.krate, &impls, canonical);
    }

    fn inherent_impls(&mut self, krate: &'c str, impls: &[Id], canonical: &str) {
        for &id in impls {
            let Some(item) = self.item(Node { krate, id }) else {
                continue;
            };

            if let Some(implementation) = &item.inner.impl_
                && implementation.trait_.is_none()
            {
                self.associated_items(krate, &implementation.items, canonical);
            }
        }
    }

    fn associated_items(&mut self, krate: &'c str, ids: &[Id], parent: &str) {
        for &id in ids {
            let Some(item) = self.item(Node { krate, id }) else {
                continue;
            };

            let kind = if item.inner.function.is_some() {
                ItemKind::AssociatedFunction
            } else if item.inner.assoc_const.is_some() {
                ItemKind::AssociatedConstant
            } else {
                continue;
            };

            let Some(name) = item.name.as_ref().filter(|_| item.visibility.is_public()) else {
                continue;
            };

            if let Stable::Since(since) = self.stability(item) {
                let item = Item { kind, since };
                self.builder
                    .insert_associated_item(parent, name.clone(), item);
            }
        }
    }

    /// Records the primitive types and their inherent methods. These are spread over
    /// several crates, e.g. `f64::abs` is defined in `core`, and `f64::sqrt` in `std`.
    fn primitives(&mut self) {
        // The crates and items are visited in a fixed order, so the same input gives the
        // same collection.
        for krate in CRATES {
            let mut primitives: Vec<(Id, &Primitive)> = self.crates[krate]
                .krate
                .index
                .iter()
                .filter_map(|(id, item)| Some((*id, item.inner.primitive.as_ref()?)))
                .collect();
            primitives.sort_by_key(|(id, _)| *id);

            for (_, primitive) in primitives {
                self.builder.insert_item(
                    primitive.name.clone(),
                    Item {
                        kind: ItemKind::Type,
                        since: semver::Version::new(1, 0, 0),
                    },
                );

                // A primitive type is in scope everywhere, by its name.
                self.builder.insert_path(
                    primitive.name.clone(),
                    PathEntry {
                        canonical: primitive.name.clone(),
                        since: semver::Version::new(1, 0, 0),
                    },
                );

                self.inherent_impls(krate, &primitive.impls, &primitive.name);
            }
        }
    }
}

fn kind(inner: &Inner) -> Option<ItemKind> {
    let kind = if inner.module.is_some() {
        ItemKind::Module
    } else if inner.function.is_some() {
        ItemKind::Function
    } else if inner.struct_.is_some()
        || inner.enum_.is_some()
        || inner.union.is_some()
        || inner.type_alias.is_some()
        || inner.primitive.is_some()
    {
        ItemKind::Type
    } else if inner.trait_.is_some() {
        ItemKind::Trait
    } else if inner.constant.is_some() || inner.static_.is_some() {
        ItemKind::Constant
    } else if inner.macro_.is_some() || inner.proc_macro.is_some() {
        ItemKind::Macro
    } else {
        return None;
    };

    Some(kind)
}
