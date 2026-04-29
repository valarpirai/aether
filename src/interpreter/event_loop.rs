use crate::interpreter::io_pool::IoResult;
use crate::interpreter::value::Value;
use std::sync::mpsc::{Receiver, TryRecvError};

pub struct EventLoopEntry {
    pub rx: Receiver<IoResult>,
    pub callback: Value,
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
    pub fn push(&mut self, rx: Receiver<IoResult>, callback: Value) -> Result<(), String> {
        if self.pending.len() >= self.max_pending {
            return Err(format!(
                "event loop queue full ({}/{} pending callbacks)",
                self.pending.len(),
                self.max_pending
            ));
        }
        self.pending.push(EventLoopEntry { rx, callback });
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Non-blocking poll. Drains all receivers that have a result ready.
    /// Returns (result_or_error, callback) for each ready entry.
    /// Entries still waiting stay in self.pending.
    pub fn drain_ready(&mut self) -> Vec<(Result<Value, String>, Value)> {
        let mut ready = Vec::new();
        let mut remaining = Vec::new();

        for entry in self.pending.drain(..) {
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
