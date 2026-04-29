use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Result type for I/O tasks (only primitive types cross thread boundaries)
#[derive(Debug)]
pub enum IoResult {
    Str(Result<String, String>),
    Unit(Result<(), String>),
}

/// Per-request HTTP options — primitive fields only, safe to send across threads.
/// `None` fields fall back to env-var defaults when the client is built.
#[derive(Debug, Default, Clone)]
pub struct HttpOptions {
    /// Override `AETHER_HTTP_TIMEOUT` (seconds)
    pub timeout_secs: Option<u64>,
    /// Override `AETHER_HTTP_USER_AGENT`
    pub user_agent: Option<String>,
}

/// I/O task submitted to the worker pool
#[derive(Debug)]
pub enum IoTask {
    HttpGet {
        url: String,
        opts: HttpOptions,
        tx: Sender<IoResult>,
    },
    HttpPost {
        url: String,
        body: String,
        opts: HttpOptions,
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

/// Build a reqwest blocking client, applying per-request opts on top of env-var defaults.
///
/// Env-var defaults:
/// - `AETHER_HTTP_TIMEOUT`: request timeout in seconds (default 30)
/// - `AETHER_HTTP_USER_AGENT`: User-Agent header value (default "aether/0.1")
pub fn build_http_client_with_opts(opts: &HttpOptions) -> reqwest::blocking::Client {
    let timeout_secs: u64 = opts
        .timeout_secs
        .or_else(|| {
            std::env::var("AETHER_HTTP_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
        })
        .unwrap_or(30);
    let user_agent = opts
        .user_agent
        .clone()
        .or_else(|| std::env::var("AETHER_HTTP_USER_AGENT").ok())
        .unwrap_or_else(|| "aether/0.1".to_string());
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent(user_agent)
        .build()
        .unwrap_or_default()
}

/// Convenience wrapper with env-var defaults only (no per-request opts).
pub fn build_http_client() -> reqwest::blocking::Client {
    build_http_client_with_opts(&HttpOptions::default())
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
                    Ok(IoTask::HttpGet { url, opts, tx }) => {
                        let client = build_http_client_with_opts(&opts);
                        let result = client
                            .get(&url)
                            .send()
                            .and_then(|r| r.text())
                            .map_err(|e| e.to_string());
                        let _ = tx.send(IoResult::Str(result));
                    }
                    Ok(IoTask::HttpPost {
                        url,
                        body,
                        opts,
                        tx,
                    }) => {
                        let client = build_http_client_with_opts(&opts);
                        let result = client
                            .post(&url)
                            .body(body)
                            .send()
                            .and_then(|r| r.text())
                            .map_err(|e| e.to_string());
                        let _ = tx.send(IoResult::Str(result));
                    }
                    Ok(IoTask::Sleep { secs, tx }) => {
                        thread::sleep(Duration::from_secs_f64(secs));
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
