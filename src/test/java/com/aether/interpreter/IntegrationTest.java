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

/** Integration tests: complete programs exercising all major language features together. */
class IntegrationTest {

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
  void simpleArithmetic() {
    assertEquals("15", eval("let x = 5\nx += 10\nx"));
  }

  @Test
  void fibonacci() {
    assertEquals("8",
        eval("fn fib(n) {\n"
            + "  if (n <= 1) { return n }\n"
            + "  return fib(n - 1) + fib(n - 2)\n"
            + "}\n"
            + "fib(6)"));
  }

  @Test
  void nestedFunctions() {
    assertEquals("42",
        eval("fn outer() {\n"
            + "  fn inner() { return 42 }\n"
            + "  return inner()\n"
            + "}\n"
            + "outer()"));
  }

  @Test
  void arityMismatchThrows() {
    assertThrows(AetherRuntimeException.ArityMismatch.class,
        () -> eval("fn add(a, b) { return a + b }\nadd(1, 2, 3)"));
  }

  @Test
  void builtinPrint() {
    // print should not throw and returns null
    assertEquals("null", eval("print(\"Hello\")\nprint(42)"));
  }

  @Test
  void builtinPrintln() {
    assertEquals("null", eval("println(\"Hello World\")\nprintln(1, 2, 3)"));
  }

  @Test
  void builtinPrintMultipleArgs() {
    assertEquals("null", eval("println(\"Sum:\", 1 + 2 + 3)\nprintln(\"Values:\", 10, 20, 30)"));
  }

  @Test
  void builtinLen() {
    assertEquals("5", eval("len(\"hello\")"));
    assertEquals("4", eval("len([1, 2, 3, 4])"));
  }

  @Test
  void builtinType() {
    assertEquals("int", eval("type(42)"));
    assertEquals("string", eval("type(\"hello\")"));
    assertEquals("bool", eval("type(true)"));
    assertEquals("null", eval("type(null)"));
    assertEquals("array", eval("type([])"));
    assertEquals("function", eval("fn f() { } type(f)"));
  }

  @Test
  void builtinInt() {
    assertEquals("42", eval("int(\"42\")"));
    assertEquals("3", eval("int(3.9)"));
    assertEquals("1", eval("int(true)"));
  }

  @Test
  void builtinFloat() {
    assertEquals("3.14", eval("float(\"3.14\")"));
    assertEquals("42.0", eval("float(42)"));
  }

  @Test
  void builtinStr() {
    assertEquals("42", eval("str(42)"));
    assertEquals("true", eval("str(true)"));
    assertEquals("3.14", eval("str(3.14)"));
  }

  @Test
  void builtinBool() {
    assertEquals("true", eval("bool(1)"));
    assertEquals("false", eval("bool(0)"));
    assertEquals("true", eval("bool(\"nonempty\")"));
    assertEquals("false", eval("bool(\"\")"));
  }

  @Test
  void comparisonOperators() {
    assertEquals("true", eval("1 < 2"));
    assertEquals("false", eval("2 < 1"));
    assertEquals("true", eval("2 <= 2"));
    assertEquals("true", eval("3 > 2"));
    assertEquals("true", eval("3 >= 3"));
    assertEquals("true", eval("5 == 5"));
    assertEquals("false", eval("5 != 5"));
  }

  @Test
  void logicalOperators() {
    assertEquals("true", eval("true && true"));
    assertEquals("false", eval("true && false"));
    assertEquals("true", eval("false || true"));
    assertEquals("false", eval("false || false"));
    assertEquals("true", eval("!false"));
    assertEquals("false", eval("!true"));
  }

  @Test
  void variableReassignment() {
    assertEquals("20", eval("let x = 10\nx = 20\nx"));
  }

  @Test
  void arrayCreation() {
    assertEquals("[1, 2, 3]", eval("[1, 2, 3]"));
  }

  @Test
  void arrayIndexing() {
    assertEquals("b", eval("[\"a\", \"b\", \"c\"][1]"));
  }

  @Test
  void functionWithLocalVariables() {
    assertEquals("15",
        eval("fn sum_to(n) {\n"
            + "  let total = 0\n"
            + "  let i = 1\n"
            + "  while (i <= n) { total += i\ni += 1 }\n"
            + "  return total\n"
            + "}\n"
            + "sum_to(5)"));
  }

  @Test
  void closure() {
    assertEquals("11",
        eval("fn make_adder(n) { return fn(x) { return n + x } }\n"
            + "let add10 = make_adder(10)\n"
            + "add10(1)"));
  }

  @Test
  void stringConcatenation() {
    assertEquals("hello world", eval("\"hello\" + \" \" + \"world\""));
  }

  @Test
  void errorUndefinedVariable() {
    assertThrows(AetherRuntimeException.UndefinedVariable.class, () -> eval("undefined_xyz"));
  }

  @Test
  void errorDivisionByZero() {
    assertThrows(AetherRuntimeException.DivisionByZero.class, () -> eval("10 / 0"));
  }

  @Test
  void errorTypeMismatch() {
    assertThrows(AetherRuntimeException.TypeError.class, () -> eval("\"a\" - 1"));
  }
}
