use crate::cli::configurators::Configure;
use crate::cli::custom_check_opts::CustomCheckOpts;
use crate::cli::find_opts::FindOpts;
use crate::cli::rust_releases_opts::RustReleasesOpts;
use crate::cli::shared_opts::SharedOpts;
use crate::cli::toolchain_opts::ToolchainOpts;
use crate::config::list::ListMsrvVariant;
use crate::config::{ConfigBuilder, ForwardedSubcommandId};
use crate::default_target::default_target;
use crate::manifest::bare_version::BareVersion;
use crate::{CargoMSRVError, Config, SubcommandId};
use clap::{Args, Parser, Subcommand};
use std::convert::{TryFrom, TryInto};
use std::ffi::{OsStr, OsString};

pub(in crate::cli) mod configurators;
pub(crate) mod custom_check_opts;
pub(crate) mod find_opts;
pub(crate) mod rust_releases_opts;
pub(crate) mod shared_opts;
pub(crate) mod toolchain_opts;

#[derive(Debug, Parser)]
#[command(version, bin_name = "cargo", max_term_width = 120)]
pub struct CargoCli {
    #[command(subcommand)]
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
    #[command(
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
    Msrv(CargoMsrvOpts),
}

#[derive(Debug, Args)]
#[command(version)]
pub(in crate::cli) struct CargoMsrvOpts {
    #[command(flatten)]
    pub(in crate::cli) find_opts: FindOpts,

    #[command(flatten)]
    pub(in crate::cli) shared_opts: SharedOpts,

    #[command(subcommand)]
    pub(in crate::cli) subcommand: Option<SubCommand>,

    /// DEPRECATED: Use the `cargo msrv verify` subcommand instead
    #[arg(long, global = false, hide = true)]
    pub(in crate::cli) verify: bool,
}

#[derive(Debug, Subcommand)]
#[command(propagate_version = true)]
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

    /// Subcommands which are ran with the MSRV toolchain and forwarded to cargo
    #[command(flatten)]
    CargoSubcommand(CargoSubcommand),
}

#[derive(Debug, Args)]
#[command(next_help_heading = "List options")]
pub(in crate::cli) struct ListOpts {
    /// Display the MSRV's of crates that your crate depends on
    #[arg(long, value_enum, default_value_t)]
    variant: ListMsrvVariant,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Set options")]
pub(in crate::cli) struct SetOpts {
    /// The version to be set as MSRV
    ///
    /// The given version must be a two- or three component Rust version number.
    /// MSRV values prior to Rust 1.56 will be written to the `package.metadata.msrv` field
    /// in the Cargo manifest. MSRV's greater or equal to 1.56 will be written to
    /// `package.rust-version` in the Cargo manifest.
    #[arg(value_name = "MSRV")]
    msrv: BareVersion,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Verify options")]
pub(in crate::cli) struct VerifyOpts {
    #[command(flatten)]
    pub(in crate::cli) rust_releases_opts: RustReleasesOpts,

    #[command(flatten)]
    pub(in crate::cli) toolchain_opts: ToolchainOpts,

    #[command(flatten)]
    pub(in crate::cli) custom_check: CustomCheckOpts,

    /// The Rust version, to check against for toolchain compatibility
    ///
    /// If not set, the MSRV will be parsed from the Cargo manifest instead.
    #[arg(long, value_name = "rust-version")]
    rust_version: Option<BareVersion>,
}

#[derive(Debug, Subcommand)]
#[command(next_help_heading = "Cargo Subcommands")]
enum CargoSubcommand {
    /// Run cargo build with the MSRV toolchain
    Build,
    /// Run cargo check with the MSRV toolchain
    Check,
    /// Run cargo test with the MSRV toolchain
    Test,
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
            CargoMsrvCli::Msrv(opts) => opts.try_into(),
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
        builder = configurators::ManifestPathConfig::configure(builder, opts)?;
        builder = configurators::Target::configure(builder, opts)?;
        builder = configurators::MinVersion::configure(builder, opts)?;
        builder = configurators::MaxVersion::configure(builder, opts)?;
        builder = configurators::SearchMethodConfig::configure(builder, opts)?;
        builder = configurators::IncludeAllPatchReleases::configure(builder, opts)?;
        builder = configurators::OutputToolchainFile::configure(builder, opts)?;
        builder = configurators::WriteMsrv::configure(builder, opts)?;
        builder = configurators::IgnoreLockfile::configure(builder, opts)?;
        builder = configurators::UserOutput::configure(builder, opts)?;
        builder = configurators::ReleaseSource::configure(builder, opts)?;
        builder = configurators::Tracing::configure(builder, opts)?;
        builder = configurators::CheckFeedback::configure(builder, opts)?;
        builder = configurators::SubCommandConfigurator::configure(builder, opts)?;

        Ok(builder.build())
    }
}

fn make_mode(opts: &CargoMsrvOpts) -> SubcommandId {
    opts.subcommand
        .as_ref()
        .map(|subcommand| match subcommand {
            SubCommand::List(_) => SubcommandId::List,
            SubCommand::Show => SubcommandId::Show,
            SubCommand::Set(_) => SubcommandId::Set,
            SubCommand::Verify(_) => SubcommandId::Verify,
            SubCommand::CargoSubcommand(subcommand) => match subcommand {
                CargoSubcommand::Build => SubcommandId::Forward(ForwardedSubcommandId::Build),
                CargoSubcommand::Check => SubcommandId::Forward(ForwardedSubcommandId::Check),
                CargoSubcommand::Test => SubcommandId::Forward(ForwardedSubcommandId::Test),
            },
        })
        .unwrap_or_else(|| {
            if opts.verify {
                SubcommandId::Verify
            } else {
                SubcommandId::Find
            }
        })
}

#[cfg(test)]
mod tests {
    use super::CargoCli;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        CargoCli::command().debug_assert();
    }
}
