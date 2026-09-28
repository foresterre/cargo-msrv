//! The subset of the rustdoc JSON format which is used by the generator.
//!
//! Only the fields which are needed are deserialized, so the large files of the standard
//! library (e.g. `core.json` is about 90 MB) can be read without keeping everything.

use serde::Deserialize;
use serde::de::IgnoredAny;
use std::collections::HashMap;

pub type Id = u32;

#[derive(Deserialize)]
pub struct Crate {
    pub root: Id,
    pub index: HashMap<Id, JsonItem>,
    pub paths: HashMap<Id, ItemSummary>,
    pub external_crates: HashMap<u32, ExternalCrate>,
}

#[derive(Deserialize)]
pub struct JsonItem {
    pub name: Option<String>,
    pub visibility: Visibility,
    pub stability: Option<Stability>,
    #[serde(deserialize_with = "inner_or_unit")]
    pub inner: Inner,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Visibility {
    /// `public`, `default` (e.g. trait items), or `crate`
    Simple(String),
    /// `pub(in path)`
    Restricted(IgnoredAny),
}

impl Visibility {
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Simple(v) if v == "public" || v == "default")
    }
}

#[derive(Deserialize)]
pub struct Stability {
    pub level: Level,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Level {
    Stable { stable: StableSince },
    Unstable(IgnoredAny),
}

#[derive(Deserialize)]
pub struct StableSince {
    pub since: String,
}

/// The kind of item. At most one of the fields is present.
#[derive(Default, Deserialize)]
pub struct Inner {
    pub module: Option<Module>,
    #[serde(rename = "use")]
    pub use_: Option<Use>,
    #[serde(rename = "struct")]
    pub struct_: Option<WithImpls>,
    #[serde(rename = "enum")]
    pub enum_: Option<WithImpls>,
    pub union: Option<WithImpls>,
    #[serde(rename = "trait")]
    pub trait_: Option<WithItems>,
    #[serde(rename = "impl")]
    pub impl_: Option<Impl>,
    pub primitive: Option<Primitive>,
    pub function: Option<IgnoredAny>,
    pub constant: Option<IgnoredAny>,
    #[serde(rename = "static")]
    pub static_: Option<IgnoredAny>,
    pub type_alias: Option<IgnoredAny>,
    #[serde(rename = "macro")]
    pub macro_: Option<IgnoredAny>,
    pub proc_macro: Option<IgnoredAny>,
    pub assoc_const: Option<IgnoredAny>,
}

/// Kinds without data, like `"extern_type"`, are a string instead of a map.
fn inner_or_unit<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Inner, D::Error> {
    struct InnerVisitor;

    impl<'de> serde::de::Visitor<'de> for InnerVisitor {
        type Value = Inner;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a map or a string")
        }

        fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<Inner, E> {
            Ok(Inner::default())
        }

        fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<Inner, A::Error> {
            Inner::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(InnerVisitor)
}

#[derive(Deserialize)]
pub struct Module {
    pub items: Vec<Id>,
}

#[derive(Deserialize)]
pub struct Use {
    pub name: String,
    pub id: Option<Id>,
    pub is_glob: bool,
}

#[derive(Deserialize)]
pub struct WithImpls {
    pub impls: Vec<Id>,
}

#[derive(Deserialize)]
pub struct WithItems {
    pub items: Vec<Id>,
}

#[derive(Deserialize)]
pub struct Impl {
    pub items: Vec<Id>,
    #[serde(rename = "trait")]
    pub trait_: Option<IgnoredAny>,
}

#[derive(Deserialize)]
pub struct Primitive {
    pub name: String,
    pub impls: Vec<Id>,
}

#[derive(Deserialize)]
pub struct ItemSummary {
    pub crate_id: u32,
    pub path: Vec<String>,
    pub kind: String,
}

#[derive(Deserialize)]
pub struct ExternalCrate {
    pub name: String,
}
