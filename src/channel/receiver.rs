use super::ChannelQueue;
use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

pub struct Receiver<T> {
    pub(super) inner: Arc<ChannelQueue<T>>,
    pub(super) marker: PhantomData<*mut ()>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum RecvError {
    Disconnected,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, RecvError> {
        loop {
            // See sync_queue for an explanation of pushing first.
            self.inner.waiters.push(thread::current());
            match self.inner.queue.pop() {
                Some(elem) => return Ok(elem),
                None => {
                    if self.inner.senders.load(Ordering::Relaxed) == 0 {
                        // No senders, and nothing in the queue.
                        return Err(RecvError::Disconnected);
                    } else {
                        // Otherwise, park, and then check again.
                        thread::park();
                    }
                }
            }
        }
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        self.inner.receivers.fetch_add(1, Ordering::Relaxed);

        Self {
            inner: self.inner.clone(),
            marker: PhantomData,
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        // Ordering: Release to make sure the atomic instruction above doesn't go after this.
        self.inner.receivers.fetch_sub(1, Ordering::Relaxed);
    }
}

// Can't send non-sendable types.
unsafe impl<T: Send> Send for Receiver<T> {}
