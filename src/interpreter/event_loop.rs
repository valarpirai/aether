use crate::interpreter::io_pool::IoResult;
use crate::interpreter::value::Value;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Instant;

pub struct EventLoopEntry {
    pub rx: Receiver<IoResult>,
    pub callback: Value,
    /// Per-task deadline. When Some and now >= deadline, the task is considered
    /// timed out: the callback fires with an error and the receiver is dropped.
    pub deadline: Option<Instant>,
}

/// Non-blocking event queue. Holds pending I/O receivers paired with Aether
/// callbacks. The main interpreter thread polls this each tick via drain_ready().
pub struct EventLoopQueue {
    pub pending: Vec<EventLoopEntry>,
    max_pending: usize,
}

impl Default for EventLoopQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLoopQueue {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            max_pending: 1024,
        }
    }

    /// Change the maximum number of entries allowed in the queue.
    pub fn set_limit(&mut self, limit: usize) {
        self.max_pending = limit;
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Push a new entry, returning an error if the queue is at capacity (backpressure).
    /// `deadline` is the per-task expiry: if set and elapsed, the task is aborted.
    pub fn push(
        &mut self,
        rx: Receiver<IoResult>,
        callback: Value,
        deadline: Option<Instant>,
    ) -> Result<(), String> {
        if self.pending.len() >= self.max_pending {
            return Err(format!(
                "event loop queue full ({}/{} pending callbacks)",
                self.pending.len(),
                self.max_pending
            ));
        }
        self.pending.push(EventLoopEntry {
            rx,
            callback,
            deadline,
        });
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Non-blocking poll. Per-task deadlines are checked first: a timed-out task
    /// is returned immediately as Err("task timed out") without waiting for the
    /// worker. Tasks still within their deadline that have no result yet stay in
    /// self.pending for the next tick.
    pub fn drain_ready(&mut self) -> Vec<(Result<Value, String>, Value)> {
        let mut ready = Vec::new();
        let mut remaining = Vec::new();
        let now = Instant::now();

        for entry in self.pending.drain(..) {
            // Per-task deadline check — abort if expired, regardless of I/O state
            if let Some(dl) = entry.deadline {
                if now >= dl {
                    ready.push((Err("task timed out".to_string()), entry.callback));
                    // Receiver is dropped here; the worker will get a SendError and discard its result
                    continue;
                }
            }

            match entry.rx.try_recv() {
                Ok(IoResult::Str(Ok(s))) => ready.push((Ok(Value::string(s)), entry.callback)),
                Ok(IoResult::Str(Err(e))) => ready.push((Err(e), entry.callback)),
                Ok(IoResult::Unit(Ok(()))) => ready.push((Ok(Value::Null), entry.callback)),
                Ok(IoResult::Unit(Err(e))) => ready.push((Err(e), entry.callback)),
                Err(TryRecvError::Empty) => remaining.push(entry),
                Err(TryRecvError::Disconnected) => {} // worker panicked — drop silently
            }
        }

        self.pending = remaining;
        ready
    }
}
