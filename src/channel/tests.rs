use super::*;
use std::thread::scope;
use std::sync::{Arc, Barrier};
use rand::distributions::uniform::Uniform;
use rand::distributions::Distribution;
use rand::thread_rng;

#[test]
fn simple_test() {
    let (sender, receiver) = channel();
    scope(|s| {
        s.spawn(move || {
            assert!(sender.send(2).is_ok());
            assert!(sender.send(3).is_ok());
            assert!(sender.send(4).is_ok());
        });
        s.spawn(move || {
            assert_eq!(receiver.recv(), Ok(2));
            assert_eq!(receiver.recv(), Ok(3));
            assert_eq!(receiver.recv(), Ok(4));
            assert_eq!(receiver.recv(), Err(RecvError::Disconnected));
            assert_eq!(receiver.recv(), Err(RecvError::Disconnected));
        });
    });
}

#[test]
fn no_receivers() {
    let (sender, receiver) = channel();
    let bar = Arc::new(Barrier::new(2));
    scope(|s| {
        {
            let bar = bar.clone();
            s.spawn(move || {
                bar.wait();
                assert_eq!(sender.send(2), Err(SendError::Disconnected));
            });
        }
        {
            s.spawn(move || {
                drop(receiver);
                bar.wait();
            });
        }
    });
}

#[test]
fn simple_pipeline() {
    let samples = 100;
    let start: Vec<_> = Uniform::new(0, 10).sample_iter(thread_rng()).take(samples).collect();

    let (sender_one, receiver_one) = channel();
    let (sender_two, receiver_two) = channel();
    let (sender_three, receiver_three) = channel();

    for x in &start {
        sender_one.send(*x).unwrap();
    }
    drop(sender_one);

    scope(move |s| {
        for _ in 0..10 {
            let receiver_one = receiver_one.clone();
            let sender_two = sender_two.clone();
            s.spawn(move || {
                while let Ok(val) = receiver_one.recv() {
                    sender_two.send(val * 2).unwrap();
                }
            });
        }
        drop(sender_two);
        drop(receiver_one);

        for _ in 0..10 {
            let receiver_two = receiver_two.clone();
            let sender_three = sender_three.clone();
            s.spawn(move || {
                while let Ok(val) = receiver_two.recv() {
                    sender_three.send(val + 1).unwrap();
                }
            });
        }
        drop(sender_three);
        drop(receiver_two);

        let mut end: Vec<_> = start.into_iter().map(|x| x * 2 + 1).collect();
        end.sort();

        let mut pipeline_end = Vec::new();
        pipeline_end.reserve(samples);
        while let Ok(val) = receiver_three.recv() {
            pipeline_end.push(val);
        }
        pipeline_end.sort();

        assert_eq!(end, pipeline_end);
    });
}
