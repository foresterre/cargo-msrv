use crate::cli::{CargoMsrvOpts, SubCommand};
use cargo_msrv_context::context::error::{Error, TResult};
use cargo_msrv_context::context::set::SetTarget;
use cargo_msrv_context::{EnvironmentContext, Project, SetContext};
use std::convert::{TryFrom, TryInto};

impl TryFrom<CargoMsrvOpts> for SetContext {
    type Error = Error;

    fn try_from(opts: CargoMsrvOpts) -> TResult<Self> {
        let CargoMsrvOpts {
            shared_opts,
            subcommand,
            ..
        } = opts;

        let set_opts = match subcommand {
            SubCommand::Set(opts) => opts,
            _ => unreachable!("This should never happen. The subcommand is not `set`!"),
        };

        let environment: EnvironmentContext = (&shared_opts).try_into()?;

        let target = if set_opts.workspace_root {
            match &environment.project {
                Project::Cargo(project) => {
                    SetTarget::WorkspaceRoot(project.workspace_root.join("Cargo.toml"))
                }
                Project::Bare => {
                    return Err(Error::CargoFlagWithoutCargoProject {
                        flag: "--workspace-root",
                        root: environment.root_crate_path,
                    });
                }
            }
        } else {
            SetTarget::Packages
        };

        Ok(Self {
            msrv: set_opts.msrv,
            target,
            rust_releases: set_opts.rust_releases_opts.into(),
            environment,
        })
    }
}
