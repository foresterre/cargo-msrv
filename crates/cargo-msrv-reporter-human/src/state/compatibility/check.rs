use crate::draw_target::Ticking;
use crate::state::{Level, Running};
use crate::view::{CheckHeader, Human};
use cargo_msrv_reporter::event::{CheckResult, CheckToolchain};

pub struct IsCompatibleView<P> {
    parent: P,
    id: usize,
    depth: usize,
    ticking: Ticking,
}

impl<P: Level> IsCompatibleView<P> {
    pub fn enter(mut parent: P, it: &CheckToolchain, id: usize) -> Self {
        let nth = parent.next_check();
        let run = parent.run();

        run.line(CheckHeader { check: it, nth });
        let ticking = run
            .printer()
            .start(format!("Rust {}", it.toolchain.version()));
        let depth = run.scopes().depth();

        Self {
            parent,
            id,
            depth,
            ticking,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn run(&mut self) -> &mut Running<P::Cmd> {
        self.parent.run()
    }

    pub fn result(&mut self, it: &CheckResult) {
        self.run().line(Human(it));
    }

    pub fn exit(self) -> P {
        let Self {
            mut parent,
            depth,
            ticking,
            ..
        } = self;

        let run = parent.run();
        run.scopes().close_to(depth);
        run.printer().stop(ticking);

        parent
    }
}
