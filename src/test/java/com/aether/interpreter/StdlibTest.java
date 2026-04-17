package com.aether.interpreter;

import static org.junit.jupiter.api.Assertions.assertEquals;

import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Stmt;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;

/** Tests for the built-in standard library modules (core, collections, math, string, testing). */
class StdlibTest {

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

  // ── core ─────────────────────────────────────────────────────────────────────

  @Nested
  class Core {

    @Test
    void rangeSingleArg() {
      assertEquals("[0, 1, 2, 3, 4]", eval("range(5)"));
    }

    @Test
    void rangeTwoArgs() {
      assertEquals("[2, 3, 4]", eval("range(2, 5)"));
    }

    @Test
    void rangeEmpty() {
      assertEquals("[]", eval("range(0)"));
    }

    @Test
    void rangeUsedInForLoop() {
      assertEquals("10", eval("let s = 0\nfor i in range(5) { s += i }\ns"));
    }

    @Test
    void enumerate() {
      assertEquals("[[0, \"a\"], [1, \"b\"], [2, \"c\"]]", eval("enumerate([\"a\", \"b\", \"c\"])"));
    }

    @Test
    void enumerateEmpty() {
      assertEquals("[]", eval("enumerate([])"));
    }
  }

  // ── collections ──────────────────────────────────────────────────────────────

  @Nested
  class Collections {

    @Test
    void map() {
      assertEquals("[2, 4, 6]", eval("map([1, 2, 3], fn(x) { return x * 2 })"));
    }

    @Test
    void mapEmpty() {
      assertEquals("[]", eval("map([], fn(x) { return x })"));
    }

    @Test
    void filter() {
      assertEquals("[3, 4]", eval("filter([1, 2, 3, 4], fn(x) { return x > 2 })"));
    }

    @Test
    void filterNoneMatch() {
      assertEquals("[]", eval("filter([1, 2, 3], fn(x) { return x > 10 })"));
    }

    @Test
    void reduce() {
      assertEquals("6", eval("reduce([1, 2, 3], fn(acc, x) { return acc + x }, 0)"));
    }

    @Test
    void reduceString() {
      assertEquals("abc", eval("reduce([\"a\", \"b\", \"c\"], fn(acc, x) { return acc + x }, \"\")"));
    }

    @Test
    void findReturnsFirst() {
      assertEquals("3", eval("find([1, 2, 3, 4], fn(x) { return x > 2 })"));
    }

    @Test
    void findReturnsNullWhenMissing() {
      assertEquals("null", eval("find([1, 2, 3], fn(x) { return x > 10 })"));
    }

    @Test
    void everyTrue() {
      assertEquals("true", eval("every([2, 4, 6], fn(x) { return x % 2 == 0 })"));
    }

    @Test
    void everyFalse() {
      assertEquals("false", eval("every([2, 3, 6], fn(x) { return x % 2 == 0 })"));
    }

    @Test
    void everyEmptyIsTrue() {
      assertEquals("true", eval("every([], fn(x) { return false })"));
    }

    @Test
    void someTrue() {
      assertEquals("true", eval("some([1, 3, 5], fn(x) { return x > 4 })"));
    }

    @Test
    void someFalse() {
      assertEquals("false", eval("some([1, 3, 5], fn(x) { return x > 10 })"));
    }

    @Test
    void someEmptyIsFalse() {
      assertEquals("false", eval("some([], fn(x) { return true })"));
    }

    @Test
    void concat() {
      assertEquals("[1, 2, 3, 4]", eval("concat([1, 2], [3, 4])"));
    }

    @Test
    void concatWithEmpty() {
      assertEquals("[1, 2]", eval("concat([1, 2], [])"));
    }

    @Test
    void sortNumbers() {
      assertEquals("[1, 2, 3, 4, 5]", eval("sort([3, 1, 4, 2, 5], null)"));
    }

    @Test
    void sortWithComparator() {
      // comparator(b, a): return true to swap → descending order
      assertEquals("[5, 4, 3, 1]",
          eval("sort([3, 1, 4, 5], fn(a, b) { return a > b })"));
    }
  }

  // ── math ─────────────────────────────────────────────────────────────────────

  @Nested
  class Math {

    @Test
    void absPositive() {
      assertEquals("5", eval("abs(5)"));
    }

    @Test
    void absNegative() {
      assertEquals("5", eval("abs(-5)"));
    }

    @Test
    void absZero() {
      assertEquals("0", eval("abs(0)"));
    }

    @Test
    void minTwoArgs() {
      assertEquals("3", eval("min(3, 5)"));
    }

    @Test
    void minArray() {
      assertEquals("1", eval("min([3, 1, 5, 2])"));
    }

    @Test
    void maxTwoArgs() {
      assertEquals("5", eval("max(3, 5)"));
    }

    @Test
    void maxArray() {
      assertEquals("5", eval("max([3, 1, 5, 2])"));
    }

    @Test
    void sum() {
      assertEquals("6", eval("sum([1, 2, 3])"));
    }

    @Test
    void sumEmpty() {
      assertEquals("0", eval("sum([])"));
    }

    @Test
    void clampAbove() {
      assertEquals("5", eval("clamp(10, 0, 5)"));
    }

    @Test
    void clampBelow() {
      assertEquals("0", eval("clamp(-1, 0, 5)"));
    }

    @Test
    void clampWithin() {
      assertEquals("3", eval("clamp(3, 0, 5)"));
    }

    @Test
    void signPositive() {
      assertEquals("1", eval("sign(42)"));
    }

    @Test
    void signNegative() {
      assertEquals("-1", eval("sign(-7)"));
    }

    @Test
    void signZero() {
      assertEquals("0", eval("sign(0)"));
    }
  }

  // ── string ───────────────────────────────────────────────────────────────────

  @Nested
  class StringUtils {

    @Test
    void join() {
      assertEquals("a,b,c", eval("join([\"a\", \"b\", \"c\"], \",\")"));
    }

    @Test
    void joinEmpty() {
      assertEquals("", eval("join([], \",\")"));
    }

    @Test
    void joinSingleElement() {
      assertEquals("x", eval("join([\"x\"], \",\")"));
    }

    @Test
    void repeat() {
      assertEquals("ababab", eval("repeat(\"ab\", 3)"));
    }

    @Test
    void repeatZero() {
      assertEquals("", eval("repeat(\"ab\", 0)"));
    }

    @Test
    void reverse() {
      assertEquals("olleh", eval("reverse(\"hello\")"));
    }

    @Test
    void reverseEmpty() {
      assertEquals("", eval("reverse(\"\")"));
    }

    @Test
    void startsWithTrue() {
      assertEquals("true", eval("starts_with(\"hello\", \"he\")"));
    }

    @Test
    void startsWithFalse() {
      assertEquals("false", eval("starts_with(\"hello\", \"wo\")"));
    }

    @Test
    void startsWithEmpty() {
      assertEquals("true", eval("starts_with(\"hello\", \"\")"));
    }

    @Test
    void endsWithTrue() {
      assertEquals("true", eval("ends_with(\"hello\", \"lo\")"));
    }

    @Test
    void endsWithFalse() {
      assertEquals("false", eval("ends_with(\"hello\", \"he\")"));
    }
  }
}
