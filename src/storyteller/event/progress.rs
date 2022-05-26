use serde::ser::SerializeStruct;
use serde::Serializer;

/// Progression indicates how far we are
pub struct Progression {
    state: State,
}

impl Progression {
    pub fn new(total: usize) -> Self {
        Init::init(total).into()
    }
}

impl From<Init> for Progression {
    fn from(instance: Init) -> Self {
        Self {
            state: State::Init(instance),
        }
    }
}

impl From<Add> for Progression {
    fn from(instance: Add) -> Self {
        Self {
            state: State::Add(instance),
        }
    }
}

impl From<Complete> for Progression {
    fn from(instance: Complete) -> Self {
        Self {
            state: State::Complete(instance),
        }
    }
}

impl serde::Serialize for Progression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut progress = serializer.serialize_struct("progress", 2)?;

        progress.serialize_field("current", &self.current())?;
        progress.serialize_field("total", &self.total())?;

        progress.end()
    }
}

// State
// -----

#[derive(serde::Serialize)]
struct Progress {
    current: usize,
    total: usize,
}

enum State {
    Init(Init),
    Add(Add),
    Complete(Complete),
}

pub struct Init(Progress);
pub struct Add(Progress);
pub struct Complete {
    total: usize,
}

impl Add {
    fn _add(progress: Progress, added_value: usize) -> Add {
        let current = progress.current;
        let total = progress.total;
        let added = current + added_value;

        // Do not scale beyond the max (i.e. beyond 'total')
        let updated_current = usize::min(added, total);

        // return the new progress
        Add {
            0: Progress {
                current: updated_current,
                total,
            },
        }
    }
}

impl Complete {
    fn _completed(progress: Progress) -> Complete {
        let total = progress.total;

        Complete { total }
    }
}

// Getters
// -------

trait GetProgression {
    fn current(&self) -> usize;

    fn total(&self) -> usize;
}

impl GetProgression for Progression {
    fn current(&self) -> usize {
        match &self.state {
            State::Init(item) => item.current(),
            State::Add(item) => item.current(),
            State::Complete(item) => item.current(),
        }
    }

    fn total(&self) -> usize {
        match &self.state {
            State::Init(item) => item.total(),
            State::Add(item) => item.total(),
            State::Complete(item) => item.total(),
        }
    }
}

impl GetProgression for Init {
    fn current(&self) -> usize {
        self.0.current
    }

    fn total(&self) -> usize {
        self.0.total
    }
}
impl GetProgression for Add {
    fn current(&self) -> usize {
        self.0.current
    }

    fn total(&self) -> usize {
        self.0.total
    }
}

impl GetProgression for Complete {
    fn current(&self) -> usize {
        self.total
    }

    fn total(&self) -> usize {
        self.total
    }
}

// Transitions
// -----------

impl Init {
    /// Start
    pub fn init(total: usize) -> Init {
        Init {
            0: Progress { current: 0, total },
        }
    }
}

impl Init {
    pub fn add(self, value: usize) -> Add {
        Add::_add(self.0, value)
    }

    pub fn complete(self) -> Complete {
        Complete::_completed(self.0)
    }
}

impl Add {
    pub fn add(self, progress: usize) -> Add {
        Add::_add(self.0, progress)
    }

    pub fn reset(self) -> Init {
        Init::init(self.0.total)
    }

    pub fn complete(self) -> Complete {
        Complete::_completed(self.0)
    }
}

