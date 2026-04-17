package com.aether.interpreter;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import com.aether.exception.AetherRuntimeException;
import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Stmt;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;

/**
 * Regression tests for confirmed interpreter bugs.
 * Each nested class documents one bug: what was wrong and what is now correct.
 */
class BugRegressionTest {

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

  // ── Bug 1: String comparison always used `< 0` regardless of operator ─────────
  // "abc" > "abd" returned true (wrong); "abd" > "abc" returned false (wrong).

  @Nested
  class StringComparison {

    @Test
    void lessTrue() {
      assertEquals("true", eval("\"abc\" < \"abd\""));
    }

    @Test
    void lessFalse() {
      assertEquals("false", eval("\"abd\" < \"abc\""));
    }

    @Test
    void greaterTrue() {
      assertEquals("true", eval("\"abd\" > \"abc\""));
    }

    @Test
    void greaterFalse() {
      assertEquals("false", eval("\"abc\" > \"abd\""));
    }

    @Test
    void lessEqualSame() {
      assertEquals("true", eval("\"abc\" <= \"abc\""));
    }

    @Test
    void greaterEqualSame() {
      assertEquals("true", eval("\"abc\" >= \"abc\""));
    }

    @Test
    void greaterEqualTrue() {
      assertEquals("true", eval("\"abd\" >= \"abc\""));
    }

    @Test
    void lessEqualFalse() {
      assertEquals("false", eval("\"abd\" <= \"abc\""));
    }

    @Test
    void sortedByComparison() {
      // Verifies comparison drives correct sorting logic
      assertEquals("true", eval("\"apple\" < \"banana\""));
      assertEquals("false", eval("\"banana\" < \"apple\""));
    }
  }

  // ── Bug 2: for-loop variable leaked into outer scope ─────────────────────────
  // After `for x in [1,2,3] {}`, `x` was visible with value 3.

  @Nested
  class ForLoopScope {

    @Test
    void loopVarNotVisibleAfterLoop() {
      assertThrows(AetherRuntimeException.UndefinedVariable.class,
          () -> eval("for x in [1, 2, 3] { }\nx"));
    }

    @Test
    void loopVarDoesNotShadowOuter() {
      // An outer `x` should retain its original value after the loop
      assertEquals("99",
          eval("let x = 99\nfor x in [1, 2, 3] { }\nx"));
    }

    @Test
    void loopVarScopeIndependentBetweenLoops() {
      // Two successive loops with the same var name must not interfere
      assertEquals("21",
          eval("let s = 0\n"
              + "for i in [1, 2, 3] { s += i }\n"
              + "for i in [4, 5, 6] { s += i }\n"
              + "s"));
    }

    @Test
    void loopBodyScopeIndependent() {
      // Variables declared inside the body must not leak out
      assertThrows(AetherRuntimeException.UndefinedVariable.class,
          () -> eval("for x in [1] { let inner = x }\ninner"));
    }
  }

  // ── Bug 3: sort comparator never returned 0 for equal elements ────────────────
  // Comparator `fn(a,b){return a>b}` on [3,1,2,1,3] produced unstable results.

  @Nested
  class SortComparator {

    @Test
    void descendingWithDuplicates() {
      assertEquals("[5, 4, 3, 1, 1]",
          eval("let a = [3, 1, 4, 1, 5]\na.sort(fn(a, b) { return a > b })\na"));
    }

    @Test
    void ascendingWithDuplicates() {
      assertEquals("[1, 1, 3, 4, 5]",
          eval("let a = [3, 1, 4, 1, 5]\na.sort(fn(a, b) { return a < b })\na"));
    }

    @Test
    void allEqualElements() {
      assertEquals("[2, 2, 2]",
          eval("let a = [2, 2, 2]\na.sort(fn(a, b) { return a > b })\na"));
    }

    @Test
    void naturalSortStillWorks() {
      assertEquals("[1, 2, 3, 4, 5]",
          eval("let a = [3, 1, 4, 2, 5]\na.sort()\na"));
    }
  }

