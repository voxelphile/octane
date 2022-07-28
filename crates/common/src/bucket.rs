use std::ops;

pub struct Bucket<T> {
    inner: T,
    dirty: bool,
}

impl<T> Bucket<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            dirty: false,
        }
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clean(&mut self) {
        self.dirty = false;
    }
}

impl<T> ops::Deref for Bucket<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> ops::DerefMut for Bucket<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty = true;
        &mut self.inner
    }
}
