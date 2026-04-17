package com.aether.parser;

import com.aether.exception.ParseException;
import com.aether.lexer.Scanner;
import com.aether.lexer.StringPart;
import com.aether.lexer.Token;
import com.aether.lexer.TokenKind;
import com.aether.parser.ast.BinaryOp;
import com.aether.parser.ast.Expr;
import com.aether.parser.ast.Stmt;
import com.aether.parser.ast.UnaryOp;
import java.util.ArrayList;
import java.util.List;

/**
 * Recursive-descent parser for Aether.
 *
 * <p>Converts a flat token list (from {@link com.aether.lexer.Scanner}) into a list of top-level
 * {@link Stmt} nodes.
 *
 * <p>Operator precedence (lowest to highest):
 *
 * <ol>
 *   <li>Logical OR ({@code ||})
 *   <li>Logical AND ({@code &&})
 *   <li>Equality ({@code ==}, {@code !=})
 *   <li>Comparison ({@code <}, {@code >}, {@code <=}, {@code >=})
 *   <li>Addition/subtraction ({@code +}, {@code -})
 *   <li>Multiplication/division/modulo ({@code *}, {@code /}, {@code %})
 *   <li>Unary ({@code -}, {@code !})
 *   <li>Postfix: call, index, slice, member access
 *   <li>Primary: literals, identifiers, grouped expressions, array/dict literals
 * </ol>
 */
public final class Parser {

  private final List<Token> tokens;
  private int current = 0;

  public Parser(List<Token> tokens) {
    this.tokens = tokens;
  }

  /**
   * Parse all top-level statements and return them.
   *
   * @throws ParseException on syntax errors
   */
  public List<Stmt> parse() {
    List<Stmt> statements = new ArrayList<>();
    while (!isAtEnd()) {
      statements.add(declaration());
    }
    return statements;
  }

  // ── Helper methods ───────────────────────────────────────────────────────────

  private boolean isAtEnd() {
    return peek().kind() == TokenKind.EOF;
  }

  private Token peek() {
    return tokens.get(current);
  }

  private Token previous() {
    return tokens.get(current - 1);
  }

  private Token advance() {
    if (!isAtEnd()) {
      current++;
    }
    return previous();
  }

  private Token peekAt(int offset) {
    int idx = current + offset;
    return idx < tokens.size() ? tokens.get(idx) : tokens.get(tokens.size() - 1);
  }

  private boolean check(TokenKind kind) {
    return !isAtEnd() && peek().kind() == kind;
  }

  private boolean matchToken(TokenKind... kinds) {
    for (TokenKind kind : kinds) {
      if (check(kind)) {
        advance();
        return true;
      }
    }
    return false;
  }

  private Token consume(TokenKind kind, String expected) {
    if (check(kind)) {
      return advance();
    }
    throw new ParseException(expected, peek());
  }

  // ── Declarations ─────────────────────────────────────────────────────────────

  private Stmt declaration() {
    if (matchToken(TokenKind.LET)) {
      return letDeclaration();
    }
    if (matchToken(TokenKind.STRUCT)) {
      return structDeclaration();
    }
    if (matchToken(TokenKind.IMPORT)) {
      return importStatement();
    }
    if (matchToken(TokenKind.FROM)) {
      return fromImportStatement();
    }
    // fn name(...) is a declaration; fn(...) is an expression
    if (check(TokenKind.FN)
        && current + 1 < tokens.size()
        && tokens.get(current + 1).kind() == TokenKind.IDENTIFIER) {
      advance(); // consume 'fn'
      return functionDeclaration();
    }
    return statement();
  }

  private Stmt letDeclaration() {
    Token name = consume(TokenKind.IDENTIFIER, "variable name");
    consume(TokenKind.EQUAL, "=");
    Expr initializer = expression();
    return new Stmt.Let(name.lexeme(), initializer);
  }

  private Stmt functionDeclaration() {
    String name = consume(TokenKind.IDENTIFIER, "function name").lexeme();
    List<String> params = parseParamList();
    consume(TokenKind.LEFT_BRACE, "{");
    Stmt body = blockStatement();
    return new Stmt.Function(name, params, body);
  }

