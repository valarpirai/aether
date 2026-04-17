package com.aether.interpreter;

import com.aether.exception.AetherRuntimeException;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.function.Function;

/**
 * All built-in (native Java) functions for the Aether interpreter.
 *
 * <p>Each method returns a {@link Value.Builtin} ready to be registered in the global {@link
 * Environment}. Arity {@code -1} means variadic.
 */
public final class Builtins {

  private static final ObjectMapper JSON = new ObjectMapper();

  private Builtins() {}

  // ── I/O ─────────────────────────────────────────────────────────────────────

  /** {@code print(...values)} — print to stdout without a newline. */
  public static Value.Builtin print() {
    return builtin("print", -1, args -> {
      for (int i = 0; i < args.size(); i++) {
        if (i > 0) System.out.print(" ");
        System.out.print(display(args.get(i)));
      }
      return Value.Null.INSTANCE;
    });
  }

  /** {@code println(...values)} — print to stdout with a trailing newline. */
  public static Value.Builtin println() {
    return builtin("println", -1, args -> {
      for (int i = 0; i < args.size(); i++) {
        if (i > 0) System.out.print(" ");
        System.out.print(display(args.get(i)));
      }
      System.out.println();
      return Value.Null.INSTANCE;
    });
  }

  /** {@code input([prompt])} — read a line from stdin, printing an optional prompt first. */
  public static Value.Builtin input() {
    return builtin("input", -1, args -> {
      if (!args.isEmpty()) {
        System.out.print(display(args.get(0)));
        System.out.flush();
      }
      try {
        BufferedReader reader = new BufferedReader(new InputStreamReader(System.in));
        String line = reader.readLine();
        return new Value.Str(line != null ? line : "");
      } catch (IOException e) {
        throw new AetherRuntimeException.InvalidOperation("input() failed: " + e.getMessage());
      }
    });
  }

  /** {@code read_file(path)} — read a file and return its contents as a string. */
  public static Value.Builtin readFile() {
    return builtin("read_file", 1, args -> {
      String path = requireStr(args.get(0), "read_file");
      try {
        return new Value.Str(Files.readString(Path.of(path)));
      } catch (IOException e) {
        throw new AetherRuntimeException.InvalidOperation("read_file failed: " + e.getMessage());
      }
    });
  }

  /** {@code write_file(path, content)} — write a string to a file. */
  public static Value.Builtin writeFile() {
    return builtin("write_file", 2, args -> {
      String path = requireStr(args.get(0), "write_file");
      String content = display(args.get(1));
      try {
        Files.writeString(Path.of(path), content);
        return Value.Null.INSTANCE;
      } catch (IOException e) {
        throw new AetherRuntimeException.InvalidOperation("write_file failed: " + e.getMessage());
      }
    });
  }

  // ── Type system ──────────────────────────────────────────────────────────────

  /** {@code type(value)} — return the type name as a string. */
  public static Value.Builtin type() {
    return builtin("type", 1, args -> new Value.Str(args.get(0).typeName()));
  }

  /** {@code len(collection)} — return the length of a string or array. */
  public static Value.Builtin len() {
    return builtin("len", 1, args -> {
      Value v = args.get(0);
      return switch (v) {
        case Value.Str(String s) -> new Value.IntVal(s.length());
        case Value.Array arr -> new Value.IntVal(arr.elements().size());
        case Value.Dict dict -> new Value.IntVal(dict.entries().size());
        default -> throw new AetherRuntimeException.TypeError("string or array", v.typeName());
      };
    });
  }

  /** {@code int(value)} — convert to integer. */
  public static Value.Builtin toInt() {
    return builtin("int", 1, args -> {
      Value v = args.get(0);
      return switch (v) {
        case Value.IntVal(long n) -> v;
        case Value.FloatVal(double f) -> new Value.IntVal((long) f);
        case Value.Str(String s) -> {
          try {
            yield new Value.IntVal(Long.parseLong(s.trim()));
          } catch (NumberFormatException e) {
            throw new AetherRuntimeException.InvalidOperation("Cannot convert '" + s + "' to int");
          }
        }
        case Value.Bool(boolean b) -> new Value.IntVal(b ? 1 : 0);
        default -> throw new AetherRuntimeException.TypeError("number, string, or bool", v.typeName());
      };
    });
  }

