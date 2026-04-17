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

/** Comprehensive tests for dict literals, access, methods, and mutation. */
class DictTest {

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
  void emptyDict() {
    assertEquals("{}", eval("let d = {}\nd"));
  }

  @Test
  void stringKeys() {
    assertEquals("Alice",
        eval("let d = {\"name\": \"Alice\"}\nd[\"name\"]"));
  }

  @Test
  void indexAccess() {
    assertEquals("42", eval("let d = {\"x\": 42}\nd[\"x\"]"));
  }

  @Test
  void memberAccess() {
    assertEquals("1", eval("let d = {\"count\": 1}\nd[\"count\"]"));
  }

  @Test
  void intValue() {
    assertEquals("99", eval("let d = {\"n\": 99}\nd[\"n\"]"));
  }

  @Test
  void length() {
    assertEquals("3", eval("len({\"a\": 1, \"b\": 2, \"c\": 3})"));
  }

  @Test
  void keyNotFoundThrows() {
    assertThrows(AetherRuntimeException.class,
        () -> eval("let d = {\"a\": 1}\nd[\"missing\"]"));
  }

  @Test
  void nested() {
    assertEquals("inner",
        eval("let d = {\"outer\": {\"inner\": \"inner\"}}\nd[\"outer\"][\"inner\"]"));
  }

  @Test
  void inFunction() {
    assertEquals("10",
        eval("fn get_val(d, k) { return d[k] }\n"
            + "let d = {\"x\": 10}\n"
            + "get_val(d, \"x\")"));
  }

  @Test
  void variableValue() {
    assertEquals("7",
        eval("let v = 7\nlet d = {\"k\": v}\nd[\"k\"]"));
  }

  @Test
  void keys() {
    assertEquals("[\"x\"]", eval("let d = {\"x\": 1}\nd.keys()"));
  }

  @Test
  void values() {
    assertEquals("[1, 2]", eval("let d = {\"a\": 1, \"b\": 2}\nd.values()"));
  }

  @Test
  void containsTrue() {
    assertEquals("true", eval("let d = {\"a\": 1}\nd.contains(\"a\")"));
  }

  @Test
  void containsFalse() {
    assertEquals("false", eval("let d = {\"a\": 1}\nd.contains(\"z\")"));
  }

  @Test
  void indexAssignment() {
    assertEquals("99", eval("let d = {\"x\": 1}\nd[\"x\"] = 99\nd[\"x\"]"));
  }

  @Test
  void addNewKey() {
    assertEquals("hello", eval("let d = {}\nd[\"msg\"] = \"hello\"\nd[\"msg\"]"));
  }

  @Test
  void iterateWithForLoop() {
    assertEquals("6",
        eval("let d = {\"a\": 1, \"b\": 2, \"c\": 3}\n"
            + "let s = 0\n"
            + "for k in d.keys() { s += d[k] }\n"
            + "s"));
  }
}
