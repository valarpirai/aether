package com.aether.interpreter;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.aether.exception.AetherRuntimeException;
import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Stmt;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

/** Comprehensive tests for try/catch/throw error handling. */
class ErrorHandlingTest {

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
  void throwString() {
    assertThrows(AetherRuntimeException.Thrown.class, () -> eval("throw \"oops\""));
  }

  @Test
  void throwInt() {
    assertThrows(AetherRuntimeException.Thrown.class, () -> eval("throw 42"));
  }

  @Test
  void catchGetsStringValue() {
    assertEquals("oops",
        eval("let r = \"\"\ntry { throw \"oops\" } catch(e) { r = e }\nr"));
  }

  @Test
  void catchGetsIntValue() {
    assertEquals("42",
        eval("let r = 0\ntry { throw 42 } catch(e) { r = e }\nr"));
  }

  @Test
  void catchRuntimeError() {
    assertEquals("caught",
        eval("let r = \"ok\"\ntry { 1 / 0 } catch(e) { r = \"caught\" }\nr"));
  }

  @Test
  void catchAllowsContinuation() {
    assertEquals("after",
        eval("try { throw \"x\" } catch(e) { }\n\"after\""));
  }

  @Test
  void rethrow() {
    assertThrows(AetherRuntimeException.Thrown.class,
        () -> eval("try { throw \"x\" } catch(e) { throw e }"));
  }

  @Test
  void propagatesAcrossCallStack() {
    assertEquals("caught",
        eval("fn inner() { throw \"boom\" }\n"
            + "fn outer() { inner() }\n"
            + "let r = \"\"\n"
            + "try { outer() } catch(e) { r = \"caught\" }\n"
            + "r"));
  }

  @Test
  void nestedTryCatch() {
    assertEquals("inner caught",
        eval("let r = \"\"\n"
            + "try {\n"
            + "  try { throw \"inner\" } catch(e) { r = \"inner caught\" }\n"
            + "} catch(e) { r = \"outer caught\" }\n"
            + "r"));
  }

  @Test
  void outerCatchesInnerEscape() {
    assertEquals("outer caught",
        eval("let r = \"\"\n"
            + "try {\n"
            + "  try { throw \"escape\" } catch(e) { throw \"rethrown\" }\n"
            + "} catch(e) { r = \"outer caught\" }\n"
            + "r"));
  }
}
