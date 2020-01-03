use spin::RwLock;
use lazy_static::lazy_static;
use alloc::collections::VecDeque;

/// A queue of processes
/// Each process is assigned a time quantum specified in this structure.
/// If a process exceeds this time quantum, it is moved to a lower-level queue.
struct Queue {

