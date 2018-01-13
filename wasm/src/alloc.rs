use std::mem;

#[no_mangle]
pub extern "C" fn alloc_byte_buffer(size: usize) -> *mut u8 {
    let mut buf: Vec<u8> = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr
}

pub extern "C" fn into_boxed_raw<T>(t: T) -> *mut T {
    let boxed = Box::new(t);
    Box::into_raw(boxed)
}
