fn main() {
    let mut vec = Vec::with_capacity(10);
    vec.extend([1]);
    vec.shrink_to(0);
}
