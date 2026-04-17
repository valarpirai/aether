package com.aether.interpreter;

import com.aether.parser.ast.Stmt;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Function;

/**
 * Sealed hierarchy of all runtime values in the Aether interpreter.
 *
 * <p>Mirrors the {@code Value} enum from the Rust implementation.
 *
 * <ul>
 *   <li>Immutable value types: {@link IntVal}, {@link FloatVal}, {@link Str}, {@link Bool}, {@link
 *       Null}
 *   <li>Container types: {@link Array}, {@link Dict} (insertion-ordered)
 *   <li>Callable types: {@link AetherFunction}, {@link Builtin}
 *   <li>Namespace type: {@link Module}
 *   <li>Struct types: {@link StructDef} (blueprint), {@link Instance} (runtime object)
 * </ul>
 */
public sealed interface Value
    permits Value.IntVal,
        Value.FloatVal,
        Value.Str,
        Value.Bool,
        Value.Null,
        Value.Array,
        Value.Dict,
        Value.AetherFunction,
        Value.Builtin,
        Value.Module,
        Value.StructDef,
        Value.Instance {

  // ── Primitive value types ────────────────────────────────────────────────────

  /** 64-bit integer. */
  record IntVal(long value) implements Value {
    @Override
    public String typeName() {
      return "int";
    }
  }

  /** 64-bit float. */
  record FloatVal(double value) implements Value {
    @Override
    public String typeName() {
      return "float";
    }
  }

  /** UTF-8 string (immutable). */
  record Str(String value) implements Value {
    @Override
    public String typeName() {
      return "string";
    }
  }

  /** Boolean value. */
  record Bool(boolean value) implements Value {
    @Override
    public String typeName() {
      return "bool";
    }
  }

  /** The null value (singleton via {@link #INSTANCE}). */
  final class Null implements Value {
    public static final Null INSTANCE = new Null();

    private Null() {}

    @Override
    public String typeName() {
      return "null";
    }

    @Override
    public String toString() {
      return "null";
    }
  }

  // ── Container types ──────────────────────────────────────────────────────────

  /**
   * Ordered array of values.
   *
   * <p>Arrays are functionally immutable in Aether: {@code push} / {@code pop} return a new list.
   * Use {@code AtomicReference<List<Value>>} as the underlying store so mutations made through a
   * shared reference are visible to all holders.
   */
  final class Array implements Value {
    private final AtomicReference<List<Value>> elements;

    public Array(List<Value> elements) {
      this.elements = new AtomicReference<>(List.copyOf(elements));
    }

    public List<Value> elements() {
      return elements.get();
    }

    public void setElements(List<Value> newElements) {
      elements.set(List.copyOf(newElements));
    }

    @Override
    public String typeName() {
      return "array";
    }
  }

  /**
   * Insertion-ordered dictionary (key → value).
   *
   * <p>Aether dicts preserve insertion order (like Python dicts). Keys must be strings, ints, or
   * bools. Backed by {@link java.util.LinkedHashMap}.
   */
  final class Dict implements Value {
    private final java.util.LinkedHashMap<Value, Value> entries;

    public Dict(java.util.LinkedHashMap<Value, Value> entries) {
      this.entries = entries;
    }

    public java.util.LinkedHashMap<Value, Value> entries() {
      return entries;
    }

    @Override
    public String typeName() {
      return "dict";
    }
  }

  // ── Callable types ───────────────────────────────────────────────────────────

  /**
   * User-defined function with a closure environment.
   *
   * <p>Mirrors {@code Value::Function { params, body, closure }} from Rust. The closure captures
   * the environment at the point of function definition.
   */
  record AetherFunction(List<String> params, Stmt body, Environment closure) implements Value {
    @Override
    public String typeName() {
      return "function";
    }
  }

  /**
   * Built-in (native Java) function.
   *
   * <p>{@code arity == -1} means variadic (any number of arguments).
   */
  record Builtin(String name, int arity, Function<List<Value>, Value> impl) implements Value {
    @Override
    public String typeName() {
      return "builtin_function";
    }
  }

  // ── Module namespace ─────────────────────────────────────────────────────────

  /** Module value: an immutable namespace of top-level bindings. */
  record Module(String name, Map<String, Value> members) implements Value {
    @Override
    public String typeName() {
      return "module";
    }
  }

  // ── Struct types ─────────────────────────────────────────────────────────────

  /**
   * Struct definition (blueprint).
   *
   * <p>Contains declared field names and a method map. Calling a StructDef creates an {@link
   * Instance}.
   */
  record StructDef(String name, List<String> fields, Map<String, MethodEntry> methods)
      implements Value {
    @Override
    public String typeName() {
      return "struct";
    }
  }

  /** A method stored inside a {@link StructDef}. */
  record MethodEntry(List<String> params, Stmt body) {}

  /**
   * Struct instance (runtime object).
   *
   * <p>Fields are held in a {@link HashMap} wrapped in a shared reference so that mutations via
   * {@code self.field = value} inside methods are reflected on the original instance.
   */
  final class Instance implements Value {
    private final String typeName;
    private final Map<String, Value> fields;
    private final Map<String, MethodEntry> methods;

    public Instance(String typeName, Map<String, Value> fields, Map<String, MethodEntry> methods) {
      this.typeName = typeName;
      this.fields = fields;
      this.methods = methods;
    }

    @Override
    public String typeName() {
      return typeName;
    }

    public Map<String, Value> fields() {
      return fields;
    }

    public Map<String, MethodEntry> methods() {
      return methods;
    }
  }

  // ── Shared helpers ───────────────────────────────────────────────────────────

  /** Returns the Aether type name string for this value. */
  String typeName();

  /**
   * Truthiness rules matching the Rust implementation.
   *
   * <ul>
   *   <li>{@code false}, {@code null}, {@code 0}, {@code 0.0}, {@code ""}, {@code []} → falsy
   *   <li>everything else → truthy
   * </ul>
   */
  default boolean isTruthy() {
    return switch (this) {
      case Bool b -> b.value();
      case Null n -> false;
      case IntVal i -> i.value() != 0;
      case FloatVal f -> f.value() != 0.0;
      case Str s -> !s.value().isEmpty();
      case Array a -> !a.elements().isEmpty();
      default -> true;
    };
  }
}