  private Stmt structDeclaration() {
    String name = consume(TokenKind.IDENTIFIER, "struct name").lexeme();
    consume(TokenKind.LEFT_BRACE, "{");

    List<String> fields = new ArrayList<>();
    List<Stmt.MethodDecl> methods = new ArrayList<>();

    while (!check(TokenKind.RIGHT_BRACE) && !isAtEnd()) {
      if (matchToken(TokenKind.FN)) {
        String methodName = consume(TokenKind.IDENTIFIER, "method name").lexeme();
        List<String> params = parseParamList();
        consume(TokenKind.LEFT_BRACE, "{");
        Stmt body = blockStatement();
        methods.add(new Stmt.MethodDecl(methodName, params, body));
      } else if (check(TokenKind.IDENTIFIER)) {
        fields.add(advance().lexeme());
        // Comma-separated fields on one line: x, y
        while (matchToken(TokenKind.COMMA)) {
          if (!check(TokenKind.IDENTIFIER)) {
            break;
          }
          fields.add(advance().lexeme());
        }
      } else {
        throw new ParseException("field name or fn", peek());
      }
    }

    consume(TokenKind.RIGHT_BRACE, "}");
    return new Stmt.StructDecl(name, fields, methods);
  }

  // ── Statements ───────────────────────────────────────────────────────────────

  private Stmt statement() {
    if (matchToken(TokenKind.IF)) {
      return ifStatement();
    }
    if (matchToken(TokenKind.WHILE)) {
      return whileStatement();
    }
    if (matchToken(TokenKind.FOR)) {
      return forStatement();
    }
    if (matchToken(TokenKind.RETURN)) {
      return returnStatement();
    }
    if (matchToken(TokenKind.BREAK)) {
      return new Stmt.Break();
    }
    if (matchToken(TokenKind.CONTINUE)) {
      return new Stmt.Continue();
    }
    if (matchToken(TokenKind.LEFT_BRACE)) {
      return blockStatement();
    }
    if (matchToken(TokenKind.TRY)) {
      return tryCatchStatement();
    }
    if (matchToken(TokenKind.THROW)) {
      return new Stmt.Throw(expression());
    }
    return expressionStatement();
  }

  private Stmt blockStatement() {
    List<Stmt> statements = new ArrayList<>();
    while (!check(TokenKind.RIGHT_BRACE) && !isAtEnd()) {
      statements.add(declaration());
    }
    consume(TokenKind.RIGHT_BRACE, "}");
    return new Stmt.Block(statements);
  }

  private Stmt ifStatement() {
    consume(TokenKind.LEFT_PAREN, "(");
    Expr condition = expression();
    consume(TokenKind.RIGHT_PAREN, ")");
    Stmt thenBranch = statement();
    Stmt elseBranch = matchToken(TokenKind.ELSE) ? statement() : null;
    return new Stmt.If(condition, thenBranch, elseBranch);
  }

  private Stmt whileStatement() {
    consume(TokenKind.LEFT_PAREN, "(");
    Expr condition = expression();
    consume(TokenKind.RIGHT_PAREN, ")");
    return new Stmt.While(condition, statement());
  }

  private Stmt forStatement() {
    String varName = consume(TokenKind.IDENTIFIER, "variable name").lexeme();
    consume(TokenKind.IN, "in");
    Expr iterable = expression();
    return new Stmt.For(varName, iterable, statement());
  }

  private Stmt returnStatement() {
    Expr value = (check(TokenKind.RIGHT_BRACE) || isAtEnd()) ? null : expression();
    return new Stmt.Return(value);
  }

  private Stmt tryCatchStatement() {
    consume(TokenKind.LEFT_BRACE, "{");
    Stmt tryBody = blockStatement();
    consume(TokenKind.CATCH, "catch");
    consume(TokenKind.LEFT_PAREN, "(");
    String errorVar = consume(TokenKind.IDENTIFIER, "error variable name").lexeme();
    consume(TokenKind.RIGHT_PAREN, ")");
    consume(TokenKind.LEFT_BRACE, "{");
    Stmt catchBody = blockStatement();
    return new Stmt.TryCatch(tryBody, errorVar, catchBody);
  }

