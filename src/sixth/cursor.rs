use std::ops::Not;

use super::{node::NodePtr, LinkedList};

pub(crate) struct RawCursor<T> {
    pub(crate) node: Option<NodePtr<T>>,
    pub(crate) index: usize,
}

impl<T> Clone for RawCursor<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node,
            index: self.index,
        }
    }
}

impl<T> Copy for RawCursor<T> {}

impl<T> RawCursor<T> {
    fn new(list: &LinkedList<T>) -> Self {
        Self {
            node: list.dummy,
            index: list.len,
        }
    }

    fn set_index(&mut self, index: usize, list: &LinkedList<T>) {
        self.index = index;
        if self.index > list.len {
            self.index %= list.len + 1;
        }
    }

    fn index_add(&mut self, inc: usize, list: &LinkedList<T>) {
        self.set_index(self.index.wrapping_add(inc), list);
    }

    fn index_sub(&mut self, dec: usize, list: &LinkedList<T>) {
        self.set_index(self.index.wrapping_sub(dec), list);
    }

    fn index(&self, list: &LinkedList<T>) -> Option<usize> {
        (self.index != list.len).then_some(self.index)
    }

    fn move_next(&mut self, list: &LinkedList<T>) {
        if let Some(node) = self.node.as_mut() {
            *node = node.next();
            self.index_add(1, list);
        }
    }

    fn move_prev(&mut self, list: &LinkedList<T>) {
        if let Some(node) = self.node.as_mut() {
            *node = node.prev();
            self.index_sub(1, list);
        }
    }

    unsafe fn current<'a>(&self, list: &'a LinkedList<T>) -> Option<&'a T> {
        self.node?.get(list)
    }

    unsafe fn current_mut<'a>(&mut self, list: &'a mut LinkedList<T>) -> Option<&'a mut T> {
        self.node?.get_mut(list)
    }

    unsafe fn peek_next<'a>(&self, list: &'a LinkedList<T>) -> Option<&'a T> {
        self.node?.next().get(list)
    }

    unsafe fn peek_next_mut<'a>(&self, list: &'a mut LinkedList<T>) -> Option<&'a mut T> {
        self.node?.next().get_mut(list)
    }

    unsafe fn peek_prev<'a>(&self, list: &'a LinkedList<T>) -> Option<&'a T> {
        self.node?.prev().get(list)
    }

    unsafe fn peek_prev_mut<'a>(&self, list: &'a mut LinkedList<T>) -> Option<&'a mut T> {
        self.node?.prev().get_mut(list)
    }

    fn init(&mut self, list: &mut LinkedList<T>) -> NodePtr<T> {
        *self.node.get_or_insert_with(|| list.init())
    }

    unsafe fn insert_after(&mut self, item: T, list: &mut LinkedList<T>) {
        let node = self.init(list);
        node.insert_after(item, list);

        if node.is_dummy(list) {
            self.index_add(1, list);
        }
    }

    unsafe fn insert_before(&mut self, item: T, list: &mut LinkedList<T>) {
        let node = self.init(list);
        node.insert_before(item, list);

        if !node.is_dummy(list) {
            self.index_add(1, list);
        }
    }

    unsafe fn remove_current(&mut self, list: &mut LinkedList<T>) -> Option<T> {
        let node = self.node.as_mut()?;
        let next = node.next();

        let item = node.pop(list)?;
        *node = next;
        Some(item)
    }

    unsafe fn remove_current_as_list(&mut self, list: &mut LinkedList<T>) -> Option<LinkedList<T>> {
        let node = self.node.as_mut()?;
        let next = node.next();

        node.is_dummy(list).not().then(|| {
            let list = NodePtr::slice_off_as_list(*node, *node, 1, list);
            *node = next;
            list
        })
    }
}

pub struct Cursor<'a, T> {
    inner: RawCursor<T>,
    list: &'a LinkedList<T>,
}

pub struct CursorMut<'a, T> {
    inner: RawCursor<T>,
    list: &'a mut LinkedList<T>,
}

impl<'a, T> Cursor<'a, T> {
    pub fn index(&self) -> Option<usize> {
        self.inner.index(self.list)
    }

    pub fn move_next(&mut self) {
        self.inner.move_next(self.list)
    }

    pub fn move_prev(&mut self) {
        self.inner.move_prev(self.list)
    }

    pub fn current(&self) -> Option<&T> {
        // Safety:`self.inner` is a node of self.list
        unsafe { self.inner.current(self.list) }
    }

    pub fn peek_next(&self) -> Option<&T> {
        // Safety:`self.inner` is a node of self.list
        unsafe { self.inner.peek_next(self.list) }
    }

    pub fn peek_prev(&self) -> Option<&T> {
        // Safety:`self.inner` is a node of self.list
        unsafe { self.inner.peek_prev(self.list) }
    }
}

impl<'a, T> CursorMut<'a, T> {
    pub fn index(&self) -> Option<usize> {
        self.inner.index(self.list)
    }

    pub fn move_next(&mut self) {
        self.inner.move_next(self.list)
    }

    pub fn move_prev(&mut self) {
        self.inner.move_prev(self.list)
    }

    pub fn current(&mut self) -> Option<&mut T> {
        // Safety:`self.inner` is a node of self.list
        unsafe { self.inner.current_mut(self.list) }
    }

    pub fn peek_next(&mut self) -> Option<&mut T> {
        // Safety:`self.inner` is a node of self.list
        unsafe { self.inner.peek_next_mut(self.list) }
    }

    pub fn peek_prev(&mut self) -> Option<&mut T> {
        // Safety:`self.inner` is a node of self.list
        unsafe { self.inner.peek_prev_mut(self.list) }
    }

    pub fn as_cursor(&self) -> Cursor<'_, T> {
        Cursor {
            inner: self.inner,
            list: self.list,
        }
    }

    pub fn insert_after(&mut self, item: T) {
        unsafe {
            self.inner.insert_after(item, self.list);
        }
    }

    pub fn insert_before(&mut self, item: T) {
        unsafe {
            self.inner.insert_before(item, self.list);
        }
    }

    pub fn remove_current(&mut self) -> Option<T> {
        unsafe { self.inner.remove_current(self.list) }
    }

    // pub fn remove_current_as_list(&mut self) -> Option<LinkedList<T>> {
    //     unsafe {
    //         self.inner.
    //     }
    // }
}
