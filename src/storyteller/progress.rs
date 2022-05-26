//! TODO: experiment with these events, and having Progression use typestate

use std::marker::PhantomData;

/// Progression indicates how far we are
#[derive(serde::Serialize)]
pub struct Progression<S: ProgressionState> {
    #[serde(flatten)]
    progress: Progress, // TODO currently is `{"Progress":{"current":0,"total":100}}`, wanted: Progress -> progress (i.e. use snake_case), but rename or rename_all doesn't seem to work with flatten?
    #[serde(skip_serializing)]
    marker: PhantomData<S>,
}

#[derive(serde::Serialize)]
struct Progress {
    current: usize,
    total: usize,
}

// State
// -----

#[derive(serde::Serialize)]
pub struct Init;
#[derive(serde::Serialize)]
pub struct Add;
#[derive(serde::Serialize)]
pub struct Complete;

pub trait ProgressionState: serde::Serialize {}

impl ProgressionState for Init {}
impl ProgressionState for Add {}
impl ProgressionState for Complete {}

impl Add {
    fn add<S: ProgressionState>(
        progress: Progression<S>,
        additional_value: usize,
    ) -> Progression<Self> {
        let current = progress.current();
        let total = progress.total();
        let added = current + additional_value;

        // Do not scale beyond the max (i.e. beyond 'total')
        let updated_current = usize::min(added, total);

        // return the new progress
        Progression {
            progress: Progress {
                current: updated_current,
                total,
            },
            marker: PhantomData,
        }
    }
}

impl Complete {
    pub fn completed<S: ProgressionState>(progress: Progression<S>) -> Progression<Complete> {
        let total = progress.total();

        Progression {
            progress: Progress {
                current: total,
                total,
            },
            marker: PhantomData,
        }
    }
}

// Getters
// -------

impl<T: ProgressionState> Progression<T> {
    pub fn current(&self) -> usize {
        self.progress.current
    }

    pub fn total(&self) -> usize {
        self.progress.total
    }
}

// Transitions
// -----------

impl Progression<Init> {
    /// Start
    pub fn init(total: usize) -> Self {
        Self {
            progress: Progress { current: 0, total },
            marker: PhantomData,
        }
    }
}

impl Progression<Init> {
    pub fn add(self, progress: usize) -> Progression<Add> {
        Add::add(self, progress)
    }

    pub fn complete(self) -> Progression<Complete> {
        Complete::completed(self)
    }
}

impl Progression<Add> {
    pub fn add(self, progress: usize) -> Progression<Add> {
        Add::add(self, progress)
    }

    pub fn reset(self) -> Progression<Init> {
        Progression::init(self.total())
    }

    pub fn complete(self) -> Progression<Complete> {
        Complete::completed(self)
    }
}

impl Progression<Complete> {
    pub fn reset(self) -> Progression<Init> {
        Progression::init(self.total())
    }
}

#[cfg(test)]
mod tests {
    use crate::storyteller::progress::Progression;

    #[test]
    fn usage() {
        let progression = Progression::init(10);
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
                let p = Progression::init(2);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 2);

                let p = p.add(0);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 2);
            }

            #[test]
            fn add_identity_plus_one() {
                let p = Progression::init(2);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 2);

                let p = p.add(1);
                assert_eq!(p.current(), 1);
                assert_eq!(p.total(), 2);
            }

            #[test]
            fn dont_go_beyond_total() {
                let p = Progression::init(2);
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
                let p = Progression::init(2);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 2);

                let p = p.complete();
                assert_eq!(p.current(), 2);
                assert_eq!(p.total(), 2);
            }

            #[test]
            fn complete_from_identity_plus_one() {
                let p = Progression::init(2);
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
                let p = Progression::init(0);
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
            let p = Progression::init(2);
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
            let p = Progression::init(2);
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
            use super::Progression;

            #[test]
            fn init() {
                let p = Progression::init(100);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 100);
            }
        }

        mod init_transitions {
            use super::Progression;

            #[test]
            fn add() {
                let p = Progression::init(100);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 100);

                let p = p.add(10);
                assert_eq!(p.current(), 10);
                assert_eq!(p.total(), 100);
            }

            #[test]
            fn complete() {
                let p = Progression::init(100);
                assert_eq!(p.current(), 0);
                assert_eq!(p.total(), 100);

                let p = p.complete();
                assert_eq!(p.current(), 100);
                assert_eq!(p.total(), 100);
            }
        }

        mod add_transitions {
            use super::Progression;

            #[test]
            fn add() {
                let p = Progression::init(100);
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
                let p = Progression::init(100);
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
                let p = Progression::init(100);
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
            use super::Progression;

            #[test]
            fn reset() {
                let p = Progression::init(100);
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
