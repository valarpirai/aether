package com.aether.parser.ast;

/** Unary operators supported in Aether expressions. */
public enum UnaryOp {
  /** Arithmetic negation: {@code -expr}. */
  NEGATE,

  /** Logical not: {@code !expr}. */
  NOT,
}
