pub fn nanoseconds() -> usize {
    super::peri::timer::nanoseconds()
}

pub fn set_trigger() {
    super::peri::timer::set_trigger(100)
}