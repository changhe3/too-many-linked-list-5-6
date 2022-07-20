use std::marker::PhantomData;

use self::node::NodePtr;

mod node;

#[derive(Debug)]
pub struct LinkedList<T> {
    dummy: Option<NodePtr<T>>,
    len: usize,
    _phantom: PhantomData<T>,
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self {
            dummy: None,
            len: 0,
            _phantom: PhantomData,
        }
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn init(&mut self) -> NodePtr<T> {
        *self.dummy.get_or_insert_with(|| NodePtr::dummy())
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push_front(&mut self, elem: T) {
        let dummy = self.init();
        let head = dummy.next();
        let new_head = NodePtr::alloc(dummy, elem, head);
        head.set_prev(new_head);
        dummy.set_next(new_head);

        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let dummy = self.dummy?;
        let (_, elem, new_head) = unsafe { dummy.next().dealloc()? };
        dummy.set_next(new_head);
        new_head.set_prev(dummy);

        self.len -= 1;
        Some(elem)
    }

    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        self.clear();
        unsafe {
            self.dummy.map(|ptr| ptr.dealloc_raw());
        }
    }
}

#[cfg(test)]
mod test {
    use super::{node::NodePtr, LinkedList};

    #[test]
    fn test_empty() {
        LinkedList::<i32>::new();
    }

    fn test_node() {
        let dummy = NodePtr::dummy();
        let node = NodePtr::alloc(dummy, 100, dummy);

        dummy.set_next(node);
        dummy.set_prev(node);

        let (_, elem, _) = unsafe { node.dealloc() }.unwrap();
        unsafe {
            Box::from_raw(dummy.as_ptr());
        }
        assert_eq!(elem, 100);
    }

    #[test]
    fn test_small() {
        let mut list = LinkedList::new();
        list.push_front(10);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn test_basic_front() {
        let mut list = LinkedList::new();

        // Try to break an empty list
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Try to break a one item list
        list.push_front(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Mess around
        list.push_front(10);
        assert_eq!(list.len(), 1);
        list.push_front(20);
        assert_eq!(list.len(), 2);
        list.push_front(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq!(list.len(), 2);
        list.push_front(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        list.push_front(10);
        list.push_front(20);
    }
}