impl Complete {
    pub fn reset(self) -> Init {
        Init::init(self.total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage() {
        let progression = Init::init(10);
        assert_eq!(progression.current(), 0);
        assert_eq!(progression.total(), 10);

        let progression = progression.add(1);
        assert_eq!(progression.current(), 1);
        assert_eq!(progression.total(), 10);

        let progression = progression.add(3);
        assert_eq!(progression.current(), 4);
        assert_eq!(progression.total(), 10);

        let progression = progression.reset();
        assert_eq!(progression.current(), 0);
        assert_eq!(progression.total(), 10);

        let progression = progression.complete();
        assert_eq!(progression.current(), 10);
        assert_eq!(progression.total(), 10);
    }

    mod impls {
        use super::*;

        mod add {
            use super::*;

            #[test]
            fn add_identity() {
                let p = Init::init(2);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 2);

                let p = p.add(0);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 2);
            }

            #[test]
            fn add_identity_plus_one() {
                let p = Init::init(2);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 2);

                let p = p.add(1);
                assert_eq!(p.current(), 1);
                assert_eq!(p.total(), 2);
            }

            #[test]
            fn dont_go_beyond_total() {
                let p = Init::init(2);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 2);

                let p = p.add(3);
                assert_eq!(p.current(), 2);
                assert_eq!(p.total(), 2);
            }
        }

        mod complete {
            use super::*;

            #[test]
            fn complete_from_identity() {
                let p = Init::init(2);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 2);

                let p = p.complete();
                assert_eq!(p.current(), 2);
                assert_eq!(p.total(), 2);
            }

            #[test]
            fn complete_from_identity_plus_one() {
                let p = Init::init(2);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 2);

                let p = p.add(1);
                assert_eq!(p.current(), 1);
                assert_eq!(p.total(), 2);

                let p = p.complete();
                assert_eq!(p.current(), 2);
                assert_eq!(p.total(), 2);
            }

            #[test]
            fn complete_total_zero() {
                let p = Init::init(0);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 0);

                let p = p.complete();
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 0);
            }
        }
    }

    mod getters {
        use super::*;

        #[test]
        fn current() {
            let p = Init::init(2);
            assert_eq!(p.current(), 0);

            let p = p.add(1);
            assert_eq!(p.current(), 1);

            let p = p.complete();
            assert_eq!(p.current(), 2);

            let p = p.reset();
            assert_eq!(p.current(), 0);
        }

        #[test]
        fn total() {
            let p = Init::init(2);
            assert_eq!(p.total(), 2);

            let p = p.add(1);
            assert_eq!(p.total(), 2);

            let p = p.complete();
            assert_eq!(p.total(), 2);

            let p = p.reset();
            assert_eq!(p.total(), 2);
        }
    }

    mod transitions {
        use super::*;

        mod init {
            use super::{GetProgression, Init};

            #[test]
            fn init() {
                let p = Init::init(100);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 100);
            }
        }

        mod init_transitions {
            use super::{GetProgression, Init};

            #[test]
            fn add() {
                let p = Init::init(100);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 100);

                let p = p.add(10);
                assert_eq!(p.current(), 10);
                assert_eq!(p.total(), 100);
            }

            #[test]
            fn complete() {
                let p = Init::init(100);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 100);

                let p = p.complete();
                assert_eq!(p.current(), 100);
                assert_eq!(p.total(), 100);
            }
        }

        mod add_transitions {
            use super::{GetProgression, Init};

            #[test]
            fn add() {
                let p = Init::init(100);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 100);

                let p = p.add(10);
                assert_eq!(p.current(), 10);
                assert_eq!(p.total(), 100);

                let p = p.add(10);
                assert_eq!(p.current(), 20);
                assert_eq!(p.total(), 100);
            }

            #[test]
            fn reset() {
                let p = Init::init(100);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 100);

                let p = p.add(10);
                assert_eq!(p.current(), 10);
                assert_eq!(p.total(), 100);

                let p = p.reset();
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 100);
            }

            #[test]
            fn complete() {
                let p = Init::init(100);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 100);

                let p = p.add(10);
                assert_eq!(p.current(), 10);
                assert_eq!(p.total(), 100);

                let p = p.complete();
                assert_eq!(p.current(), 100);
                assert_eq!(p.total(), 100);
            }
        }

        mod complete_transitions {
            use super::{GetProgression, Init};

            #[test]
            fn reset() {
                let p = Init::init(100);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 100);

                let p = p.complete();
                assert_eq!(p.current(), 100);
                assert_eq!(p.total(), 100);

                let p = p.reset();
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 100);
            }
        }
    }
}
