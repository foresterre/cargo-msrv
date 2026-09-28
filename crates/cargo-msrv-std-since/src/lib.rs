//! The Rust versions in which items of the Rust standard library were stabilized.
//!
//! The standard library annotates its public items with an attribute like
//! `#[stable(feature = "is_some_and", since = "1.70.0")]`. This crate holds a collection
//! of these versions for the `std`, `core`, and `alloc` crates, so we can tell which Rust
//! version is at least required to use an item.
//!
//! The collection is generated from the rustdoc JSON of the standard library, by the
//! `cargo-msrv-std-since-generator` crate, and consists of:
//!
//! * the items, with the version in which they were stabilized, and their associated
//!   functions and constants,
//! * every public path of these items, including re-exports, with the version since which
//!   the path is usable (e.g. `core::ffi::CStr` is usable since 1.64, while `std::ffi::CStr`
//!   is usable since 1.0), and
//! * the lowest version of the associated functions with a given name, for method calls.
//!
//! The collection is an [rkyv](https://docs.rs/rkyv) archive. It is accessed in place,
//! without deserializing it, so a lookup only decodes the entries it needs.
//!
//! A collection is bundled with this crate, and a newer one can be read from a file or
//! downloaded, see [`source`].

mod archive;
mod builder;
mod index;
mod item;
pub mod source;

pub use builder::{BuildError, CollectionBuilder, PathEntry};
pub use index::{ItemId, ParseError, PathMatch, StdSinceIndex};
pub use item::{Item, ItemKind};
