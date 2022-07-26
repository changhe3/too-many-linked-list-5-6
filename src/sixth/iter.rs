use std::marker::PhantomData;

use super::{node::NodePtr, LinkedList};

#[derive(Debug)]
pub(crate) struct RawIter<T> {
    front: NodePtr<T>,
    back: NodePtr<T>,
    len: usize,
}

impl<T> RawIter<T> {
    pub(crate) fn new(front: NodePtr<T>, back: NodePtr<T>, len: usize) -> Self {
        Self { front, back, len }
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl<T> LinkedList<T> {
    pub(crate) unsafe fn raw_iter(&self) -> Option<RawIter<T>> {
        self.dummy.map(|dummy| RawIter {
            front: dummy.next(),
            back: dummy.prev(),
            len: self.len,
        })
    }
}

impl<T> Iterator for RawIter<T> {
    type Item = NodePtr<T>;

    fn next(&mut self) -> Option<Self::Item> {
        (self.len != 0).then(|| {
            // Rather than using a simpler design where we return the pointer we are at, we return the pointer that has been advanced past.
            // This way, we can do whatever we want with the returned pointers, e.g. deallocating them.

            let front = self.front;

            self.front = front.next();
            self.len = self.len.saturating_sub(1);
            front
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> DoubleEndedIterator for RawIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        (self.len != 0).then(|| {
            let back = self.back;

            self.back = back.prev();
            self.len = self.len.saturating_sub(1);
            back
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

pub struct DrainFilter<'a, T, F> {
    inner: Option<RawIter<T>>,
    retained: usize,
    pred: F,
    list: &'a mut LinkedList<T>,
}

impl<'a, T, F> DrainFilter<'a, T, F> {
    pub(crate) fn new(list: &'a mut LinkedList<T>, pred: F) -> Self {
        let inner = unsafe { list.raw_iter() };

        Self {
            inner,
            retained: 0,
            pred,
            list,
        }
    }
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
            .map(|ptr| unsafe { ptr.get_unchecked() })
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
            .map(|ptr| unsafe { ptr.get_mut_unchecked() })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a, T, F: FnMut(&mut T) -> bool> Iterator for DrainFilter<'a, T, F> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        for ptr in self.inner.as_mut()? {
            let to_remove = {
                let item: &'a mut T = unsafe { ptr.get_mut_unchecked() };
                (self.pred)(item)
            };

            if to_remove {
                unsafe {
                    return Some(ptr.pop_unchecked(self.list));
                }
            } else {
                self.retained = self.retained.saturating_add(1);
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.list.len - self.retained))
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
            .map(|ptr| unsafe { ptr.get_unchecked() })
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .as_mut()?
            .next_back()
            .map(|ptr| unsafe { ptr.get_mut_unchecked() })
    }
}

impl<'a, T, F: FnMut(&mut T) -> bool> DoubleEndedIterator for DrainFilter<'a, T, F> {
    fn next_back(&mut self) -> Option<Self::Item> {
        for ptr in self.inner.as_mut()?.rev() {
            let to_remove = {
                let item: &'a mut T = unsafe { ptr.get_mut_unchecked() };
                (self.pred)(item)
            };

            if to_remove {
                unsafe {
                    return Some(ptr.pop_unchecked(self.list));
                }
            } else {
                self.retained = self.retained.saturating_add(1);
            }
        }

        None
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

unsafe impl<'a, T: Send> Send for Iter<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Iter<'a, T> {}

unsafe impl<'a, T: Send> Send for IterMut<'a, T> {}
unsafe impl<'a, T: Sync> Sync for IterMut<'a, T> {}

/// ```compile_fail
/// use too_many_linked_list::sixth::IterMut;
///
/// fn iter_mut_covariant<'i, 'a, T>(x: IterMut<'i, &'static T>) -> IterMut<'i, &'a T> { x }
/// ```
#[allow(unused)]
fn iter_mut_invariant() {}
