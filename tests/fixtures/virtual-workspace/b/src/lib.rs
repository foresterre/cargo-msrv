pub fn hello_from_b(name: &str) -> String {
	format!("Hello {}, I'm B", name)
}

#[cfg(test)]
mod tests {
	use super::hello_from_b;

    #[test]
    fn b_works() {
        let result = hello_from_b("Christopher");
        assert_eq!(result.as_str(), "Hello Christopher, I'm B");
    }
}
