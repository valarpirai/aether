---
layout: default
title: "Aether — Format"
---

[Home](../index.html) › Language Reference › Format

# Format

`format(fmt, ...args)` renders a template string using `{}` placeholders, similar to Rust's `format!` or Python's `str.format`.

## Basic placeholders

`{}` is replaced by the next argument, in order, using its default string representation.

```aether
format("Hello, {}!", "Alice")      // "Hello, Alice!"
format("{} + {} = {}", 1, 2, 3)    // "1 + 2 = 3"
```

Extra arguments beyond the number of placeholders are ignored.

## Format specs — `{:spec}`

A placeholder may include a spec after a colon: `{:[[fill]align][width][.precision][type]}`.

### Precision — floats

```aether
format("{:.2f}", 3.14159)   // "3.14"
format("{:.0f}", 9.7)       // "10"
```

### Width and alignment

`<` left-aligns, `>` right-aligns, `^` centers. Numbers default to right-aligned; strings default to left-aligned.

```aether
format("{:>10}", "hi")   // "        hi"
format("{:<10}", "hi")   // "hi        "
format("{:^10}", "hi")   // "    hi    "
```

### Fill character

A character placed immediately before the alignment symbol is used as padding instead of a space.

```aether
format("{:0>5d}", 42)    // "00042"
format("{:->8}", "x")    // "-------x"
```

### Type specifiers

| Type | Meaning |
|------|---------|
| `f` | float, with optional `.precision` (default 6) |
| `d` | integer, decimal |
| `x` | integer, lowercase hex |
| `o` | integer, octal |
| `b` | integer, binary |
| `s` | string (default) |

```aether
format("{:x}", 255)   // "ff"
format("{:o}", 8)     // "10"
format("{:b}", 5)     // "101"
```

If a value's width exceeds the requested width, the value is not truncated.

## Escaping braces

`{{` and `}}` produce a literal `{` and `}`.

```aether
format("{{not a placeholder}}")   // "{not a placeholder}"
```

## Errors

`format()` throws when:
- the format string has more `{}` placeholders than arguments given
- a `{` is unclosed, or a `}` is unmatched
- a type specifier is not one of `f`, `d`, `x`, `o`, `b`, `s`
- a value's type doesn't match the requested type specifier (e.g. `{:f}` on a string)

## Related

- [Strings](STRINGS.html) — interpolation with `"${expr}"`, indexing, slicing

---
[← Strings](STRINGS.html) &nbsp;&nbsp; [Error Handling →](ERROR_HANDLING.html)
