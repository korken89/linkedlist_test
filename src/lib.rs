use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::ptr;

pub trait LinkedListIndex: Copy {
    #[doc(hidden)]
    unsafe fn new_unchecked(val: usize) -> Self;
    #[doc(hidden)]
    unsafe fn get_unchecked(self) -> usize;
    #[doc(hidden)]
    fn option(self) -> Option<usize>;
    #[doc(hidden)]
    fn none() -> Self;
}

/// Marker for Min sorted [`LinkedList`].
pub struct Min;

/// Marker for Max sorted [`LinkedList`].
pub struct Max;

/// Sealed traits and implementations for [`LinkedList`].
mod kind {
    use super::{Max, Min};
    use core::cmp::Ordering;

    /// The linked list kind: minimum first or maximum first.
    pub unsafe trait Kind {
        #[doc(hidden)]
        fn ordering() -> Option<Ordering>;
    }

    unsafe impl Kind for Min {
        #[inline]
        fn ordering() -> Option<Ordering> {
            Some(Ordering::Less)
        }
    }

    unsafe impl Kind for Max {
        #[inline]
        fn ordering() -> Option<Ordering> {
            Some(Ordering::Greater)
        }
    }
}

/// A node in the linked list.
pub struct Node<T, Idx> {
    val: MaybeUninit<T>,
    next: Idx,
}

/// The linked list.
pub struct LinkedList<T, Idx, Kind, const N: usize>
where
    Idx: LinkedListIndex,
{
    list: [Node<T, Idx>; N],
    head: Idx,
    free: Idx,
    _kind: PhantomData<Kind>,
}

macro_rules! impl_index_and_const_new {
    ($name:ident, $ty:ty, $new_name:ident, $max_val:literal) => {
        #[doc = concat!(
            "Index for the [`LinkedList`] using `", stringify!($ty), "` as backing storage.")
        ]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name($ty);

        impl LinkedListIndex for $name {
            #[doc = concat!("Safety: `val` is <= `", stringify!($max_val), "`.")]
            #[inline(always)]
            unsafe fn new_unchecked(val: usize) -> Self {
                Self::new_unchecked(val as $ty)
            }

            /// This is only valid if `self.option()` is not `None`.
            #[inline(always)]
            unsafe fn get_unchecked(self) -> usize {
                self.0 as usize
            }

            #[inline(always)]
            fn option(self) -> Option<usize> {
                if self.0 == <$ty>::MAX {
                    None
                } else {
                    Some(self.0 as usize)
                }
            }

            #[inline(always)]
            fn none() -> Self {
                Self::none()
            }
        }

        impl $name {
            /// Needed for a `const fn new()`.
            #[inline]
            const unsafe fn new_unchecked(value: $ty) -> Self {
                $name(value)
            }

            /// Needed for a `const fn new()`.
            #[inline]
            const fn none() -> Self {
                $name(<$ty>::MAX)
            }
        }

        impl<T, Kind, const N: usize> LinkedList<T, $name, Kind, N> {
            const UNINIT: Node<T, $name> = Node {
                val: MaybeUninit::uninit(),
                next: $name::none(),
            };

            /// Create a new linked list.
            pub const fn $new_name() -> Self {
                let mut list = LinkedList {
                    list: [Self::UNINIT; N],
                    head: $name::none(),
                    free: unsafe { $name::new_unchecked(0) },
                    _kind: PhantomData,
                };

                if N == 0 {
                    list.free = $name::none();
                    return list;
                }

                let mut free = 0;

                // Initialize indexes
                while free < N - 1 {
                    list.list[free].next = unsafe { $name::new_unchecked(free as $ty + 1) };
                    free += 1;
                }

                list
            }
        }
    }
}

impl_index_and_const_new!(LinkedIndexU8, u8, new_u8, 254); // val is 2^8 - 2 (one less than max)
impl_index_and_const_new!(LinkedIndexU16, u16, new_u16, 65_534); // val is 2^16 - 2
impl_index_and_const_new!(LinkedIndexUsize, usize, new_usize, 4_294_967_294); // val is 2^32 - 2

