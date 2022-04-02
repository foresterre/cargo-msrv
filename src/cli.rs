use crate::cli::configurators::Configure;
use crate::cli::custom_check_opts::CustomCheckOpts;
use crate::cli::find_opts::FindOpts;
use crate::cli::rust_releases_opts::RustReleasesOpts;
use crate::cli::shared_opts::SharedOpts;
use crate::cli::toolchain_opts::ToolchainOpts;
use crate::config::list::ListMsrvVariant;
use crate::config::ConfigBuilder;
use crate::fetch::default_target;
use crate::manifest::bare_version::BareVersion;
use crate::{CargoMSRVError, Config, ModeIntent};
use clap::{AppSettings, Args, Parser, Subcommand};
use std::convert::{TryFrom, TryInto};
use std::ffi::{OsStr, OsString};

pub(in crate::cli) mod configurators;
pub(crate) mod custom_check_opts;
pub(crate) mod find_opts;
pub(crate) mod rust_releases_opts;
pub(crate) mod shared_opts;
pub(crate) mod toolchain_opts;

#[derive(Debug, Parser)]
#[clap(version, bin_name = "cargo", max_term_width = 120)]
pub struct CargoCli {
    #[clap(subcommand)]
    subcommand: CargoMsrvCli,
}

impl CargoCli {
    pub fn parse_args<I: IntoIterator<Item = T>, T: Into<OsString> + Clone>(args: I) -> Self {
        let modified_args = modify_args(args);
        CargoCli::parse_from(modified_args)
    }
}

// When we call cargo-msrv with cargo, cargo will supply the msrv subcommand, in addition
// to the binary name itself. As a result, when you call cargo-msrv without cargo, for example
// `cargo-msrv` (without cargo) instead of `cargo msrv` (with cargo), the process will receive
// too many arguments, and you will have to specify the subcommand again like so: `cargo-msrv msrv`.
// This function removes the subcommand when it's present in addition to the program name.
fn modify_args<I: IntoIterator<Item = T>, T: Into<OsString> + Clone>(
    args: I,
) -> impl IntoIterator<Item = OsString> {
    let mut args = args.into_iter().map(Into::into).collect::<Vec<_>>();

    if args.len() >= 2 {
        let program: &OsStr = args[0].as_os_str();
        let program = program.to_string_lossy();

        // when `cargo-msrv(.exe)` or `msrv` are present
        if program.ends_with("cargo-msrv") || program.ends_with("cargo-msrv.exe") {
            // remove `msrv`, or `cargo-msrv(.exe)`
            args[0] = OsStr::new("cargo").to_os_string();

            let cargo_msrv_subcmd: &OsStr = args[1].as_os_str();
            let cargo_msrv_subcmd = cargo_msrv_subcmd.to_string_lossy();

            if cargo_msrv_subcmd != "msrv" {
                args.insert(1, OsStr::new("msrv").to_os_string());
            }
        }
    }

    args
}

#[derive(Debug, Subcommand)]
pub(in crate::cli) enum CargoMsrvCli {
    /// Find your Minimum Supported Rust Version!
    #[clap(
        author = "Martijn Gribnau <garm@ilumeo.com>",
        after_help = r#"
            You may provide a custom compatibility `check` command as the last argument (only
            when this argument is provided via the double dash syntax, e.g. `$ cargo msrv -- custom
            command`.
            This custom check command will then be used to validate whether a Rust version is
            compatible. 
            A custom `check` command should be runnable by rustup, as they will be passed on to
            rustup like so: `rustup run <toolchain> <COMMAND...>`. NB: You only need to provide the
            <COMMAND...> part.
            
            By default, the custom check command is `cargo check`.
        "#
    )]
    MSRV(CargoMsrvOpts),
}

#[derive(Debug, Args)]
#[clap(version)]
pub(in crate::cli) struct CargoMsrvOpts {
    #[clap(flatten)]
    pub(in crate::cli) find_opts: FindOpts,

    #[clap(flatten)]
    pub(in crate::cli) shared_opts: SharedOpts,

    #[clap(subcommand)]
    pub(in crate::cli) subcommand: Option<SubCommand>,

    /// DEPRECATED: Use the `cargo msrv verify` subcommand instead
    #[clap(long, global = false, hide = true)]
    pub(in crate::cli) verify: bool,
}

