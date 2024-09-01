use super::inner::context;

#[derive(Clone)]
#[repr(C)]
pub struct TrapContext {
    // 通用的 trap 上下文，用长度为 34 的数组表示，这个地方就体现了 usize 的优势了
    pub x: [usize; 34],
}

impl TrapContext {
    pub fn new(entry: usize) -> Self {
        Self { x: context::create_ctx(entry) }
    }
}