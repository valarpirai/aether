//! Redis plugin for Aether.
//!
//! Demonstrates using the Rust `redis` crate from Aether through the FFI
//! plugin system. Connections persist: `conn_open(url)` opens a connection,
//! stores it in a process-global registry, and returns an integer handle id.
//! Command functions (`conn_get`, `conn_set`, ...) take that id as their first
//! argument and reuse the same live connection. `conn_close(id)` drops it.
//!
//! The FFI boundary is scalar-only (int, String, Vec, HashMap), so the handle
//! id is an `int`. The `RedisConn` struct in examples/redis_plugin_demo.ae
//! wraps this so callers write `conn.get(key)` instead of passing the id.

use std::collections::HashMap;
use std::sync::Mutex;

use aether_plugin::*;
use redis::Commands;

/// Registry of live connections, keyed by an integer handle id.
static CONNECTIONS: Mutex<Option<HashMap<i64, redis::Connection>>> = Mutex::new(None);

/// Monotonic source of handle ids. Never reused, so a stale id fails cleanly
/// rather than aliasing a different connection.
static NEXT_ID: Mutex<i64> = Mutex::new(1);

/// Run `f` with the connection for `id`, returning a boundary-friendly error
/// if the id is unknown (closed or never opened).
fn with_conn<T>(
    id: i64,
    f: impl FnOnce(&mut redis::Connection) -> Result<T, String>,
) -> Result<T, String> {
    let mut guard = CONNECTIONS
        .lock()
        .map_err(|_| "connection registry lock poisoned".to_string())?;
    let map = guard.get_or_insert_with(HashMap::new);
    match map.get_mut(&id) {
        Some(c) => f(c),
        None => Err(format!("invalid or closed connection handle: {id}")),
    }
}

/// Open a connection to `url` and return its integer handle id.
///
/// Returns a `String` error (surfaced to Aether as a catchable plugin error)
/// rather than panicking, since panics across the FFI boundary are undefined
/// behaviour.
#[aether_export]
fn conn_open(url: String) -> Result<i64, String> {
    let client = redis::Client::open(url).map_err(|e| format!("open client: {e}"))?;
    let connection = client
        .get_connection()
        .map_err(|e| format!("connect: {e}"))?;

    let id = {
        let mut next = NEXT_ID.lock().map_err(|_| "id lock poisoned".to_string())?;
        let id = *next;
        *next += 1;
        id
    };

    let mut guard = CONNECTIONS
        .lock()
        .map_err(|_| "connection registry lock poisoned".to_string())?;
    guard
        .get_or_insert_with(HashMap::new)
        .insert(id, connection);
    Ok(id)
}

/// Close the connection for `id`, dropping the socket. Returns 1 if a
/// connection was closed, 0 if the id was already gone.
#[aether_export]
fn conn_close(id: i64) -> Result<i64, String> {
    let mut guard = CONNECTIONS
        .lock()
        .map_err(|_| "connection registry lock poisoned".to_string())?;
    let removed = guard.get_or_insert_with(HashMap::new).remove(&id).is_some();
    Ok(if removed { 1 } else { 0 })
}

/// PING the server on connection `id`. Returns the server's reply ("PONG").
#[aether_export]
fn conn_ping(id: i64) -> Result<String, String> {
    with_conn(id, |c| {
        redis::cmd("PING")
            .query(c)
            .map_err(|e| format!("PING: {e}"))
    })
}

/// SET a string key to a string value. Returns "OK".
#[aether_export]
fn conn_set(id: i64, key: String, value: String) -> Result<String, String> {
    with_conn(id, |c| {
        c.set(&key, &value).map_err(|e| format!("SET {key}: {e}"))
    })
}

/// GET a string value by key. Returns the value, or an empty string if the
/// key is absent (Redis nil maps to "").
#[aether_export]
fn conn_get(id: i64, key: String) -> Result<String, String> {
    with_conn(id, |c| {
        let value: Option<String> = c.get(&key).map_err(|e| format!("GET {key}: {e}"))?;
        Ok(value.unwrap_or_default())
    })
}

/// DEL a key. Returns the number of keys removed (0 or 1).
#[aether_export]
fn conn_del(id: i64, key: String) -> Result<i64, String> {
    with_conn(id, |c| c.del(&key).map_err(|e| format!("DEL {key}: {e}")))
}

/// EXISTS check. Returns 1 if the key exists, 0 otherwise.
#[aether_export]
fn conn_exists(id: i64, key: String) -> Result<i64, String> {
    with_conn(id, |c| {
        let n: i64 = c.exists(&key).map_err(|e| format!("EXISTS {key}: {e}"))?;
        Ok(n)
    })
}

/// INCR a key, returning the new value. Creates the key at 1 if absent.
#[aether_export]
fn conn_incr(id: i64, key: String) -> Result<i64, String> {
    with_conn(id, |c| {
        c.incr(&key, 1).map_err(|e| format!("INCR {key}: {e}"))
    })
}

/// INCRBY: add an integer amount to a key, returning the new value.
#[aether_export]
fn conn_incr_by(id: i64, key: String, amount: i64) -> Result<i64, String> {
    with_conn(id, |c| {
        c.incr(&key, amount)
            .map_err(|e| format!("INCRBY {key}: {e}"))
    })
}

/// KEYS matching a glob pattern (e.g. "*", "user:*"). Returns an array of
/// key names. Note: KEYS scans the whole keyspace; use only on small sets.
#[aether_export]
fn conn_keys(id: i64, pattern: String) -> Result<Vec<String>, String> {
    with_conn(id, |c| {
        c.keys(&pattern).map_err(|e| format!("KEYS {pattern}: {e}"))
    })
}

// Register all V2 functions.
aether_plugin_init_v2!(
    conn_open,
    conn_close,
    conn_ping,
    conn_set,
    conn_get,
    conn_del,
    conn_exists,
    conn_incr,
    conn_incr_by,
    conn_keys
);
