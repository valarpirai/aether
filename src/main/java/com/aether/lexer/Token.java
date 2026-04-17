package com.aether.lexer;

import java.util.List;

/**
 * A single token produced by the scanner, carrying its kind, raw text, and source position.
 *
 * <p>For {@link TokenKind#INTEGER} tokens, {@link #intValue()} holds the parsed value. For {@link
 * TokenKind#FLOAT} tokens, {@link #floatValue()} holds the parsed value. For {@link
 * TokenKind#STRING_INTERP} tokens, {@link #interpParts()} holds the interpolation segments.
 */
public record Token(
    TokenKind kind,
    String lexeme,
    int line,
    int column,
    long intValue,
    double floatValue,
    String stringValue,
    List<StringPart> interpParts) {

  // ── Factory helpers ──────────────────────────────────────────────────────────

  /** Create a simple (non-literal) token. */
  public static Token of(TokenKind kind, String lexeme, int line, int column) {
    return new Token(kind, lexeme, line, column, 0, 0.0, null, null);
  }

  /** Create an integer literal token. */
  public static Token ofInt(long value, String lexeme, int line, int column) {
    return new Token(TokenKind.INTEGER, lexeme, line, column, value, 0.0, null, null);
  }

  /** Create a float literal token. */
  public static Token ofFloat(double value, String lexeme, int line, int column) {
    return new Token(TokenKind.FLOAT, lexeme, line, column, 0, value, null, null);
  }

  /** Create a plain string literal token. */
  public static Token ofString(String value, String lexeme, int line, int column) {
    return new Token(TokenKind.STRING, lexeme, line, column, 0, 0.0, value, null);
  }

  /** Create a string-interpolation token. */
  public static Token ofStringInterp(List<StringPart> parts, String lexeme, int line, int column) {
    return new Token(TokenKind.STRING_INTERP, lexeme, line, column, 0, 0.0, null, parts);
  }

  @Override
  public String toString() {
    return "Token[" + kind + " '" + lexeme + "' @" + line + ":" + column + "]";
  }
}
