use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Copy, Clone, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Scope {
    pub id: usize,
    pub marker: Marker,
}

impl Scope {
    pub fn new(id: usize, marker: Marker) -> Self {
        Self { id, marker }
    }

    /// Tests whether this is marked as the start of the scope or not.
    pub fn is_start(&self) -> bool {
        matches!(self.marker, Marker::Start)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Marker {
    Start,
    End,
}

pub trait SupplyScopeGenerator {
    type ScopeGen: ScopeGenerator;

    fn scope_generator(&self) -> &Self::ScopeGen;
}

/// Generator to generate a unique scope for a scoped event.
pub trait ScopeGenerator {
    /// Returns a pair of scope's with opposite markers. The first
    /// scope will mark the start of the scope, while the second scope
    /// will mark the end of the scope.
    ///
    /// The id of the scope data structures can be used to identify a scope.
    /// They will be identical for the returned pair.
    /// While the generated id must be unique for the programs' execution, no
    /// other guarantees are made about the id itself.
    fn generate(&self) -> (Scope, Scope);
}

/// A counter based scope generator.
pub struct ScopeCounter {
    counter: AtomicUsize,
}

impl ScopeCounter {
    pub const fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }
}

impl ScopeGenerator for ScopeCounter {
    fn generate(&self) -> (Scope, Scope) {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);

        (Scope::new(id, Marker::Start), Scope::new(id, Marker::End))
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct TestScopeGenerator;

#[cfg(test)]
impl ScopeGenerator for TestScopeGenerator {
    fn generate(&self) -> (Scope, Scope) {
        let id = 0;

        (Scope::new(id, Marker::Start), Scope::new(id, Marker::End))
    }
}

#[cfg(test)]
mod tests {
    use crate::reporter::event::scope::{ScopeCounter, ScopeGenerator};
    use crate::reporter::event::Marker;
    use std::sync::atomic::Ordering;

    #[test]
    fn unused() {
        let gen = ScopeCounter::new();
        assert_eq!(gen.counter.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn first_id() {
        let gen = ScopeCounter::new();
        let (start, end) = gen.generate();

        assert_eq!(start.id, 0);
        assert_eq!(end.id, 0);

        assert_eq!(start.marker, Marker::Start);
        assert_eq!(end.marker, Marker::End);
    }

    #[test]
    fn second_id() {
        let gen = ScopeCounter::new();
        let (start, end) = gen.generate();

        assert_eq!(start.id, 0);
        assert_eq!(end.id, 0);

        assert_eq!(start.marker, Marker::Start);
        assert_eq!(end.marker, Marker::End);

        let (start, end) = gen.generate();

        assert_eq!(start.id, 1);
        assert_eq!(end.id, 1);

        assert_eq!(start.marker, Marker::Start);
        assert_eq!(end.marker, Marker::End);
    }

    #[test]
    fn thousand() {
        let gen = ScopeCounter::new();

        for _ in 0..1000 {
            gen.generate();
        }

        let (start, end) = gen.generate();

        assert_eq!(start.id, 1000);
        assert_eq!(end.id, 1000);

        assert_eq!(start.marker, Marker::Start);
        assert_eq!(end.marker, Marker::End);
    }
}
