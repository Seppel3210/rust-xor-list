//! This crate implements an xor doubly-linked list i.e. the `previous` and `next` pointers are
//! xored together in the lists nodes.
//! Otherwise this implementation is mostly analogous to `alloc::collections::LinkedList`
#![cfg_attr(not(test), no_std)]
extern crate alloc;

use alloc::boxed::Box;
use core::iter::FromIterator;
use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct LinkedList<E> {
    head: Option<NonNull<Node<E>>>,
    tail: Option<NonNull<Node<E>>>,
    len: usize,
    phantom: PhantomData<Box<Node<E>>>,
}

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

    fn other_ptr(&self, first: Option<NonNull<Self>>) -> Option<NonNull<Self>> {
        let first = first.map(|nn| nn.as_ptr() as usize).unwrap_or(0);
        let other = (self.prev_x_next ^ first) as *mut Self;
        NonNull::new(other)
    }

    fn xor(&mut self, other: Option<NonNull<Self>>) {
        let other = other.map(|nn| nn.as_ptr() as usize).unwrap_or(0);
        self.prev_x_next ^= other;
    }

    fn into_element(self: Box<Self>) -> E {
        self.element
    }
}

impl<E> LinkedList<E> {
    fn push_front_node(&mut self, mut node: Box<Node<E>>) {
        unsafe {
            node.xor(self.head);
            let node = Some(Box::leak(node).into());
            match self.head {
                None => self.tail = node,
                Some(head) => (*head.as_ptr()).xor(node),
            }
            self.head = node;
            self.len += 1;
        }
    }

    fn pop_front_node(&mut self) -> Option<Box<Node<E>>> {
        self.head.map(|node_ptr| unsafe {
            let node = Box::from_raw(node_ptr.as_ptr());
            self.head = node.other_ptr(None);

            match self.head {
                None => self.tail = None,
                Some(head) => (*head.as_ptr()).xor(Some(node_ptr)),
            }
            self.len -= 1;
            node
        })
    }

    fn push_back_node(&mut self, mut node: Box<Node<E>>) {
        unsafe {
            node.xor(self.tail);
            let node = Some(Box::leak(node).into());
            match self.tail {
                None => self.head = node,
                Some(tail) => (*tail.as_ptr()).xor(node),
            }
            self.tail = node;
            self.len += 1;
        }
    }

    fn pop_back_node(&mut self) -> Option<Box<Node<E>>> {
        self.tail.map(|node_ptr| unsafe {
            let node = Box::from_raw(node_ptr.as_ptr());
            self.tail = node.other_ptr(None);

            match self.tail {
                None => self.head = None,
                Some(tail) => (*tail.as_ptr()).xor(Some(node_ptr)),
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
                        tail.as_mut().xor(Some(other_head));
                        other_head.as_mut().xor(Some(tail));
                    }

                    self.tail = other.tail.take();
                    self.len += mem::replace(&mut other.len, 0);
                }
            }
        }
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
