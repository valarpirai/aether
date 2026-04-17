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

/** Tests for array built-in methods: push, pop, sort, concat, contains, and length. */
class ArrayMethodsTest {

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
  void push() {
    assertEquals("[1, 2, 3]",
        eval("let a = [1, 2]\na.push(3)\na"));
  }

  @Test
  void pushReturnsNull() {
    assertEquals("null", eval("let a = []\na.push(1)"));
  }

  @Test
  void pushMultiple() {
    assertEquals("[1, 2, 3]",
        eval("let a = []\na.push(1)\na.push(2)\na.push(3)\na"));
  }

  @Test
  void pop() {
    assertEquals("3", eval("let a = [1, 2, 3]\na.pop()"));
  }

  @Test
  void popModifiesArray() {
    assertEquals("[1, 2]",
        eval("let a = [1, 2, 3]\na.pop()\na"));
  }

  @Test
  void popEmptyArrayReturnsNull() {
    assertEquals("null", eval("let a = []\na.pop()"));
  }

  @Test
  void pushPopCombo() {
    assertEquals("3",
        eval("let a = [1, 2]\na.push(3)\na.pop()"));
  }

  @Test
  void sortNumbers() {
    assertEquals("[1, 2, 3, 4, 5]",
        eval("let a = [3, 1, 4, 2, 5]\na.sort()\na"));
  }

  @Test
  void sortStrings() {
    assertEquals("[\"apple\", \"banana\", \"cherry\"]",
        eval("let a = [\"banana\", \"apple\", \"cherry\"]\na.sort()\na"));
  }

  @Test
  void sortWithComparator() {
    // Descending sort: return true if a > b
    assertEquals("[5, 4, 3, 2, 1]",
        eval("let a = [3, 1, 4, 2, 5]\na.sort(fn(a, b) { return a > b })\na"));
  }

  @Test
  void concat() {
    assertEquals("[1, 2, 3, 4]",
        eval("let a = [1, 2]\nlet b = [3, 4]\na.concat(b)"));
  }

  @Test
  void concatDoesNotMutate() {
    assertEquals("[1, 2]",
        eval("let a = [1, 2]\nlet b = [3, 4]\na.concat(b)\na"));
  }

  @Test
  void containsTrue() {
    assertEquals("true", eval("let a = [1, 2, 3]\na.contains(2)"));
  }

  @Test
  void containsFalse() {
    assertEquals("false", eval("let a = [1, 2, 3]\na.contains(99)"));
  }

  @Test
  void length() {
    assertEquals("4", eval("let a = [10, 20, 30, 40]\na.length"));
  }
}
