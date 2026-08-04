use crate::context::EnvironmentContext;

#[derive(Debug)]
pub struct ShowContext {
    /// Resolved environment options
    pub environment: EnvironmentContext,
}
