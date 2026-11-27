use crate::draw_target::Printer;
use crate::state::{Done, Level, Next, ScopeStack, Summary, Unexpected, unexpected_event};
use crate::sub_command::{CompatibilityChecks, ResultOnly, SubCommand};
use crate::view::Human;
use cargo_msrv_reporter::event::{Message, SubcommandResult};
use cargo_msrv_reporter::{Event, Marker, Scope, TerminateWithFailure};
use std::fmt::Display;
use std::marker::PhantomData;

pub struct Running<C> {
    out: Printer,
    scopes: ScopeStack,
    summary: Summary,
    nth: u32,
    command: PhantomData<C>,
}

impl<C: SubCommand> Running<C> {
    pub fn new(mut out: Printer) -> Self {
        if C::SPINNER {
            out.idle_spinner();
        }

        Self {
            out,
            scopes: ScopeStack::new(),
            summary: Summary::new(),
            nth: 1,
            command: PhantomData,
        }
    }

    pub fn result(&mut self, result: &SubcommandResult) -> Result<(), Unexpected> {
        C::result(self, result)
    }

    pub fn finish(self) -> Done {
        self.print_summary();
        Done
    }

    pub fn fail(self, termination: &TerminateWithFailure) -> Done {
        let mut out = self.print_summary();
        out.line(Human(termination));
        Done
    }

    fn print_summary(self) -> Printer {
        let Self {
            mut out, summary, ..
        } = self;

        if summary.is_shown() {
            out.line(Human(&summary));
        }

        out
    }
}

impl<C> Running<C> {
    pub fn line(&mut self, message: impl Display) {
        self.out.line(message);
    }

    pub fn printer(&mut self) -> &mut Printer {
        &mut self.out
    }

    pub fn scopes(&mut self) -> &mut ScopeStack {
        &mut self.scopes
    }

    pub fn summary(&mut self) -> &mut Summary {
        &mut self.summary
    }

    pub fn plain_scope(&mut self, event: &Event, label: &'static str) {
        match event.scope() {
            Some(Scope {
                id,
                marker: Marker::Start,
            }) => self.scopes.push(id, label),
            Some(Scope {
                id,
                marker: Marker::End,
            }) => self.scopes.pop(id),
            None => tracing::warn!(label, "expected a scoped event"),
        }
    }
}

impl<C: CompatibilityChecks> Level for Running<C> {
    type Cmd = C;

    fn run(&mut self) -> &mut Running<C> {
        self
    }

    fn next_check(&mut self) -> u32 {
        let nth = self.nth;
        self.nth += 1;
        nth
    }
}

impl<C: ResultOnly> Running<C> {
    pub fn step(mut self, event: &Event) -> Next<Self> {
        match event.message() {
            Message::FetchIndex(_) => {
                self.plain_scope(event, "fetch index");
                Next::Continue(self)
            }
            Message::UnableToConfirmValidReleaseVersion(it) => {
                self.line(Human(it));
                Next::Continue(self)
            }
            Message::AuxiliaryOutput(_) => Next::Continue(self),
            Message::SubcommandResult(result) => match self.result(result) {
                Ok(()) => Next::Continue(self),
                Err(Unexpected) => Next::Continue(unexpected_event(self, event)),
            },
            Message::TerminateWithFailure(it) => Next::Done(self.fail(it)),
            Message::Meta(_)
            | Message::ContextFinalized(_)
            | Message::SubcommandInit(_)
            | Message::CheckPackage(_)
            | Message::CheckToolchain(_)
            | Message::SetupToolchain(_)
            | Message::CheckMethod(_)
            | Message::CheckResult(_)
            | Message::FindMsrv(_)
            | Message::Progress(_) => Next::Continue(unexpected_event(self, event)),
        }
    }
}
