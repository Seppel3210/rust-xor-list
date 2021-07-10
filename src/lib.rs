//! This crate implements an xor doubly-linked list i.e. the `previous` and `next` pointers are
//! xored together in the lists nodes.
//! Otherwise this implementation is mostly analogous to `alloc::collections::LinkedList`
#![cfg_attr(not(test), no_std)]
extern crate alloc;

use alloc::boxed::Box;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::iter::FromIterator;
use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;

#[cfg(test)]
mod tests;

pub struct LinkedList<E> {
    head: Option<NonNull<Node<E>>>,
    tail: Option<NonNull<Node<E>>>,
    len: usize,
    phantom: PhantomData<Box<Node<E>>>,
}

impl<E> LinkedList<E> {
    fn push_front_node(&mut self, mut node: Box<Node<E>>) {
        unsafe {
            node.xor_assign(self.head);
            let node = Some(Box::leak(node).into());
            match self.head {
                None => self.tail = node,
                Some(head) => (*head.as_ptr()).xor_assign(node),
            }
            self.head = node;
            self.len += 1;
        }
    }

    fn pop_front_node(&mut self) -> Option<Box<Node<E>>> {
        self.head.map(|node_ptr| unsafe {
            let node = Box::from_raw(node_ptr.as_ptr());
            self.head = node.xor(None);

            match self.head {
                None => self.tail = None,
                Some(head) => (*head.as_ptr()).xor_assign(Some(node_ptr)),
            }
            self.len -= 1;
            node
        })
    }

    fn push_back_node(&mut self, mut node: Box<Node<E>>) {
        unsafe {
            node.xor_assign(self.tail);
            let node = Some(Box::leak(node).into());
            match self.tail {
                None => self.head = node,
                Some(tail) => (*tail.as_ptr()).xor_assign(node),
            }
            self.tail = node;
            self.len += 1;
        }
    }

    fn pop_back_node(&mut self) -> Option<Box<Node<E>>> {
        self.tail.map(|node_ptr| unsafe {
            let node = Box::from_raw(node_ptr.as_ptr());
            self.tail = node.xor(None);

            match self.tail {
                None => self.head = None,
                Some(tail) => (*tail.as_ptr()).xor_assign(Some(node_ptr)),
            }
            self.len -= 1;
            node
        })
    }
}

impl<E> LinkedList<E> {
    pub fn new() -> Self {
        LinkedList {
            head: None,
            tail: None,
            len: 0,
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push_front(&mut self, elem: E) {
        self.push_front_node(Box::new(Node::new(elem)));
    }

    pub fn pop_front(&mut self) -> Option<E> {
        self.pop_front_node().map(Node::into_element)
    }

    pub fn push_back(&mut self, elem: E) {
        self.push_back_node(Box::new(Node::new(elem)));
    }

    pub fn pop_back(&mut self) -> Option<E> {
        self.pop_back_node().map(Node::into_element)
    }

    pub fn append(&mut self, other: &mut Self) {
        match self.tail {
            None => mem::swap(self, other),
            Some(mut tail) => {
                // `as_mut` is okay here becaute we have exclusive access to the
                // entirety of both lists.
                if let Some(mut other_head) = other.head.take() {
                    unsafe {
                        tail.as_mut().xor_assign(Some(other_head));
                        other_head.as_mut().xor_assign(Some(tail));
                    }

                    self.tail = other.tail.take();
                    self.len += mem::replace(&mut other.len, 0);
                }
            }
        }
    }
    pub fn iter(&self) -> Iter<'_, E> {
        Iter {
            head: self.head,
            prev_head: None,
            tail: self.tail,
            prev_tail: None,
            len: self.len,
            marker: PhantomData,
        }
    }
}

impl<E> Default for LinkedList<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E> FromIterator<E> for LinkedList<E> {
    fn from_iter<I: IntoIterator<Item = E>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

impl<E> Extend<E> for LinkedList<E> {
    fn extend<I: IntoIterator<Item = E>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |elem| self.push_back(elem));
    }
}

impl<E: PartialEq> PartialEq for LinkedList<E> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }

    fn ne(&self, other: &Self) -> bool {
        self.len() != other.len() || self.iter().ne(other)
    }
}

impl<E: Eq> Eq for LinkedList<E> {}

