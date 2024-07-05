use core::{mem::MaybeUninit, ops::Deref, ptr::NonNull};

#[derive(Debug)]
pub struct UnallocatedVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

#[derive(Debug)]
pub struct AllocatedVec<T> {
    data: NonNull<T>,
    cap: usize,
    len: usize,
}

impl<T> AllocatedVec<T> {
    fn grow(&mut self) {
        let layout =
            alloc::alloc::Layout::array::<T>(self.cap).expect("Failed to create layout for vector");

        let ptr: NonNull<T> = unsafe {
            let ptr = alloc::alloc::realloc(
                self.data.as_ptr().cast(),
                layout,
                core::mem::size_of::<T>() * self.cap * 2,
            );
            if ptr.is_null() {
                alloc::alloc::handle_alloc_error(layout);
            }
            NonNull::new_unchecked(ptr.cast())
        };

        self.cap *= 2;
        self.data = ptr;
    }

    fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }
        unsafe { self.data.as_ptr().add(self.len).write(value) };
        self.len += 1;
    }
}

/// A vector that starts off allocated on the stack, but as it grows might get moved to the heap.
#[derive(Debug)]
pub enum StackVec<T, const N: usize = 32> {
    /// Not yet heap-allocated
    Unallocated(UnallocatedVec<T, N>),
    /// Heap-allocated vector
    Allocated(AllocatedVec<T>),
}

impl<T, const N: usize> Default for StackVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> StackVec<T, N> {
    const ARRAY_INIT: MaybeUninit<T> = MaybeUninit::uninit();

    /// Creates a new empty vector
    ///
    /// # Panics
    ///
    /// Panics if the size of `T` in bytes is zero.
    pub fn new() -> Self {
        if core::mem::size_of::<T>() == 0 {
            panic!("ZSTs are not yet supported");
        }

        Self::Unallocated(UnallocatedVec {
            data: [Self::ARRAY_INIT; N],
            len: 0,
        })
    }

    /// Inserts an element at the back of the vector
    pub fn push(&mut self, value: T) {
        match self {
            Self::Unallocated(v) if v.len < N => {
                v.data[v.len] = MaybeUninit::new(value);
                v.len += 1;
            }
            Self::Unallocated(v) => {
                let layout = core::alloc::Layout::array::<T>(v.len + 1)
                    .expect("Failed to create layout for vector");
                let ptr: NonNull<T> = unsafe {
                    let ptr = alloc::alloc::alloc(layout);
                    if ptr.is_null() {
                        alloc::alloc::handle_alloc_error(layout);
                    }
                    NonNull::new_unchecked(ptr.cast())
                };
                unsafe {
                    std::ptr::copy(
                        (&v.data as *const MaybeUninit<T>).cast(),
                        ptr.as_ptr(),
                        v.len,
                    );
                    std::ptr::write(ptr.as_ptr().add(v.len), value);
                };

                let alloc_vec = AllocatedVec {
                    data: ptr,
                    cap: v.len + 1,
                    len: v.len + 1,
                };

                let _ = core::mem::replace(self, Self::Allocated(alloc_vec));
            }
            Self::Allocated(v) => v.push(value),
        }
    }

    /// Removes the last element of the vector and returns it.
    pub fn pop(&mut self) -> Option<T> {
        match self {
            Self::Unallocated(v) => {
                if v.len > 0 {
                    let val = core::mem::replace(&mut v.data[v.len - 1], MaybeUninit::uninit());
                    v.len -= 1;
                    Some(unsafe { val.assume_init() })
                } else {
                    None
                }
            }
            Self::Allocated(v) => {
                if v.len > 0 {
                    let val = unsafe { core::ptr::read(v.data.as_ptr().add(v.len - 1)) };
                    v.len -= 1;
                    Some(val)
                } else {
                    None
                }
            }
        }
    }