impl<T, Idx, Kind, const N: usize> LinkedList<T, Idx, Kind, N>
where
    Idx: LinkedListIndex,
{
    /// Internal access helper
    #[inline(always)]
    fn node_at(&self, index: usize) -> &Node<T, Idx> {
        // Safety: The entire `self.list` is initialized in `new`, which makes this safe.
        unsafe { self.list.get_unchecked(index) }
    }

    /// Internal access helper
    #[inline(always)]
    fn node_at_mut(&mut self, index: usize) -> &mut Node<T, Idx> {
        // Safety: The entire `self.list` is initialized in `new`, which makes this safe.
        unsafe { self.list.get_unchecked_mut(index) }
    }

    /// Internal access helper
    #[inline(always)]
    fn write_data_in_node_at(&mut self, index: usize, data: T) {
        // Safety: The entire `self.list` is initialized in `new`, which makes this safe.
        unsafe {
            self.node_at_mut(index).val.as_mut_ptr().write(data);
        }
    }

    /// Internal access helper
    #[inline(always)]
    fn read_data_in_node_at(&self, index: usize) -> &T {
        // Safety: The entire `self.list` is initialized in `new`, which makes this safe.
        unsafe { &*self.node_at(index).val.as_ptr() }
    }

    /// Internal access helper
    #[inline(always)]
    fn read_mut_data_in_node_at(&mut self, index: usize) -> &mut T {
        // Safety: The entire `self.list` is initialized in `new`, which makes this safe.
        unsafe { &mut *self.node_at_mut(index).val.as_mut_ptr() }
    }

    /// Internal access helper
    #[inline(always)]
    fn extract_data_in_node_at(&mut self, index: usize) -> T {
        // Safety: The entire `self.list` is initialized in `new`, which makes this safe.
        unsafe { self.node_at(index).val.as_ptr().read() }
    }
}

