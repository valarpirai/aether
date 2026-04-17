package com.aether.interpreter;

import static org.junit.jupiter.api.Assertions.assertEquals;

import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Stmt;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

/** Tests for string interpolation: "text ${expr} text". */
class StringInterpTest {

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
  void variable() {
    assertEquals("Hello Aether", eval("let name = \"Aether\"\n\"Hello ${name}\""));
  }

  @Test
  void number() {
    assertEquals("Value: 42", eval("let n = 42\n\"Value: ${n}\""));
  }

  @Test
  void arithmeticExpression() {
    assertEquals("Result: 6", eval("\"Result: ${1 + 2 + 3}\""));
  }

  @Test
  void multiple() {
    assertEquals("3 + 4 = 7", eval("let a = 3\nlet b = 4\n\"${a} + ${b} = ${a + b}\""));
  }

  @Test
  void noPlaceholder() {
    assertEquals("plain string", eval("\"plain string\""));
  }

  @Test
  void boolValue() {
    assertEquals("flag is true", eval("let flag = true\n\"flag is ${flag}\""));
  }

  @Test
  void nestedCall() {
    assertEquals("double(5) = 10",
        eval("fn double(x) { return x * 2 }\n\"double(5) = ${double(5)}\""));
  }

  @Test
  void atStart() {
    assertEquals("99 bottles", eval("let x = 99\n\"${x} bottles\""));
  }

  @Test
  void atEnd() {
    assertEquals("count: 3", eval("let x = 3\n\"count: ${x}\""));
  }

  @Test
  void onlyInterpolation() {
    assertEquals("42", eval("let x = 42\n\"${x}\""));
  }

  @Test
  void nullValue() {
    assertEquals("got null", eval("let x = null\n\"got ${x}\""));
  }

  @Test
  void arrayValue() {
    assertEquals("arr: [1, 2, 3]", eval("let a = [1, 2, 3]\n\"arr: ${a}\""));
  }

  @Test
  void stringValue() {
    assertEquals("say hi", eval("let s = \"hi\"\n\"say ${s}\""));
  }

  @Test
  void interpolationInLoop() {
    assertEquals("x=0 x=1 x=2",
        eval("let parts = []\n"
            + "let i = 0\n"
            + "while (i < 3) { parts.push(\"x=${i}\")\ni += 1 }\n"
            + "parts[0] + \" \" + parts[1] + \" \" + parts[2]"));
  }
}
