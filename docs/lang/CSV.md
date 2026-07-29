---
layout: default
title: "Aether — CSV"
---

[Home](../index.md) › Language Reference › CSV

# CSV

Aether has two built-in functions for working with CSV (comma-separated values): `csv_parse()` converts a CSV string into an array of arrays, and `csv_stringify()` converts an array of arrays back into a CSV string.

Both functions accept an optional delimiter so they work with TSV, semicolon-delimited files, and other formats.

## csv_parse(text)  /  csv_parse(text, delimiter)

Parses a CSV string and returns an array of rows, where each row is an array of string fields.

```aether
let text = "name,age,city\nAlice,30,New York\nBob,25,London"
let rows = csv_parse(text)
println(rows[0])       // ["name", "age", "city"]
println(rows[1][0])    // Alice
println(rows[1][2])    // New York
```

**Quoted fields** containing commas or newlines are handled automatically:

```aether
let text = "\"Smith, John\",42,\"New York, NY\""
let row = csv_parse(text)[0]
println(row[0])   // Smith, John
println(row[2])   // New York, NY
```

**Custom delimiter** — pass a second argument to parse TSV or other formats:

```aether
let tsv = "a\tb\tc\n1\t2\t3"
let rows = csv_parse(tsv, "\t")
println(rows[0])   // ["a", "b", "c"]
println(rows[1])   // ["1", "2", "3"]
```

## csv_stringify(rows)  /  csv_stringify(rows, delimiter)

Converts an array of row arrays into a CSV string. Fields containing the delimiter, double-quotes, or newlines are automatically quoted and escaped.

```aether
let data = [
    ["product", "price", "note"],
    ["Apple", "1.50", "fresh"],
    ["Banana", "0.75", "large, ripe"]
]
println(csv_stringify(data))
```

Output:

```
product,price,note
Apple,1.50,fresh
Banana,0.75,"large, ripe"
```

**Custom delimiter:**

```aether
let ssv = csv_stringify(data, ";")
println(ssv)
// product;price;note
// Apple;1.50;fresh
// Banana;0.75;large, ripe
```

## Roundtrip

`csv_stringify` followed by `csv_parse` with the same delimiter reproduces the original data:

```aether
fn main() {
    let original = [
        ["city", "note"],
        ["New York", "big, busy"],
        ["say \"hi\"", "quoted value"]
    ]
    let encoded = csv_stringify(original)
    let decoded = csv_parse(encoded)
    println(decoded.equals(original))   // true
}
```

## Type notes

- All fields are returned as **strings** by `csv_parse`. Convert with `int()` or `float()` as needed.
- `csv_stringify` calls `str()` on each field before encoding, so non-string values (int, float, bool) are accepted.
- Empty fields produce empty strings `""`.

## Related

- [JSON](JSON.md) — structured data with types via `json_parse` / `json_stringify`
- [File I/O](STDLIB.md#file-io) — read CSV files with `read_file`

---
[← JSON](JSON.md) &nbsp;&nbsp; [HTTP →](HTTP.md)
