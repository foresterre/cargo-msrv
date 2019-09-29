use std::collections::VecDeque;

fn main() {
    let mut example: VecDeque<_> = (0..5).collect();
    example.rotate_left(4);
}
