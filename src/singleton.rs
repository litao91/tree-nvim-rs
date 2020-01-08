use core::cell::UnsafeCell;
use std::sync::Mutex;

// The singleton wrapper
// idea from: https://github.com/RusPiRo/ruspiro-singleton/blob/master/src/lib.rs
pub struct Singleton<T: 'static> {
    lock: Mutex<()>,
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for Singleton<T> {}

unsafe impl<T> Send for Singleton<T> {}

impl<T: 'static> Singleton<T> {
    // Create a new singleton instance to be used in a static variable.
    pub fn new(data: T) -> Self {
        Self {
            lock: Mutex::new(()),
            inner: UnsafeCell::new(data),
        }
    }

    // Take the stored singleton for whatever operation and prevent usage by other cores
    // Safe access to singleton mutable instance is guarantied inside the given closure
    // # Example
    // ```
    // # fn doc() {
    //     MY_SINGLETON.take_for(|my| {
    //         // do something with [my]
    //         my.any_mutable_function();
    // # }
    // ```
    pub fn take_for<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        // lock
        let _ = self.lock.lock().unwrap();
        f(unsafe { &mut *self.inner.get() })
    }

    pub fn use_for<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(unsafe { &*self.inner.get() })
    }
}
