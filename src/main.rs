pub mod linked_list;

use generic_array::typenum::consts::*;
use linked_list::{LinkedList, Max, Min};

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
    // println!experimental(experimental"{:#?}", ll);
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

    println!("List: {:?}", ll);
    let mut find = ll.find_mut(|v| {
        println!("v: {}", *v);
        *v == 5
    }).unwrap();

    *find += 1000;
    find.finish();

    println!("List: {:?}", ll);

    println!("");

    println!("Iter");
    for v in ll.iter() {
        println!("v: {}", v);
    }

    println!("");

    println!("Head: {:?}", ll.peek());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.peek());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.peek());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.peek());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.peek());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.peek());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.peek());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.peek());
    println!("Pop: {:?}", ll.pop());
    println!("Head: {:?}", ll.peek());
    println!("Pop: {:?}", ll.pop());
}