  private Stmt expressionStatement() {
    Expr expr = expression();

    if (matchToken(TokenKind.EQUAL)) {
      validateAssignTarget(expr);
      return new Stmt.Assign(expr, expression());
    }
    if (matchToken(
        TokenKind.PLUS_EQUAL, TokenKind.MINUS_EQUAL,
        TokenKind.STAR_EQUAL, TokenKind.SLASH_EQUAL)) {
      validateAssignTarget(expr);
      BinaryOp op =
          switch (previous().kind()) {
            case PLUS_EQUAL -> BinaryOp.ADD;
            case MINUS_EQUAL -> BinaryOp.SUBTRACT;
            case STAR_EQUAL -> BinaryOp.MULTIPLY;
            case SLASH_EQUAL -> BinaryOp.DIVIDE;
            default -> throw new ParseException("compound assignment operator");
          };
      return new Stmt.CompoundAssign(expr, op, expression());
    }
    return new Stmt.ExprStmt(expr);
  }

  private void validateAssignTarget(Expr expr) {
    if (!(expr instanceof Expr.Identifier)
        && !(expr instanceof Expr.Index)
        && !(expr instanceof Expr.Member)) {
      throw new ParseException("Invalid assignment target");
    }
  }

  // ── Expressions ──────────────────────────────────────────────────────────────

  private Expr expression() {
    return logicalOr();
  }

  private Expr logicalOr() {
    Expr expr = logicalAnd();
    while (matchToken(TokenKind.OR)) {
      expr = new Expr.Binary(expr, BinaryOp.OR, logicalAnd());
    }
    return expr;
  }

  private Expr logicalAnd() {
    Expr expr = equality();
    while (matchToken(TokenKind.AND)) {
      expr = new Expr.Binary(expr, BinaryOp.AND, equality());
    }
    return expr;
  }

  private Expr equality() {
    Expr expr = comparison();
    while (matchToken(TokenKind.EQUAL_EQUAL, TokenKind.NOT_EQUAL)) {
      BinaryOp op = previous().kind() == TokenKind.EQUAL_EQUAL ? BinaryOp.EQUAL : BinaryOp.NOT_EQUAL;
      expr = new Expr.Binary(expr, op, comparison());
    }
    return expr;
  }

  private Expr comparison() {
    Expr expr = addition();
    while (matchToken(
        TokenKind.LESS, TokenKind.GREATER, TokenKind.LESS_EQUAL, TokenKind.GREATER_EQUAL)) {
      BinaryOp op =
          switch (previous().kind()) {
            case LESS -> BinaryOp.LESS;
            case GREATER -> BinaryOp.GREATER;
            case LESS_EQUAL -> BinaryOp.LESS_EQUAL;
            case GREATER_EQUAL -> BinaryOp.GREATER_EQUAL;
            default -> throw new ParseException("comparison operator");
          };
      expr = new Expr.Binary(expr, op, addition());
    }
    return expr;
  }

  private Expr addition() {
    Expr expr = multiplication();
    while (matchToken(TokenKind.PLUS, TokenKind.MINUS)) {
      BinaryOp op = previous().kind() == TokenKind.PLUS ? BinaryOp.ADD : BinaryOp.SUBTRACT;
      expr = new Expr.Binary(expr, op, multiplication());
    }
    return expr;
  }

  private Expr multiplication() {
    Expr expr = unary();
    while (matchToken(TokenKind.STAR, TokenKind.SLASH, TokenKind.PERCENT)) {
      BinaryOp op =
          switch (previous().kind()) {
            case STAR -> BinaryOp.MULTIPLY;
            case SLASH -> BinaryOp.DIVIDE;
            case PERCENT -> BinaryOp.MODULO;
            default -> throw new ParseException("operator");
          };
      expr = new Expr.Binary(expr, op, unary());
    }
    return expr;
  }

