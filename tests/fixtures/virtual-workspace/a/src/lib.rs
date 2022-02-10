pub fn hello_from_a(name: &str) -> String {
	format!("Hello {}, I'm A", name)
}

#[cfg(test)]
mod tests {
	use super::hello_from_a;

    #[test]
    fn a_works() {
        let result = hello_from_a("Christopher");
        assert_eq!(result.as_str(), "Hello Christopher, I'm A");
    }
}
