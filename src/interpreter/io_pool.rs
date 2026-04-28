use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

/// Result type for I/O tasks (only primitive types cross thread boundaries)
#[derive(Debug)]
pub enum IoResult {
    Str(Result<String, String>),
    Unit(Result<(), String>),
}

/// I/O task submitted to the worker pool
#[derive(Debug)]
pub enum IoTask {
    HttpGet {
        url: String,
        tx: Sender<IoResult>,
    },
    HttpPost {
        url: String,
        body: String,
        tx: Sender<IoResult>,
    },
    Sleep {
        secs: f64,
        tx: Sender<IoResult>,
    },
    ReadFile {
        path: String,
        tx: Sender<IoResult>,
    },
    WriteFile {
        path: String,
        content: String,
        tx: Sender<IoResult>,
    },
}

/// Thread pool for offloading blocking I/O. Worker threads only see primitive
/// types (String, f64) — never Value or Rc<T> — preserving single-threaded
/// Rc<T> safety on the main interpreter thread.
pub struct IoPool {
    task_tx: Sender<IoTask>,
}

impl IoPool {
    pub fn new(num_workers: usize) -> Self {
        let (task_tx, task_rx) = channel::<IoTask>();
        let task_rx = Arc::new(Mutex::new(task_rx));

        for _ in 0..num_workers {
            let rx = Arc::clone(&task_rx);
            thread::spawn(move || loop {
                let task = rx.lock().unwrap().recv();
                match task {
                    Ok(IoTask::HttpGet { url, tx }) => {
                        let result = reqwest::blocking::get(&url)
                            .and_then(|r| r.text())
                            .map_err(|e| e.to_string());
                        let _ = tx.send(IoResult::Str(result));
                    }
                    Ok(IoTask::HttpPost { url, body, tx }) => {
                        let client = reqwest::blocking::Client::new();
                        let result = client
                            .post(&url)
                            .body(body)
                            .send()
                            .and_then(|r| r.text())
                            .map_err(|e| e.to_string());
                        let _ = tx.send(IoResult::Str(result));
                    }
                    Ok(IoTask::Sleep { secs, tx }) => {
                        thread::sleep(std::time::Duration::from_secs_f64(secs));
                        let _ = tx.send(IoResult::Unit(Ok(())));
                    }
                    Ok(IoTask::ReadFile { path, tx }) => {
                        let result = std::fs::read_to_string(&path).map_err(|e| e.to_string());
                        let _ = tx.send(IoResult::Str(result));
                    }
                    Ok(IoTask::WriteFile { path, content, tx }) => {
                        let result = std::fs::write(&path, content).map_err(|e| e.to_string());
                        let _ = tx.send(IoResult::Unit(result));
                    }
                    Err(_) => break, // Channel closed, worker exits
                }
            });
        }

        Self { task_tx }
    }

    pub fn submit(&self, task: IoTask) {
        let _ = self.task_tx.send(task);
    }

    /// Default worker count: max(available_parallelism - 1, 4)
    pub fn default_workers() -> usize {
        let cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        cpus.saturating_sub(1).max(4)
    }
}