  /** {@code float(value)} — convert to float. */
  public static Value.Builtin toFloat() {
    return builtin("float", 1, args -> {
      Value v = args.get(0);
      return switch (v) {
        case Value.IntVal(long n) -> new Value.FloatVal(n);
        case Value.FloatVal ignored -> v;
        case Value.Str(String s) -> {
          try {
            yield new Value.FloatVal(Double.parseDouble(s.trim()));
          } catch (NumberFormatException e) {
            throw new AetherRuntimeException.InvalidOperation(
                "Cannot convert '" + s + "' to float");
          }
        }
        case Value.Bool(boolean b) -> new Value.FloatVal(b ? 1.0 : 0.0);
        default -> throw new AetherRuntimeException.TypeError("number, string, or bool", v.typeName());
      };
    });
  }

  /** {@code str(value)} — convert to string. */
  public static Value.Builtin toStr() {
    return builtin("str", 1, args -> new Value.Str(display(args.get(0))));
  }

  /** {@code bool(value)} — convert to boolean using truthiness rules. */
  public static Value.Builtin toBool() {
    return builtin("bool", 1, args -> new Value.Bool(args.get(0).isTruthy()));
  }

  // ── Time ─────────────────────────────────────────────────────────────────────

  /** {@code clock()} — seconds since Unix epoch as a float. */
  public static Value.Builtin clock() {
    return builtin("clock", 0, args ->
        new Value.FloatVal(System.currentTimeMillis() / 1000.0));
  }

  /** {@code sleep(seconds)} — pause execution for the given duration. */
  public static Value.Builtin sleep() {
    return builtin("sleep", 1, args -> {
      Value v = args.get(0);
      double seconds = switch (v) {
        case Value.IntVal(long n) -> n;
        case Value.FloatVal(double f) -> f;
        default -> throw new AetherRuntimeException.TypeError("number", v.typeName());
      };
      try {
        Thread.sleep((long) (seconds * 1000));
      } catch (InterruptedException e) {
        Thread.currentThread().interrupt();
      }
      return Value.Null.INSTANCE;
    });
  }

  // ── JSON ─────────────────────────────────────────────────────────────────────

  /** {@code json_parse(string)} — parse a JSON string into Aether values. */
  public static Value.Builtin jsonParse() {
    return builtin("json_parse", 1, args -> {
      String json = requireStr(args.get(0), "json_parse");
      try {
        return jsonNodeToValue(JSON.readTree(json));
      } catch (IOException e) {
        throw new AetherRuntimeException.InvalidOperation("json_parse failed: " + e.getMessage());
      }
    });
  }

  /** {@code json_stringify(value)} — serialise an Aether value to a JSON string. */
  public static Value.Builtin jsonStringify() {
    return builtin("json_stringify", 1, args -> {
      try {
        return new Value.Str(JSON.writeValueAsString(valueToJsonNode(args.get(0))));
      } catch (IOException e) {
        throw new AetherRuntimeException.InvalidOperation(
            "json_stringify failed: " + e.getMessage());
      }
    });
  }

  // ── Display logic ────────────────────────────────────────────────────────────

