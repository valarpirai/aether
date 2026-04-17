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

/** Comprehensive tests for struct declaration, instantiation, fields, and methods. */
class StructTest {

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
  void declareAndInstantiate() {
    assertEquals("1",
        eval("struct Point { x, y }\nlet p = Point { x: 1, y: 2 }\np.x"));
  }

  @Test
  void fieldY() {
    assertEquals("7",
        eval("struct Point { x, y }\nlet p = Point { x: 3, y: 7 }\np.y"));
  }

  @Test
  void display() {
    assertEquals("Point { x: 1, y: 2 }",
        eval("struct Point { x, y }\nlet p = Point { x: 1, y: 2 }\nstr(p)"));
  }

  @Test
  void typeName() {
    assertEquals("Point",
        eval("struct Point { x, y }\nlet p = Point { x: 0, y: 0 }\ntype(p)"));
  }

  @Test
  void fieldAssignment() {
    assertEquals("99",
        eval("struct Point { x, y }\nlet p = Point { x: 1, y: 2 }\np.x = 99\np.x"));
  }

  @Test
  void fieldMutationIndependent() {
    // Mutating x should not affect y
    assertEquals("2",
        eval("struct Point { x, y }\nlet p = Point { x: 1, y: 2 }\np.x = 10\np.y"));
  }

  @Test
  void unsetFieldIsNull() {
    assertEquals("null",
        eval("struct Point { x, y }\nlet p = Point { x: 5 }\np.y"));
  }

  @Test
  void methodCall() {
    assertEquals("1",
        eval("struct Counter { count\nfn increment(self) { self.count = self.count + 1 } }\n"
            + "let c = Counter { count: 0 }\nc.increment()\nc.count"));
  }

  @Test
  void methodWithReturn() {
    assertEquals("12",
        eval("struct Rectangle { width, height\nfn area(self) { return self.width * self.height } }\n"
            + "let r = Rectangle { width: 3, height: 4 }\nr.area()"));
  }

  @Test
  void methodWithParam() {
    assertEquals("15",
        eval("struct Adder { base\nfn add(self, n) { return self.base + n } }\n"
            + "let a = Adder { base: 10 }\na.add(5)"));
  }

  @Test
  void multipleInstancesIndependent() {
    // Mutating p1 should not affect p2
    assertEquals("10",
        eval("struct Point { x, y }\n"
            + "let p1 = Point { x: 1, y: 2 }\n"
            + "let p2 = Point { x: 10, y: 20 }\n"
            + "p1.x = 99\n"
            + "p2.x"));
  }

  @Test
  void nested() {
    assertEquals("5",
        eval("struct Point { x, y }\n"
            + "struct Line { start, end }\n"
            + "let p1 = Point { x: 0, y: 0 }\n"
            + "let p2 = Point { x: 5, y: 5 }\n"
            + "let line = Line { start: p1, end: p2 }\n"
            + "line.end.x"));
  }

  @Test
  void undefinedFieldErrors() {
    assertThrows(AetherRuntimeException.class,
        () -> eval("struct Point { x, y }\nlet p = Point { x: 1, y: 2 }\np.z"));
  }

  @Test
  void undefinedMethodErrors() {
    assertThrows(AetherRuntimeException.class,
        () -> eval("struct Point { x, y }\nlet p = Point { x: 1, y: 2 }\np.move()"));
  }
}
