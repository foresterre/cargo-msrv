mod discard_output;
mod human;
mod json;
mod minimal;

#[cfg(test)]
mod testing;

pub use discard_output::DiscardOutputHandler;
pub use human::HumanProgressHandler;
pub use json::JsonHandler;
pub use minimal::MinimalOutputHandler;

#[cfg(test)]
pub use testing::TestingHandler;
