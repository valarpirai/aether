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
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;

/**
 * Additional evaluator tests covering JSON, modules, control flow, dicts, closures, and
 * error propagation.
 */
class MoreEvaluatorTest {

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

  // ── JSON ─────────────────────────────────────────────────────────────────────

  @Nested
  class Json {

    @Test
    void parseObject() {
      assertEquals("42", eval("json_parse(\"{\\\"x\\\": 42}\")[\"x\"]"));
    }

    @Test
    void parseArray() {
      assertEquals("2", eval("json_parse(\"[1, 2, 3]\")[1]"));
    }

    @Test
    void parseString() {
      assertEquals("hello", eval("json_parse(\"\\\"hello\\\"\")"));
    }

    @Test
    void parseNumber() {
      assertEquals("99", eval("json_parse(\"99\")"));
    }

    @Test
    void parseBool() {
      assertEquals("true", eval("json_parse(\"true\")"));
    }

    @Test
    void parseNull() {
      assertEquals("null", eval("json_parse(\"null\")"));
    }

    @Test
    void stringifyInt() {
      assertEquals("42", eval("json_stringify(42)"));
    }

    @Test
    void stringifyString() {
      assertEquals("\"hello\"", eval("json_stringify(\"hello\")"));
    }

    @Test
    void stringifyArray() {
      assertEquals("[1,2,3]", eval("json_stringify([1, 2, 3])"));
    }

    @Test
    void roundTrip() {
      assertEquals("99", eval("json_parse(json_stringify({\"v\": 99}))[\"v\"]"));
    }
  }

  // ── Module system ─────────────────────────────────────────────────────────────

  @Nested
  class Modules {

    @Test
    void importMath() {
      assertEquals("5", eval("import math\nmath.abs(-5)"));
    }

    @Test
    void importAs() {
      assertEquals("3", eval("import math as m\nm.min(3, 7)"));
    }

    @Test
    void fromImport() {
      assertEquals("7", eval("from math import max\nmax(3, 7)"));
    }

    @Test
    void fromImportMultiple() {
      // max(1,2)=2, min(3,2)=2
      assertEquals("2", eval("from math import min, max\nmin(3, max(1, 2))"));
    }

    @Test
    void importCollections() {
      assertEquals("[2, 4, 6]",
          eval("from collections import map\nmap([1, 2, 3], fn(x) { return x * 2 })"));
    }

    @Test
    void importCore() {
      assertEquals("[0, 1, 2]", eval("from core import range\nrange(3)"));
    }

    @Test
    void unknownModuleThrows() {
      assertThrows(AetherRuntimeException.InvalidOperation.class,
          () -> eval("import nonexistent"));
    }
  }

  // ── Control flow ─────────────────────────────────────────────────────────────

  @Nested
  class ControlFlow {

    @Test
    void continueSkipsIteration() {
      // sum of [1,2,3,4] skipping 3 → 1+2+4 = 7
      assertEquals("7",
          eval("let s = 0\nfor x in [1, 2, 3, 4] { if (x == 3) { continue }\ns += x }\ns"));
    }

    @Test
    void continueInWhile() {
      // i in 1..5, skip i=3 → 1+2+4+5 = 12
      assertEquals("12",
          eval("let s = 0\nlet i = 0\nwhile (i < 5) { i += 1\nif (i == 3) { continue }\ns += i }\ns"));
    }

    @Test
    void nestedBreak() {
      // Only breaks the inner loop
      assertEquals("3",
          eval(
              "let count = 0\n"
              + "for i in [1, 2, 3] {\n"
              + "  let j = 0\n"
              + "  while (true) { j += 1\nif (j == 1) { break } }\n"
              + "  count += 1\n"
              + "}\n"
              + "count"));
    }

    @Test
    void earlyReturnFromFunction() {
      assertEquals("1",
          eval("fn first_positive(arr) {\n"
              + "  for x in arr { if (x > 0) { return x } }\n"
              + "  return -1\n"
              + "}\n"
              + "first_positive([-3, -1, 1, 2])"));
    }
  }

  // ── Dict ─────────────────────────────────────────────────────────────────────

  @Nested
  class Dicts {

    @Test
    void values() {
      // values() order matches insertion order
      assertEquals("[1]", eval("let d = {\"a\": 1}\nd.values()"));
    }

    @Test
    void multipleValues() {
      assertEquals("[1, 2]", eval("let d = {\"a\": 1, \"b\": 2}\nd.values()"));
    }

    @Test
    void dictAssignment() {
      assertEquals("99", eval("let d = {\"x\": 1}\nd[\"x\"] = 99\nd[\"x\"]"));
    }

    @Test
    void dictLen() {
      assertEquals("3", eval("len({\"a\": 1, \"b\": 2, \"c\": 3})"));
    }

    @Test
    void iterateKeys() {
      assertEquals("6",
          eval("let d = {\"a\": 1, \"b\": 2, \"c\": 3}\n"
              + "let s = 0\n"
              + "for k in d.keys() { s += d[k] }\n"
              + "s"));
    }
  }

  // ── Closures & higher-order ──────────────────────────────────────────────────

  @Nested
  class Closures {

    @Test
    void closureCapture() {
      assertEquals("15",
          eval("fn make_adder(n) { return fn(x) { return n + x } }\n"
              + "let add5 = make_adder(5)\n"
              + "add5(10)"));
    }

    @Test
    void closureMutatesCapture() {
      assertEquals("3",
          eval("fn counter() {\n"
              + "  let n = 0\n"
              + "  return fn() { n += 1\nreturn n }\n"
              + "}\n"
              + "let c = counter()\n"
              + "c()\nc()\nc()"));
    }

