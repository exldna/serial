use crate::raw_vec::RawVec;
use std::ptr;

pub struct Vec<T> {
    buf: RawVec<T>,
    len: usize,
}

impl<T> Vec<T> {
    pub const fn new() -> Self {
        Self {
            buf: RawVec::new(),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: RawVec::with_capacity(capacity),
            len: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// # Safety
    ///  len must be less than capacity, and data must be allocated.
    pub unsafe fn set_len(&mut self, len: usize) {
        self.len = len
    }

    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    pub fn as_ptr(&self) -> *const T {
        self.buf.ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.buf.ptr()
    }

    pub fn as_slice(&self) -> &[T] {
        self
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Vec<T> {
        Vec::new()
    }
}

impl<T> Vec<T> {
    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(self.len, additional);
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.buf.capacity() {
            self.buf.reserve_to_push(self.len);
        }
        unsafe {
            let end = self.as_mut_ptr().add(self.len);
            ptr::write(end, value);
            self.len += 1;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.as_ptr().add(self.len))) }
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        #[cold]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("insertion index (is {index}) should be <= len (is {len})");
        }

        if index > self.len {
            assert_failed(index, self.len);
        }

        // space for the new element
        if self.len == self.buf.capacity() {
            self.buf.reserve_to_push(self.len);
        }

        unsafe {
            // infallible
            // The spot to put the new value
            {
                let p = self.as_mut_ptr().add(index);
                if index < self.len {
                    // Shift everything over to make space. (Duplicating the
                    // `index`th element into two consecutive places.)
                    ptr::copy(p, p.add(1), self.len - index);
                }
                // Write it in, overwriting the first copy of the `index`th
                // element.
                ptr::write(p, value);
            }
            self.len += 1;
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        #[cold]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("swap_remove index (is {index}) should be < len (is {len})");
        }

        if index >= self.len {
            assert_failed(index, self.len);
        }
        unsafe {
            // We replace self[index] with the last element. Note that if the
            // bounds check above succeeds there must be a last element (which
            // can be self[index] itself).
            self.len -= 1;
            let value = ptr::read(self.as_ptr().add(index));
            let base_ptr = self.as_mut_ptr();
            ptr::copy(base_ptr.add(self.len), base_ptr.add(index), 1);
            value
        }
    }

    pub fn append(&mut self, other: &mut Self) {
        unsafe {
            self.append_elements(other.as_slice() as _);
            other.set_len(0);
        }
    }

    unsafe fn append_elements(&mut self, other: *const [T]) {
        let count = unsafe { (*other).len() };
        self.reserve(count);
        unsafe {
            let end = self.as_mut_ptr().add(self.len);
            ptr::copy_nonoverlapping(other as *const T, end, count)
        };
        self.len += count;
    }
}

unsafe impl<#[may_dangle] T> Drop for Vec<T> {
    fn drop(&mut self) {
        unsafe {
            // use drop for [T]
            // use a raw slice to refer to the elements of the vector as weakest necessary type;
            // could avoid questions of validity in certain cases
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len))
        }
        // RawVec handles deallocation
    }
}

impl<T> std::ops::Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len) }
    }
}

impl<T> std::ops::DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }
}
