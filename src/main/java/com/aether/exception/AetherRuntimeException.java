package com.aether.exception;

/**
 * Base class for all Aether runtime errors.
 *
 * <p>Subclasses correspond to the {@code RuntimeError} enum variants in the Rust source.
 */
public abstract sealed class AetherRuntimeException extends RuntimeException
    permits AetherRuntimeException.UndefinedVariable,
        AetherRuntimeException.TypeError,
        AetherRuntimeException.DivisionByZero,
        AetherRuntimeException.IndexOutOfBounds,
        AetherRuntimeException.InvalidOperation,
        AetherRuntimeException.ArityMismatch,
        AetherRuntimeException.StackOverflow,
        AetherRuntimeException.Thrown {

  protected AetherRuntimeException(String message) {
    super(message);
  }

  /** A variable was referenced before it was defined. */
  public static final class UndefinedVariable extends AetherRuntimeException {
    public UndefinedVariable(String name) {
      super("Undefined variable '" + name + "'");
    }
  }

  /** A value of the wrong type was supplied to an operation. */
  public static final class TypeError extends AetherRuntimeException {
    public TypeError(String expected, String got) {
      super("Type error: expected " + expected + ", got " + got);
    }
  }

  /** Division or modulo by zero. */
  public static final class DivisionByZero extends AetherRuntimeException {
    public DivisionByZero() {
      super("Division by zero");
    }
  }

  /** Array or string index out of range. */
  public static final class IndexOutOfBounds extends AetherRuntimeException {
    public IndexOutOfBounds(long index, int length) {
      super("Index " + index + " out of bounds for length " + length);
    }
  }

  /** Generic runtime operation failure. */
  public static final class InvalidOperation extends AetherRuntimeException {
    public InvalidOperation(String message) {
      super(message);
    }
  }

  /** Wrong number of arguments passed to a function. */
  public static final class ArityMismatch extends AetherRuntimeException {
    public ArityMismatch(int expected, int got) {
      super("Expected " + expected + " argument(s), got " + got);
    }
  }

  /** Call stack depth exceeded the configured limit. */
  public static final class StackOverflow extends AetherRuntimeException {
    public StackOverflow(int depth, int limit) {
      super("Stack overflow: depth " + depth + " exceeds limit " + limit);
    }
  }

  /** Value thrown by the Aether {@code throw} statement. */
  public static final class Thrown extends AetherRuntimeException {
    public Thrown(String message) {
      super(message);
    }
  }
}
