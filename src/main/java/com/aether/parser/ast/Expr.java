package com.aether.parser.ast;

import java.util.List;

/**
 * Sealed hierarchy of all expression node types in the Aether AST.
 *
 * <p>Use {@code switch} with pattern matching (Java 21+) to exhaustively handle every variant.
 */
public sealed interface Expr
    permits Expr.IntLiteral,
        Expr.FloatLiteral,
        Expr.StringLiteral,
        Expr.BoolLiteral,
        Expr.NullLiteral,
        Expr.Identifier,
        Expr.Binary,
        Expr.Unary,
        Expr.Call,
        Expr.Array,
        Expr.Dict,
        Expr.Index,
        Expr.Slice,
        Expr.Spread,
        Expr.Member,
        Expr.FunctionExpr,
        Expr.StringInterp,
        Expr.StructInit {

  /** Integer literal: {@code 42}. */
  record IntLiteral(long value) implements Expr {}

  /** Float literal: {@code 3.14}. */
  record FloatLiteral(double value) implements Expr {}

  /** Plain string literal: {@code "hello"}. */
  record StringLiteral(String value) implements Expr {}

  /** Boolean literal: {@code true} or {@code false}. */
  record BoolLiteral(boolean value) implements Expr {}

  /** Null literal: {@code null}. */
  record NullLiteral() implements Expr {}

  /** Variable reference: {@code myVar}. */
  record Identifier(String name) implements Expr {}

  /** Binary operation: {@code left op right}. */
  record Binary(Expr left, BinaryOp op, Expr right) implements Expr {}

  /** Unary operation: {@code op operand}. */
  record Unary(UnaryOp op, Expr operand) implements Expr {}

  /** Function or method call: {@code callee(args)}. */
  record Call(Expr callee, List<Expr> args) implements Expr {}

  /** Array literal: {@code [e1, e2, ...]}. Elements may include {@link Spread}. */
  record Array(List<Expr> elements) implements Expr {}

  /** Dictionary literal: {@code {k1: v1, k2: v2}}. */
  record Dict(List<DictEntry> entries) implements Expr {}

  /** A single key-value pair inside a dict literal. */
  record DictEntry(Expr key, Expr value) {}

  /** Index access: {@code obj[index]}. */
  record Index(Expr object, Expr index) implements Expr {}

  /**
   * Slice access: {@code obj[start:end]}.
   *
   * <p>{@code start} and {@code end} are optional (null means "from beginning" / "to end").
   */
  record Slice(Expr object, Expr start, Expr end) implements Expr {}

  /** Spread expression: {@code ...expr} — valid only inside array literals. */
  record Spread(Expr inner) implements Expr {}

  /** Member access: {@code obj.member}. */
  record Member(Expr object, String member) implements Expr {}

  /** Anonymous function expression: {@code fn(params) \{ body \}}. */
  record FunctionExpr(List<String> params, Stmt body) implements Expr {}

  /**
   * String interpolation: alternating {@link StringLiteral} and expression nodes.
   *
   * <p>E.g. {@code "Hello ${name}!"} → {@code [StringLiteral("Hello "), Identifier("name"),
   * StringLiteral("!")]}.
   */
  record StringInterp(List<Expr> parts) implements Expr {}

  /** Struct instantiation: {@code Point \{ x: 1, y: 2 \}}. */
  record StructInit(String name, List<FieldInit> fields) implements Expr {}

  /** A single field initialiser inside a struct literal. */
  record FieldInit(String name, Expr value) {}
}