    /// Removes the element at `index`, replacing it's position in the vector with the last
    /// element
    pub fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }

        if !self.is_empty() && index == self.len() - 1 {
            return None;
        }

        match self {
            Self::Unallocated(v) => {
                if v.len > 1 {
                    // SAFETY: Since we have a self, the last element
                    // should be initialized
                    let back = unsafe {
                        core::mem::replace(&mut v.data[v.len - 1], MaybeUninit::uninit())
                            .assume_init()
                    };

                    v.len -= 1;

                    // SAFETY: Since we have a self, and index is in `[0..len)`, this element
                    // should be initialized
                    Some(unsafe {
                        core::mem::replace(&mut v.data[index], MaybeUninit::new(back)).assume_init()
                    })
                } else {
                    None
                }
            }
            Self::Allocated(v) => {
                if v.len > 1 {
                    // SAFETY: Since we have a `&self`, and index is in [0..len], the last element pointer
                    // is valid, so dereferencing is fine.
                    let back = unsafe { v.data.as_ptr().add(v.len - 1).read() };
                    v.len -= 1;

                    // SAFETY: Since we have a `&self`, and index is in [0..len], this pointer is
                    // valid, so dereferencing is fine.
                    Some(std::mem::replace(
                        unsafe { &mut *v.data.as_ptr().add(index) },
                        back,
                    ))
                } else {
                    None
                }
            }
        }
    }

    /// Returns the number of elements in the vector
    pub fn len(&self) -> usize {
        match self {
            Self::Allocated(v) => v.len,
            Self::Unallocated(v) => v.len,
        }
    }

    /// Returns `true` if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of elements the vector can hold without reallocating.
    pub fn capacity(&self) -> usize {
        match self {
            Self::Unallocated(_) => N,
            Self::Allocated(v) => v.cap,
        }
    }

    /// Returns `true` if the vector is on the heap
    pub fn is_allocated(&self) -> bool {
        matches!(self, Self::Allocated(_))
    }

    /// Removes all elements from the vector
    pub fn clear(&mut self) {
        match self {
            Self::Unallocated(v) => {
                v.data[..v.len]
                    .iter_mut()
                    .for_each(|val| unsafe { val.assume_init_drop() });
                v.data = [Self::ARRAY_INIT; N];
                v.len = 0;
            }
            Self::Allocated(v) => {
                (0..v.len).for_each(|i| {
                    unsafe { v.data.as_ptr().add(i).drop_in_place() };
                });
                v.len = 0;
            }
        }
    }
}

impl<T, const N: usize> core::ops::Deref for StackVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Unallocated(v) => unsafe {
                core::slice::from_raw_parts((&v.data as *const MaybeUninit<T>).cast(), v.len)
            },
            Self::Allocated(v) => unsafe { core::slice::from_raw_parts(v.data.as_ptr(), v.len) },
        }
    }
}

impl<T, const N: usize> core::ops::DerefMut for StackVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Unallocated(v) => unsafe {
                core::slice::from_raw_parts_mut((&mut v.data as *mut MaybeUninit<T>).cast(), v.len)
            },
            Self::Allocated(v) => unsafe {
                core::slice::from_raw_parts_mut(v.data.as_ptr(), v.len)
            },
        }
    }
}

impl<T, const N: usize> core::ops::Drop for StackVec<T, N> {
    fn drop(&mut self) {
        match self {
            Self::Unallocated(v) => v.data[..v.len]
                .iter_mut()
                .for_each(|val| unsafe { val.assume_init_drop() }),
            Self::Allocated(v) => {
                for i in 0..v.len {
                    unsafe { v.data.as_ptr().add(i).drop_in_place() };
                }
                let layout = alloc::alloc::Layout::array::<T>(v.cap)
                    .expect("Failed to create layout for vec");
                unsafe { alloc::alloc::dealloc(v.data.as_ptr().cast(), layout) }
            }
        }
    }
}

impl<T, const N: usize> core::ops::Index<usize> for StackVec<T, N> {
    type Output = T;

    /// Returns a reference to the element at `index`.
    ///
    /// # Panics
    ///
    /// Panics if `index` is greater than or equal to `len`
    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Self::Unallocated(v) => {
                if index >= v.len {
                    panic!("Vector indexed out of bounds");
                }

                unsafe { v.data[index].assume_init_ref() }
            }
            Self::Allocated(v) => {
                if index >= v.len {
                    panic!("Vector indexed out of bounds");
                }

                unsafe { &*v.data.as_ptr().add(index) }
            }
        }
    }
}

impl<T, const N: usize> core::ops::IndexMut<usize> for StackVec<T, N> {
    /// Returns a mutable reference to the element at `index`.
    ///
    /// # Panics
    ///
    /// Panics if `index` is greater than or equal to `len`
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            Self::Unallocated(v) => {
                if index >= v.len {
                    panic!("Vector indexed out of bounds");
                }

                unsafe { v.data[index].assume_init_mut() }
            }
            Self::Allocated(v) => {
                if index >= v.len {
                    panic!("Vector indexed out of bounds");
                }

                unsafe { &mut *v.data.as_ptr().add(index) }
            }
        }
    }
}