impl<T, Idx, Kind, const N: usize> LinkedList<T, Idx, Kind, N>
where
    T: PartialOrd,
    Idx: LinkedListIndex,
    Kind: kind::Kind,
{
    /// Pushes a value onto the list without checking if the list is full.
    ///
    /// Complexity is worst-case `O(N)`.
    ///
    /// # Safety
    ///
    /// Assumes that the list is not full.
    pub unsafe fn push_unchecked(&mut self, value: T) {
        let new = self.free.get_unchecked();

        // Store the data and update the next free spot
        self.write_data_in_node_at(new, value);
        self.free = self.node_at(new).next;

        if let Some(head) = self.head.option() {
            // Check if we need to replace head
            if self
                .read_data_in_node_at(head as usize)
                .partial_cmp(self.read_data_in_node_at(new as usize))
                != Kind::ordering()
            {
                self.node_at_mut(new as usize).next = self.head;
                self.head = Idx::new_unchecked(new);
            } else {
                // It's not head, search the list for the correct placement
                let mut current = head;

                while let Some(next) = self.node_at(current).next.option() {
                    if self
                        .read_data_in_node_at(next as usize)
                        .partial_cmp(self.read_data_in_node_at(new as usize))
                        != Kind::ordering()
                    {
                        break;
                    }

                    current = next;
                }

                self.node_at_mut(new).next = self.node_at(current).next;
                self.node_at_mut(current).next = Idx::new_unchecked(new);
            }
        } else {
            self.node_at_mut(new).next = self.head;
            self.head = Idx::new_unchecked(new);
        }
    }

    /// Pushes an element to the linked list and sorts it into place.
    ///
    /// Complexity is worst-case `O(N)`.
    ///
    /// # Example
    ///
    /// ```
    /// use linked_list::{LinkedList, Max};
    /// let mut ll: LinkedList<_, _, Max, 3> = LinkedList::new_usize();
    ///
    /// // The largest value will always be first
    /// ll.push(1).unwrap();
    /// assert_eq!(ll.peek(), Some(&1));
    ///
    /// ll.push(2).unwrap();
    /// assert_eq!(ll.peek(), Some(&2));
    ///
    /// ll.push(3).unwrap();
    /// assert_eq!(ll.peek(), Some(&3));
    ///
    /// // This will not fit in the queue.
    /// assert_eq!(ll.push(4), Err(4));
    /// ```
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if !self.is_full() {
            Ok(unsafe { self.push_unchecked(value) })
        } else {
            Err(value)
        }
    }

    /// Get an iterator over the sorted list.
    ///
    /// # Example
    ///
    /// ```
    /// use linked_list::{LinkedList, Max};
    /// let mut ll: LinkedList<_, _, Max, 3> = LinkedList::new_usize();
    ///
    /// ll.push(1).unwrap();
    /// ll.push(2).unwrap();
    ///
    /// let mut iter = ll.iter();
    ///
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<'_, T, Idx, Kind, N> {
        Iter {
            list: self,
            index: self.head,
        }
    }

    /// Find an element in the list that can be changed and resorted.
    ///
    /// # Example
    ///
    /// ```
    /// use linked_list::{LinkedList, Max};
    /// let mut ll: LinkedList<_, _, Max, 3> = LinkedList::new_usize();
    ///
    /// ll.push(1).unwrap();
    /// ll.push(2).unwrap();
    /// ll.push(3).unwrap();
    ///
    /// // Find a value and update it
    /// let mut find = ll.find_mut(|v| *v == 2).unwrap();
    /// *find += 1000;
    /// find.finish();
    ///
    /// assert_eq!(ll.pop(), Ok(1002));
    /// assert_eq!(ll.pop(), Ok(3));
    /// assert_eq!(ll.pop(), Ok(1));
    /// assert_eq!(ll.pop(), Err(()));
    /// ```
    pub fn find_mut<F>(&mut self, mut f: F) -> Option<FindMut<'_, T, Idx, Kind, N>>
    where
        F: FnMut(&T) -> bool,
    {
        let head = self.head.option()?;

        // Special-case, first element
        if f(self.read_data_in_node_at(head as usize)) {
            return Some(FindMut {
                is_head: true,
                prev_index: Idx::none(),
                index: self.head,
                list: self,
                maybe_changed: false,
            });
        }

        let mut current = head;

        while let Some(next) = self.node_at(current as usize).next.option() {
            if f(self.read_data_in_node_at(next as usize)) {
                return Some(FindMut {
                    is_head: false,
                    prev_index: unsafe { Idx::new_unchecked(current) },
                    index: unsafe { Idx::new_unchecked(next) },
                    list: self,
                    maybe_changed: false,
                });
            }

            current = next;
        }

        None
    }

    /// Peek at the first element.
    ///
    /// # Example
    ///
    /// ```
    /// use linked_list::{LinkedList, Max, Min};
    /// let mut ll_max: LinkedList<_, _, Max, 3> = LinkedList::new_usize();
    ///
    /// // The largest value will always be first
    /// ll_max.push(1).unwrap();
    /// assert_eq!(ll_max.peek(), Some(&1));
    /// ll_max.push(2).unwrap();
    /// assert_eq!(ll_max.peek(), Some(&2));
    /// ll_max.push(3).unwrap();
    /// assert_eq!(ll_max.peek(), Some(&3));
    ///
    /// let mut ll_min: LinkedList<_, _, Min, 3> = LinkedList::new_usize();
    ///
    /// // The Smallest value will always be first
    /// ll_min.push(3).unwrap();
    /// assert_eq!(ll_min.peek(), Some(&3));
    /// ll_min.push(2).unwrap();
    /// assert_eq!(ll_min.peek(), Some(&2));
    /// ll_min.push(1).unwrap();
    /// assert_eq!(ll_min.peek(), Some(&1));
    /// ```
    pub fn peek(&self) -> Option<&T> {
        self.head
            .option()
            .map(|head| self.read_data_in_node_at(head))
    }

    /// Pop an element from the list without checking so the list is not empty.
    ///
    /// # Safety
    ///
    /// Assumes that the list is not empty.
    pub unsafe fn pop_unchecked(&mut self) -> T {
        let head = self.head.get_unchecked();
        let current = head;
        self.head = self.node_at(head).next;
        self.node_at_mut(current).next = self.free;
        self.free = Idx::new_unchecked(current);

        self.extract_data_in_node_at(current)
    }

    /// Pops the first element in the list.
    ///
    /// Complexity is worst-case `O(1)`.
    ///
    /// # Example
    ///
    /// ```
    /// use linked_list::{LinkedList, Max};
    /// let mut ll: LinkedList<_, _, Max, 3> = LinkedList::new_usize();
    ///
    /// ll.push(1).unwrap();
    /// ll.push(2).unwrap();
    ///
    /// assert_eq!(ll.pop(), Ok(2));
    /// assert_eq!(ll.pop(), Ok(1));
    /// assert_eq!(ll.pop(), Err(()));
    /// ```
    pub fn pop(&mut self) -> Result<T, ()> {
        if !self.is_empty() {
            Ok(unsafe { self.pop_unchecked() })
        } else {
            Err(())
        }
    }

    /// Checks if the linked list is full.
    ///
    /// # Example
    ///
    /// ```
    /// use linked_list::{LinkedList, Max};
    /// let mut ll: LinkedList<_, _, Max, 3> = LinkedList::new_usize();
    ///
    /// assert_eq!(ll.is_full(), false);
    ///
    /// ll.push(1).unwrap();
    /// assert_eq!(ll.is_full(), false);
    /// ll.push(2).unwrap();
    /// assert_eq!(ll.is_full(), false);
    /// ll.push(3).unwrap();
    /// assert_eq!(ll.is_full(), true);
    /// ```
    #[inline]
    pub fn is_full(&self) -> bool {
        self.free.option().is_none()
    }

    /// Checks if the linked list is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use linked_list::{LinkedList, Max};
    /// let mut ll: LinkedList<_, _, Max, 3> = LinkedList::new_usize();
    ///
    /// assert_eq!(ll.is_empty(), true);
    ///
    /// ll.push(1).unwrap();
    /// assert_eq!(ll.is_empty(), false);
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head.option().is_none()
    }
}

