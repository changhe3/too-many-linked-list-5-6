use std::{
    fmt::Debug,
    mem::MaybeUninit,
    ops::Not,
    ptr::{self, NonNull},
};

#[cfg(feature = "debug-alloc")]
use std::backtrace::Backtrace;

use super::{iter::RawIter, LinkedList};

#[derive(Debug)]
pub struct Node<T> {
    pub(crate) prev: NodePtr<T>,
    pub(crate) next: NodePtr<T>,
    pub(crate) item: MaybeUninit<T>,
}

impl<T> Node<T> {
    pub(crate) fn new(prev: NodePtr<T>, item: MaybeUninit<T>, next: NodePtr<T>) -> Self {
        Self { prev, next, item }
    }
}

pub struct NodePtr<T> {
    ptr: NonNull<Node<T>>,
}

impl<T> Debug for NodePtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodePtr").field("ptr", &self.ptr).finish()
    }
}

impl<T> Clone for NodePtr<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T> Copy for NodePtr<T> {}

impl<T> PartialEq for NodePtr<T> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.as_ptr(), other.as_ptr())
    }
}

impl<T> Eq for NodePtr<T> {}

#[allow(unused)]
impl<T> NodePtr<T> {
    pub unsafe fn dangling() -> Self {
        Self {
            ptr: NonNull::dangling(),
        }
    }

    pub unsafe fn raw_alloc(prev: Self, item: MaybeUninit<T>, next: Self) -> Self {
        let ptr = Box::into_raw(Box::new(Node::new(prev, item, next)));
        let ptr = NonNull::new_unchecked(ptr);

        #[cfg(feature = "debug-alloc")]
        {
            println!(
                "Allocated {} bytes at ptr {:p}: ",
                std::mem::size_of::<Node<T>>(),
                ptr.as_ptr()
            );
            println!("{}\n", Backtrace::capture());
        }

        Self { ptr }
    }

    pub unsafe fn alloc_dangling(item: T) -> Self {
        let dangling = Self::dangling();
        Self::alloc(dangling, item, dangling)
    }

    pub fn alloc(prev: Self, item: T, next: Self) -> Self {
        unsafe { Self::raw_alloc(prev, MaybeUninit::new(item), next) }
    }

    pub fn dummy() -> Self {
        unsafe {
            let dangling = Self::dangling();

            let dummy = Self::raw_alloc(dangling, MaybeUninit::uninit(), dangling);
            dummy.set_prev(dummy);
            dummy.set_next(dummy);

            dummy
        }
    }

    pub fn prev(self) -> Self {
        unsafe { (*self.as_ptr()).prev }
    }

    pub fn set_prev(self, ptr: Self) {
        unsafe {
            (*self.as_ptr()).prev = ptr;
        }
    }

    pub fn next(self) -> Self {
        unsafe { (*self.as_ptr()).next }
    }

    pub fn set_next(self, ptr: Self) {
        unsafe {
            (*self.as_ptr()).next = ptr;
        }
    }

    pub fn link(self, ptr: Self) {
        self.set_next(ptr);
        ptr.set_prev(self);
    }

    pub fn is_dummy(self, list: &LinkedList<T>) -> bool {
        list.dummy.map_or(false, |dummy| self == dummy)
    }

    pub fn as_ptr(self) -> *mut Node<T> {
        self.ptr.as_ptr()
    }

    pub unsafe fn as_ref<'a>(self) -> &'a Node<T> {
        self.ptr.as_ref()
    }

    pub unsafe fn as_mut<'a>(mut self) -> &'a mut Node<T> {
        self.ptr.as_mut()
    }

    pub unsafe fn get_raw(self, list: &LinkedList<T>) -> Option<NonNull<T>> {
        self.is_dummy(list)
            .not()
            .then(|| unsafe { self.get_raw_unchecked() })
    }

    pub unsafe fn get(self, list: &LinkedList<T>) -> Option<&T> {
        self.get_raw(list).map(|ptr| unsafe { ptr.as_ref() })
    }

