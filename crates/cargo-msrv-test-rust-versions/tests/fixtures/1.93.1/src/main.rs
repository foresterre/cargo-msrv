fn main() {
    let values: &[u8] = &[1, 2];
    let _: Option<&[u8; 2]> = values.as_array();
}
