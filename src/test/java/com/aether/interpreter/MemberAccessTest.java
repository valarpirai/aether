package com.aether.interpreter;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import com.aether.exception.AetherRuntimeException;
import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Stmt;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

/** Tests for member access: array.length, string.length, and property error cases. */
class MemberAccessTest {

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
  void arrayLengthProperty() {
    assertEquals("3", eval("[1, 2, 3].length"));
  }

  @Test
  void stringLengthProperty() {
    assertEquals("5", eval("\"hello\".length"));
  }

  @Test
  void emptyArrayLength() {
    assertEquals("0", eval("[].length"));
  }

  @Test
  void emptyStringLength() {
    assertEquals("0", eval("\"\".length"));
  }

  @Test
  void directLiteralMemberAccess() {
    assertEquals("2", eval("[10, 20].length"));
  }

  @Test
  void memberAccessInExpression() {
    assertEquals("8", eval("let s = \"hello\"\nlet a = [1, 2, 3]\ns.length + a.length"));
  }

  @Test
  void undefinedPropertyError() {
    assertThrows(AetherRuntimeException.class, () -> eval("[1, 2].nonexistent"));
  }

  @Test
  void memberAccessOnNonObject() {
    assertThrows(AetherRuntimeException.class, () -> eval("42.length"));
  }
}
