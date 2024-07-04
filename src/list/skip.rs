#![allow(unused)]

use core::{alloc::Layout, ptr::NonNull};

/// A linked list with multiple links created in a probabilistic fashion in order to provide an
/// average lookup complexity of `O(log(n))`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkipList<T, const L: usize = 8> {
    head: [Option<NonNull<SkipListNode<T, L>>>; L],
    len: usize,
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct SkipListNode<T, const L: usize> {
    value: T,
    level: usize,
    next: [Option<NonNull<SkipListNode<T, L>>>; L],
}

impl<T, const L: usize> SkipList<T, L> {
    const HEAD_INIT_VALUE: Option<NonNull<SkipListNode<T, L>>> = None;

    /// Creates a new empty skip-list
    pub fn new() -> Self {
        Self {
            head: [Self::HEAD_INIT_VALUE; L],
            len: 0,
        }
    }

    /// Inserts a new element into the list.
    pub fn insert(&mut self, element: T)
    where
        T: PartialOrd,
    {
        let mut level = 0;

        while rand::prelude::random::<f32>() < 0.5 && level <= L {
            level += 1;
        }

        let layout = Layout::new::<SkipListNode<T, L>>();

        // SAFETY: Since layout generates valid layouts and we check for the allocation being null,
        // creating a NonNull from the pointer should be safe.
        let mut ptr: NonNull<SkipListNode<T, L>> = unsafe {
            let ptr = std::alloc::alloc(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            NonNull::new_unchecked(ptr.cast())
        };

        // SAFETY: Since ptr is still valid, and no-one else is currently writing to it, mutably
        // dereferencing is fine.
        let new_node = unsafe {
            *ptr.as_ptr() = SkipListNode {
                value: element,
                level,
                next: [Self::HEAD_INIT_VALUE; L],
            };
            ptr.as_mut()
        };

        for k in (0..L).rev() {}
    }
}

impl<T, const N: usize> Default for SkipList<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
