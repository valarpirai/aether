# JSON Support in Aether

**Status**: ✅ Complete  
**Added**: Phase 5  
**Tests**: 25 tests passing  
**Backend**: serde_json

## Overview

Aether provides built-in JSON parsing and serialization through `json_parse()` and `json_stringify()` functions.

## Functions

### json_parse(string)

Parse a JSON string into Aether values:

```aether
let json_text = '{"name": "Alice", "age": 30}'
let data = json_parse(json_text)
println(data["name"])  // Alice
println(data["age"])   // 30
```

**Mapping**:
- JSON object → Aether dict
- JSON array → Aether array
- JSON string → Aether string
- JSON number → Aether int or float
- JSON boolean → Aether bool
- JSON null → Aether null

### json_stringify(value)

Convert Aether values to JSON string:

```aether
let data = {"name": "Bob", "active": true, "score": 95}
let json = json_stringify(data)
println(json)  // {"name":"Bob","active":true,"score":95}
```

**Supported Types**:
- Dict → JSON object (keys must be strings)
- Array → JSON array
- String → JSON string
- Int/Float → JSON number
- Bool → JSON boolean
- Null → JSON null

## Examples

### Example 1: API Response Parsing

```aether
fn parse_user_data(json_response) {
    let user = json_parse(json_response)
    return {
        "id": user["id"],
        "name": user["name"],
        "email": user["email"]
    }
}

fn main() {
    let response = '{"id": 123, "name": "Alice", "email": "alice@example.com"}'
    let user = parse_user_data(response)
    println("User:", user["name"])
}
```

### Example 2: Configuration Files

```aether
fn load_config(json_string) {
    let config = json_parse(json_string)
    return {
        "debug": config["debug"],
        "port": config["server"]["port"],
        "host": config["server"]["host"]
    }
}

fn main() {
    let config_json = read_file("config.json")
    let config = load_config(config_json)
    println("Server: ${config["host"]}:${config["port"]}")
}
```

### Example 3: Data Serialization

```aether
fn save_state(state) {
    let json = json_stringify(state)
    write_file("state.json", json)
    println("State saved")
}

fn main() {
    let state = {
        "level": 5,
        "score": 1000,
        "inventory": ["sword", "shield", "potion"]
    }
    save_state(state)
}
```

### Example 4: Array Processing

```aether
fn process_items(json_array) {
    let items = json_parse(json_array)
    let total = 0
    for item in items {
        total = total + item["price"]
    }
    return total
}

fn main() {
    let data = '[{"name":"apple","price":1},{"name":"banana","price":2}]'
    let total = process_items(data)
    println("Total: ${total}")  // Total: 3
}
```

## Error Handling

### Invalid JSON

```aether
try {
    let data = json_parse("{invalid json}")
} catch(e) {
    println("Parse error:", e)
}
```

### Non-String Keys in Stringify

```aether
let bad_dict = {1: "one", 2: "two"}  // Int keys
try {
    json_stringify(bad_dict)
} catch(e) {
    println("Error: dict keys must be strings for JSON")
}
```

## Best Practices

### 1. Validate After Parsing

```aether
fn safe_parse(json_text) {
    try {
        return json_parse(json_text)
    } catch(e) {
        println("JSON parse failed:", e)
        return null
    }
}
```

### 2. Use String Keys for JSON-Compatible Dicts

```aether
// Good - will serialize to JSON
let data = {"name": "Alice", "age": 30}

// Won't work with json_stringify
let data = {1: "one", true: "yes"}
```

### 3. Pretty Print for Debugging

```aether
// json_stringify doesn't pretty-print, but you can format manually
fn debug_json(data) {
    println(json_stringify(data))
}
```

## Limitations

- **No Pretty Printing**: JSON output is compact (no indentation)
- **String Keys Only**: Dict keys must be strings for `json_stringify()`
- **No Custom Serialization**: Functions, sets, and custom types can't be serialized
- **No Comments**: JSON doesn't support comments

## See Also

- [DESIGN.md](DESIGN.md) - Language specification
- [examples/](../examples/) - Usage examples

---

**Last Updated**: 2026-04-28  
**Status**: Complete and stable