/// Iterator for the linked list.
pub struct Iter<'a, T, Idx, Kind, const N: usize>
where
    T: PartialOrd,
    Idx: LinkedListIndex,
    Kind: kind::Kind,
{
    list: &'a LinkedList<T, Idx, Kind, N>,
    index: Idx,
}

impl<'a, T, Idx, Kind, const N: usize> Iterator for Iter<'a, T, Idx, Kind, N>
where
    T: PartialOrd,
    Idx: LinkedListIndex,
    Kind: kind::Kind,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index.option()?;

        let node = self.list.node_at(index);
        self.index = node.next;

        Some(self.list.read_data_in_node_at(index))
    }
}

/// Comes from [`LinkedList::find_mut`].
pub struct FindMut<'a, T, Idx, Kind, const N: usize>
where
    T: PartialOrd,
    Idx: LinkedListIndex,
    Kind: kind::Kind,
{
    list: &'a mut LinkedList<T, Idx, Kind, N>,
    is_head: bool,
    prev_index: Idx,
    index: Idx,
    maybe_changed: bool,
}

impl<'a, T, Idx, Kind, const N: usize> FindMut<'a, T, Idx, Kind, N>
where
    T: PartialOrd,
    Idx: LinkedListIndex,
    Kind: kind::Kind,
{
    fn pop_internal(&mut self) -> T {
        if self.is_head {
            // If it is the head element, we can do a normal pop
            unsafe { self.list.pop_unchecked() }
        } else {
            // Somewhere in the list
            let prev = unsafe { self.prev_index.get_unchecked() };
            let curr = unsafe { self.index.get_unchecked() };

            // Re-point the previous index
            self.list.node_at_mut(prev).next = self.list.node_at_mut(curr).next;

            // Release the index into the free queue
            self.list.node_at_mut(curr).next = self.list.free;
            self.list.free = self.index;

            self.list.extract_data_in_node_at(curr)
        }
    }

    /// This will pop the element from the list.
    ///
    /// Complexity is worst-case `O(1)`.
    ///
    /// # Example
    ///
    /// ```
    /// use linked_list::{LinkedList, Max};
    /// let mut ll: LinkedList<_, _, Max, 3> = LinkedList::new_usize();
    ///
    /// ll.push(1).unwrap();
    /// ll.push(2).unwrap();
    /// ll.push(3).unwrap();
    ///
    /// // Find a value and update it
    /// let mut find = ll.find_mut(|v| *v == 2).unwrap();
    /// find.pop();
    ///
    /// assert_eq!(ll.pop(), Ok(3));
    /// assert_eq!(ll.pop(), Ok(1));
    /// assert_eq!(ll.pop(), Err(()));
    /// ```
    #[inline]
    pub fn pop(mut self) -> T {
        self.pop_internal()
    }

    /// This will resort the element into the correct position in the list if needed. The resorting
    /// will only happen if the element has been accessed mutably.
    ///
    /// Same as calling `drop`.
    ///
    /// Complexity is worst-case `O(N)`.
    ///
    /// # Example
    ///
    /// ```
    /// use linked_list::{LinkedList, Max};
    /// let mut ll: LinkedList<_, _, Max, 3> = LinkedList::new_usize();
    ///
    /// ll.push(1).unwrap();
    /// ll.push(2).unwrap();
    /// ll.push(3).unwrap();
    ///
    /// let mut find = ll.find_mut(|v| *v == 2).unwrap();
    /// find.finish(); // No resort, we did not access the value.
    ///
    /// let mut find = ll.find_mut(|v| *v == 2).unwrap();
    /// *find += 1000;
    /// find.finish(); // Will resort, we accessed (and updated) the value.
    ///
    /// assert_eq!(ll.pop(), Ok(1002));
    /// assert_eq!(ll.pop(), Ok(3));
    /// assert_eq!(ll.pop(), Ok(1));
    /// assert_eq!(ll.pop(), Err(()));
    /// ```
    #[inline]
    pub fn finish(self) {
        drop(self)
    }
}