impl<T, const N: usize> AsRef<[T]> for StackVec<T, N> {
    fn as_ref(&self) -> &[T] {
        self.deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let v = StackVec::<i32>::new();
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn small() {
        let mut v = StackVec::<i32, 4>::new();
        v.push(0);
        v.push(1);
        v.push(2);
        v.push(3);

        assert_eq!(v.len(), 4);
        assert!(matches!(v, StackVec::Unallocated(_)));
        assert_eq!(v[0], 0);
        assert_eq!(v[1], 1);
        assert_eq!(v[2], 2);
        assert_eq!(v[3], 3);
    }

    #[test]
    fn stack_to_heap() {
        let mut v = StackVec::<i32, 4>::new();
        v.push(0);
        v.push(1);
        v.push(2);
        v.push(3);

        assert_eq!(v.len(), 4);
        assert!(matches!(v, StackVec::Unallocated(_)));
        assert_eq!(v[0], 0);
        assert_eq!(v[1], 1);
        assert_eq!(v[2], 2);
        assert_eq!(v[3], 3);

        v.push(4);
        assert_eq!(v.len(), 5);
        assert!(matches!(v, StackVec::Allocated(_)));
        assert_eq!(v[0], 0);
        assert_eq!(v[1], 1);
        assert_eq!(v[2], 2);
        assert_eq!(v[3], 3);
        assert_eq!(v[4], 4);
    }

    #[test]
    fn stack_to_heap_plus_one() {
        let mut v = StackVec::<i32, 4>::new();
        v.push(0);
        v.push(1);
        v.push(2);
        v.push(3);

        assert_eq!(v.len(), 4);
        assert!(matches!(v, StackVec::Unallocated(_)));
        assert_eq!(v[0], 0);
        assert_eq!(v[1], 1);
        assert_eq!(v[2], 2);
        assert_eq!(v[3], 3);

        v.push(4);
        v.push(5);
        v.push(6);
        assert_eq!(v.len(), 7);
        assert!(matches!(v, StackVec::Allocated(_)));

        assert_eq!(v[0], 0);
        assert_eq!(v[1], 1);
        assert_eq!(v[2], 2);
        assert_eq!(v[3], 3);
        assert_eq!(v[4], 4);
        assert_eq!(v[5], 5);
        assert_eq!(v[6], 6);
        assert_eq!(v.capacity(), 10);
    }

    #[test]
    fn pop_small() {
        let mut v = StackVec::<i32, 8>::new();
        v.push(32);
        assert_eq!(v.pop(), Some(32));
        assert_eq!(v.len(), 0);

        for i in 0..8 {
            v.push(i);
            assert_eq!(v.len(), i as usize + 1);
            assert_eq!(v.capacity(), 8);
        }

        for i in (0..8).rev() {
            assert_eq!(v.pop(), Some(i));
            assert_eq!(v.capacity(), 8);
        }
    }

    #[test]
    fn pop_large() {
        let mut v = StackVec::<i32, 2>::new();
        for i in 0..100 {
            v.push(i);
            assert_eq!(v.len(), i as usize + 1);
        }

        for i in (0..100).rev() {
            assert_eq!(v.pop(), Some(i));
            assert_eq!(v.len(), i as usize);
        }
    }

    #[test]
    fn small_slice() {
        let mut v = StackVec::<i32, 8>::new();

        v.push(0);
        v.push(1);
        v.push(2);
        v.push(3);

        assert_eq!(&*v, &[0, 1, 2, 3]);
    }

    #[test]
    fn large_slice() {
        let mut v = StackVec::<i32, 2>::new();

        v.push(0);
        v.push(1);
        v.push(2);
        v.push(3);

        assert_eq!(&*v, &[0, 1, 2, 3]);
    }

    #[test]
    fn swap_remove_small() {
        let mut v = StackVec::<_, 8>::new();
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);
        v.push(5);

        assert_eq!(v.swap_remove(2), Some(3));
        assert_eq!(v[2], 5);
    }

    #[test]
    fn swap_remove_two() {
        let mut v = StackVec::<_, 8>::new();
        v.push(1);
        v.push(2);

        assert_eq!(v.swap_remove(0), Some(1));
        assert_eq!(v[0], 2);
    }

    #[test]
    fn swap_remove_large() {
        let mut v = StackVec::<_, 2>::new();

        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);
        v.push(5);

        assert!(matches!(v, StackVec::Allocated(_)));
        assert_eq!(v.swap_remove(2), Some(3));
        assert_eq!(v[2], 5);
    }
}
