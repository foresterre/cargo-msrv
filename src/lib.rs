//! Documentation can be found on the project [README](https://github.com/foresterre/cargo-msrv/blob/main/README.md) page
//! and in the cargo-msrv [book](https://foresterre.github.io/cargo-msrv/). If you can't find an answer on your question,
//! feel free to open an [issue](https://github.com/foresterre/cargo-msrv/issues/new).
//!
//! Issues and ideas may be reported via the [issue tracker](https://github.com/foresterre/cargo-msrv/issues),
//! and questions can be asked on the [discussion forum](https://github.com/foresterre/cargo-msrv/discussions).
//!
//! The docs focus on how to use `cargo-msrv` from the command line. If you want to also use it as a library,
//! please feel free to open an [issue](https://github.com/foresterre/cargo-msrv/issues/new).

#![deny(clippy::all)]
#![allow(
    clippy::upper_case_acronyms,
    clippy::unnecessary_wraps,
    clippy::uninlined_format_args,
    clippy::items_after_test_module
)]

extern crate core;
#[macro_use]
extern crate tracing;

pub use crate::context::{Context, OutputFormat, TracingOptions, TracingTargetOption};
pub use crate::outcome::Outcome;
pub use crate::sub_command::{Find, List, Set, Show, SubCommand, Verify};

use crate::check::RustupToolchainCheck;
use crate::context::ReleaseSource;
use crate::error::{CargoMSRVError, TResult};
use crate::reporter::event::{Meta, SubcommandInit};
use crate::reporter::{Event, Reporter};
use rust_releases::semver;

pub mod check;
pub mod cli;

pub mod error;
pub mod exit_code;
pub mod io;
pub mod reporter;
pub mod toolchain;

mod context;
pub(crate) mod default_target;
pub(crate) mod dependency_graph;
mod external_command;
pub(crate) mod formatting;
pub(crate) mod lockfile;
pub(crate) mod log_level;
pub(crate) mod manifest;
pub(crate) mod msrv;
pub(crate) mod outcome;
mod release_index;
pub(crate) mod releases_filter;
mod rust_release;
pub(crate) mod search_method;
pub(crate) mod setup_toolchain;
pub(crate) mod sub_command;
pub(crate) mod typed_bool;
pub(crate) mod writer;

pub fn run_app(ctx: &Context, reporter: &impl Reporter) -> TResult<()> {
    reporter.report_event(Meta::default())?;
    reporter.report_event(SubcommandInit::new(ctx.reporting_name()))?;

    match ctx {
        Context::Find(ctx) => {
            let index = release_index::fetch_index(reporter, ctx.rust_releases.release_source)?;

            let runner = RustupToolchainCheck::new(
                reporter,
                ctx.ignore_lockfile,
                ctx.no_check_feedback,
                &ctx.environment,
                ctx.run_command(),
            );
            Find::new(&index, runner).run(ctx, reporter)?;
        }
        Context::List(ctx) => {
            List.run(ctx, reporter)?;
        }
        Context::Set(ctx) => {
            let index = release_index::fetch_index(reporter, ctx.rust_releases.release_source).ok();
            Set::new(index.as_ref()).run(ctx, reporter)?;
        }
        Context::Show(ctx) => {
            Show.run(ctx, reporter)?;
        }
        Context::Verify(ctx) => {
            let index = release_index::fetch_index(reporter, ctx.rust_releases.release_source)?;

            let runner = RustupToolchainCheck::new(
                reporter,
                ctx.ignore_lockfile,
                ctx.no_check_feedback,
                &ctx.environment,
                ctx.run_command(),
            );

            Verify::new(&index, runner).run(ctx, reporter)?;
        }
    }

    Ok(())
}
