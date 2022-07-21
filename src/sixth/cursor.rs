use std::{marker::PhantomData, ops::Rem, ptr::NonNull};

use super::{node::NodePtr, LinkedList};

pub(crate) struct RawCursor<T> {
    pub(crate) node: Option<NodePtr<T>>,
    pub(crate) index: usize,
}

pub struct Cursor<'a, T> {
    pub(crate) inner: RawCursor<T>,
    pub(crate) list: &'a LinkedList<T>,
}

pub struct CursorMut<'a, T> {
    pub(crate) inner: RawCursor<T>,
    pub(crate) list: &'a mut LinkedList<T>,
}

impl<T> RawCursor<T> {
    fn reset_index(&mut self, list: &LinkedList<T>) {
        if self.index > list.len() {
            self.index %= list.len() + 1;
        }
    }

    pub fn index(&self, list: &LinkedList<T>) -> Option<usize> {
        let index = self.index;
        (index != list.len()).then_some(index)
    }

    pub fn move_next(&mut self, list: &LinkedList<T>) {
        if let Some(node) = self.node.as_mut() {
            *node = node.next();
            self.index = self.index.wrapping_add(1);
            self.reset_index(list);
        }
    }

    pub fn move_prev(&mut self, list: &LinkedList<T>) {
        if let Some(node) = self.node.as_mut() {
            *node = node.prev();
            self.index = self.index.wrapping_sub(1);
            self.reset_index(list);
        }
    }

    pub fn current<'a>(&self, list: &'a LinkedList<T>) -> Option<&'a T> {
        self.node?.get(list)
    }

    pub fn current_mut<'a>(&self, list: &'a mut LinkedList<T>) -> Option<&'a mut T> {
        self.node?.get_mut(list)
    }
}

impl<'a, T> Cursor<'a, T> {
    pub fn index(&self) -> Option<usize> {
        self.inner.index(self.list)
    }
}

impl<'a, T> CursorMut<'a, T> {
    pub fn index(&self) -> Option<usize> {
        self.inner.index(self.list)
    }
}
