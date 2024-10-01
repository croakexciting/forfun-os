pub fn nanoseconds() -> usize {
    super::inner::timer::nanoseconds()
}

pub fn set_trigger() {
    super::inner::timer::set_trigger(100)
}