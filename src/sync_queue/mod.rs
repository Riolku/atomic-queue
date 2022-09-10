use crate::AsyncQueue;
use std::thread;

pub struct SyncQueue<T> {
    inner: AsyncQueue<T>,
    waiters: AsyncQueue<thread::Thread>,
}

impl<T> SyncQueue<T> {
    pub fn new() -> Self {
        Self {
            inner: AsyncQueue::new(),
            waiters: AsyncQueue::new(),
        }
    }

    pub fn push(&self, elem: T) {
        self.inner.push(elem);
        match self.waiters.pop() {
            Some(waiter) => waiter.unpark(),
            None => {}
        }
    }

    // Block until something becomes available.
    pub fn pop(&self) -> T {
        loop {
            // We have to push first, or we might lose a notification.
            // If we are unparked but still end up finding something, that's fine,
            // worst case we do an extra iteration next time.
            // Anybody who is _actually_ parked will be popped before us anyway.
            self.waiters.push(thread::current());
            match self.inner.pop() {
                Some(elem) => return elem,
                None => {
                    thread::park();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
