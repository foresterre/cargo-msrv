mod discard_output;
mod human;
mod json;
mod minimal;

#[cfg(any(test, feature = "testing"))]
mod testing;

pub use discard_output::DiscardOutputHandler;
pub use human::HumanProgressHandler;
pub use json::JsonHandler;
pub use minimal::MinimalOutputHandler;

#[cfg(any(test, feature = "testing"))]
pub use testing::TestingHandler;
