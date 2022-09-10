use crate::AsyncQueue;
use std::marker::PhantomData;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::thread;

struct ChannelQueue<T> {
    queue: AsyncQueue<T>,
    receivers: AtomicUsize,
    senders: AtomicUsize,
    waiters: AsyncQueue<thread::Thread>,
}

mod receiver;
pub use receiver::{Receiver, RecvError};

mod sender;
pub use sender::{SendError, Sender};

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(ChannelQueue {
        queue: AsyncQueue::new(),
        receivers: AtomicUsize::new(1),
        senders: AtomicUsize::new(1),
        waiters: AsyncQueue::new(),
    });

    (
        Sender {
            inner: inner.clone(),
            marker: PhantomData,
        },
        Receiver {
            inner,
            marker: PhantomData,
        },
    )
}

#[cfg(test)]
mod tests;
