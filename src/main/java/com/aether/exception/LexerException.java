package com.aether.exception;

import lombok.Getter;

/** Thrown when the scanner encounters invalid source text. */
@Getter
public final class LexerException extends RuntimeException {

  private final int line;
  private final int column;

  public LexerException(String message, int line, int column) {
    super(message + " at line " + line + ", column " + column);
    this.line = line;
    this.column = column;
  }
}
