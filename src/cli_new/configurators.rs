use crate::cli_new::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

mod check_feedback;
mod custom_check;
mod ignore_lockfile;
mod max_version;
mod min_version;
mod output_toolchain_file;
mod path;
mod release_source;
mod search_method;
mod search_space;
mod sub_command_configurator;
mod target;
mod tracing_configurator;
mod user_output;

pub(in crate::cli_new) use check_feedback::CheckFeedback;
pub(in crate::cli_new) use custom_check::CustomCheckCommand;
pub(in crate::cli_new) use ignore_lockfile::IgnoreLockfile;
pub(in crate::cli_new) use max_version::MaxVersion;
pub(in crate::cli_new) use min_version::MinVersion;
pub(in crate::cli_new) use output_toolchain_file::OutputToolchainFile;
pub(in crate::cli_new) use path::PathConfig;
pub(in crate::cli_new) use release_source::ReleaseSource;
pub(in crate::cli_new) use search_method::SearchMethodConfig;
pub(in crate::cli_new) use search_space::IncludeAllPatchReleases;
pub(in crate::cli_new) use sub_command_configurator::SubCommandConfigurator;
pub(in crate::cli_new) use target::Target;
pub(in crate::cli_new) use tracing_configurator::Tracing;
pub(in crate::cli_new) use user_output::UserOutput;

/// Used to turn the CLI front-end into a flattened Config.
///
/// Allows scaffolding of methods which otherwise would become one big pile of conditions,
/// all in one unified type signature.
///
/// Will probably be replaced as soon as we refactor Config into a layered config approach.
pub(in crate::cli_new) trait Configure {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>>;
}
