package com.aether.parser.ast;

/** Binary operators supported in Aether expressions. */
public enum BinaryOp {
  // Arithmetic
  ADD,
  SUBTRACT,
  MULTIPLY,
  DIVIDE,
  MODULO,

  // Comparison
  EQUAL,
  NOT_EQUAL,
  LESS,
  GREATER,
  LESS_EQUAL,
  GREATER_EQUAL,

  // Logical
  AND,
  OR,
}
