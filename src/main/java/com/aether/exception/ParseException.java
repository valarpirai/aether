package com.aether.exception;

import com.aether.lexer.Token;
import lombok.Getter;

/** Thrown by the parser when it encounters syntactically invalid input. */
@Getter
public final class ParseException extends RuntimeException {

  private final Token found;

  public ParseException(String expected, Token found) {
    super(
        "Expected "
            + expected
            + ", found '"
            + found.lexeme()
            + "' at line "
            + found.line()
            + ":"
            + found.column());
    this.found = found;
  }

  public ParseException(String message) {
    super(message);
    this.found = null;
  }
}
