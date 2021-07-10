use super::*;

use std::thread;
use std::vec::Vec;

use rand::{thread_rng, RngCore};

fn list_from<T: Clone>(v: &[T]) -> LinkedList<T> {
    v.iter().cloned().collect()
}

pub fn check_links<T>(list: &LinkedList<T>) {
    unsafe {
        let mut len = 0;
        let mut last_ptr: Option<&Node<T>> = None;
        let mut node_ptr: &Node<T>;
        match list.head {
            None => {
                // tail node should also be None.
                assert!(list.tail.is_none());
                assert_eq!(0, list.len);
                return;
            }
            Some(node) => node_ptr = &*node.as_ptr(),
        }
        loop {
            match node_ptr.xor(last_ptr.map(NonNull::from)) {
                Some(next) => {
                    last_ptr = Some(node_ptr);
                    node_ptr = &*next.as_ptr();
                    len += 1;
                }
                None => {
                    len += 1;
                    break;
                }
            }
        }

        // verify that the tail node points to the last node.
        let tail = list.tail.as_ref().expect("some tail node").as_ref();
        assert_eq!(tail as *const Node<T>, node_ptr as *const Node<T>);
        // check that len matches interior links.
        assert_eq!(len, list.len);
    }
}

#[test]
fn test_append() {
    // Empty to empty
    {
        let mut m = LinkedList::<i32>::new();
        let mut n = LinkedList::new();
        m.append(&mut n);
        check_links(&m);
        assert_eq!(m.len(), 0);
        assert_eq!(n.len(), 0);
    }
    // Non-empty to empty
    {
        let mut m = LinkedList::new();
        let mut n = LinkedList::new();
        n.push_back(2);
        m.append(&mut n);
        check_links(&m);
        assert_eq!(m.len(), 1);
        assert_eq!(m.pop_back(), Some(2));
        assert_eq!(n.len(), 0);
        check_links(&m);
    }
    // Empty to non-empty
    {
        let mut m = LinkedList::new();
        let mut n = LinkedList::new();
        m.push_back(2);
        m.append(&mut n);
        check_links(&m);
        assert_eq!(m.len(), 1);
        assert_eq!(m.pop_back(), Some(2));
        check_links(&m);
    }

    // Non-empty to non-empty
    let v = vec![1, 2, 3, 4, 5];
    let u = vec![9, 8, 1, 2, 3, 4, 5];
    let mut m = list_from(&v);
    let mut n = list_from(&u);
    m.append(&mut n);
    check_links(&m);
    let mut sum = v;
    sum.extend_from_slice(&u);
    assert_eq!(sum.len(), m.len());
    for elt in sum {
        assert_eq!(m.pop_front(), Some(elt))
    }
    assert_eq!(n.len(), 0);
    // Let's make sure it's working properly, since we
    // did some direct changes to private members.
    n.push_back(3);
    assert_eq!(n.len(), 1);
    assert_eq!(n.pop_front(), Some(3));
    check_links(&n);
}
