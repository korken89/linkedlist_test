use std::fmt;
use std::marker::{PhantomData, PhantomPinned};
use std::mem::MaybeUninit;
use std::ptr;

#[derive(Debug, Clone, Copy)]
struct Node<T> {
    val: T,
    next: *mut Node<T>,
}

impl<T> Default for Node<T>
where
    T: Default,
{
    fn default() -> Self {
        Node {
            val: T::default(),
            next: ptr::null_mut(),
        }
    }
}

struct LinkedList {
    list: [MaybeUninit<Node<i32>>; 8],
    head: *mut Node<i32>,
    free: *mut Node<i32>,
    _marker: PhantomData<*const ()>,
    _pinned: PhantomPinned,
}

impl fmt::Debug for LinkedList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LinkedList")
            .field("head", &self.head)
            .field("free", &self.free)
            .field(
                "list",
                &format_args!("{:#?}", unsafe {
                    std::slice::from_raw_parts(self.list.as_ptr() as *mut Node<i32>, 7)
                }),
            )
            .finish()
    }
}

impl LinkedList {
    const fn new() -> Self {
        LinkedList {
            list: [MaybeUninit::uninit(); 8],
            head: ptr::null_mut(),
            free: ptr::null_mut(),
            _marker: PhantomData,
            _pinned: PhantomPinned,
        }
    }

    unsafe fn init(&mut self) {
        let mut list = self.list.as_mut_ptr() as *mut Node<i32>;
        self.free = list;

        for n in &mut self.list[..6] {
            list = list.add(1);
            n.as_mut_ptr().write(Node { val: 0, next: list });
        }

        list.add(1).write(Node {
            val: 0,
            next: ptr::null_mut(),
        });
    }

    unsafe fn push(&mut self, value: i32) -> Result<(), i32> {
        if self.free.is_null() {
            return Err(value);
        }

        let new = self.free;
        (*new).val = value;

        self.free = (*self.free).next;

        if self.head.is_null() || (*self.head).val <= value {
            (*new).next = self.head;
            self.head = new;
        } else {
            let mut current = self.head;

            while !(*current).next.is_null() {
                if (*(*current).next).val < value {
                    break;
                }
                current = (*current).next;
            }

            (*new).next = (*current).next;
            (*current).next = new;
        }

        Ok(())
    }

    unsafe fn head(&self) -> Option<i32> {
        if !self.head.is_null() {
            Some((*self.head).val)
        } else {
            None
        }
    }

    unsafe fn pop(&mut self) -> Result<(), ()> {
        let current = self.head;

        if !self.head.is_null() {
            self.head = (*self.head).next;
            (*current).next = self.free;
            self.free = current;

            Ok(())
        } else {
            Err(())
        }
    }
}

static mut LL: LinkedList = LinkedList::new();

fn main() {
    unsafe {
        let ll = &mut LL;

        ll.init();

        ll.push(1).ok();
        ll.push(70).ok();
        ll.push(2).ok();
        ll.push(3).ok();
        ll.push(7).ok();
        ll.push(5).ok();
        ll.push(6).ok();

        // if let Err(v) = ll.push(8) {
        //     println!("Rejected {}", v);
        // }

        println!("{:#?}", ll);

        println!("Head: {:?}", ll.head());
        ll.pop().ok();
        println!("Head: {:?}", ll.head());
        ll.pop().ok();
        println!("Head: {:?}", ll.head());
        ll.pop().ok();
        println!("Head: {:?}", ll.head());
        ll.pop().ok();
        println!("Head: {:?}", ll.head());
        ll.pop().ok();
        println!("Head: {:?}", ll.head());
        ll.pop().ok();
        println!("Head: {:?}", ll.head());
        ll.pop().ok();
        println!("Head: {:?}", ll.head());
        ll.pop().ok();
        println!("Head: {:?}", ll.head());
    }
}
