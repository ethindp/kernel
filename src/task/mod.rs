/// The cooperative module contains code for the cooperative multitasking scheduler.
pub mod cooperative;
use core::sync::atomic::{AtomicU64, Ordering};
use core::pin::Pin;
use alloc::boxed::Box;
use core::future::Future;
use core::task::{Context, Poll};

// Common definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Tid(u64);

impl Tid {
fn new() ->Self {
static NEXT_ID: AtomicU64 = AtomicU64::new(0);
Tid (NEXT_ID.fetch_add(1, Ordering::Relaxed))
}
}

pub struct AsyncTask {
id: Tid,
future: Pin<Box<dyn Future<Output = ()>>>,
}

impl AsyncTask {
pub fn new(future: impl Future<Output = ()> + 'static) -> AsyncTask {
AsyncTask {
id: Tid::new(),
future: Box::pin(future)
}
}

fn poll(&mut self, context: &mut Context) -> Poll<()> {
self.future.as_mut().poll(context)
}
}
