use super::AsyncQueue;
use std::sync::Barrier;
use std::thread::scope;

#[test]
fn simple_test() {
    let q = AsyncQueue::new();
    q.push(4);
    q.push(3);
    assert_eq!(q.pop(), Some(4));
    assert_eq!(q.pop(), Some(3));
    assert_eq!(q.pop(), None);
    q.push(5);
    assert_eq!(q.pop(), Some(5));
}

#[test]
fn threaded_data_consistent() {
    let q = AsyncQueue::new();
    let bar = Barrier::new(2);
    scope(|s| {
        s.spawn(|| {
            q.push(2);
            q.push(3);
            q.push(4);
            bar.wait();
        });
        s.spawn(|| {
            bar.wait();
            assert_eq!(q.pop(), Some(2));
            assert_eq!(q.pop(), Some(3));
            assert_eq!(q.pop(), Some(4));
            assert_eq!(q.pop(), None);
        });
    });
}
