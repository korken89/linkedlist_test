pub mod linked_list;

use linked_list::{LinkedList, LinkedIndexU16, Max, Min};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct S(pub u32);

impl Drop for S {
    fn drop(&mut self) {
        println!("Dropping {}", self.0)
    }
}

fn main() {
    let mut ll: LinkedList<S, LinkedIndexU16, Min, 8> = LinkedList::new_u16();

    ll.push(S(1)).unwrap();
    ll.push(S(70)).unwrap();
    ll.push(S(2)).unwrap();
    ll.push(S(0)).unwrap();
    ll.push(S(7)).unwrap();
    ll.push(S(5)).unwrap();
    ll.push(S(6)).unwrap();
    ll.push(S(11)).unwrap();

    if let Err(v) = ll.push(S(8)) {
        println!("Rejected {}", v.0);
    } else {
        panic!("Sould have been rejected")
    }

    // println!("{:#?}", ll);

    println!("List: {:?}", ll);
    let mut find = ll
        .find_mut(|v| {
            println!("v: {}", v.0);
            *v == S(5)
        })
        .unwrap();

    find.0 += 1000;
    find.finish();

    println!("List: {:?}", ll);

    println!("");

    println!("Iter");
    for v in ll.iter() {
        println!("v: {}", v.0);
    }

    println!("");

    ll.find_mut(|v| v.0 == 1005).unwrap().pop();

    // println!("pop");
    // ll.pop();
    // println!("pop after");

    // println!("Head: {:?}", ll.peek());
    // println!("Pop: {:?}", ll.pop());
    // println!("Head: {:?}", ll.peek());
    // println!("Pop: {:?}", ll.pop());
    // println!("Head: {:?}", ll.peek());
    // println!("Pop: {:?}", ll.pop());
    // println!("Head: {:?}", ll.peek());
    // println!("Pop: {:?}", ll.pop());
    // println!("Head: {:?}", ll.peek());
    // println!("Pop: {:?}", ll.pop());
    // println!("Head: {:?}", ll.peek());
    // println!("Pop: {:?}", ll.pop());
    // println!("Head: {:?}", ll.peek());
    // println!("Pop: {:?}", ll.pop());
    // println!("Head: {:?}", ll.peek());
    // println!("Pop: {:?}", ll.pop());
    // println!("Head: {:?}", ll.peek());
    // println!("Pop: {:?}", ll.pop());
}
