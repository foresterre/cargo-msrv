use crate::state::{Level, Running, Unexpected};
use crate::sub_command::CompatibilityChecks;
use crate::view::Human;
use cargo_msrv_context::SelectedPackage;
use cargo_msrv_reporter::SubcommandResult;
use cargo_msrv_reporter::event::CheckPackage;

pub struct Package<C> {
    run: Running<C>,
    package: SelectedPackage,
    id: usize,
    nth: u32,
    has_result: bool,
}

impl<C: CompatibilityChecks> Package<C> {
    pub fn enter(mut run: Running<C>, it: &CheckPackage, id: usize) -> Self {
        run.line(Human(it));

        Self {
            run,
            package: it.package.clone(),
            id,
            nth: 1,
            has_result: false,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn result(&mut self, result: &SubcommandResult) -> Result<(), Unexpected> {
        self.run.result(result)?;
        self.has_result = true;
        Ok(())
    }

    pub fn exit(self) -> Running<C> {
        if !self.has_result {
            tracing::warn!(package = %self.package.name, "package was checked without a result");
        }

        self.run
    }

    pub fn into_running(self) -> Running<C> {
        self.run
    }
}

impl<C: CompatibilityChecks> Level for Package<C> {
    type Cmd = C;

    fn run(&mut self) -> &mut Running<C> {
        &mut self.run
    }

    fn next_check(&mut self) -> u32 {
        let nth = self.nth;
        self.nth += 1;
        nth
    }
}
