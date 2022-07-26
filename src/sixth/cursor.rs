use super::{node::NodePtr, LinkedList};

pub struct RawCursor<T> {
    pub(crate) node: Option<NodePtr<T>>,
    pub(crate) index: usize,
}

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
}
