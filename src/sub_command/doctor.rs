use crate::cli::rust_releases_opts::Edition;
use crate::context::DoctorContext;
use crate::error::TResult;
use crate::manifest::bare_version::BareVersion;
use crate::reporter::Reporter;
use crate::sub_command::verify::RustVersion;
use crate::SubCommand;

#[derive(Default)]
pub struct Doctor;

impl SubCommand for Doctor {
    type Context = DoctorContext;
    type Output = ();

    fn run(&self, _ctx: &Self::Context, _reporter: &impl Reporter) -> TResult<Self::Output> {
        todo!("Implement cargo msrv doctor!")
    }
}

struct IssueAnalyzer {
    msrv_cargo_toml: BareVersion,
    edition: Edition,
}

struct MsrvSource {
    cargo_toml: Option<BareVersion>,
    clippy_toml: Option<BareVersion>,
}
