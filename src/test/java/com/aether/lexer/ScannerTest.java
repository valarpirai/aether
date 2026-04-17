package com.aether.lexer;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import com.aether.exception.LexerException;
import java.util.List;
import org.junit.jupiter.api.Test;

/** Unit tests for {@link Scanner}. */
class ScannerTest {

  private List<Token> scan(String source) {
    return new Scanner(source).scanTokens();
  }

  @Test
  void emptySourceProducesOnlyEof() {
    List<Token> tokens = scan("");
    assertEquals(1, tokens.size());
    assertEquals(TokenKind.EOF, tokens.get(0).kind());
  }

  @Test
  void integerLiteral() {
    List<Token> tokens = scan("42");
    assertEquals(TokenKind.INTEGER, tokens.get(0).kind());
    assertEquals(42L, tokens.get(0).intValue());
  }

  @Test
  void floatLiteral() {
    List<Token> tokens = scan("3.14");
    assertEquals(TokenKind.FLOAT, tokens.get(0).kind());
    assertEquals(3.14, tokens.get(0).floatValue(), 0.001);
  }

  @Test
  void stringLiteral() {
    List<Token> tokens = scan("\"hello\"");
    assertEquals(TokenKind.STRING, tokens.get(0).kind());
    assertEquals("hello", tokens.get(0).stringValue());
  }

  @Test
  void stringWithEscapes() {
    List<Token> tokens = scan("\"a\\nb\"");
    assertEquals("a\nb", tokens.get(0).stringValue());
  }

  @Test
  void stringInterpolation() {
    List<Token> tokens = scan("\"hello ${name}!\"");
    assertEquals(TokenKind.STRING_INTERP, tokens.get(0).kind());
    List<StringPart> parts = tokens.get(0).interpParts();
    assertEquals(3, parts.size());
    assertEquals(new StringPart.Literal("hello "), parts.get(0));
    assertEquals(new StringPart.Placeholder("name"), parts.get(1));
    assertEquals(new StringPart.Literal("!"), parts.get(2));
  }

  @Test
  void keywords() {
    List<Token> tokens = scan("let fn return if else while for in");
    assertEquals(TokenKind.LET, tokens.get(0).kind());
    assertEquals(TokenKind.FN, tokens.get(1).kind());
    assertEquals(TokenKind.RETURN, tokens.get(2).kind());
    assertEquals(TokenKind.IF, tokens.get(3).kind());
    assertEquals(TokenKind.ELSE, tokens.get(4).kind());
    assertEquals(TokenKind.WHILE, tokens.get(5).kind());
    assertEquals(TokenKind.FOR, tokens.get(6).kind());
    assertEquals(TokenKind.IN, tokens.get(7).kind());
  }

  @Test
  void boolAndNullLiterals() {
    List<Token> tokens = scan("true false null");
    assertEquals(TokenKind.TRUE, tokens.get(0).kind());
    assertEquals(TokenKind.FALSE, tokens.get(1).kind());
    assertEquals(TokenKind.NULL, tokens.get(2).kind());
  }

  @Test
  void arithmeticOperators() {
    List<Token> tokens = scan("+ - * / %");
    assertEquals(TokenKind.PLUS, tokens.get(0).kind());
    assertEquals(TokenKind.MINUS, tokens.get(1).kind());
    assertEquals(TokenKind.STAR, tokens.get(2).kind());
    assertEquals(TokenKind.SLASH, tokens.get(3).kind());
    assertEquals(TokenKind.PERCENT, tokens.get(4).kind());
  }

  @Test
  void comparisonOperators() {
    List<Token> tokens = scan("== != < > <= >=");
    assertEquals(TokenKind.EQUAL_EQUAL, tokens.get(0).kind());
    assertEquals(TokenKind.NOT_EQUAL, tokens.get(1).kind());
    assertEquals(TokenKind.LESS, tokens.get(2).kind());
    assertEquals(TokenKind.GREATER, tokens.get(3).kind());
    assertEquals(TokenKind.LESS_EQUAL, tokens.get(4).kind());
    assertEquals(TokenKind.GREATER_EQUAL, tokens.get(5).kind());
  }

  @Test
  void spreadOperator() {
    List<Token> tokens = scan("...");
    assertEquals(TokenKind.SPREAD, tokens.get(0).kind());
    assertEquals(TokenKind.EOF, tokens.get(1).kind());
  }

  @Test
  void compoundAssignOperators() {
    List<Token> tokens = scan("+= -= *= /=");
    assertEquals(TokenKind.PLUS_EQUAL, tokens.get(0).kind());
    assertEquals(TokenKind.MINUS_EQUAL, tokens.get(1).kind());
    assertEquals(TokenKind.STAR_EQUAL, tokens.get(2).kind());
    assertEquals(TokenKind.SLASH_EQUAL, tokens.get(3).kind());
  }

  @Test
  void lineCommentSkipped() {
    List<Token> tokens = scan("42 // this is ignored\n99");
    assertEquals(2, tokens.size() - 1); // 42, 99, EOF
    assertEquals(42L, tokens.get(0).intValue());
    assertEquals(99L, tokens.get(1).intValue());
  }

  @Test
  void blockCommentSkipped() {
    List<Token> tokens = scan("1 /* ignored */ 2");
    assertEquals(2, tokens.size() - 1);
    assertEquals(1L, tokens.get(0).intValue());
    assertEquals(2L, tokens.get(1).intValue());
  }

  @Test
  void unexpectedCharacterThrows() {
    assertThrows(LexerException.class, () -> scan("@"));
  }

  @Test
  void unterminatedStringThrows() {
    assertThrows(LexerException.class, () -> scan("\"unterminated"));
  }
}
