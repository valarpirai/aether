package com.aether.lexer;

import com.aether.exception.LexerException;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

/**
 * Lexical analyser for Aether source code.
 *
 * <p>Converts raw source text into a flat list of {@link Token}s terminated by {@link
 * TokenKind#EOF}. Supports:
 *
 * <ul>
 *   <li>Integer and float literals
 *   <li>String literals with escape sequences and {@code ${…}} interpolation
 *   <li>Single-line ({@code //}) and multi-line ({@code /* … * /}) comments
 *   <li>All Aether operators and delimiters including {@code ...} (spread)
 * </ul>
 */
public final class Scanner {

  private static final Map<String, TokenKind> KEYWORDS =
      Map.ofEntries(
          Map.entry("let", TokenKind.LET),
          Map.entry("fn", TokenKind.FN),
          Map.entry("return", TokenKind.RETURN),
          Map.entry("if", TokenKind.IF),
          Map.entry("else", TokenKind.ELSE),
          Map.entry("while", TokenKind.WHILE),
          Map.entry("for", TokenKind.FOR),
          Map.entry("in", TokenKind.IN),
          Map.entry("break", TokenKind.BREAK),
          Map.entry("continue", TokenKind.CONTINUE),
          Map.entry("true", TokenKind.TRUE),
          Map.entry("false", TokenKind.FALSE),
          Map.entry("null", TokenKind.NULL),
          Map.entry("import", TokenKind.IMPORT),
          Map.entry("from", TokenKind.FROM),
          Map.entry("as", TokenKind.AS),
          Map.entry("try", TokenKind.TRY),
          Map.entry("catch", TokenKind.CATCH),
          Map.entry("throw", TokenKind.THROW),
          Map.entry("struct", TokenKind.STRUCT));

  private final String source;
  private final List<Token> tokens = new ArrayList<>();
  private int start = 0;
  private int current = 0;
  private int line = 1;
  private int column = 1;

  public Scanner(String source) {
    this.source = source;
  }

  /**
   * Scans all tokens from the source and returns the resulting list.
   *
   * @return immutable list of tokens ending with EOF
   * @throws LexerException on unexpected characters or unterminated strings
   */
  public List<Token> scanTokens() {
    while (!isAtEnd()) {
      start = current;
      scanToken();
    }
    tokens.add(Token.of(TokenKind.EOF, "", line, column));
    return List.copyOf(tokens);
  }

  // ── Private scan methods ─────────────────────────────────────────────────────

  private void scanToken() {
    char c = advance();
    int startCol = column - 1;

    switch (c) {
      case ' ', '\r', '\t' -> {} // skip whitespace
      case '\n' -> {} // line/column already updated in advance()
      case '(' -> addTokenAt(TokenKind.LEFT_PAREN, startCol);
      case ')' -> addTokenAt(TokenKind.RIGHT_PAREN, startCol);
      case '{' -> addTokenAt(TokenKind.LEFT_BRACE, startCol);
      case '}' -> addTokenAt(TokenKind.RIGHT_BRACE, startCol);
      case '[' -> addTokenAt(TokenKind.LEFT_BRACKET, startCol);
      case ']' -> addTokenAt(TokenKind.RIGHT_BRACKET, startCol);
      case ',' -> addTokenAt(TokenKind.COMMA, startCol);
      case ':' -> addTokenAt(TokenKind.COLON, startCol);
      case '%' -> addTokenAt(TokenKind.PERCENT, startCol);
      case '.' -> {
        if (peek() == '.' && peekNext() == '.') {
          advance();
          advance();
          addTokenAt(TokenKind.SPREAD, startCol);
        } else {
          addTokenAt(TokenKind.DOT, startCol);
        }
      }
      case '+' -> addTokenAt(matchChar('=') ? TokenKind.PLUS_EQUAL : TokenKind.PLUS, startCol);
      case '-' -> addTokenAt(matchChar('=') ? TokenKind.MINUS_EQUAL : TokenKind.MINUS, startCol);
      case '*' -> addTokenAt(matchChar('=') ? TokenKind.STAR_EQUAL : TokenKind.STAR, startCol);
      case '/' -> {
        if (matchChar('/')) {
          skipLineComment();
        } else if (matchChar('*')) {
          skipBlockComment();
        } else if (matchChar('=')) {
          addTokenAt(TokenKind.SLASH_EQUAL, startCol);
        } else {
          addTokenAt(TokenKind.SLASH, startCol);
        }
      }
      case '!' -> addTokenAt(matchChar('=') ? TokenKind.NOT_EQUAL : TokenKind.NOT, startCol);
      case '=' -> addTokenAt(matchChar('=') ? TokenKind.EQUAL_EQUAL : TokenKind.EQUAL, startCol);
      case '<' -> addTokenAt(matchChar('=') ? TokenKind.LESS_EQUAL : TokenKind.LESS, startCol);
      case '>' ->
          addTokenAt(matchChar('=') ? TokenKind.GREATER_EQUAL : TokenKind.GREATER, startCol);
      case '&' -> {
        if (matchChar('&')) {
          addTokenAt(TokenKind.AND, startCol);
        } else {
          throw new LexerException("Unexpected character '&'", line, startCol);
        }
      }
      case '|' -> {
        if (matchChar('|')) {
          addTokenAt(TokenKind.OR, startCol);
        } else {
          throw new LexerException("Unexpected character '|'", line, startCol);
        }
      }
      case '"' -> scanString(startCol);
      default -> {
        if (Character.isDigit(c)) {
          scanNumber(startCol);
        } else if (Character.isLetter(c) || c == '_') {
          scanIdentifier(startCol);
        } else {
          throw new LexerException("Unexpected character '" + c + "'", line, startCol);
        }
      }
    }
  }