  private Expr unary() {
    if (matchToken(TokenKind.MINUS)) {
      return new Expr.Unary(UnaryOp.NEGATE, unary());
    }
    if (matchToken(TokenKind.NOT)) {
      return new Expr.Unary(UnaryOp.NOT, unary());
    }
    return call();
  }

  private Expr call() {
    Expr expr = primary();

    while (true) {
      if (matchToken(TokenKind.LEFT_PAREN)) {
        expr = finishCall(expr);
      } else if (peek().kind() == TokenKind.LEFT_BRACKET
          && peek().line() == previous().line()) {
        // Same-line indexing to avoid ambiguity with standalone array literals
        advance(); // consume '['
        expr = parseIndexOrSlice(expr);
      } else if (matchToken(TokenKind.DOT)) {
        String member = consume(TokenKind.IDENTIFIER, "property name").lexeme();
        expr = new Expr.Member(expr, member);
      } else {
        break;
      }
    }
    return expr;
  }

  private Expr finishCall(Expr callee) {
    List<Expr> args = new ArrayList<>();
    if (!check(TokenKind.RIGHT_PAREN)) {
      do {
        args.add(expression());
      } while (matchToken(TokenKind.COMMA));
    }
    consume(TokenKind.RIGHT_PAREN, ")");
    return new Expr.Call(callee, args);
  }

  private Expr parseIndexOrSlice(Expr object) {
    if (matchToken(TokenKind.COLON)) {
      // [:end] or [:]
      Expr end = check(TokenKind.RIGHT_BRACKET) ? null : expression();
      consume(TokenKind.RIGHT_BRACKET, "]");
      return new Expr.Slice(object, null, end);
    }
    Expr first = expression();
    if (matchToken(TokenKind.COLON)) {
      // [start:end] or [start:]
      Expr end = check(TokenKind.RIGHT_BRACKET) ? null : expression();
      consume(TokenKind.RIGHT_BRACKET, "]");
      return new Expr.Slice(object, first, end);
    }
    consume(TokenKind.RIGHT_BRACKET, "]");
    return new Expr.Index(object, first);
  }

  private Expr primary() {
    Token token = advance();
    return switch (token.kind()) {
      case INTEGER -> new Expr.IntLiteral(token.intValue());
      case FLOAT -> new Expr.FloatLiteral(token.floatValue());
      case STRING -> new Expr.StringLiteral(token.stringValue());
      case STRING_INTERP -> parseStringInterp(token);
      case TRUE -> new Expr.BoolLiteral(true);
      case FALSE -> new Expr.BoolLiteral(false);
      case NULL -> new Expr.NullLiteral();
      case IDENTIFIER -> {
        String name = token.lexeme();
        // Same-line '{' with struct-init pattern → struct literal
        if (peek().kind() == TokenKind.LEFT_BRACE
            && peek().line() == previous().line()
            && isStructInit()) {
          advance(); // consume '{'
          yield structInit(name);
        }
        yield new Expr.Identifier(name);
      }
      case FN -> functionExpression();
      case LEFT_PAREN -> {
        Expr inner = expression();
        consume(TokenKind.RIGHT_PAREN, ")");
        yield inner;
      }
      case LEFT_BRACKET -> {
        List<Expr> elements = new ArrayList<>();
        if (!check(TokenKind.RIGHT_BRACKET)) {
          do {
            if (matchToken(TokenKind.SPREAD)) {
              elements.add(new Expr.Spread(expression()));
            } else {
              elements.add(expression());
            }
          } while (matchToken(TokenKind.COMMA));
        }
        consume(TokenKind.RIGHT_BRACKET, "]");
        yield new Expr.Array(elements);
      }
      case LEFT_BRACE -> {
        List<Expr.DictEntry> pairs = new ArrayList<>();
        if (!check(TokenKind.RIGHT_BRACE)) {
          do {
            Expr key = expression();
            consume(TokenKind.COLON, ":");
            pairs.add(new Expr.DictEntry(key, expression()));
          } while (matchToken(TokenKind.COMMA));
        }
        consume(TokenKind.RIGHT_BRACE, "}");
        yield new Expr.Dict(pairs);
      }
      default -> throw new ParseException("expression", token);
    };
  }