#[derive(Debug, Subcommand)]
#[clap(propagate_version = true)]
pub(in crate::cli) enum SubCommand {
    /// Display the MSRV's of dependencies
    List(ListOpts),
    /// Set the MSRV of the current crate to a given Rust version
    Set(SetOpts),
    /// Show the MSRV of your crate, as specified in the Cargo manifest
    Show,
    /// Verify whether the MSRV is satisfiable. The MSRV must be specified using the
    /// 'package.rust-version' or 'package.metadata.msrv' key in the Cargo.toml manifest.
    Verify(VerifyOpts),
}

#[derive(Debug, Args)]
#[clap(next_help_heading = "LIST OPTIONS", setting = AppSettings::DeriveDisplayOrder)]
pub(in crate::cli) struct ListOpts {
    /// Display the MSRV's of crates that your crate depends on
    #[clap(long, possible_values = ListMsrvVariant::variants(), default_value_t)]
    variant: ListMsrvVariant,
}

#[derive(Debug, Args)]
#[clap(next_help_heading = "SET OPTIONS", setting = AppSettings::DeriveDisplayOrder)]
pub(in crate::cli) struct SetOpts {
    /// The version to be set as MSRV
    ///
    /// The given version must be a two- or three component Rust version number.
    /// MSRV values prior to Rust 1.56 will be written to the `package.metadata.msrv` field
    /// in the Cargo manifest. MSRV's greater or equal to 1.56 will be written to
    /// `package.rust-version` in the Cargo manifest.
    #[clap(value_name = "MSRV")]
    msrv: BareVersion,
}

#[derive(Debug, Args)]
#[clap(
    next_help_heading = "VERIFY OPTIONS",
    setting = AppSettings::DeriveDisplayOrder,
)]
pub(in crate::cli) struct VerifyOpts {
    #[clap(flatten)]
    pub(in crate::cli) rust_releases_opts: RustReleasesOpts,

    #[clap(flatten)]
    pub(in crate::cli) toolchain_opts: ToolchainOpts,

    #[clap(flatten)]
    pub(in crate::cli) custom_check: CustomCheckOpts,
}

// Interpret the CLI config frontend as general Config
impl<'opts> TryFrom<&'opts CargoCli> for Config<'opts> {
    type Error = CargoMSRVError;

    fn try_from(cli: &'opts CargoCli) -> Result<Self, Self::Error> {
        (&cli.subcommand).try_into()
    }
}

// Interpret the CLI config frontend as general Config
impl<'opts> TryFrom<&'opts CargoMsrvCli> for Config<'opts> {
    type Error = CargoMSRVError;

    fn try_from(cli: &'opts CargoMsrvCli) -> Result<Self, Self::Error> {
        match cli {
            CargoMsrvCli::MSRV(opts) => opts.try_into(),
        }
    }
}

impl<'opts> TryFrom<&'opts CargoMsrvOpts> for Config<'opts> {
    type Error = CargoMSRVError;

    fn try_from(opts: &'opts CargoMsrvOpts) -> Result<Self, Self::Error> {
        let mode = make_mode(opts);
        let target = default_target()?;

        let mut builder = ConfigBuilder::new(mode, &target);

        builder = configurators::CustomCheckCommand::configure(builder, opts)?;
        builder = configurators::PathConfig::configure(builder, opts)?;
        builder = configurators::Target::configure(builder, opts)?;
        builder = configurators::MinVersion::configure(builder, opts)?;
        builder = configurators::MaxVersion::configure(builder, opts)?;
        builder = configurators::SearchMethodConfig::configure(builder, opts)?;
        builder = configurators::IncludeAllPatchReleases::configure(builder, opts)?;
        builder = configurators::OutputToolchainFile::configure(builder, opts)?;
        builder = configurators::IgnoreLockfile::configure(builder, opts)?;
        builder = configurators::UserOutput::configure(builder, opts)?;
        builder = configurators::ReleaseSource::configure(builder, opts)?;
        builder = configurators::Tracing::configure(builder, opts)?;
        builder = configurators::CheckFeedback::configure(builder, opts)?;
        builder = configurators::SubCommandConfigurator::configure(builder, opts)?;

        Ok(builder.build())
    }
}

fn make_mode(opts: &CargoMsrvOpts) -> ModeIntent {
    opts.subcommand
        .as_ref()
        .map(|subcommand| match subcommand {
            SubCommand::List(_) => ModeIntent::List,
            SubCommand::Show => ModeIntent::Show,
            SubCommand::Set(_) => ModeIntent::Set,
            SubCommand::Verify(_) => ModeIntent::Verify,
        })
        .unwrap_or_else(|| {
            if opts.verify {
                ModeIntent::Verify
            } else {
                ModeIntent::Find
            }
        })
}
