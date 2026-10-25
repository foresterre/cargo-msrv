//! Renders tables for the terminal output of `cargo-msrv`
//!
//! ```
//! use cargo_msrv_table::{Style, Table, Width};
//!
//! let mut table = Table::new(Style::Rounded, Width::AtMost(80));
//! table.header(["Name", "MSRV"]).row(["foo", "1.70.0"]);
//!
//! let expected = "\
//! ╭──────┬────────╮
//! │ Name │ MSRV   │
//! ├──────┼────────┤
//! │ foo  │ 1.70.0 │
//! ╰──────┴────────╯";
//!
//! assert_eq!(table.to_string(), expected);
//! ```

mod layout;
mod style;
mod table;
mod wrap;

pub use layout::Width;
pub use style::Style;
pub use table::{Margin, Table};
