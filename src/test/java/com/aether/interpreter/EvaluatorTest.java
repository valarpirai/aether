package com.aether.interpreter;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertThrows;

import com.aether.exception.AetherRuntimeException;
import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Stmt;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

/** Integration-style tests for {@link Evaluator}. */
class EvaluatorTest {

  private Evaluator evaluator;

  @BeforeEach
  void setUp() {
    evaluator = Evaluator.withoutStdlib();
  }

  /** Convenience: parse + execute source, return last expression value as display string. */
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

  // ── Arithmetic ───────────────────────────────────────────────────────────────

  @Test
  void intArithmetic() {
    assertEquals("7", eval("3 + 4"));
    assertEquals("2", eval("5 - 3"));
    assertEquals("12", eval("3 * 4"));
    assertEquals("2", eval("7 / 3"));
    assertEquals("1", eval("7 % 3"));
  }

  @Test
  void floatArithmetic() {
    assertEquals("1.5", eval("3.0 / 2.0"));
    assertEquals("6.28", eval("3.14 * 2.0"));
  }

  @Test
  void mixedIntFloat() {
    assertEquals("5.0", eval("2 + 3.0"));
    assertEquals("1.5", eval("3 / 2.0"));
  }

  @Test
  void stringConcatenation() {
    assertEquals("hello world", eval("\"hello\" + \" world\""));
  }

  @Test
  void divisionByZeroThrows() {
    assertThrows(AetherRuntimeException.DivisionByZero.class, () -> eval("1 / 0"));
  }

  // ── Booleans ─────────────────────────────────────────────────────────────────

  @Test
  void booleanLogic() {
    assertEquals("true", eval("true && true"));
    assertEquals("false", eval("true && false"));
    assertEquals("true", eval("false || true"));
    assertEquals("true", eval("!false"));
  }

  @Test
  void comparison() {
    assertEquals("true", eval("3 < 5"));
    assertEquals("false", eval("5 < 3"));
    assertEquals("true", eval("5 == 5"));
    assertEquals("false", eval("5 != 5"));
    assertEquals("true", eval("5 >= 5"));
  }

  // ── Truthiness ───────────────────────────────────────────────────────────────

  @Test
  void truthinessRules() {
    // 0, null, "" are falsy → !falsy = true
    assertEquals("true", eval("!0"));
    assertEquals("true", eval("!null"));
    assertEquals("true", eval("!\"\""));
    // 1, "a" are truthy → !truthy = false
    assertEquals("false", eval("!1"));
    assertEquals("false", eval("!\"a\""));
  }

  // ── Variables ────────────────────────────────────────────────────────────────

  @Test
  void letAndLookup() {
    assertEquals("42", eval("let x = 42\nx"));
  }

  @Test
  void assignment() {
    assertEquals("10", eval("let x = 5\nx = 10\nx"));
  }

  @Test
  void compoundAssignment() {
    assertEquals("8", eval("let x = 5\nx += 3\nx"));
  }

  @Test
  void undefinedVariableThrows() {
    assertThrows(AetherRuntimeException.UndefinedVariable.class, () -> eval("xyz"));
  }

  // ── Control flow ─────────────────────────────────────────────────────────────

  @Test
  void ifTrue() {
    assertEquals("yes", eval("let r = \"no\"\nif (true) { r = \"yes\" }\nr"));
  }

  @Test
  void ifElse() {
    assertEquals("b", eval("let r = \"a\"\nif (false) { r = \"a\" } else { r = \"b\" }\nr"));
  }

  @Test
  void whileLoop() {
    assertEquals("3", eval("let i = 0\nwhile (i < 3) { i += 1 }\ni"));
  }

  @Test
  void forLoop() {
    assertEquals("6", eval("let sum = 0\nfor x in [1, 2, 3] { sum += x }\nsum"));
  }

  @Test
  void breakInWhile() {
    assertEquals("2", eval("let i = 0\nwhile (true) { i += 1\nif (i == 2) { break } }\ni"));
  }

  // ── Functions ────────────────────────────────────────────────────────────────

  @Test
  void functionDeclarationAndCall() {
    assertEquals("7", eval("fn add(a, b) { return a + b }\nadd(3, 4)"));
  }

  @Test
  void optionalParams() {
    assertEquals("null", eval("fn f(a, b) { return b }\nf(1)"));
  }

  @Test
  void closures() {
    assertEquals("11",
        eval("fn make_adder(n) { return fn(x) { return n + x } }\nlet add10 = make_adder(10)\nadd10(1)"));
  }

  @Test
  void recursion() {
    assertEquals("120", eval("fn fact(n) { if (n <= 1) { return 1 } return n * fact(n - 1) }\nfact(5)"));
  }

  @Test
  void stackOverflowThrows() {
    assertThrows(
        AetherRuntimeException.StackOverflow.class,
        () -> eval("fn infinite(x) { infinite(x + 1) }\ninfinite(0)"));
  }

  // ── Strings ──────────────────────────────────────────────────────────────────

