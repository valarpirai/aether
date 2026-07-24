//! Redis plugin for Aether (V2 protocol prototype).
//!
//! Demonstrates using the Rust `redis` crate from Aether through the FFI
//! plugin system. Because the FFI boundary has no opaque-handle type yet,
//! there is no persistent connection object: the target URL is held in a
//! process-global static (set via `connect`, default `redis://127.0.0.1/`)
//! and every command opens a short-lived connection.
//!
//! Type mapping at the boundary is scalar-only, so commands are limited to
//! `int`- and `string`-shaped values.

use std::sync::Mutex;

use aether_plugin::*;
use redis::Commands;

/// Process-global Redis URL. Set by `connect`; defaults to localhost.
static REDIS_URL: Mutex<Option<String>> = Mutex::new(None);

const DEFAULT_URL: &str = "redis://127.0.0.1/";

/// Open a fresh connection to the configured URL.
///
/// Returns a `String` error (surfaced to Aether as a catchable plugin error)
/// rather than panicking, since panics across the FFI boundary are undefined
/// behaviour.
fn conn() -> Result<redis::Connection, String> {
    let url = REDIS_URL
        .lock()
        .map_err(|_| "Redis URL lock poisoned".to_string())?
        .clone()
        .unwrap_or_else(|| DEFAULT_URL.to_string());

    let client = redis::Client::open(url).map_err(|e| format!("open client: {e}"))?;
    client.get_connection().map_err(|e| format!("connect: {e}"))
}

/// Set the Redis URL used by subsequent commands (e.g. "redis://127.0.0.1/").
///
/// Verifies the URL parses before storing it. Returns "OK" on success.
#[aether_export]
fn connect(url: String) -> Result<String, String> {
    // Validate by attempting to build a client.
    redis::Client::open(url.clone()).map_err(|e| format!("invalid url: {e}"))?;
    *REDIS_URL.lock().map_err(|_| "lock poisoned".to_string())? = Some(url);
    Ok("OK".to_string())
}

/// PING the server. Returns the server's reply ("PONG").
#[aether_export]
fn ping() -> Result<String, String> {
    let mut c = conn()?;
    redis::cmd("PING")
        .query(&mut c)
        .map_err(|e| format!("PING: {e}"))
}

/// SET a string key to a string value. Returns "OK".
#[aether_export]
fn set(key: String, value: String) -> Result<String, String> {
    let mut c = conn()?;
    c.set(&key, &value).map_err(|e| format!("SET {key}: {e}"))
}

/// GET a string value by key. Returns the value, or an empty string if the
/// key is absent (Redis nil maps to "").
#[aether_export]
fn get(key: String) -> Result<String, String> {
    let mut c = conn()?;
    let value: Option<String> = c.get(&key).map_err(|e| format!("GET {key}: {e}"))?;
    Ok(value.unwrap_or_default())
}

/// DEL a key. Returns the number of keys removed (0 or 1).
#[aether_export]
fn del(key: String) -> Result<i64, String> {
    let mut c = conn()?;
    c.del(&key).map_err(|e| format!("DEL {key}: {e}"))
}

/// EXISTS check. Returns 1 if the key exists, 0 otherwise.
#[aether_export]
fn exists(key: String) -> Result<i64, String> {
    let mut c = conn()?;
    let n: i64 = c.exists(&key).map_err(|e| format!("EXISTS {key}: {e}"))?;
    Ok(n)
}

/// INCR a key, returning the new value. Creates the key at 1 if absent.
#[aether_export]
fn incr(key: String) -> Result<i64, String> {
    let mut c = conn()?;
    c.incr(&key, 1).map_err(|e| format!("INCR {key}: {e}"))
}

/// INCRBY: add an integer amount to a key, returning the new value.
#[aether_export]
fn incr_by(key: String, amount: i64) -> Result<i64, String> {
    let mut c = conn()?;
    c.incr(&key, amount)
        .map_err(|e| format!("INCRBY {key}: {e}"))
}

/// KEYS matching a glob pattern (e.g. "*", "user:*"). Returns an array of
/// key names. Note: KEYS scans the whole keyspace; use only on small sets.
#[aether_export]
fn keys(pattern: String) -> Result<Vec<String>, String> {
    let mut c = conn()?;
    c.keys(&pattern).map_err(|e| format!("KEYS {pattern}: {e}"))
}

// Register all V2 functions.
aether_plugin_init_v2!(connect, ping, set, get, del, exists, incr, incr_by, keys);