  // ── Bug 4: empty dict was truthy (missing Dict case in isTruthy) ──────────────
  // `if ({}) { ... }` executed the branch — empty dict should be falsy.

  @Nested
  class DictTruthiness {

    @Test
    void emptyDictIsFalsy() {
      assertEquals("falsy", eval("let d = {}\nlet r = \"falsy\"\nif (d) { r = \"truthy\" }\nr"));
    }

    @Test
    void nonEmptyDictIsTruthy() {
      assertEquals("truthy",
          eval("let d = {\"a\": 1}\nlet r = \"falsy\"\nif (d) { r = \"truthy\" }\nr"));
    }

    @Test
    void notEmptyDict() {
      assertEquals("true", eval("let d = {}\n!d"));
    }

    @Test
    void notNonEmptyDict() {
      assertEquals("false", eval("let d = {\"x\": 1}\n!d"));
    }

    @Test
    void emptyArrayStillFalsy() {
      // Ensure array truthiness was not broken
      assertEquals("falsy", eval("let a = []\nlet r = \"falsy\"\nif (a) { r = \"truthy\" }\nr"));
    }

    @Test
    void nonEmptyArrayStillTruthy() {
      assertEquals("truthy",
          eval("let a = [1]\nlet r = \"falsy\"\nif (a) { r = \"truthy\" }\nr"));
    }
  }

  // ── Bug 5: float modulo was not supported (only int % int worked) ─────────────
  // `5.5 % 2.5` threw TypeError; `7 % 2.5` threw TypeError.

  @Nested
  class FloatModulo {

    @Test
    void floatModFloat() {
      assertEquals("0.5", eval("5.5 % 2.5"));
    }

    @Test
    void intModFloat() {
      assertEquals("1.0", eval("7 % 2.0"));
    }

    @Test
    void floatModInt() {
      assertEquals("1.5", eval("7.5 % 3"));
    }

    @Test
    void floatModZeroThrows() {
      assertThrows(AetherRuntimeException.DivisionByZero.class,
          () -> eval("5.5 % 0.0"));
    }
  }

  // ── Bug 6: float division by zero threw instead of returning Infinity ─────────
  // `1.0 / 0.0` threw DivisionByZero; IEEE 754 says it should be Infinity.

  @Nested
  class FloatDivisionByZero {

    @Test
    void floatDivByFloatZeroIsInfinity() {
      assertEquals("Infinity", eval("1.0 / 0.0"));
    }

    @Test
    void intDivByFloatZeroIsInfinity() {
      assertEquals("Infinity", eval("1 / 0.0"));
    }

    @Test
    void floatDivByIntZeroIsInfinity() {
      assertEquals("Infinity", eval("1.0 / 0"));
    }

    @Test
    void negativeInfinity() {
      assertEquals("-Infinity", eval("-1.0 / 0.0"));
    }

    @Test
    void intDivByIntZeroStillThrows() {
      assertThrows(AetherRuntimeException.DivisionByZero.class,
          () -> eval("1 / 0"));
    }
  }

  // ── Bug 7: struct init accepted undeclared fields silently ────────────────────
  // `Point{x:1, z:99}` on a struct with only `x` field should throw.

  @Nested
  class StructFieldValidation {

    @Test
    void initRejectsUndeclaredField() {
      assertThrows(AetherRuntimeException.InvalidOperation.class,
          () -> eval("struct P { x }\nP { x: 1, z: 99 }"));
    }

    @Test
    void assignRejectsUndeclaredField() {
      assertThrows(AetherRuntimeException.InvalidOperation.class,
          () -> eval("struct P { x }\nlet p = P { x: 1 }\np.z = 99"));
    }

    @Test
    void initWithDeclaredFieldsWorks() {
      assertEquals("42", eval("struct P { x }\nlet p = P { x: 42 }\np.x"));
    }

    @Test
    void assignDeclaredFieldWorks() {
      assertEquals("99", eval("struct P { x }\nlet p = P { x: 1 }\np.x = 99\np.x"));
    }
  }
}