impl<T, Idx, Kind, const N: usize> Drop for FindMut<'_, T, Idx, Kind, N>
where
    T: PartialOrd,
    Idx: LinkedListIndex,
    Kind: kind::Kind,
{
    fn drop(&mut self) {
        // Only resort the list if the element has changed
        if self.maybe_changed {
            let val = self.pop_internal();
            unsafe { self.list.push_unchecked(val) };
        }
    }
}

impl<T, Idx, Kind, const N: usize> Deref for FindMut<'_, T, Idx, Kind, N>
where
    T: PartialOrd,
    Idx: LinkedListIndex,
    Kind: kind::Kind,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.list
            .read_data_in_node_at(unsafe { self.index.get_unchecked() })
    }
}

impl<T, Idx, Kind, const N: usize> DerefMut for FindMut<'_, T, Idx, Kind, N>
where
    T: PartialOrd,
    Idx: LinkedListIndex,
    Kind: kind::Kind,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.maybe_changed = true;
        self.list
            .read_mut_data_in_node_at(unsafe { self.index.get_unchecked() })
    }
}

// /// Useful for debug during development.
// impl<T, Idx, Kind, const N: usize> fmt::Debug for FindMut<'_, T, Idx, Kind, N>
// where
//     T: PartialOrd + core::fmt::Debug,
//     Idx: LinkedListIndex,
//     Kind: kind::Kind,
// {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("FindMut")
//             .field("prev_index", &self.prev_index.option())
//             .field("index", &self.index.option())
//             .field(
//                 "prev_value",
//                 &self
//                     .list
//                     .read_data_in_node_at(self.prev_index.option().unwrap()),
//             )
//             .field(
//                 "value",
//                 &self.list.read_data_in_node_at(self.index.option().unwrap()),
//             )
//             .finish()
//     }
// }

impl<T, Idx, Kind, const N: usize> fmt::Debug for LinkedList<T, Idx, Kind, N>
where
    T: PartialOrd + core::fmt::Debug,
    Idx: LinkedListIndex,
    Kind: kind::Kind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T, Idx, Kind, const N: usize> Drop for LinkedList<T, Idx, Kind, N>
