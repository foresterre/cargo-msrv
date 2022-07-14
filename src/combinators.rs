pub trait ThenSome {
    fn then_some<T>(self, t: T) -> Option<T>;
}

impl ThenSome for bool {
    fn then_some<T>(self, t: T) -> Option<T> {
        if self {
            Some(t)
        } else {
            None
        }
    }
}
