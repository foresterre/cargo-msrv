fn main() {
    let values: &[u8] = &[1, 2, 3];
    let _ = values.array_windows::<2>().count();
}