where
    Idx: LinkedListIndex,
{
    fn drop(&mut self) {
        let mut index = self.head;

        while let Some(i) = index.option() {
            let node = self.node_at_mut(i);
            index = node.next;

            unsafe {
                ptr::drop_in_place(node.val.as_mut_ptr());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_new() {
        static mut _V1: LinkedList<u32, LinkedIndexU8, Max, 100> = LinkedList::new_u8();
        static mut _V2: LinkedList<u32, LinkedIndexU16, Max, 10_000> = LinkedList::new_u16();
        static mut _V3: LinkedList<u32, LinkedIndexUsize, Max, 100_000> = LinkedList::new_usize();
    }

    #[test]
    fn test_peek() {
        let mut ll: LinkedList<u32, LinkedIndexUsize, Max, 3> = LinkedList::new_usize();

        ll.push(1).unwrap();
        assert_eq!(ll.peek().unwrap(), &1);

        ll.push(2).unwrap();
        assert_eq!(ll.peek().unwrap(), &2);

        ll.push(3).unwrap();
        assert_eq!(ll.peek().unwrap(), &3);

        let mut ll: LinkedList<u32, LinkedIndexUsize, Min, 3> = LinkedList::new_usize();

        ll.push(2).unwrap();
        assert_eq!(ll.peek().unwrap(), &2);

        ll.push(1).unwrap();
        assert_eq!(ll.peek().unwrap(), &1);

        ll.push(3).unwrap();
        assert_eq!(ll.peek().unwrap(), &1);
    }

    #[test]
    fn test_full() {
        let mut ll: LinkedList<u32, LinkedIndexUsize, Max, 3> = LinkedList::new_usize();
        ll.push(1).unwrap();
        ll.push(2).unwrap();
        ll.push(3).unwrap();

        assert!(ll.is_full())
    }

    #[test]
    fn test_empty() {
        let ll: LinkedList<u32, LinkedIndexUsize, Max, 3> = LinkedList::new_usize();

        assert!(ll.is_empty())
    }

    #[test]
    fn test_zero_size() {
        let ll: LinkedList<u32, LinkedIndexUsize, Max, 0> = LinkedList::new_usize();

        assert!(ll.is_empty());
        assert!(ll.is_full());
    }

    #[test]
    fn test_rejected_push() {
        let mut ll: LinkedList<u32, LinkedIndexUsize, Max, 3> = LinkedList::new_usize();
        ll.push(1).unwrap();
        ll.push(2).unwrap();
        ll.push(3).unwrap();

        // This won't fit
        let r = ll.push(4);

        assert_eq!(r, Err(4));
    }

    #[test]
    fn test_updating() {
        let mut ll: LinkedList<u32, LinkedIndexUsize, Max, 3> = LinkedList::new_usize();
        ll.push(1).unwrap();
        ll.push(2).unwrap();
        ll.push(3).unwrap();

        let mut find = ll.find_mut(|v| *v == 2).unwrap();

        *find += 1000;
        find.finish();

        assert_eq!(ll.peek().unwrap(), &1002);

        let mut find = ll.find_mut(|v| *v == 3).unwrap();

        *find += 1000;
        find.finish();

        assert_eq!(ll.peek().unwrap(), &1003);

        // Remove largest element
        ll.find_mut(|v| *v == 1003).unwrap().pop();

        assert_eq!(ll.peek().unwrap(), &1002);
    }

    #[test]
    #[test]
    fn test_updating_1() {
        let mut ll: LinkedList<u32, LinkedIndexUsize, Max, 3> = LinkedList::new_usize();
        ll.push(1).unwrap();

        let v = ll.pop().unwrap();

        assert_eq!(v, 1);
    }

    #[test]
    fn test_updating_2() {
        let mut ll: LinkedList<u32, LinkedIndexUsize, Max, 3> = LinkedList::new_usize();
        ll.push(1).unwrap();

        let mut find = ll.find_mut(|v| *v == 1).unwrap();

        *find += 1000;
        find.finish();

        assert_eq!(ll.peek().unwrap(), &1001);
    }
}
