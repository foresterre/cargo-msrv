use std::io;

// Alias trait for Write + Send
pub trait SendWriter: io::Write + Send {}

impl SendWriter for io::Stdout {}

impl SendWriter for io::Stderr {}

#[cfg(test)]
impl SendWriter for Vec<u8> {}
