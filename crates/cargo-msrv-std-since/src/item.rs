use std::fmt;

/// The kind of item which was annotated with a `#[stable]` attribute.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ItemKind {
    Function,
    AssociatedFunction,
    AssociatedConstant,
    Macro,
    /// A struct, enum, union, type alias, or primitive type.
    Type,
    Trait,
    /// A constant or static.
    Constant,
    Module,
}

impl From<&ArchivedItemKind> for ItemKind {
    fn from(kind: &ArchivedItemKind) -> Self {
        match kind {
            ArchivedItemKind::Function => Self::Function,
            ArchivedItemKind::AssociatedFunction => Self::AssociatedFunction,
            ArchivedItemKind::AssociatedConstant => Self::AssociatedConstant,
            ArchivedItemKind::Macro => Self::Macro,
            ArchivedItemKind::Type => Self::Type,
            ArchivedItemKind::Trait => Self::Trait,
            ArchivedItemKind::Constant => Self::Constant,
            ArchivedItemKind::Module => Self::Module,
        }
    }
}

impl fmt::Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self {
            Self::Function => "function",
            Self::AssociatedFunction => "associated function",
            Self::AssociatedConstant => "associated constant",
            Self::Macro => "macro",
            Self::Type => "type",
            Self::Trait => "trait",
            Self::Constant => "constant",
            Self::Module => "module",
        };

        f.write_str(kind)
    }
}

/// An item, and the version in which it was stabilized.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    pub kind: ItemKind,
    pub since: semver::Version,
}
