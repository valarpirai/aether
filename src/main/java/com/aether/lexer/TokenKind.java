package com.aether.lexer;

/** All token types produced by the Aether scanner. */
public enum TokenKind {
  // Literals
  INTEGER,
  FLOAT,
  STRING,
  STRING_INTERP,
  TRUE,
  FALSE,
  NULL,

  // Identifier
  IDENTIFIER,

  // Keywords
  LET,
  FN,
  RETURN,
  IF,
  ELSE,
  WHILE,
  FOR,
  IN,
  BREAK,
  CONTINUE,
  IMPORT,
  FROM,
  AS,
  TRY,
  CATCH,
  THROW,
  STRUCT,

  // Arithmetic operators
  PLUS,
  MINUS,
  STAR,
  SLASH,
  PERCENT,

  // Compound assignment
  PLUS_EQUAL,
  MINUS_EQUAL,
  STAR_EQUAL,
  SLASH_EQUAL,

  // Comparison operators
  EQUAL_EQUAL,
  NOT_EQUAL,
  LESS,
  GREATER,
  LESS_EQUAL,
  GREATER_EQUAL,

  // Assignment
  EQUAL,

  // Logical operators
  AND,
  OR,
  NOT,

  // Delimiters
  LEFT_PAREN,
  RIGHT_PAREN,
  LEFT_BRACE,
  RIGHT_BRACE,
  LEFT_BRACKET,
  RIGHT_BRACKET,
  COMMA,
  DOT,
  SPREAD,
  COLON,

  // Special
  EOF,
}