    pub unsafe fn get_mut(self, list: &mut LinkedList<T>) -> Option<&mut T> {
        self.get_raw(list).map(|mut ptr| unsafe { ptr.as_mut() })
    }

    pub unsafe fn get_raw_unchecked(self) -> NonNull<T> {
        NonNull::new_unchecked((*self.as_ptr()).item.as_mut_ptr())
    }

    pub unsafe fn get_unchecked<'a>(self) -> &'a T {
        self.get_raw_unchecked().as_ref()
    }

    pub unsafe fn get_mut_unchecked<'a>(self) -> &'a mut T {
        self.get_raw_unchecked().as_mut()
    }

    // need to guarantee that self is a node in list
    pub unsafe fn insert_after(self, item: T, list: &mut LinkedList<T>) {
        let new_node = Self::alloc_dangling(item);
        self.splice_after(new_node, new_node, 1, list);
    }

    // need to guarantee that self is a node in list
    pub unsafe fn insert_before(self, item: T, list: &mut LinkedList<T>) {
        let new_node = Self::alloc_dangling(item);
        self.splice_before(new_node, new_node, 1, list);
    }

    // need to guarantee that self is a node in list
    pub unsafe fn pop(self, list: &mut LinkedList<T>) -> Option<T> {
        self.is_dummy(list).not().then(|| self.pop_unchecked(list))
    }

    pub unsafe fn pop_unchecked(self, list: &mut LinkedList<T>) -> T {
        Self::slice_off(self, self, 1, list).next().unwrap()
    }

    // slice off a part of the linked list
    // the slice CANNOT include the dummy node
    pub unsafe fn slice_off(
        front: Self,
        back: Self,
        len: usize,
        list: &mut LinkedList<T>,
    ) -> impl Iterator<Item = T> {
        front.prev().link(back.next());
        list.len = list.len.saturating_sub(len);

        let mut iter = RawIter::new(front, back, len);
        iter.map(|node| {
            let (_, item, _) = node.dealloc_unchecked();
            item
        })
    }

    // slice off a part of the linked list
    // the slice CANNOT include the dummy node
    pub unsafe fn slice_off_as_list(
        front: Self,
        back: Self,
        len: usize,
        list: &mut LinkedList<T>,
    ) -> LinkedList<T> {
        front.prev().link(back.next());
        list.len = list.len.saturating_sub(len);

        let mut res = LinkedList::new();
        res.init().splice_after(front, back, len, &mut res);

        res
    }

    pub unsafe fn splice_after(
        self,
        front: Self,
        back: Self,
        len: usize,
        list: &mut LinkedList<T>,
    ) {
        debug_assert!(list.dummy.is_some());

        let next = self.next();

        self.link(front);
        back.link(next);

        list.len = list.len.saturating_add(len);
    }

    pub unsafe fn splice_before(
        self,
        front: Self,
        back: Self,
        len: usize,
        list: &mut LinkedList<T>,
    ) {
        debug_assert!(list.dummy.is_some());

        let prev = self.prev();

        prev.link(front);
        back.link(self);

        list.len = list.len.saturating_add(len);
    }

    pub unsafe fn dealloc(self, list: &mut LinkedList<T>) -> Option<(Self, T, Self)> {
        self.is_dummy(list).not().then(|| {
            let Node {
                prev, next, item, ..
            } = self.dealloc_raw();
            (prev, item.assume_init(), next)
        })
    }

    pub unsafe fn dealloc_unchecked(self) -> (Self, T, Self) {
        let Node { prev, next, item } = self.dealloc_raw();
        (prev, item.assume_init(), next)
    }

    pub unsafe fn dealloc_raw(self) -> Node<T> {
        #[cfg(feature = "debug-alloc")]
        {
            println!(
                "Deallocated {} bytes at ptr {:p}",
                std::mem::size_of::<Node<T>>(),
                self.as_ptr()
            );
            println!("{}\n", Backtrace::capture());
        }

        *Box::from_raw(self.as_ptr())
    }
}