  @Test
  void stringIndexing() {
    assertEquals("h", eval("\"hello\"[0]"));
    assertEquals("o", eval("\"hello\"[4]"));
  }

  @Test
  void stringMethods() {
    assertEquals("HELLO", eval("\"hello\".upper()"));
    assertEquals("hello", eval("\"HELLO\".lower()"));
    assertEquals("hi", eval("\"  hi  \".trim()"));
  }

  @Test
  void stringSplit() {
    assertEquals("[\"a\", \"b\", \"c\"]", eval("\"a,b,c\".split(\",\")"));
  }

  @Test
  void stringInterpolation() {
    assertEquals("hello world", eval("let name = \"world\"\n\"hello ${name}\""));
  }

  @Test
  void stringLength() {
    assertEquals("5", eval("\"hello\".length"));
  }

  // ── Arrays ───────────────────────────────────────────────────────────────────

  @Test
  void arrayLiteral() {
    assertEquals("[1, 2, 3]", eval("[1, 2, 3]"));
  }

  @Test
  void arrayIndex() {
    assertEquals("2", eval("[1, 2, 3][1]"));
  }

  @Test
  void arrayLength() {
    assertEquals("3", eval("[1, 2, 3].length"));
  }

  @Test
  void arrayPushPop() {
    assertEquals("3", eval("let a = [1, 2]\na.push(3)\na[2]"));
    assertEquals("2", eval("let a = [1, 2]\na.pop()"));
  }

  @Test
  void arraySlice() {
    assertEquals("[2, 3]", eval("[1, 2, 3, 4][1:3]"));
  }

  @Test
  void spreadOperator() {
    assertEquals("[1, 2, 3, 4]", eval("let a = [1, 2]\n[...a, 3, 4]"));
  }

  // ── Dicts ────────────────────────────────────────────────────────────────────

  @Test
  void dictLiteralAndAccess() {
    assertEquals("42", eval("let d = {\"x\": 42}\nd[\"x\"]"));
  }

  @Test
  void dictKeys() {
    assertEquals("[\"a\"]", eval("let d = {\"a\": 1}\nd.keys()"));
  }

  @Test
  void dictContains() {
    assertEquals("true", eval("let d = {\"a\": 1}\nd.contains(\"a\")"));
    assertEquals("false", eval("let d = {\"a\": 1}\nd.contains(\"b\")"));
  }

  // ── Error handling ───────────────────────────────────────────────────────────

  @Test
  void tryCatch() {
    assertEquals("caught", eval("let r = \"ok\"\ntry { throw \"oops\" } catch(e) { r = \"caught\" }\nr"));
  }

  @Test
  void tryCatchRuntimeError() {
    assertEquals("caught", eval("let r = \"ok\"\ntry { 1 / 0 } catch(e) { r = \"caught\" }\nr"));
  }

  // ── Structs ──────────────────────────────────────────────────────────────────

  @Test
  void structDeclarationAndInstantiation() {
    assertEquals("1",
        eval("struct Point { x, y }\nlet p = Point { x: 1, y: 2 }\np.x"));
  }

  @Test
  void structMethod() {
    assertEquals("3",
        eval("struct Point { x, y\nfn sum(self) { return self.x + self.y } }\nlet p = Point { x: 1, y: 2 }\np.sum()"));
  }

  @Test
  void structFieldMutation() {
    assertEquals("10",
        eval("struct Box { val }\nlet b = Box { val: 1 }\nb.val = 10\nb.val"));
  }

  // ── Builtins ─────────────────────────────────────────────────────────────────

  @Test
  void typeBuiltin() {
    assertEquals("int", eval("type(42)"));
    assertEquals("string", eval("type(\"hi\")"));
    assertEquals("bool", eval("type(true)"));
    assertEquals("null", eval("type(null)"));
    assertEquals("array", eval("type([])"));
  }

  @Test
  void lenBuiltin() {
    assertEquals("3", eval("len([1, 2, 3])"));
    assertEquals("5", eval("len(\"hello\")"));
  }

  @Test
  void conversionBuiltins() {
    assertEquals("42", eval("int(\"42\")"));
    assertEquals("42", eval("int(42.9)"));
    assertEquals("3.14", eval("float(\"3.14\")"));
    assertEquals("42", eval("str(42)"));
    assertEquals("true", eval("bool(1)"));
    assertEquals("false", eval("bool(0)"));
  }

  @Test
  void clockReturnsPositiveFloat() {
    assertInstanceOf(Value.FloatVal.class, evaluator.evalExpr(
        new com.aether.parser.ast.Expr.Call(
            new com.aether.parser.ast.Expr.Identifier("clock"),
            List.of())));
  }

  @Test
  void typeError() {
    assertThrows(AetherRuntimeException.TypeError.class, () -> eval("\"a\" - 1"));
  }

  @Test
  void indexOutOfBounds() {
    assertThrows(AetherRuntimeException.IndexOutOfBounds.class, () -> eval("[1, 2][5]"));
  }
}
