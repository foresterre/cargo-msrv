use std::io;

pub trait SendWriter: io::Write + Send + 'static {}

impl SendWriter for io::Stderr {}

#[cfg(test)]
impl SendWriter for Vec<u8> {}
