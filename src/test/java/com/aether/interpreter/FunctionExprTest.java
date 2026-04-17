package com.aether.interpreter;

import static org.junit.jupiter.api.Assertions.assertEquals;

import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Stmt;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

/** Comprehensive tests for function expressions, closures, and higher-order functions. */
class FunctionExprTest {

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
  void noParams() {
    assertEquals("42", eval("let f = fn() { return 42 }\nf()"));
  }

  @Test
  void withParams() {
    assertEquals("7", eval("let add = fn(a, b) { return a + b }\nadd(3, 4)"));
  }

  @Test
  void inVariable() {
    assertEquals("10", eval("let double = fn(x) { return x * 2 }\ndouble(5)"));
  }

  @Test
  void immediateCallSimple() {
    assertEquals("9", eval("fn(x) { return x * x }(3)"));
  }

  @Test
  void storedNotCalled() {
    // Storing a function in a variable should not call it
    assertEquals("function",
        eval("let f = fn() { return 99 }\ntype(f)"));
  }

  @Test
  void arrayOfFunctionsNotCalled() {
    assertEquals("array",
        eval("let fns = [fn(x) { return x + 1 }, fn(x) { return x * 2 }]\ntype(fns)"));
  }

  @Test
  void regularFunctionDeclaration() {
    assertEquals("8", eval("fn cube(x) { return x * x * x }\ncube(2)"));
  }

  @Test
  void closureSimple() {
    assertEquals("15",
        eval("fn make_adder(n) { return fn(x) { return n + x } }\n"
            + "let add5 = make_adder(5)\n"
            + "add5(10)"));
  }

  @Test
  void closureMultipleVariables() {
    assertEquals("7",
        eval("fn make_adder(a, b) { return fn(x) { return x + a + b } }\n"
            + "let f = make_adder(2, 3)\n"
            + "f(2)"));
  }

  @Test
  void returnFunctionExpression() {
    assertEquals("function",
        eval("fn get_fn() { return fn(x) { return x + 1 } }\n"
            + "let f = get_fn()\n"
            + "type(f)"));
  }

  @Test
  void withMap() {
    // Inline higher-order function mimicking map
    assertEquals("[2, 4, 6]",
        eval("fn my_map(arr, f) {\n"
            + "  let result = []\n"
            + "  for x in arr { result.push(f(x)) }\n"
            + "  return result\n"
            + "}\n"
            + "my_map([1, 2, 3], fn(x) { return x * 2 })"));
  }

  @Test
  void withFilter() {
    assertEquals("[2, 4]",
        eval("fn my_filter(arr, f) {\n"
            + "  let result = []\n"
            + "  for x in arr { if (f(x)) { result.push(x) } }\n"
            + "  return result\n"
            + "}\n"
            + "my_filter([1, 2, 3, 4], fn(x) { return x % 2 == 0 })"));
  }

  @Test
  void withReduce() {
    assertEquals("10",
        eval("fn my_reduce(arr, f, acc) {\n"
            + "  for x in arr { acc = f(acc, x) }\n"
            + "  return acc\n"
            + "}\n"
            + "my_reduce([1, 2, 3, 4], fn(a, b) { return a + b }, 0)"));
  }
}
