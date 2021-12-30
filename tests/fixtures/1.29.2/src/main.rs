fn main() {
    let words = vec!["hello", "world"];

    // .flatten was stabilized in 1.29
    let concatenated: String = words.into_iter().map(str::chars).flatten().collect();

    assert_eq!(concatenated.as_str(), "helloworld")
}
