use std::alloc::{Allocator, Layout};
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;

use crate::try_reserve_error::handle_reserve;
use crate::TryReserveError;
use crate::TryReserveErrorKind::*;

pub(crate) struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
    _pd: PhantomData<T>,
}

// Direct API
impl<T> RawVec<T> {
    pub(crate) const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
            _pd: PhantomData,
        }
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        handle_reserve(Self::try_allocate(capacity))
    }

    pub(crate) fn capacity(&self) -> usize {
        if mem::size_of::<T>() == 0 {
            usize::MAX
        } else {
            self.cap
        }
    }

    pub(crate) fn ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

// Reserve API
impl<T> RawVec<T> {
    pub(crate) fn reserve(&mut self, len: usize, additional: usize) {
        #[cold]
        fn do_reserve_and_handle<T>(slf: &mut RawVec<T>, len: usize, additional: usize) {
            handle_reserve(slf.grow_amortized(len, additional));
        }

        if self.needs_to_grow(len, additional) {
            do_reserve_and_handle(self, len, additional);
        }
    }

    pub(crate) fn reserve_to_push(&mut self, len: usize) {
        handle_reserve(self.grow_amortized(len, 1));
    }
}

// Growing
impl<T> RawVec<T> {
    const MIN_NON_ZERO_CAP: usize = if mem::size_of::<T>() == 1 {
        8
    } else if mem::size_of::<T>() <= 1024 {
        4
    } else {
        1
    };

    fn needs_to_grow(&self, len: usize, additional: usize) -> bool {
        additional > self.capacity().wrapping_sub(len)
    }

    fn current_memory(&self) -> Option<(NonNull<u8>, Layout)> {
        if mem::size_of::<T>() == 0 || self.cap == 0 {
            None
        } else {
            // We could use Layout::array here which ensures the absence of isize and usize overflows
            // and could hypothetically handle differences between stride and size, but this memory
            // has already been allocated, so we know it can't overflow and currently rust does not
            // support such types. So we can do better by skipping some checks and avoid an unwrap.
            assert_eq!(mem::size_of::<T>() % mem::align_of::<T>(), 0);
            unsafe {
                let align = mem::align_of::<T>();
                let size = mem::size_of::<T>() * self.cap;
                let layout = Layout::from_size_align_unchecked(size, align);
                Some((self.ptr.cast(), layout))
            }
        }
    }

    unsafe fn set_ptr_and_cap(&mut self, ptr: NonNull<[u8]>, cap: usize) {
        // Allocators currently return a `NonNull<[u8]>` whose length matches
        // the size requested. If that ever changes, the capacity here should
        // change to `ptr.len() / mem::size_of::<T>()`.
        self.ptr = unsafe { NonNull::new_unchecked(ptr.cast().as_ptr()) };
        self.cap = cap;
    }

    fn try_allocate(capacity: usize) -> Result<Self, TryReserveError> {
        if mem::size_of::<T>() == 0 {
            return Ok(Self::new());
        }

        let layout = match Layout::array::<T>(capacity) {
            Ok(layout) => layout,
            Err(_) => return Err(CapacityOverflow.into()),
        };

        alloc_guard(layout.size())?;

        let result = std::alloc::System.allocate(layout);
        let ptr = match result {
            Ok(ptr) => ptr,
            Err(_) => return Err(AllocError { layout }.into()),
        };

        // Allocators currently return a `NonNull<[u8]>` whose length
        // matches the size requested. If that ever changes, the capacity
        // here should change to `ptr.len() / mem::size_of::<T>()`.
        Ok(Self {
            ptr: ptr.cast(),
            cap: capacity,
            _pd: PhantomData,
        })
    }

    fn grow_amortized(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        debug_assert!(additional > 0);

        if mem::size_of::<T>() == 0 {
            return Err(CapacityOverflow.into());
        }

        let required_cap = len.checked_add(additional).ok_or(CapacityOverflow)?;

        let cap = std::cmp::max(self.cap * 2, required_cap);
        let cap = std::cmp::max(Self::MIN_NON_ZERO_CAP, cap);

        let new_layout = Layout::array::<T>(cap).map_err(|_| CapacityOverflow)?;
        alloc_guard(new_layout.size())?;

        let alloc_result: Result<NonNull<[u8]>, TryReserveError> =
            (if let Some((ptr, old_layout)) = self.current_memory() {
                debug_assert_eq!(old_layout.align(), new_layout.align());
                unsafe {
                    // The allocator checks for alignment equality
                    // hint::assert_unchecked(old_layout.align() == new_layout.align());
                    std::alloc::System.grow(ptr, old_layout, new_layout)
                }
            } else {
                std::alloc::System.allocate(new_layout)
            })
            .map_err(|_| AllocError { layout: new_layout }.into());

        let ptr = alloc_result?;

        // SAFETY: if the allocation is valid, then the capacity is too
        unsafe {
            self.set_ptr_and_cap(ptr, cap);
        }

        Ok(())
    }
}

unsafe impl<#[may_dangle] T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if let Some((ptr, layout)) = self.current_memory() {
            unsafe { std::alloc::System.deallocate(ptr, layout) }
        }
    }
}

fn alloc_guard(alloc_size: usize) -> Result<(), TryReserveError> {
    if usize::BITS < 64 && alloc_size > isize::MAX as usize {
        Err(CapacityOverflow.into())
    } else {
        Ok(())
    }
}
