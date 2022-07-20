use crate::cli::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

mod check_feedback;
mod crate_path;
mod custom_check;
mod ignore_lockfile;
mod manifest_path;
mod max_version;
mod min_version;
mod output_toolchain_file;
mod release_source;
mod search_method;
mod search_space;
mod sub_command_configurator;
mod target;
mod tracing_configurator;
mod user_output;
mod write_msrv;

pub(in crate::cli) use check_feedback::CheckFeedback;
pub(in crate::cli) use crate_path::CratePathConfig;
pub(in crate::cli) use custom_check::CustomCheckCommand;
pub(in crate::cli) use ignore_lockfile::IgnoreLockfile;
pub(in crate::cli) use manifest_path::ManifestPathConfig;
pub(in crate::cli) use max_version::MaxVersion;
pub(in crate::cli) use min_version::MinVersion;
pub(in crate::cli) use output_toolchain_file::OutputToolchainFile;
pub(in crate::cli) use release_source::ReleaseSource;
pub(in crate::cli) use search_method::SearchMethodConfig;
pub(in crate::cli) use search_space::IncludeAllPatchReleases;
pub(in crate::cli) use sub_command_configurator::SubCommandConfigurator;
pub(in crate::cli) use target::Target;
pub(in crate::cli) use tracing_configurator::Tracing;
pub(in crate::cli) use user_output::UserOutput;
pub(in crate::cli) use write_msrv::WriteMsrv;

/// Used to turn the CLI front-end into a flattened Config.
///
/// Allows scaffolding of methods which otherwise would become one big pile of conditions,
/// all in one unified type signature.
///
/// Will probably be replaced as soon as we refactor Config into a layered config approach.
pub(in crate::cli) trait Configure {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>>;
}
