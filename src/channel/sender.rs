use super::ChannelQueue;
use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct Sender<T> {
    pub(super) inner: Arc<ChannelQueue<T>>,
    pub(super) marker: PhantomData<*mut ()>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SendError {
    Disconnected,
}

impl<T> Sender<T> {
    pub fn send(&self, message: T) -> Result<(), SendError> {
        if self.inner.receivers.load(Ordering::Relaxed) == 0 {
            Err(SendError::Disconnected)
        } else {
            self.inner.queue.push(message);
            match self.inner.waiters.pop() {
                Some(waiter) => waiter.unpark(),
                None => {}
            };
            Ok(())
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.inner.senders.fetch_add(1, Ordering::Relaxed);

        Self {
            inner: self.inner.clone(),
            marker: PhantomData,
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        // Ordering: Release to make sure the atomic instruction above doesn't go after this.
        if self.inner.senders.fetch_sub(1, Ordering::Release) == 1 {
            // No more senders.
            // Flush the queue of receivers.
            while let Some(waiter) = self.inner.waiters.pop() {
                waiter.unpark();
            }
        }
    }
}

// Can't send non-sendable types.
unsafe impl<T: Send> Send for Sender<T> {}
