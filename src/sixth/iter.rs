use std::{marker::PhantomData, ptr::NonNull};

use super::{node::NodePtr, LinkedList};

#[derive(Debug)]
pub(crate) struct RawIter<T> {
    front: NodePtr<T>,
    back: NodePtr<T>,
    len: usize,
}

impl<T> RawIter<T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<T> LinkedList<T> {
    pub(crate) unsafe fn raw_iter(&self) -> Option<RawIter<T>> {
        self.dummy.map(|dummy| RawIter {
            front: dummy,
            back: dummy,
            len: self.len,
        })
    }
}

impl<T> Iterator for RawIter<T> {
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        (self.len != 0).then(|| {
            self.front = self.front.next();
            self.len = self.len.saturating_sub(1);

            debug_assert!(!self.front.is_dummy());
            unsafe { self.front.get_raw_unchecked() }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> DoubleEndedIterator for RawIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        (self.len != 0).then(|| {
            self.back = self.back.prev();
            self.len = self.len.saturating_sub(1);

            debug_assert!(!self.back.is_dummy());
            unsafe { self.back.get_raw_unchecked() }
        })
    }
}

impl<T> ExactSizeIterator for RawIter<T> {
    fn len(&self) -> usize {
        self.len
    }
}

pub struct Iter<'a, T> {
    inner: Option<RawIter<T>>,
    _phantom: PhantomData<&'a T>,
}

pub struct IterMut<'a, T> {
    inner: Option<RawIter<T>>,
    _phantom: PhantomData<&'a mut T>,
}

pub struct IntoIter<T> {
    inner: LinkedList<T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .as_mut()?
            .next()
            .map(|ptr| unsafe { ptr.as_ref() })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .as_mut()?
            .next()
            .map(|mut ptr| unsafe { ptr.as_mut() })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.inner.len, Some(self.inner.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .as_mut()?
            .next_back()
            .map(|ptr| unsafe { ptr.as_ref() })
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .as_mut()?
            .next_back()
            .map(|mut ptr| unsafe { ptr.as_mut() })
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.pop_back()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.inner.as_ref().map_or(0, RawIter::len)
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.inner.as_ref().map_or(0, RawIter::len)
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.inner.len
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { inner: self }
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = unsafe { self.raw_iter() };
        Iter {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = unsafe { self.raw_iter() };
        IterMut {
            inner,
            _phantom: PhantomData,
        }
    }
}