  private Expr parseStringInterp(Token token) {
    List<Expr> parts = new ArrayList<>();
    for (StringPart part : token.interpParts()) {
      switch (part) {
        case StringPart.Literal(String text) -> parts.add(new Expr.StringLiteral(text));
        case StringPart.Placeholder(String src) -> {
          List<Token> inner = new Scanner(src).scanTokens();
          // Parse the placeholder as a single expression
          parts.add(new Parser(inner).expression());
        }
      }
    }
    return new Expr.StringInterp(parts);
  }

  private Expr functionExpression() {
    List<String> params = parseParamList();
    consume(TokenKind.LEFT_BRACE, "{");
    return new Expr.FunctionExpr(params, blockStatement());
  }

  private Expr structInit(String name) {
    List<Expr.FieldInit> fields = new ArrayList<>();
    if (!check(TokenKind.RIGHT_BRACE)) {
      do {
        String fieldName = consume(TokenKind.IDENTIFIER, "field name").lexeme();
        consume(TokenKind.COLON, ":");
        fields.add(new Expr.FieldInit(fieldName, expression()));
      } while (matchToken(TokenKind.COMMA));
    }
    consume(TokenKind.RIGHT_BRACE, "}");
    return new Expr.StructInit(name, fields);
  }

  /**
   * Returns true if the next '{' starts a struct initialiser (not a block or dict).
   *
   * <p>peek() must be LEFT_BRACE before calling. Struct init requires either '{}' (empty) or
   * 'identifier :' inside.
   */
  private boolean isStructInit() {
    // peekAt(0) is '{', peekAt(1) is what's inside
    Token inside = peekAt(1);
    return switch (inside.kind()) {
      case RIGHT_BRACE -> true; // empty braces: Name {}
      case IDENTIFIER -> peekAt(2).kind() == TokenKind.COLON;
      default -> false;
    };
  }

  // ── Import statements ────────────────────────────────────────────────────────

  private Stmt importStatement() {
    String moduleName = consume(TokenKind.IDENTIFIER, "module name").lexeme();
    if (matchToken(TokenKind.AS)) {
      String alias = consume(TokenKind.IDENTIFIER, "alias name").lexeme();
      return new Stmt.ImportAs(moduleName, alias);
    }
    return new Stmt.Import(moduleName);
  }

  private Stmt fromImportStatement() {
    String moduleName = consume(TokenKind.IDENTIFIER, "module name").lexeme();
    consume(TokenKind.IMPORT, "import");

    List<Stmt.ImportAlias> aliased = new ArrayList<>();
    boolean hasAlias = false;

    do {
      String item = consume(TokenKind.IDENTIFIER, "item name").lexeme();
      if (matchToken(TokenKind.AS)) {
        hasAlias = true;
        String alias = consume(TokenKind.IDENTIFIER, "alias name").lexeme();
        aliased.add(new Stmt.ImportAlias(item, alias));
      } else {
        aliased.add(new Stmt.ImportAlias(item, item));
      }
    } while (matchToken(TokenKind.COMMA));

    if (hasAlias) {
      return new Stmt.FromImportAs(moduleName, aliased);
    }
    List<String> items = aliased.stream().map(Stmt.ImportAlias::item).toList();
    return new Stmt.FromImport(moduleName, items);
  }

  // ── Shared helpers ────────────────────────────────────────────────────────────

  private List<String> parseParamList() {
    consume(TokenKind.LEFT_PAREN, "(");
    List<String> params = new ArrayList<>();
    if (!check(TokenKind.RIGHT_PAREN)) {
      do {
        params.add(consume(TokenKind.IDENTIFIER, "parameter name").lexeme());
      } while (matchToken(TokenKind.COMMA));
    }
    consume(TokenKind.RIGHT_PAREN, ")");
    return params;
  }
}