  private void skipLineComment() {
    while (peek() != '\n' && !isAtEnd()) {
      advance();
    }
  }

  private void skipBlockComment() {
    while (!isAtEnd()) {
      if (peek() == '*' && peekNext() == '/') {
        advance(); // consume '*'
        advance(); // consume '/'
        return;
      }
      advance();
    }
  }

  private void scanString(int startCol) {
    StringBuilder currentLiteral = new StringBuilder();
    List<StringPart> parts = new ArrayList<>();
    boolean hasInterpolation = false;

    while (peek() != '"' && !isAtEnd()) {
      if (peek() == '$' && peekNext() == '{') {
        hasInterpolation = true;
        advance(); // consume '$'
        advance(); // consume '{'

        parts.add(new StringPart.Literal(currentLiteral.toString()));
        currentLiteral.setLength(0);

        // Collect expression source until matching '}'
        StringBuilder exprSrc = new StringBuilder();
        int depth = 1;
        while (!isAtEnd() && depth > 0) {
          char ch = advance();
          if (ch == '{') {
            depth++;
            exprSrc.append(ch);
          } else if (ch == '}') {
            depth--;
            if (depth > 0) {
              exprSrc.append(ch);
            }
          } else {
            exprSrc.append(ch);
          }
        }
        parts.add(new StringPart.Placeholder(exprSrc.toString()));
      } else if (peek() == '\\') {
        advance(); // consume backslash
        if (isAtEnd()) {
          throw new LexerException("Unterminated string", line, startCol);
        }
        char escaped =
            switch (peek()) {
              case 'n' -> '\n';
              case 't' -> '\t';
              case '\\' -> '\\';
              case '"' -> '"';
              default -> peek();
            };
        currentLiteral.append(escaped);
        advance();
      } else {
        currentLiteral.append(advance());
      }
    }

    if (isAtEnd()) {
      throw new LexerException("Unterminated string", line, startCol);
    }
    advance(); // closing "

    if (hasInterpolation) {
      parts.add(new StringPart.Literal(currentLiteral.toString()));
      tokens.add(Token.ofStringInterp(parts, source.substring(start, current), line, startCol));
    } else {
      tokens.add(
          Token.ofString(currentLiteral.toString(), source.substring(start, current), line, startCol));
    }
  }

  private void scanNumber(int startCol) {
    while (Character.isDigit(peek())) {
      advance();
    }

    boolean isFloat = peek() == '.' && Character.isDigit(peekNext());
    if (isFloat) {
      advance(); // consume '.'
      while (Character.isDigit(peek())) {
        advance();
      }
    }

    String text = source.substring(start, current);
    if (isFloat) {
      tokens.add(Token.ofFloat(Double.parseDouble(text), text, line, startCol));
    } else {
      tokens.add(Token.ofInt(Long.parseLong(text), text, line, startCol));
    }
  }

  private void scanIdentifier(int startCol) {
    while (Character.isLetterOrDigit(peek()) || peek() == '_') {
      advance();
    }

    String text = source.substring(start, current);
    TokenKind kind = KEYWORDS.getOrDefault(text, TokenKind.IDENTIFIER);
    tokens.add(Token.of(kind, text, line, startCol));
  }

  // ── Helpers ──────────────────────────────────────────────────────────────────

  private boolean isAtEnd() {
    return current >= source.length();
  }

  private char advance() {
    char c = source.charAt(current++);
    if (c == '\n') {
      line++;
      column = 1;
    } else {
      column++;
    }
    return c;
  }

  private char peek() {
    return isAtEnd() ? '\0' : source.charAt(current);
  }

  private char peekNext() {
    return current + 1 >= source.length() ? '\0' : source.charAt(current + 1);
  }

  private boolean matchChar(char expected) {
    if (isAtEnd() || source.charAt(current) != expected) {
      return false;
    }
    advance();
    return true;
  }

  private void addToken(TokenKind kind) {
    String lexeme = source.substring(start, current);
    tokens.add(Token.of(kind, lexeme, line, column - lexeme.length()));
  }

  private void addTokenAt(TokenKind kind, int col) {
    String lexeme = source.substring(start, current);
    tokens.add(Token.of(kind, lexeme, line, col));
  }
}
