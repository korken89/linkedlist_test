use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use generic_array::{ArrayLength, GenericArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LinkedIndex(u8);

impl LinkedIndex {
    // #[inline]
    // fn new(value: u8) -> Self {
    //     assert!(value < u8::MAX);

    //     LinkedIndex(value)
    // }

    #[inline]
    const unsafe fn new_unchecked(value: u8) -> Self {
        LinkedIndex(value)
    }

    #[inline]
    const fn none() -> Self {
        LinkedIndex(u8::MAX)
    }

    #[inline]
    const fn option(self) -> Option<u8> {
        if self.0 == u8::MAX {
            None
        } else {
            Some(self.0)
        }
    }
}

#[derive(Debug)]
pub struct Node<T> {
    val: MaybeUninit<T>,
    next: LinkedIndex,
}

pub struct LinkedList<T, Kind, N>
where
    T: PartialOrd,
    Kind: kind::Kind,
    N: ArrayLength<Node<T>>,
{
    list: MaybeUninit<GenericArray<Node<T>, N>>,
    // data_list: MaybeUninit<[T; N]>,
    // index_list: [Option<usize>; N],
    head: LinkedIndex,
    free: LinkedIndex,
    _kind: PhantomData<Kind>,
}

impl<T, Kind, N> LinkedList<T, Kind, N>
where
    T: PartialOrd,
    Kind: kind::Kind,
    N: ArrayLength<Node<T>>,
{
    /// Internal helper to not do pointer arithmetic all over the place.
    #[inline]
    fn node_at(&self, index: usize) -> &Node<T> {
        // Safety: The entire `self.list` is initialized in `new`, which makes this safe.
        unsafe { &*(self.list.as_ptr() as *const Node<T>).add(index) }
    }

    /// Internal helper to not do pointer arithmetic all over the place.
    #[inline]
    fn node_at_mut(&mut self, index: usize) -> &mut Node<T> {
        // Safety: The entire `self.list` is initialized in `new`, which makes this safe.
        unsafe { &mut *(self.list.as_mut_ptr() as *mut Node<T>).add(index) }
    }

    /// Internal helper to not do pointer arithmetic all over the place.
    #[inline]
    fn write_data_in_node_at(&mut self, index: usize, data: T) {
        unsafe {
            self.node_at_mut(index).val.as_mut_ptr().write(data);
        }
    }

    /// Internal helper to not do pointer arithmetic all over the place.
    #[inline]
    fn read_data_in_node_at(&self, index: usize) -> &T {
        unsafe { &*self.node_at(index).val.as_ptr() }
    }

    /// Internal helper to not do pointer arithmetic all over the place.
    #[inline]
    fn extract_data_in_node_at(&mut self, index: usize) -> T {
        unsafe { self.node_at(index).val.as_ptr().read() }
    }

    /// Internal helper to not do pointer arithmetic all over the place.
    /// Safety: This can overwrite existing allocated nodes if used improperly, meaning their
    /// `Drop` methods won't run.
    #[inline]
    unsafe fn write_node_at(&mut self, index: usize, node: Node<T>) {
        (self.list.as_mut_ptr() as *mut Node<T>)
            .add(index)
            .write(node)
    }

    pub fn new() -> Self {
        let mut list = LinkedList {
            list: MaybeUninit::uninit(),
            head: LinkedIndex::none(),
            free: unsafe { LinkedIndex::new_unchecked(0) },
            _kind: PhantomData,
        };

        let len = N::to_u8();
        let mut free = 0;

        while free < len - 1 {
            unsafe {
                list.write_node_at(
                    free as usize,
                    Node {
                        val: MaybeUninit::uninit(),
                        next: LinkedIndex::new_unchecked(free + 1),
                    },
                );
            }
            free += 1;
        }

        unsafe {
            list.write_node_at(
                free as usize,
                Node {
                    val: MaybeUninit::uninit(),
                    next: LinkedIndex::none(),
                },
            );
        }

        list
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if let Some(new) = self.free.option() {
            self.write_data_in_node_at(new as usize, value);
            self.free = self.node_at(new as usize).next;

            if let Some(head) = self.head.option() {
                if self
                    .read_data_in_node_at(head as usize)
                    .partial_cmp(self.read_data_in_node_at(new as usize))
                    != Kind::ordering()
                {
                    // Replace head
                    self.node_at_mut(new as usize).next = self.head;
                    self.head = unsafe { LinkedIndex::new_unchecked(new) };
                } else {
                    let mut current = head;

                    while let Some(next) = self.node_at(current as usize).next.option() {
                        if self
                            .read_data_in_node_at(next as usize)
                            .partial_cmp(self.read_data_in_node_at(new as usize))
                            != Kind::ordering()
                        {
                            break;
                        }

                        current = next;
                    }

                    self.node_at_mut(new as usize).next = self.node_at(current as usize).next;
                    self.node_at_mut(current as usize).next =
                        unsafe { LinkedIndex::new_unchecked(new) };
                }
            } else {
                self.node_at_mut(new as usize).next = self.head;
                self.head = unsafe { LinkedIndex::new_unchecked(new) };
            }

            Ok(())
        } else {
            Err(value)
        }

        // Ptr version:
        // if self.free.is_null() {
        //     return Err(value);
        // }

        // let new = self.free;
        // (*new).val = value;

        // self.free = (*self.free).next;

        // if self.head.is_null() || (*self.head).val <= value {
        //     (*new).next = self.head;
        //     self.head = new;
        // } else {
        //     let mut current = self.head;

        //     while !(*current).next.is_null() {
        //         if (*(*current).next).val < value {
        //             break;
        //         }
        //         current = (*current).next;
        //     }

        //     (*new).next = (*current).next;
        //     (*current).next = new;
        // }
    }

    pub fn head(&self) -> Option<&T> {
        self.head
            .option()
            .map(|head| self.read_data_in_node_at(head as usize))

        // if !self.head.is_null() {
        //     Some((*self.head).val)
        // } else {
        //     None
        // }
    }

    pub fn pop(&mut self) -> Result<T, ()> {
        if let Some(head) = self.head.option() {
            let current = head;
            self.head = self.node_at(head as usize).next;
            self.node_at_mut(current as usize).next = self.free;
            self.free = unsafe { LinkedIndex::new_unchecked(current) };

            Ok(self.extract_data_in_node_at(current as usize))
        } else {
            Err(())
        }

        // let current = self.head;
        //
        // if !self.head.is_null() {
        //     self.head = (*self.head).next;
        //     (*current).next = self.free;
        //     self.free = current;
        //
        //     Ok(())
        // } else {
        //     Err(())
        // }
    }

    pub fn full(&self) -> bool {
        self.free.option().is_none()
    }

    pub fn empty(&self) -> bool {
        self.head.option().is_none()
    }
}

impl<T, Kind, N> fmt::Debug for LinkedList<T, Kind, N>
where
    T: PartialOrd + core::fmt::Debug,
    Kind: kind::Kind,
    N: ArrayLength<Node<T>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LinkedList")
            .field("head", &self.head)
            .field("free", &self.free)
            .field(
                "list",
                &format_args!("{:#?}", unsafe {
                    std::slice::from_raw_parts(self.list.as_ptr() as *mut Node<u32>, 8)
                }),
            )
            .finish()
    }
}

pub struct Min;
pub struct Max;

/// Sealed traits and implementations for `binary_heap`
pub mod kind {
    use super::{Max, Min};
    use core::cmp::Ordering;

    /// The binary heap kind: min-heap or max-heap
    pub unsafe trait Kind {
        #[doc(hidden)]
        fn ordering() -> Option<Ordering>;
    }

    unsafe impl Kind for Min {
        fn ordering() -> Option<Ordering> {
            Some(Ordering::Less)
        }
    }

    unsafe impl Kind for Max {
        fn ordering() -> Option<Ordering> {
            Some(Ordering::Greater)
        }
    }
}
