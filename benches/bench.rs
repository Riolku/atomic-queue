use atomic_queue::AsyncQueue;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::random;
use std::thread::scope;

fn bench_mpmc() {
    let q = AsyncQueue::new();
    scope(|s| {
        for _ in 0..100 {
            s.spawn(|| {
                for i in 0..1000 {
                    if random::<bool>() {
                        q.push(i);
                    } else {
                        q.pop();
                    }
                }
            });
        }
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("AsyncQueue", |b| b.iter(|| bench_mpmc()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
