#![cfg_attr(feature = "nightly", feature(drain))]
#![deny(missing_docs)]
#![cfg_attr(test, deny(dead_code))]
#![cfg_attr(not(test), allow(dead_code))]

//! Small wrapper for `thread::spawn` that optionally auto-joins threads

use std::thread;
#[cfg(not(feature = "nightly"))] use std::mem;

/// A wrapper for `thread::spawn` that optionally auto-joins threads.
struct Spawner {
    threads: Vec<thread::JoinHandle<()>>
}

impl Spawner {
    /// Create a new Spawner object
    pub fn new() -> Spawner { Spawner { threads: Vec::new() } }

    /// Spawn a thread that will be auto-joined when the Spawner is dropped
    ///
    /// The thread function should be a move closure returning ()
    pub fn spawn_collected<F>(&mut self, f: F)
    where F: FnOnce(), F: Send + 'static
    {
        self.threads.push(thread::spawn(f));
    }
    
    /// Spawn a thread that won't be auto-joine
    ///
    /// The thread function should be a move closure
    pub fn spawn<F, T>(&mut self, f: F) -> thread::JoinHandle<T>
    where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static
    {
        thread::spawn(f)
    }
}

impl Drop for Spawner {
    #[cfg(feature = "nightly")]
    fn drop(&mut self) {
        self.threads.drain(..)
            .map(thread::JoinHandle::join)
            .collect::<Result<Vec<_>,_>>()
            .unwrap();
    }

    #[cfg(not(feature = "nightly"))]
    fn drop(&mut self) {
        mem::replace(&mut self.threads, Vec::new())
            .into_iter()
            .map(thread::JoinHandle::join)
            .collect::<Result<Vec<_>,_>>()
            .unwrap();
    }
}

#[test]
fn spawn_some_threads() {
    let mut spawner = Spawner::new();
    spawner.spawn(move || println!("0")).join().unwrap();
    for i in 1..10 {
        spawner.spawn_collected(move || println!("{}", i));
    }
}

