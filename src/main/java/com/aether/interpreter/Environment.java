package com.aether.interpreter;

import com.aether.exception.AetherRuntimeException;
import java.util.HashMap;
import java.util.Map;
import lombok.Getter;

/**
 * Lexically scoped variable environment.
 *
 * <p>Each function call or block creates a child environment that delegates unknown lookups to its
 * parent chain — implementing lexical (static) scoping.
 */
public final class Environment {

  private final Map<String, Value> values = new HashMap<>();
  @Getter private final Environment parent;

  /** Create a root (global) environment with no parent. */
  public Environment() {
    this.parent = null;
  }

  /** Create a child environment with the given parent scope. */
  public Environment(Environment parent) {
    this.parent = parent;
  }

  /**
   * Define a new binding in the current scope.
   *
   * <p>Shadows any binding with the same name in parent scopes.
   */
  public void define(String name, Value value) {
    values.put(name, value);
  }

  /**
   * Look up a variable, searching this scope then parent scopes.
   *
   * @throws AetherRuntimeException.UndefinedVariable if the name is not found anywhere
   */
  public Value get(String name) {
    if (values.containsKey(name)) {
      return values.get(name);
    }
    if (parent != null) {
      return parent.get(name);
    }
    throw new AetherRuntimeException.UndefinedVariable(name);
  }

  /**
   * Assign a value to an existing binding in the nearest enclosing scope that defines it.
   *
   * @throws AetherRuntimeException.UndefinedVariable if the name is not defined anywhere
   */
  public void set(String name, Value value) {
    if (values.containsKey(name)) {
      values.put(name, value);
      return;
    }
    if (parent != null) {
      parent.set(name, value);
      return;
    }
    throw new AetherRuntimeException.UndefinedVariable(name);
  }

  /**
   * Return all bindings defined directly in this scope (not parents).
   *
   * <p>Used when importing a module's top-level definitions.
   */
  public Map<String, Value> localBindings() {
    return Map.copyOf(values);
  }
}
