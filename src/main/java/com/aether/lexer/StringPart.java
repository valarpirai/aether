package com.aether.lexer;

/** A segment of an interpolated string literal. */
public sealed interface StringPart permits StringPart.Literal, StringPart.Placeholder {

  /** A plain text segment. */
  record Literal(String text) implements StringPart {}

  /** An expression placeholder: the raw source between <code>${</code> and <code>}</code>. */
  record Placeholder(String source) implements StringPart {}
}
