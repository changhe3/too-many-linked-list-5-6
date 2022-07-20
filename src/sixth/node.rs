use std::{
    fmt::Debug,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Not,
    ptr::{self, NonNull},
};

#[cfg(feature = "debug-alloc")]
use std::backtrace::Backtrace;

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

    pub fn is_dummy(self) -> bool {
        self.prev() == self.next() && self.prev() == self
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

    pub fn get_raw(self) -> Option<NonNull<T>> {
        self.is_dummy()
            .not()
            .then(|| unsafe { self.get_raw_unchecked() })
    }

    pub unsafe fn get<'a>(self) -> Option<&'a T> {
        self.get_raw().map(|ptr| ptr.as_ref())
    }

    pub unsafe fn get_mut<'a>(self) -> Option<&'a mut T> {
        self.get_raw().map(|mut ptr| ptr.as_mut())
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

    pub unsafe fn dealloc(self) -> Option<(Self, T, Self)> {
        self.is_dummy().not().then(|| {
            let Node {
                prev, next, item, ..
            } = self.dealloc_raw();
            (prev, item.assume_init(), next)
        })
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
