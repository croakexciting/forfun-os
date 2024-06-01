pub fn set_bit(value: usize, pos: u32) -> usize {
    value | (1 << pos)
}

pub fn clear_bit(value: usize, pos: usize) -> usize {
    value & !(1 << pos)
}