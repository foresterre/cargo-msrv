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

pub use crate::outcome::Compatibility;
pub use crate::sub_command::{Find, List, Set, Show, SubCommand, Verify};
pub use cargo_msrv_context::types::{OutputFormat, TracingTargetOption};
pub use cargo_msrv_context::{Context, TracingOptions};

use crate::compatibility::{RunCommandProvider, RustupToolchainCheck};
use crate::error::{CargoMSRVError, TResult};
use crate::reporter::event::{Meta, SelectedPackages, SubcommandInit};
use crate::reporter::{Event, Reporter};
use cargo_msrv_context::types::ReleaseSource;
use rust::release_index;
use rust_releases::semver;

pub use cargo_msrv_cli::cli;
pub use cargo_msrv_context::context;
pub use cargo_msrv_manifest as manifest;

pub mod compatibility;

pub mod dependency_graph;
pub mod error;
pub mod exit_code;
mod external_command;
pub mod io;
pub mod lockfile;
pub mod msrv;
pub mod outcome;
pub mod reporter;
pub mod rust;
pub mod search_method;
pub mod sub_command;
pub mod typed_bool;
pub mod writer;

pub fn run_app(ctx: &Context, reporter: &impl Reporter) -> TResult<()> {
    reporter.report_event(Meta::default())?;
    reporter.report_event(SelectedPackages::new(
        ctx.environment_context().workspace_packages.selected(),
    ))?;
    reporter.report_event(SubcommandInit::new(ctx.reporting_name()))?;

    match ctx {
        Context::Find(ctx) => {
            let index = release_index::fetch_index(reporter, ctx.rust_releases.release_source)?;

            let runner = RustupToolchainCheck::new(
                reporter,
                ctx.ignore_lockfile,
                ctx.no_check_feedback,
                ctx.skip_unavailable_toolchains,
                &ctx.environment,
                ctx.provide_run_command(),
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
                false,
                &ctx.environment,
                ctx.provide_run_command(),
            );

            Verify::new(&index, runner).run(ctx, reporter)?;
        }
    }

    Ok(())
}