  /** Display a Value as Aether would print it. */
  public static String display(Value value) {
    return switch (value) {
      case Value.IntVal(long n) -> Long.toString(n);
      case Value.FloatVal(double f) -> {
        if (f == Math.floor(f) && !Double.isInfinite(f)) {
          yield Long.toString((long) f) + ".0";
        }
        yield Double.toString(f);
      }
      case Value.Str(String s) -> s;
      case Value.Bool(boolean b) -> Boolean.toString(b);
      case Value.Null ignored -> "null";
      case Value.Array arr -> {
        StringBuilder sb = new StringBuilder("[");
        List<Value> elements = arr.elements();
        for (int i = 0; i < elements.size(); i++) {
          if (i > 0) sb.append(", ");
          sb.append(displayRepr(elements.get(i)));
        }
        sb.append("]");
        yield sb.toString();
      }
      case Value.Dict dict -> {
        StringBuilder sb = new StringBuilder("{");
        boolean first = true;
        for (Map.Entry<Value, Value> e : dict.entries().entrySet()) {
          if (!first) sb.append(", ");
          sb.append(displayRepr(e.getKey())).append(": ").append(displayRepr(e.getValue()));
          first = false;
        }
        sb.append("}");
        yield sb.toString();
      }
      case Value.AetherFunction fn -> "<fn " + fn.params() + ">";
      case Value.Builtin b -> "<builtin " + b.name() + ">";
      case Value.Module(String name, Map<String, Value> members) -> "<module " + name + ">";
      case Value.StructDef sd -> "<struct " + sd.name() + ">";
      case Value.Instance inst -> {
        StringBuilder sb = new StringBuilder(inst.typeName() + " { ");
        boolean first = true;
        for (Map.Entry<String, Value> e : inst.fields().entrySet()) {
          if (!first) sb.append(", ");
          sb.append(e.getKey()).append(": ").append(displayRepr(e.getValue()));
          first = false;
        }
        sb.append(" }");
        yield sb.toString();
      }
    };
  }

  /** Display with quotes around strings (for use inside collections). */
  private static String displayRepr(Value value) {
    if (value instanceof Value.Str(String s)) {
      return "\"" + s + "\"";
    }
    return display(value);
  }

  // ── JSON helpers ─────────────────────────────────────────────────────────────

  private static Value jsonNodeToValue(JsonNode node) {
    if (node.isNull()) return Value.Null.INSTANCE;
    if (node.isBoolean()) return new Value.Bool(node.booleanValue());
    if (node.isIntegralNumber()) return new Value.IntVal(node.longValue());
    if (node.isFloatingPointNumber()) return new Value.FloatVal(node.doubleValue());
    if (node.isTextual()) return new Value.Str(node.textValue());
    if (node.isArray()) {
      List<Value> elements = new ArrayList<>();
      for (JsonNode child : node) {
        elements.add(jsonNodeToValue(child));
      }
      return new Value.Array(elements);
    }
    if (node.isObject()) {
      LinkedHashMap<Value, Value> map = new LinkedHashMap<>();
      Iterator<Map.Entry<String, JsonNode>> fields = node.fields();
      while (fields.hasNext()) {
        Map.Entry<String, JsonNode> field = fields.next();
        map.put(new Value.Str(field.getKey()), jsonNodeToValue(field.getValue()));
      }
      return new Value.Dict(map);
    }
    return Value.Null.INSTANCE;
  }

  private static JsonNode valueToJsonNode(Value value) {
    return switch (value) {
      case Value.Null ignored -> JSON.nullNode();
      case Value.Bool(boolean b) -> JSON.getNodeFactory().booleanNode(b);
      case Value.IntVal(long n) -> JSON.getNodeFactory().numberNode(n);
      case Value.FloatVal(double f) -> JSON.getNodeFactory().numberNode(f);
      case Value.Str(String s) -> JSON.getNodeFactory().textNode(s);
      case Value.Array arr -> {
        ArrayNode arr2 = JSON.createArrayNode();
        for (Value v : arr.elements()) arr2.add(valueToJsonNode(v));
        yield arr2;
      }
      case Value.Dict dict -> {
        ObjectNode obj = JSON.createObjectNode();
        for (Map.Entry<Value, Value> e : dict.entries().entrySet()) {
          obj.set(display(e.getKey()), valueToJsonNode(e.getValue()));
        }
        yield obj;
      }
      default ->
          throw new AetherRuntimeException.InvalidOperation(
              "Cannot serialize " + value.typeName() + " to JSON");
    };
  }

  // ── Helpers ──────────────────────────────────────────────────────────────────

  private static Value.Builtin builtin(
      String name, int arity, Function<List<Value>, Value> impl) {
    return new Value.Builtin(name, arity, impl);
  }

  private static String requireStr(Value v, String fnName) {
    if (v instanceof Value.Str(String s)) return s;
    throw new AetherRuntimeException.TypeError("string", v.typeName());
  }
}