    @Test
    void higherOrderCompose() {
      assertEquals("9",
          eval("fn compose(f, g) { return fn(x) { return f(g(x)) } }\n"
              + "fn double(x) { return x * 2 }\n"
              + "fn inc(x) { return x + 1 }\n"
              + "let double_then_inc = compose(inc, double)\n"
              + "double_then_inc(4)"));
    }

    @Test
    void functionStoredInArray() {
      assertEquals("6",
          eval("let fns = [fn(x) { return x + 1 }, fn(x) { return x * 2 }]\n"
              + "fns[1](3)"));
    }

    @Test
    void functionStoredInDict() {
      assertEquals("10",
          eval("let ops = {\"double\": fn(x) { return x * 2 }}\n"
              + "ops[\"double\"](5)"));
    }
  }

  // ── Error handling ───────────────────────────────────────────────────────────

  @Nested
  class ErrorHandling {

    @Test
    void catchGetsErrorValue() {
      assertEquals("oops", eval("let msg = \"\"\ntry { throw \"oops\" } catch(e) { msg = e }\nmsg"));
    }

    @Test
    void errorPropagatesAcrossCallStack() {
      assertEquals("caught",
          eval("fn inner() { throw \"boom\" }\n"
              + "fn outer() { inner() }\n"
              + "let r = \"\"\n"
              + "try { outer() } catch(e) { r = \"caught\" }\n"
              + "r"));
    }

    @Test
    void tryCatchAllowsContinuation() {
      assertEquals("after",
          eval("try { throw \"x\" } catch(e) { }\n"
              + "\"after\""));
    }

    @Test
    void rethrow() {
      assertThrows(AetherRuntimeException.Thrown.class,
          () -> eval("try { throw \"x\" } catch(e) { throw e }"));
    }

    @Test
    void catchIntError() {
      assertEquals("42", eval("let r = 0\ntry { throw 42 } catch(e) { r = e }\nr"));
    }

    @Test
    void finallyStyleCleanup() {
      // Simulate cleanup with a flag; no finally keyword yet
      assertEquals("cleaned",
          eval("let state = \"dirty\"\n"
              + "try { throw \"err\" } catch(e) { state = \"cleaned\" }\n"
              + "state"));
    }
  }

  // ── Strings ──────────────────────────────────────────────────────────────────

  @Nested
  class Strings {

    @Test
    void interpolationWithExpression() {
      assertEquals("result: 7", eval("let a = 3\nlet b = 4\n\"result: ${a + b}\""));
    }

    @Test
    void interpolationNested() {
      assertEquals("value is 10",
          eval("fn get() { return 10 }\n\"value is ${get()}\""));
    }

    @Test
    void multipleInterpolations() {
      assertEquals("3 + 4 = 7", eval("let a = 3\nlet b = 4\n\"${a} + ${b} = ${a + b}\""));
    }

    @Test
    void stringEscapeNewline() {
      assertEquals("a\nb", eval("\"a\\nb\""));
    }

    @Test
    void stringEscapeTab() {
      assertEquals("a\tb", eval("\"a\\tb\""));
    }

    @Test
    void stringEscapeQuote() {
      assertEquals("say \"hi\"", eval("\"say \\\"hi\\\"\""));
    }

    @Test
    void negativeIndex() {
      // negative indexing not supported — just verify positive boundary works
      assertEquals("o", eval("\"hello\"[4]"));
    }
  }

  // ── Structs ──────────────────────────────────────────────────────────────────

  @Nested
  class Structs {

    @Test
    void multipleMethodsOnStruct() {
      assertEquals("5",
          eval("struct Rect { w, h\n"
              + "fn area(self) { return self.w * self.h }\n"
              + "fn perimeter(self) { return 2 * (self.w + self.h) }\n"
              + "}\n"
              + "let r = Rect { w: 1, h: 5 }\n"
              + "r.area()"));
    }

    @Test
    void structMethodChain() {
      assertEquals("12",
          eval("struct Rect { w, h\n"
              + "fn perimeter(self) { return 2 * (self.w + self.h) }\n"
              + "}\n"
              + "let r = Rect { w: 2, h: 4 }\n"
              + "r.perimeter()"));
    }

    @Test
    void structInArray() {
      assertEquals("10",
          eval("struct Point { x, y }\n"
              + "let points = [Point { x: 1, y: 2 }, Point { x: 3, y: 4 }]\n"
              + "points[0].x + points[0].y + points[1].x + points[1].y"));
    }

    @Test
    void structMethodMutatesField() {
      assertEquals("6",
          eval("struct Counter { n\n"
              + "fn inc(self) { self.n += 1 }\n"
              + "}\n"
              + "let c = Counter { n: 3 }\n"
              + "c.inc()\nc.inc()\nc.inc()\n"
              + "c.n"));
    }
  }

  // ── Type system ──────────────────────────────────────────────────────────────

  @Nested
  class Types {

    @Test
    void typeOfFunction() {
      assertEquals("function", eval("fn f() { }\ntype(f)"));
    }

    @Test
    void typeOfArray() {
      assertEquals("array", eval("type([1, 2])"));
    }

    @Test
    void typeOfDict() {
      assertEquals("dict", eval("type({\"a\": 1})"));
    }

    @Test
    void typeOfFloat() {
      assertEquals("float", eval("type(3.14)"));
    }

    @Test
    void sleepZeroReturnsNull() {
      assertEquals("null", eval("sleep(0)"));
    }
  }
}
