use crate::state::{Done, Next, Running, Unexpected, unexpected_event};
use crate::sub_command::CompatibilityChecks;
use cargo_msrv_reporter::event::{CheckPackage, CheckToolchain, Message};
use cargo_msrv_reporter::{Event, Marker, Scope};

pub use check::IsCompatibleView;
pub use package::Package;

mod check;
mod package;

pub trait Level {
    type Cmd: CompatibilityChecks;

    fn run(&mut self) -> &mut Running<Self::Cmd>;

    fn next_check(&mut self) -> u32;
}

pub enum Phase<C> {
    Running(Running<C>),
    Check(IsCompatibleView<Running<C>>),
    Package(Package<C>),
    PackageCheck(IsCompatibleView<Package<C>>),
}

impl<C: CompatibilityChecks> Phase<C> {
    pub fn step(mut self, event: &Event) -> Next<Self> {
        match event.message() {
            Message::FetchIndex(_) => {
                self.run().plain_scope(event, "fetch index");
                Next::Continue(self)
            }
            Message::FindMsrv(_) => {
                self.run().plain_scope(event, "find msrv");
                Next::Continue(self)
            }
            Message::SetupToolchain(_) => {
                self.run().plain_scope(event, "setup toolchain");
                Next::Continue(self)
            }
            Message::CheckPackage(it) => Next::Continue(self.check_package(it, event)),
            Message::CheckToolchain(it) => Next::Continue(self.check_toolchain(it, event)),
            Message::CheckResult(it) => Next::Continue(match self {
                Phase::Check(mut check) => {
                    check.result(it);
                    Phase::Check(check)
                }
                Phase::PackageCheck(mut check) => {
                    check.result(it);
                    Phase::PackageCheck(check)
                }
                phase => unexpected_event(phase, event),
            }),
            Message::SubcommandResult(result) => Next::Continue(match self {
                Phase::Running(mut run) => match run.result(result) {
                    Ok(()) => Phase::Running(run),
                    Err(Unexpected) => unexpected_event(Phase::Running(run), event),
                },
                Phase::Package(mut package) => match package.result(result) {
                    Ok(()) => Phase::Package(package),
                    Err(Unexpected) => unexpected_event(Phase::Package(package), event),
                },
                phase => unexpected_event(phase, event),
            }),
            Message::CheckMethod(_) | Message::Progress(_) | Message::AuxiliaryOutput(_) => {
                Next::Continue(self)
            }
            Message::TerminateWithFailure(it) => Next::Done(self.unwind().fail(it)),
            Message::Meta(_)
            | Message::ContextFinalized(_)
            | Message::SubcommandInit(_)
            | Message::UnableToConfirmValidReleaseVersion(_) => {
                Next::Continue(unexpected_event(self, event))
            }
        }
    }

    pub fn finish(self) -> Done {
        self.unwind().finish()
    }

    fn check_package(self, it: &CheckPackage, event: &Event) -> Self {
        match (self, event.scope()) {
            (
                Phase::Running(run),
                Some(Scope {
                    id,
                    marker: Marker::Start,
                }),
            ) => Phase::Package(Package::enter(run, it, id)),
            (
                Phase::Package(package),
                Some(Scope {
                    id,
                    marker: Marker::End,
                }),
            ) if package.id() == id => Phase::Running(package.exit()),
            (phase, _) => unexpected_event(phase, event),
        }
    }

    fn check_toolchain(self, it: &CheckToolchain, event: &Event) -> Self {
        match (self, event.scope()) {
            (
                Phase::Running(run),
                Some(Scope {
                    id,
                    marker: Marker::Start,
                }),
            ) => Phase::Check(IsCompatibleView::enter(run, it, id)),
            (
                Phase::Package(package),
                Some(Scope {
                    id,
                    marker: Marker::Start,
                }),
            ) => Phase::PackageCheck(IsCompatibleView::enter(package, it, id)),
            (
                Phase::Check(check),
                Some(Scope {
                    id,
                    marker: Marker::End,
                }),
            ) if check.id() == id => Phase::Running(check.exit()),
            (
                Phase::PackageCheck(check),
                Some(Scope {
                    id,
                    marker: Marker::End,
                }),
            ) if check.id() == id => Phase::Package(check.exit()),
            (phase, _) => unexpected_event(phase, event),
        }
    }

    fn run(&mut self) -> &mut Running<C> {
        match self {
            Phase::Running(run) => run,
            Phase::Check(check) => check.run(),
            Phase::Package(package) => package.run(),
            Phase::PackageCheck(check) => check.run(),
        }
    }

    fn unwind(self) -> Running<C> {
        if !matches!(self, Phase::Running(_)) {
            tracing::warn!("closing package or check scopes which were never closed");
        }

        match self {
            Phase::Running(run) => run,
            Phase::Check(check) => check.exit(),
            Phase::Package(package) => package.into_running(),
            Phase::PackageCheck(check) => check.exit().into_running(),
        }
    }
}
