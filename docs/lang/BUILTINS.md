---
layout: default
title: "Aether — Built-in Functions"
---

[Home](../index.md) › Language Reference › Built-in Functions

# Built-in Functions

Built-ins are implemented in Rust and are always available — no import needed.
They cover the things the language cannot express in itself: I/O, type
inspection, native conversions, and network sockets.

For functions written in Aether, see [STDLIB.md](STDLIB.md). The split between
the two is explained in
[DEVELOPMENT.md](../dev/DEVELOPMENT.md#feature-implementation-decision-tree).

Every name on this page is also registered in the `BUILTINS` slice in
`src/checker.rs`, which is what stops `aether check` reporting them as undefined.

## Quick Reference

| Function | Signature | Returns |
|----------|-----------|---------|
| `print` | `print(...args)` | null — writes args, no trailing newline |
| `println` | `println(...args)` | null — writes args plus a newline |
| `input` | `input([prompt])` | string — one line from stdin |
| `type` | `type(value)` | string — `int`, `float`, `string`, `bool`, `null`, `array`, `dict`, `set`, `function`, … |
| `len` | `len(value)` | int — length of string, array, dict, or set |
| `int` | `int(value[, base])` | int — parses with `base` when given (e.g. `int("ff", 16)` → 255) |
| `float` | `float(value)` | float |
| `str` | `str(value)` | string |
| `bool` | `bool(value)` | bool |
| `hex` | `hex(n)` | string — `0x`-prefixed (`hex(255)` → `"0xff"`) |
| `oct` | `oct(n)` | string — `0o`-prefixed (`oct(8)` → `"0o10"`) |
| `bin` | `bin(n)` | string — `0b`-prefixed (`bin(5)` → `"0b101"`) |
| `base64_encode` | `base64_encode(s)` | string |
| `base64_decode` | `base64_decode(s)` | string |
| `format` | `format(fmt, ...args)` | string — see [FORMAT.md](FORMAT.md) |
| `set` | `set(array)` | set — deduplicates |
| `copy` | `copy(value)` | depth-1 shallow clone |
| `id` | `id(value)` | int — object identity |
| `make_weak` | `make_weak(value)` | weak reference |
| `upgrade_weak` | `upgrade_weak(w)` | the value, or null if dropped |
| `is_weak` | `is_weak(value)` | bool |
| `clock` | `clock()` | float — Unix epoch seconds |
| `sleep` | `sleep(secs)` | promise — see [TIME.md](TIME.md) |
| `random` | `random()` | float in `[0, 1)` |
| `rand_int` | `rand_int(n)` | int in `[0, n)` |
| `read_file` | `read_file(path)` | promise → string (async) |
| `write_file` | `write_file(path, contents)` | promise → null (async) |
| `append_file` | `append_file(path, contents)` | null |
| `read_lines` | `read_lines(path)` | array of strings |
| `lines_iter` | `lines_iter(path)` | iterator — streams lines without loading the file |
| `read_bytes` | `read_bytes(path)` | array of ints |
| `write_bytes` | `write_bytes(path, bytes)` | null |
| `file_exists` | `file_exists(path)` | bool |
| `is_file` | `is_file(path)` | bool |
| `is_dir` | `is_dir(path)` | bool |
| `mkdir` | `mkdir(path)` | null — creates parents |
| `list_dir` | `list_dir(path)` | array of names |
| `path_join` | `path_join(a, b, ...)` | string — needs 2+ args |
| `rename` | `rename(from, to)` | null |
| `rm` | `rm(path)` | null |
| `json_parse` | `json_parse(s)` | value — see [JSON.md](JSON.md) |
| `json_stringify` | `json_stringify(value)` | string |
| `csv_parse` | `csv_parse(s[, delim])` | array of rows — see [CSV.md](CSV.md) |
| `csv_stringify` | `csv_stringify(rows[, delim])` | string |
| `http_get` | `http_get(url[, opts])` | promise — see [HTTP.md](HTTP.md) |
| `http_post` | `http_post(url, body[, opts])` | promise |
| `tcp_listen` | `tcp_listen(addr[, opts])` | server — see [TCP.md](TCP.md) |
| `tcp_connect` | `tcp_connect(addr)` | client |
| `udp_bind` | `udp_bind(addr)` | socket |
| `load_plugin` | `load_plugin(path)` | plugin — see [PLUGINS.md](PLUGINS.md) |
| `set_workers` | `set_workers(n)` | null — I/O pool size, see [CONFIGURATION.md](CONFIGURATION.md) |
| `args` | `args` | array of command-line args (a value, not a call) |
| `pi` / `e` / `tau` | — | float constants |

Arity is enforced at runtime: calling a built-in with the wrong number of
arguments raises a `RuntimeError` naming the function and the expected count.

## Output and Input

### print(...args) / println(...args)

`print` writes each argument with no separator and no trailing newline.
`println` adds a newline. Both accept any number of arguments of any type.

```aether
print("a", "b")      // ab
println("a", "b")    // ab\n
println(42, true)    // 42true\n
```

### input([prompt])

Reads one line from stdin and returns it as a string, without the trailing
newline. With a `prompt` argument, writes the prompt first.

```aether
let name = input("Name: ")
```

## Type Inspection and Conversion

### type(value)

Returns the type name as a string. The full set:

`int`, `float`, `string`, `bool`, `null`, `array`, `dict`, `set`, `function`,
`builtin_function`, `async_function`, `struct`, `iterator`, `promise`, `error`,
`module`, `file_lines`, `enum`, `enum_constructor`, `weak`, `tcp_server`,
`tcp_connection`, `udp_socket`, `plugin`, `plugin_function`.

```aether
type(1)         // "int"
type("a")       // "string"
type([1])       // "array"
type(null)      // "null"
```

Struct instances and enum variants return their own type name, not a generic one:

```aether
struct Point { x, y }
type(Point { x: 1, y: 2 })      // "Point"
```

### len(value)

Length of a string (characters), array, dict (number of keys), or set. Raises on
other types.

### int(value[, base]) / float(value) / str(value) / bool(value)

Conversions. `int` takes an optional base for string input:

```aether
int("10")           // 10
int("ff", 16)       // 255
int(3.7)            // 3     (truncates)
float("1.5")        // 1.5
str(12)             // "12"
bool(0)             // false
```

## Number and String Encoding

`hex`, `oct`, and `bin` return prefixed strings; `int(s, base)` is the inverse.

```aether
hex(255)                    // "0xff"
oct(8)                      // "0o10"
bin(5)                      // "0b101"
base64_encode("hi")         // "aGk="
base64_decode("aGk=")       // "hi"
```

See [FORMAT.md](FORMAT.md) for `format()` and its full placeholder grammar.

## Identity, Copying, and Weak References

Arrays, dicts, and struct instances have reference semantics — `==` compares
identity, not contents.

```aether
let a = [1, 2]
let b = a
let c = copy(a)         // depth-1 shallow clone

id(a) == id(b)          // true  — same object
id(a) == id(c)          // false — different object
a.equals(c)             // true  — depth-1 structural comparison
```

`set(array)` builds a set, discarding duplicates:

```aether
set([1, 2, 2, 3])       // set(1, 2, 3)
```

Weak references let you hold a value without keeping it alive — used to break
`Rc` cycles. See [MEMORY_MANAGEMENT.md](../dev/MEMORY_MANAGEMENT.md).

```aether
let w = make_weak(obj)
is_weak(w)              // true
upgrade_weak(w)         // obj, or null once obj is dropped
```

## File System

Five built-ins run on the I/O thread pool and return a promise you `await`:
`read_file`, `write_file`, `http_get`, `http_post`, and `sleep`. Every other
file-system built-in below is synchronous.

`main()` is always a plain `fn` — `await` works directly inside it:

```aether
fn main() {
    let text = await read_file("data.txt")
    await write_file("out.txt", text)
}
```

### Reading

| Function | Behaviour |
|---|---|
| `read_file(path)` | Whole file as one string. Async. |
| `read_lines(path)` | Array of lines. Loads the whole file. |
| `lines_iter(path)` | Iterator over lines. Streams — use this for large files. |
| `read_bytes(path)` | Array of ints, one per byte. |

`lines_iter` is the memory-safe choice when the file may not fit in RAM:

```aether
for line in lines_iter("huge.log") {
    if (contains(line, "ERROR")) {
        println(line)
    }
}
```

### Writing

| Function | Behaviour |
|---|---|
| `write_file(path, contents)` | Overwrites. Async. |
| `append_file(path, contents)` | Appends. Synchronous. |
| `write_bytes(path, bytes)` | Writes an array of ints as raw bytes. |

### Inspecting and Manipulating Paths

| Function | Returns |
|---|---|
| `file_exists(path)` | bool — true for files and directories |
| `is_file(path)` | bool |
| `is_dir(path)` | bool |
| `list_dir(path)` | array of entry names |
| `mkdir(path)` | null — creates missing parent directories |
| `path_join(a, b, ...)` | joined path string; requires at least 2 arguments |
| `rename(from, to)` | null |
| `rm(path)` | null |

```aether
path_join("a", "b", "c")        // "a/b/c"
mkdir("out/nested")             // creates both levels
list_dir("src")                 // ["main.rs", "lib.rs", ...]
```

## Data Formats

`json_parse` / `json_stringify` and `csv_parse` / `csv_stringify` are documented
in full in [JSON.md](JSON.md) and [CSV.md](CSV.md).

```aether
json_stringify({"a": 1})                        // "{\"a\":1}"
csv_stringify([["a", "b"], ["1", "2"]])         // "a,b\n1,2"
csv_parse("a,b\n1,2")                           // [["a", "b"], ["1", "2"]]
```

Both CSV functions take an optional delimiter as their second argument.

## Time and Randomness

```aether
clock()             // 1753800000.123 — Unix epoch seconds as a float
await sleep(0.5)    // async; returns a promise
random()            // float in [0, 1)
rand_int(6)         // int in [0, 6)
```

Details in [TIME.md](TIME.md) and [RANDOM.md](RANDOM.md).

## Network

| Function | Doc |
|---|---|
| `http_get(url[, opts])` | [HTTP.md](HTTP.md) |
| `http_post(url, body[, opts])` | [HTTP.md](HTTP.md) |
| `tcp_listen(addr[, opts])` | [TCP.md](TCP.md) |
| `tcp_connect(addr)` | [TCP.md](TCP.md) |
| `udp_bind(addr)` | [TCP.md](TCP.md) |

`opts` for the HTTP functions is a dict accepting `timeout` and `user_agent`.

## Runtime and Process

### args

The command-line arguments passed after the script path, as an array of strings.
It is a value, not a function — do not call it.

```aether
// aether script.ae foo bar  →  args == ["foo", "bar"]
for a in args {
    println(a)
}
```

At most 100 script arguments are accepted.

### set_workers(n)

Sets the I/O thread pool size at runtime. The `AETHER_IO_WORKERS` environment
variable does the same thing at startup. See
[CONFIGURATION.md](CONFIGURATION.md).

### load_plugin(path)

Loads a Rust shared library and exposes its functions as methods. See
[PLUGINS.md](PLUGINS.md) and [PLUGIN_GUIDE.md](PLUGIN_GUIDE.md).

## Math Constants

`pi`, `e`, and `tau` are float values, not functions:

```aether
pi      // 3.141592653589793
e       // 2.718281828459045
tau     // 6.283185307179586
```

Math *functions* (`sqrt`, `abs`, `floor`, …) are stdlib, not built-ins — see
[STDLIB.md](STDLIB.md).

---

## Adding a Built-in

Implement it in `src/interpreter/builtins.rs`, register the name in the
`BUILTINS` slice in `src/checker.rs`, and follow
[.claude/rules/add-builtin.md](../../.claude/rules/add-builtin.md). For a
built-in that performs blocking I/O, use
[.claude/rules/add-async-builtin.md](../../.claude/rules/add-async-builtin.md)
instead — it must go through `try_submit_io_task`.

---
[← Standard Library](STDLIB.md) &nbsp;&nbsp; [Format →](FORMAT.md)
