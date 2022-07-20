use std::ptr::NonNull;

#[derive(Debug)]
pub struct List<T> {
    inner: Option<Inner<T>>,
}

#[derive(Debug)]
struct Inner<T> {
    head: NonNull<Node<T>>,
    tail: NonNull<Node<T>>,
}

#[derive(Debug)]
struct Node<T> {
    item: T,
    next: Link<T>,
}

type Link<T> = Option<NonNull<Node<T>>>;

impl<T> Default for List<T> {
    fn default() -> Self {
        Self { inner: None }
    }
}

unsafe impl<T> Send for List<T> where Vec<T>: Send {}
unsafe impl<T> Sync for List<T> where Vec<T>: Sync {}

impl<T> List<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, item: T) {
        let new_tail = Box::new(Node { item, next: None });
        let new_tail = unsafe { NonNull::new_unchecked(Box::into_raw(new_tail)) };

        let new_head = if let Some(Inner { head, tail }) = self.inner.take() {
            unsafe {
                (*tail.as_ptr()).next = Some(new_tail);
            }
            head
        } else {
            new_tail
        };

        self.inner = Some(Inner {
            head: new_head,
            tail: new_tail,
        });
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.take().map(|Inner { head, tail }| {
            let Node { item, next } = unsafe { *Box::from_raw(head.as_ptr()) };
            self.inner = next.map(|head| Inner { head, tail });
            item
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|Node { item, next }| {
            self.next = unsafe { next.as_ref().map(|node| node.as_ref()) };
            item
        })
    }
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|Node { item, next }| {
            self.next = unsafe { next.as_mut().map(|node| node.as_mut()) };
            item
        })
    }
}

pub struct IntoIter<T>(List<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<T> List<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self
                .inner
                .as_ref()
                .map(|Inner { head, .. }| unsafe { head.as_ref() }),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self
                .inner
                .as_mut()
                .map(|Inner { head, .. }| unsafe { head.as_mut() }),
        }
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl<T> List<T> {
    pub fn peek(&self) -> Option<&T> {
        self.inner
            .as_ref()
            .map(|Inner { head, .. }| unsafe { &head.as_ref().item })
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.inner
            .as_mut()
            .map(|Inner { head, .. }| unsafe { &mut head.as_mut().item })
    }
}

#[cfg(test)]
mod test {
    use super::List;
    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn miri_food() {
        let mut list = List::new();

        list.push(1);
        list.push(2);
        list.push(3);

        assert!(list.pop() == Some(1));
        list.push(4);
        assert!(list.pop() == Some(2));
        list.push(5);

        assert!(list.peek() == Some(&3));
        list.push(6);
        if let Some(x) = list.peek_mut() {
            *x *= 10;
        }

        assert!(list.peek() == Some(&30));
        assert!(list.pop() == Some(30));

        for item in list.iter_mut() {
            *item *= 100;
        }

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&400));
        assert_eq!(iter.next(), Some(&500));
        assert_eq!(iter.next(), Some(&600));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        assert!(list.pop() == Some(400));
        if let Some(x) = list.peek_mut() {
            *x *= 10;
        }
        assert!(list.peek() == Some(&5000));
        list.push(7);

        // Drop it on the ground and let the dtor exercise itself
    }
}
