use crate::check::RunCommand;
use crate::cli::CargoMsrvOpts;
use crate::command::cargo_command::CargoCommand;
use crate::context::{
    CheckCommandContext, EnvironmentContext, RustReleasesContext, SearchMethod, ToolchainContext,
    UserOutputContext,
};
use crate::error::CargoMSRVError;
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub struct FindContext {
    /// Use a binary (bisect) or linear search to find the MSRV
    pub search_method: SearchMethod,

    /// Write the toolchain file if the MSRV is found
    pub write_toolchain_file: bool,

    /// Ignore the lockfile for the MSRV search
    pub ignore_lockfile: bool,

    /// Don't print the result of compatibility checks
    pub no_check_feedback: bool,

    /// Write the MSRV to the Cargo manifest
    pub write_msrv: bool,

    /// The context for Rust releases
    pub rust_releases: RustReleasesContext,

    /// The context for Rust toolchains
    pub toolchain: ToolchainContext,

    /// The context for checks to be used with rustup
    pub check_cmd: CheckCommandContext,

    /// Resolved environment options
    pub environment: EnvironmentContext,

    /// User output options
    pub user_output: UserOutputContext,
}

impl TryFrom<CargoMsrvOpts> for FindContext {
    type Error = CargoMSRVError;

    fn try_from(opts: CargoMsrvOpts) -> Result<Self, Self::Error> {
        let CargoMsrvOpts {
            find_opts,
            shared_opts,
            ..
        } = opts;

        let toolchain = find_opts.toolchain_opts.try_into()?;
        let environment = (&shared_opts).try_into()?;

        Ok(Self {
            search_method: if find_opts.linear {
                SearchMethod::Linear
            } else {
                SearchMethod::Bisect
            },
            write_toolchain_file: find_opts.write_toolchain_file,
            ignore_lockfile: find_opts.ignore_lockfile,
            no_check_feedback: find_opts.no_check_feedback,
            write_msrv: find_opts.write_msrv,
            rust_releases: find_opts.rust_releases_opts.into(),
            toolchain,
            check_cmd: find_opts.custom_check_opts.into(),
            environment,
            user_output: shared_opts.user_output_opts.into(),
        })
    }
}

impl FindContext {
    pub fn run_command(&self) -> RunCommand {
        if let Some(custom) = &self.check_cmd.rustup_command {
            RunCommand::custom(custom.clone())
        } else {
            let cargo_command = CargoCommand::default()
                .target(Some(self.toolchain.target.clone()))
                .features(self.check_cmd.cargo_features.clone())
                .all_features(self.check_cmd.cargo_all_features)
                .no_default_features(self.check_cmd.cargo_no_default_features);

            RunCommand::default(cargo_command)
        }
    }
}
