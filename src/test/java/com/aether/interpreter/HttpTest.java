package com.aether.interpreter;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import com.aether.exception.AetherRuntimeException;
import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Stmt;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.Test;

/** Integration tests for http_get() and http_post() builtins. Disabled: require network access. */
@Disabled("Requires network access to httpbin.org")
class HttpTest {

  private Evaluator evaluator;

  @BeforeEach
  void setUp() {
    evaluator = Evaluator.withoutStdlib();
  }

  private String eval(String source) {
    List<Stmt> stmts = new Parser(new Scanner(source).scanTokens()).parse();
    for (int i = 0; i < stmts.size() - 1; i++) {
      evaluator.execStmt(stmts.get(i));
    }
    if (!stmts.isEmpty() && stmts.getLast() instanceof Stmt.ExprStmt es) {
      return Builtins.display(evaluator.evalExpr(es.expr()));
    }
    if (!stmts.isEmpty()) {
      evaluator.execStmt(stmts.getLast());
    }
    return "null";
  }

  @Test
  void httpGetReturnsNonEmptyString() {
    String result = eval("http_get(\"https://httpbin.org/get\")");
    assertFalse(result.isEmpty());
  }

  @Test
  void httpPostReturnsNonEmptyString() {
    String result = eval("http_post(\"https://httpbin.org/post\", \"hello\")");
    assertFalse(result.isEmpty());
  }

  @Test
  void httpGetInvalidUrlThrowsError() {
    assertThrows(AetherRuntimeException.InvalidOperation.class,
        () -> eval("http_get(\"not_a_valid_url\")"));
  }
}
