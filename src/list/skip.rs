use alloc::boxed::Box;

/// A linked list with multiple links created in a probabilistic fashion in order to provide an
/// average lookup complexity of `O(log(n))`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SkipList<T> {
    head: Option<SkipListNode<T>>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SkipListNode<T> {
    value: T,
    next: Option<Box<SkipListNode<T>>>,
}

impl<T> SkipList<T> {
    /// Creates a new empty skip-list
    pub fn new() -> Self {
        Self { head: None }
    }
}
