use alloc::boxed::Box;

/// A binary tree containing key-value pairs where the keys can be ordered.
///
/// It should be noted that for most applications, a `HashMap` will offer
/// superior performance to that of a binary tree, since each node in the tree requires a heap
/// allocation (apart from the root). Hash maps also provided amortized-constant lookup times where
/// a binary tree gives `O(log(n))`.
///
/// For efficiency, the tree maintains a count of the number of elements inserted so that the
/// `len` and `is_empty` methods are constant-time complexity.
///
/// # Examples
///
/// This example shows how the binary tree functions much like a `HashMap`, but
/// gives `O(log(n))` lookup time for keys that are of an ordinal type.
/// ```
/// use collect_me::BinaryTree;
///
/// let mut tree = BinaryTree::new();
/// tree.insert(0, "John");
/// tree.insert(42, "Neo");
/// tree.insert(2, "Alice");
///
/// assert_eq!(tree.get(&0), Some(&"John"));
/// assert_eq!(tree.get(&42), Some(&"Neo"));
/// assert_eq!(tree.get(&2), Some(&"Alice"));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BinaryTree<K, V> {
    root: Option<BinaryTreeNode<K, V>>,
    len: usize,
}

type NodeChild<K, V> = Option<Box<BinaryTreeNode<K, V>>>;

#[doc(hidden)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct BinaryTreeNode<K, V> {
    key: K,
    value: V,
    children: (NodeChild<K, V>, NodeChild<K, V>),
}

impl<K, V> BinaryTree<K, V> {
    /// Constructs an empty tree
    pub fn new() -> Self {
        Self { root: None, len: 0 }
    }
}

impl<K, V> BinaryTree<K, V>
where
    K: PartialOrd + Eq,
{
    /// Inserts a key-value pair into the [`BinaryTree`].
    ///
    /// Returns [`None`] if the key did not exist, otherwise updates
    /// the value and returns [`Some`] with the old value.
    ///
    /// # Note
    ///
    /// Like with `HashMap` the key does not get updated.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let result = if let Some(ref mut root) = self.root {
            root.insert(key, value)
        } else {
            self.root = Some(BinaryTreeNode {
                key,
                value,
                children: (None, None),
            });
            None
        };

        if result.is_none() {
            self.len += 1;
        }

        result
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: core::borrow::Borrow<Q>,
        Q: PartialOrd + Eq,
    {
        self.root.as_ref().and_then(|root| root.get(key))
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: core::borrow::Borrow<Q>,
        Q: PartialOrd + Eq,
    {
        self.root.as_mut().and_then(|root| root.get_mut(key))
    }

    /// Returns the number of elements in the tree with constant-time complexity.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<K, V> BinaryTreeNode<K, V>
where
    K: PartialOrd + Eq,
{
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        if key < self.key {
            if let Some(ref mut child) = self.children.0 {
                child.insert(key, value)
            } else {
                self.children.0 = Some(Box::new(BinaryTreeNode {
                    key,
                    value,
                    children: (None, None),
                }));
                None
            }
        } else if key > self.key {
            if let Some(ref mut child) = self.children.1 {
                child.insert(key, value)
            } else {
                self.children.1 = Some(Box::new(BinaryTreeNode {
                    key,
                    value,
                    children: (None, None),
                }));
                None
            }
        } else {
            Some(core::mem::replace(&mut self.value, value))
        }
    }

    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: core::borrow::Borrow<Q>,
        Q: PartialOrd + Eq,
    {
        if *key == *self.key.borrow() {
            Some(&self.value)
        } else if *key < *self.key.borrow() {
            self.children.0.as_ref().and_then(|child| child.get(key))
        } else {
            self.children.1.as_ref().and_then(|child| child.get(key))
        }
    }

    fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: core::borrow::Borrow<Q>,
        Q: PartialOrd + Eq,
    {
        if *key == *self.key.borrow() {
            Some(&mut self.value)
        } else if *key < *self.key.borrow() {
            self.children
                .0
                .as_mut()
                .and_then(|child| child.get_mut(key))
        } else {
            self.children
                .1
                .as_mut()
                .and_then(|child| child.get_mut(key))
        }
    }
}

impl<K, V> core::ops::Index<&K> for BinaryTree<K, V>
where
    K: PartialOrd + Eq,
{
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the binary tree.
    fn index(&self, index: &K) -> &Self::Output {
        self.get(index)
            .expect("Key is not present in the binary tree")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let tree: BinaryTree<i32, i32> = BinaryTree::new();
        assert_eq!(tree.get(&0), None);
        assert_eq!(tree.len(), 0);
        assert!(tree.is_empty());
    }

    #[test]
    fn one_key() {
        let mut tree = BinaryTree::new();
        tree.insert(3, "Hello");
        assert_eq!(tree.get(&3), Some(&"Hello"));
        assert_eq!(tree.insert(3, "World"), Some("Hello"));
        assert_eq!(tree.get(&3), Some(&"World"));
    }

    #[test]
    fn three_keys() {
        let mut tree = BinaryTree::new();
        tree.insert(1, 'A');
        tree.insert(0, 'B');
        tree.insert(2, 'C');

        assert_eq!(tree.get(&1), Some(&'A'));
        assert_eq!(tree.get(&0), Some(&'B'));
        assert_eq!(tree.get(&2), Some(&'C'));
        assert_eq!(tree.get(&-1), None);
    }

    #[test]
    fn get_mut() {
        let mut tree = BinaryTree::new();
        tree.insert(1, 'A');
        tree.insert(2, 'B');
        tree.insert(0, 'C');

        assert_eq!(tree.get(&0), Some(&'C'));

        let val = tree.get_mut(&1).expect("Failed to mutably reference value");
        *val = 'X';

        let val = tree.get_mut(&2).expect("Failed to mutably reference value");
        *val = 'Y';

        let val = tree.get_mut(&0).expect("Failed to mutably reference value");
        *val = 'Z';

        assert_eq!(tree.get(&1), Some(&'X'));
        assert_eq!(tree.get(&2), Some(&'Y'));
        assert_eq!(tree.get(&0), Some(&'Z'));
    }

    #[test]
    fn index() {
        let mut tree = BinaryTree::new();
        tree.insert(0, 'A');
        assert_eq!(tree[&0], 'A');
    }

    #[test]
    #[should_panic]
    fn index_nonexistent() {
        let mut tree = BinaryTree::new();
        tree.insert(0, 'A');
        let _ = tree[&1];
    }
}
