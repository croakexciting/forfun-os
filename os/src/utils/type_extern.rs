use core::cell::{RefCell, RefMut};

pub struct RefCellWrap<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for RefCellWrap<T> {}

impl<T> RefCellWrap<T> {
    pub unsafe fn new(v: T) -> Self {
        Self {
            inner: RefCell::new(v),
        }
    }

    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}