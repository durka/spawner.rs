#![cfg_attr(feature = "nightly", feature(drain))]
#![deny(missing_docs)]
#![cfg_attr(test, deny(dead_code))]
#![cfg_attr(not(test), allow(dead_code))]

//! Small wrapper for `thread::spawn` that optionally auto-joins threads

use std::thread;
#[cfg(not(feature = "nightly"))] use std::mem;

/// A wrapper for `thread::spawn` that optionally auto-joins threads.
pub struct Spawner {
    threads: Vec<thread::JoinHandle<()>>
}

impl Spawner {
    /// Create a new Spawner object
    pub fn new() -> Spawner { Spawner { threads: vec![] } }

    /// Spawn a thread that will be auto-joined when the Spawner is dropped
    ///
    /// The thread function should be a move closure returning ()
    pub fn spawn_collected<F>(&mut self, f: F)
    where F: FnOnce(), F: Send + 'static
    {
        self.threads.push(thread::spawn(f));
    }
    
    /// Spawn a thread that won't be auto-joined
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
        assert!(self.threads.drain(..)
                    .map(thread::JoinHandle::join)
                    .find(Result::is_err)
                    .is_none());
    }

    #[cfg(not(feature = "nightly"))]
    fn drop(&mut self) {
        assert!(mem::replace(&mut self.threads, vec![])
                    .into_iter()
                    .map(thread::JoinHandle::join)
                    .find(Result::is_err)
                    .is_none());
    }
}

#[cfg(test)] mod tests {
    use super::Spawner;

    #[test]
    fn spawn_some_threads() {
        use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
        use std::thread::sleep;
        use std::time::Duration;

        static ACTIVE: AtomicUsize = ATOMIC_USIZE_INIT;

        {
            let mut spawner = Spawner::new();

            // an manually joined thread
            assert!(
                spawner.spawn(move || {
                                ACTIVE.fetch_add(1, Ordering::SeqCst);
                                sleep(Duration::from_millis(100));
                                ACTIVE.fetch_sub(1, Ordering::SeqCst);
                             })
                       .join()
                       .is_ok()
            );

            // make sure the manual thread finished
            assert_eq!(
                ACTIVE.load(Ordering::SeqCst),
                0
            );

            // some collected threads
            for _ in 1..10 {
                spawner.spawn_collected(move || {
                                          ACTIVE.fetch_add(1, Ordering::SeqCst);
                                          sleep(Duration::from_millis(100));
                                          ACTIVE.fetch_sub(1, Ordering::SeqCst);
                                       });
            }

            // make sure at least some have started
            assert!(
                ACTIVE.load(Ordering::SeqCst) > 0
            );

            // collected threads implicitly joined here
        }

        // make sure they all finished
        assert_eq!(
            ACTIVE.load(Ordering::SeqCst),
            0
        );
    }

    #[test]
    #[should_panic]
    fn failing_thread() {
        {
            let mut spawner = Spawner::new();
            spawner.spawn_collected(move || panic!());
        }
    }
}

