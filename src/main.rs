use core::marker::PhantomData;
use core::mem::MaybeUninit;
use generic_array::{ArrayLength, GenericArray};

use core::fmt;
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

#[derive(Debug)]
pub struct Node<T> {
    val: MaybeUninit<T>,
    next: Option<usize>,
}

struct Min;
struct Max;

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

pub struct LinkedList<T, Kind, N>
where
    T: PartialOrd,
    Kind: kind::Kind,
    N: ArrayLength<Node<T>>,
{
    list: MaybeUninit<GenericArray<Node<T>, N>>,
    head: Option<usize>,
    free: Option<usize>,
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
            head: None,
            free: Some(0),
            _kind: PhantomData,
        };

        let len = N::to_usize();
        let mut free = 0;

        while free < len - 1 {
            unsafe {
                list.write_node_at(
                    free,
                    Node {
                        val: MaybeUninit::uninit(),
                        next: Some(free + 1),
                    },
                );
            }
            free += 1;
        }

        unsafe {
            list.write_node_at(
                free,
                Node {
                    val: MaybeUninit::uninit(),
                    next: None,
                },
            );
        }

        list
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if let Some(new) = self.free {
            self.write_data_in_node_at(new, value);
            self.free = self.node_at(new).next;

            if let Some(head) = self.head {
                if self
                    .read_data_in_node_at(head)
                    .partial_cmp(self.read_data_in_node_at(new))
                    != Kind::ordering()
                {
                    // Replace head
                    self.node_at_mut(new).next = self.head;
                    self.head = Some(new);
                } else {
                    let mut current = head;

                    while let Some(next) = self.node_at(current).next {
                        if self
                            .read_data_in_node_at(next)
                            .partial_cmp(self.read_data_in_node_at(new))
                            != Kind::ordering()
                        {
                            break;
                        }

                        current = next;
                    }

                    self.node_at_mut(new).next = self.node_at(current).next;
                    self.node_at_mut(current).next = Some(new);
                }
            } else {
                self.node_at_mut(new).next = self.head;
                self.head = Some(new);
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
        self.head.map(|head| self.read_data_in_node_at(head))

        // if !self.head.is_null() {
        //     Some((*self.head).val)
        // } else {
        //     None
        // }
    }

    pub fn pop(&mut self) -> Result<T, ()> {
        if let Some(head) = self.head {
            let current = head;
            self.head = self.node_at(head).next;
            self.node_at_mut(current).next = self.free;
            self.free = Some(current);

            Ok(self.extract_data_in_node_at(current))
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
        self.free.is_none()
    }

    pub fn empty(&self) -> bool {
        self.head.is_none()
    }
}

use generic_array::typenum::consts::*;

fn main() {
    let mut ll: LinkedList<u32, Max, U8> = LinkedList::new();

    // println!("{:#?}", ll);
    // println!("{:#?}", ll);
    ll.push(1).unwrap();
    ll.push(70).unwrap();
    // println!("{:#?}", ll);
    ll.push(2).unwrap();
    // println!("{:#?}", ll);
    ll.push(0).unwrap();
    // println!("{:#?}", ll);
    ll.push(7).unwrap();
    // println!("{:#?}", ll);
    ll.push(5).unwrap();
    // println!("{:#?}", ll);
    ll.push(6).unwrap();
    // println!("{:#?}", ll);
    ll.push(11).unwrap();
    // println!("{:#?}", ll);

    if let Err(v) = ll.push(8) {
        println!("Rejected {}", v);
    } else {
        panic!("Sould have been rejected")
    }

    // println!("{:#?}", ll);

    println!("Head: {:?}", ll.head());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.head());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.head());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.head());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.head());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.head());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.head());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.head());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.head());
    println!("Pop: {:?}", ll.pop());
}
