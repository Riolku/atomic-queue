use std::mem::MaybeUninit;
use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

pub struct AsyncQueue<T> {
    front: AtomicPtr<Node<T>>,
    back: AtomicPtr<Node<T>>,
    node_count: AtomicUsize,
}

struct Node<T> {
    data: MaybeUninit<T>,
    next: AtomicPtr<Node<T>>,
}

unsafe impl<T: Send> Sync for AsyncQueue<T> {}
unsafe impl<T: Send> Send for AsyncQueue<T> {}

impl<T> AsyncQueue<T> {
    pub fn new() -> Self {
        let stub: *mut Node<T> = Box::into_raw(Box::new(Node {
            data: MaybeUninit::uninit(),
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        Self {
            front: AtomicPtr::new(stub),
            back: AtomicPtr::new(stub),
            node_count: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, elem: T) {
        unsafe {
            let new_node: *mut Node<T> = Box::into_raw(Box::new(Node {
                data: MaybeUninit::new(elem),
                next: AtomicPtr::new(ptr::null_mut()),
            }));

            // Ordering: We don't want any operations seeping into our critical section.
            let prev = self.back.swap(new_node, Ordering::Release);

            // If the thread is preempted here, we have a little bit of an inconsistent state where the queue is disconnected.
            // Ordering: we need to release data to popping threads.
            (*prev).next.store(new_node, Ordering::Release);

            // Add only after the node is connected.
            // Ordering: we want to fence any memory operations from seeping above into our critical section.
            self.node_count.fetch_add(1, Ordering::Acquire);
        }
    }

    pub fn pop(&self) -> Option<T> {
        unsafe {
            loop {
                // Ordering: fence operations to occur before this (our critical section).
                let prev_head = self.front.swap(ptr::null_mut(), Ordering::Release);
                // Critical section begins. If the thread is preempted, we block all poppers.

                if prev_head == ptr::null_mut() {
                    if self.node_count.load(Ordering::Relaxed) == 0 {
                        return None;
                    } else {
                        continue;
                    }
                }

                // Ordering: this is where we sync with pushing threads.
                let next_ptr = (*prev_head).next.load(Ordering::Relaxed);
                if next_ptr == ptr::null_mut() {
                    // Stub
                    self.front.swap(prev_head, Ordering::Relaxed);
                    return None;
                }

                let data = (*next_ptr).data.assume_init_read();

                // Ordering: fence operations to occur after this (our critical section).
                self.front.swap(next_ptr, Ordering::Acquire);
                // Critical section ends.

                self.node_count.fetch_sub(1, Ordering::Relaxed);

                // Have to deallocate the previous head, and return the value from `next_ptr`.
                drop(Box::from_raw(prev_head));

                return Some(data);
            }
        }
    }
}

impl<T> std::ops::Drop for AsyncQueue<T> {
    fn drop(&mut self) {
        unsafe {
            // Drop the first node (stub)
            let mut cur_node = self.front.load(Ordering::Relaxed);
            assert!(cur_node != ptr::null_mut());
            let next_node = (*cur_node).next.load(Ordering::Relaxed);
            drop(Box::from_raw(cur_node));
            cur_node = next_node;

            while cur_node != ptr::null_mut() {
                let next_node = (*cur_node).next.load(Ordering::Relaxed);

                (*cur_node).data.assume_init_drop();

                drop(Box::from_raw(cur_node));
                cur_node = next_node;
            }
        }
    }
}

#[cfg(test)]
mod tests;
