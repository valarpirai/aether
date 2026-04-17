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

/** Tests for the stdlib testing framework (assert_eq, assert_true, test, test_summary). */
class TestingFrameworkTest {

  private Evaluator evaluator;

  @BeforeEach
  void setUp() {
    evaluator = Evaluator.withStdlib();
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

  // ── assert_eq ─────────────────────────────────────────────────────────────────

  @Test
  void assertEqPassesWhenEqual() {
    eval("assert_eq(1, 1)");
  }

  @Test
  void assertEqPassesForStrings() {
    eval("assert_eq(\"hello\", \"hello\")");
  }

  @Test
  void assertEqFailsWhenNotEqual() {
    assertThrows(AetherRuntimeException.Thrown.class, () -> eval("assert_eq(1, 2)"));
  }

  @Test
  void assertEqFailsWithDescriptiveMessage() {
    AetherRuntimeException.Thrown ex = assertThrows(
        AetherRuntimeException.Thrown.class, () -> eval("assert_eq(\"a\", \"b\")"));
    assertTrue(ex.getMessage().contains("a") || ex.getMessage().contains("assert"));
  }

  // ── assert_true / assert_false ────────────────────────────────────────────────

  @Test
  void assertTruePasses() {
    eval("assert_true(true)");
  }

  @Test
  void assertTrueFails() {
    assertThrows(AetherRuntimeException.Thrown.class, () -> eval("assert_true(false)"));
  }

  @Test
  void assertFalsePasses() {
    eval("assert_false(false)");
  }

  @Test
  void assertFalseFails() {
    assertThrows(AetherRuntimeException.Thrown.class, () -> eval("assert_false(true)"));
  }

  // ── assert_null / assert_not_null ─────────────────────────────────────────────

  @Test
  void assertNullPasses() {
    eval("assert_null(null)");
  }

  @Test
  void assertNullFails() {
    assertThrows(AetherRuntimeException.Thrown.class, () -> eval("assert_null(42)"));
  }

  @Test
  void assertNotNullPasses() {
    eval("assert_not_null(42)");
  }

  @Test
  void assertNotNullFails() {
    assertThrows(AetherRuntimeException.Thrown.class, () -> eval("assert_not_null(null)"));
  }

  // ── expect_error ─────────────────────────────────────────────────────────────

  @Test
  void expectErrorPassesWhenFnThrows() {
    eval("expect_error(fn() { throw \"boom\" })");
  }

  @Test
  void expectErrorFailsWhenFnDoesNotThrow() {
    assertThrows(AetherRuntimeException.Thrown.class,
        () -> eval("expect_error(fn() { 1 + 1 })"));
  }

  // ── test() runner ─────────────────────────────────────────────────────────────

  @Test
  void testRunnerPassesOnSuccess() {
    String result = eval("test(\"my test\", fn() { assert_eq(1, 1) })");
    assertEquals("true", eval(
        "let r = test(\"ok\", fn() { assert_eq(2, 2) })\nr[\"passed\"]"));
  }

  @Test
  void testRunnerDoesNotThrowOnFailure() {
    eval("test(\"fail\", fn() { assert_eq(1, 2) })");
  }

  @Test
  void testReturnsPassedDict() {
    assertEquals("true", eval("let r = test(\"t\", fn() { assert_eq(1, 1) })\nr[\"passed\"]"));
  }

  @Test
  void testReturnsFailureDict() {
    assertEquals("false", eval("let r = test(\"t\", fn() { assert_eq(1, 2) })\nr[\"passed\"]"));
  }

  // ── test_summary ─────────────────────────────────────────────────────────────

  @Test
  void testSummaryWithResults() {
    eval("let r1 = test(\"a\", fn() { assert_eq(1, 1) })\n"
        + "let r2 = test(\"b\", fn() { assert_eq(2, 2) })\n"
        + "test_summary([r1, r2])");
  }

  // ── helper ───────────────────────────────────────────────────────────────────

  private static void assertTrue(boolean value) {
    if (!value) throw new AssertionError("Expected true");
  }
}
