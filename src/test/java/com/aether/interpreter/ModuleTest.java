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

/** Comprehensive tests for the stdlib module system (import, from...import, aliases). */
class ModuleTest {

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

  // ── from ... import ───────────────────────────────────────────────────────────

  @Test
  void fromImportSingle() {
    assertEquals("5", eval("from math import abs\nabs(-5)"));
  }

  @Test
  void fromImportMultiple() {
    // max(5, 3) = 5, min(10, 5) = 5
    assertEquals("5", eval("from math import min, max\nmin(10, max(5, 3))"));
  }

  @Test
  void fromImportString() {
    assertEquals("olleh", eval("from string import reverse\nreverse(\"hello\")"));
  }

  @Test
  void fromImportCollections() {
    assertEquals("[2, 4, 6]",
        eval("from collections import map\nmap([1, 2, 3], fn(x) { return x * 2 })"));
  }

  @Test
  void fromImportCore() {
    assertEquals("[0, 1, 2]", eval("from core import range\nrange(3)"));
  }

  // ── import namespace ──────────────────────────────────────────────────────────

  @Test
  void importModuleNamespace() {
    assertEquals("5", eval("import math\nmath.abs(-5)"));
  }

  @Test
  void importModuleNamespaceMultipleMembers() {
    assertEquals("5", eval("import math\nmath.min(10, math.max(3, 5))"));
  }

  // ── import ... as alias ───────────────────────────────────────────────────────

  @Test
  void importModuleAsAlias() {
    assertEquals("5", eval("import math as m\nm.abs(-5)"));
  }

  @Test
  void fromImportWithAlias() {
    assertEquals("5", eval("from math import abs as absolute\nabsolute(-5)"));
  }

  @Test
  void fromImportMultipleWithAliases() {
    // maximum(3, 5) = 5, minimum(10, 5) = 5
    assertEquals("5",
        eval("from math import min as minimum, max as maximum\nminimum(10, maximum(3, 5))"));
  }

  // ── import same module twice ──────────────────────────────────────────────────

  @Test
  void importSameModuleTwice() {
    assertEquals("8",
        eval("from math import abs\nfrom math import max\nabs(-3) + max(1, 5)"));
  }

  // ── error cases ───────────────────────────────────────────────────────────────

  @Test
  void unknownModuleThrows() {
    assertThrows(AetherRuntimeException.class, () -> eval("import nonexistent_module_xyz"));
  }

  @Test
  void memberNotFoundThrows() {
    AetherRuntimeException ex = assertThrows(AetherRuntimeException.class,
        () -> eval("import math\nmath.nonexistent_fn"));
    assertTrue(ex.getMessage() != null);
  }
}
