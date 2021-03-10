/// The cooperative module contains code for the cooperative multitasking scheduler.
pub mod cooperative;
use alloc::boxed::Box;
use core::fmt;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll};

// Common definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Tid(u64);

impl Tid {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Tid(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for Tid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An asynchronous task
pub struct AsyncTask {
    id: Tid,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl AsyncTask {
    /// Creates a new asynchronous task.
    pub fn new(future: impl Future<Output = ()> + 'static) -> AsyncTask {
        AsyncTask {
            id: Tid::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
