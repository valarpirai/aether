# JSON

Aether has two built-in functions for working with JSON: `json_parse()` converts a JSON string into Aether values, and `json_stringify()` converts Aether values back into a JSON string.

## json_parse(string)

Parses a JSON string and returns the equivalent Aether value.

```aether
let data = json_parse('{"name": "Alice", "age": 30}')
println(data["name"])  // Alice
println(data["age"])   // 30
```

**Type mapping:**

| JSON | Aether |
|------|--------|
| object | dict |
| array | array |
| string | string |
| number | int or float |
| boolean | bool |
| null | null |

Throws an error if the input is not valid JSON.

## json_stringify(value)

Converts an Aether value to a compact JSON string.

```aether
let data = {"name": "Bob", "active": true, "score": 95}
println(json_stringify(data))  // {"name":"Bob","active":true,"score":95}
```

Dict keys must be strings; int/bool keys will cause an error. Functions, sets, and struct instances cannot be serialized.

## Examples

### Parse an API response

```aether
fn main() {
    let json = read_file("response.json")
    let user = json_parse(json)
    println("Hello, ${user["name"]}")
}
```

### Save and load state

```aether
fn main() {
    let state = {"level": 5, "score": 1000, "items": ["sword", "shield"]}
    write_file("save.json", json_stringify(state))

    let loaded = json_parse(read_file("save.json"))
    println("Level: ${loaded["level"]}")
}
```

### Safe parse

```aether
fn safe_parse(text) {
    try {
        return json_parse(text)
    } catch(e) {
        return null
    }
}
```

## Limitations

- Output is compact — no indentation or pretty printing
- Dict keys must be strings for `json_stringify`
- Functions, sets, and struct instances cannot be serialized
