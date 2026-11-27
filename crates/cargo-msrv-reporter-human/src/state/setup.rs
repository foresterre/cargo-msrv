use crate::draw_target::Printer;
use crate::state::{Done, Phase, Running, State, unexpected_event};
use crate::view::Human;
use cargo_msrv_reporter::Event;
use cargo_msrv_reporter::event::{Message, SubcommandId};

pub struct Setup {
    out: Printer,
}

impl Setup {
    pub fn new() -> Self {
        Self {
            out: Printer::new(),
        }
    }

    pub fn step(mut self, event: &Event) -> State {
        match event.message() {
            Message::Meta(it) => {
                self.out.line(Human(it));
                State::Setup(self)
            }
            Message::ContextFinalized(it) => {
                self.out.line(Human(it));
                State::Setup(self)
            }
            Message::SubcommandInit(it) => match it.subcommand_id() {
                SubcommandId::Find => State::Find(Phase::Running(Running::new(self.out))),
                SubcommandId::List => State::List(Running::new(self.out)),
                SubcommandId::Set => State::Set(Running::new(self.out)),
                SubcommandId::Show => State::Show(Running::new(self.out)),
                SubcommandId::Verify => State::Verify(Phase::Running(Running::new(self.out))),
            },
            Message::TerminateWithFailure(it) => {
                self.out.line(Human(it));
                State::Done(Done)
            }
            Message::FetchIndex(_)
            | Message::UnableToConfirmValidReleaseVersion(_)
            | Message::CheckPackage(_)
            | Message::CheckToolchain(_)
            | Message::SetupToolchain(_)
            | Message::CheckMethod(_)
            | Message::CheckResult(_)
            | Message::AuxiliaryOutput(_)
            | Message::FindMsrv(_)
            | Message::Progress(_)
            | Message::SubcommandResult(_) => State::Setup(unexpected_event(self, event)),
        }
    }
}