impl<E: PartialOrd> PartialOrd for LinkedList<E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<E: Ord> Ord for LinkedList<E> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other)
    }
}

impl<E: Clone> Clone for LinkedList<E> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
    // TODO: fn clone_from
}

impl<E: fmt::Debug> fmt::Debug for LinkedList<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<E: Hash> Hash for LinkedList<E> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for elt in self {
            elt.hash(state);
        }
    }
}

impl<E> Drop for LinkedList<E> {
    fn drop(&mut self) {
        struct DropGuard<'a, E>(&'a mut LinkedList<E>);

        impl<'a, E> Drop for DropGuard<'a, E> {
            fn drop(&mut self) {
                // Continuo the same loop we do below. This only runs when a destructor
                // has panicked. If another one panics this will abort.
                while self.0.pop_front_node().is_some() {}
            }
        }

        while let Some(node) = self.pop_front_node() {
            let guard = DropGuard(self);
            drop(node);
            mem::forget(guard);
        }
    }
}

unsafe impl<E: Send> Send for LinkedList<E> {}
unsafe impl<E: Sync> Sync for LinkedList<E> {}

unsafe impl<E: Send> Send for Iter<'_, E> {}
unsafe impl<E: Sync> Sync for Iter<'_, E> {}

#[derive(Debug)]
struct Node<E> {
    prev_x_next: usize,
    element: E,
}

impl<E> Node<E> {
    fn new(element: E) -> Self {
        Node {
            prev_x_next: 0,
            element,
        }
    }

    fn xor(&self, other: Option<NonNull<Self>>) -> Option<NonNull<Self>> {
        let other = other.map(|nn| nn.as_ptr() as usize).unwrap_or(0);
        let result = other ^ self.prev_x_next;
        NonNull::new(result as *mut _)
    }

    fn xor_assign(&mut self, other: Option<NonNull<Self>>) {
        let other = other.map(|nn| nn.as_ptr() as usize).unwrap_or(0);
        self.prev_x_next ^= other;
    }

    fn into_element(self: Box<Self>) -> E {
        self.element
    }
}

pub struct Iter<'a, E: 'a> {
    head: Option<NonNull<Node<E>>>,
    prev_head: Option<NonNull<Node<E>>>,
    tail: Option<NonNull<Node<E>>>,
    prev_tail: Option<NonNull<Node<E>>>,
    len: usize,
    marker: PhantomData<&'a Node<E>>,
}

impl<E: fmt::Debug> fmt::Debug for Iter<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let head = self.head.clone();
        let tail = self.tail.clone();
        head.map(|nn| unsafe {
            (&mut *nn.as_ptr()).xor_assign(self.prev_head);
        });
        tail.map(|nn| unsafe {
            (&mut *nn.as_ptr()).xor_assign(self.prev_tail);
        });

        f.debug_tuple("Iter")
            .field(&*mem::ManuallyDrop::new(LinkedList {
                head,
                tail,
                len: self.len,
                phantom: PhantomData,
            }))
            .field(&self.len)
            .finish()
    }
}

impl<T> Clone for Iter<'_, T> {
    fn clone(&self) -> Self {
        Iter { ..*self }
    }
}

impl<'a, E> Iterator for Iter<'a, E> {
    type Item = &'a E;

    fn next(&mut self) -> Option<&'a E> {
        if self.len == 0 {
            None
        } else {
            self.head.map(|node| unsafe {
                let node = &*node.as_ptr();
                self.len -= 1;
                self.head = node.xor(self.prev_head);
                self.prev_head = Some(node.into());
                &node.element
            })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    fn last(mut self) -> Option<&'a E> {
        self.next_back()
    }
}

impl<'a, E> DoubleEndedIterator for Iter<'a, E> {
    fn next_back(&mut self) -> Option<&'a E> {
        if self.len == 0 {
            None
        } else {
            self.tail.map(|node| unsafe {
                let node = &*node.as_ptr();
                self.len -= 1;
                self.tail = node.xor(self.prev_tail);
                self.prev_tail = Some(node.into());
                &node.element
            })
        }
    }
}

impl<'a, E> IntoIterator for &'a LinkedList<E> {
    type Item = &'a E;
    type IntoIter = Iter<'a, E>;

    fn into_iter(self) -> Iter<'a, E> {
        self.iter()
    }
}
