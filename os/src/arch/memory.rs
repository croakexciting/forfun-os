use alloc::vec::Vec;

pub unsafe fn copy_with_user(to: *mut u8, from: *const u8, n: usize) {
    super::inner::memory::copy_with_user(to, from, n)
}

pub fn copy_from_user_into_vector(from: *const u8, n: usize) -> Vec<u8> {
    let mut vec = Vec::new();
    vec.resize(n, 0);
    unsafe { copy_with_user(vec.as_mut_ptr(), from, n) }
    vec
}

pub fn copy_vector_to_user(v: Vec<u8>, dst: *mut u8) -> usize {
    unsafe { copy_with_user(dst, v.as_ptr(), v.len()) }
    v.len()
}

pub fn copy_user_page_to_vector(vpn: VirtPage) -> Vec<u8> {
    let src = vpn.bytes_array().as_ptr();
    copy_from_user_into_vector(src, 4096)
}

pub fn copy_vector_to_user_page(v: Vec<u8>, vpn: VirtPage) {
    let dst = vpn.bytes_array().as_mut_ptr();
    copy_vector_to_user(v, dst);
}
