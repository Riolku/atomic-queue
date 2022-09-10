use super::SyncQueue;
use std::thread::scope;

#[test]
fn simple_test() {
    let q = SyncQueue::new();
    q.push(4);
    q.push(3);
    assert_eq!(q.pop(), 4);
    assert_eq!(q.pop(), 3);
    q.push(5);
    assert_eq!(q.pop(), 5);
}

#[test]
fn threaded_data_consistent() {
    let q = SyncQueue::new();
    scope(|s| {
        s.spawn(|| {
            q.push(2);
            q.push(3);
            q.push(4);
        });
        s.spawn(|| {
            assert_eq!(q.pop(), 2);
            assert_eq!(q.pop(), 3);
            assert_eq!(q.pop(), 4);
        });
    });
}
