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

/** Comprehensive tests for built-in string methods: upper, lower, trim, split, length. */
class StringMethodsTest {

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
  void upper() {
    assertEquals("HELLO", eval("\"hello\".upper()"));
  }

  @Test
  void upperAlreadyUpper() {
    assertEquals("HELLO", eval("\"HELLO\".upper()"));
  }

  @Test
  void lower() {
    assertEquals("hello", eval("\"HELLO\".lower()"));
  }

  @Test
  void lowerAlreadyLower() {
    assertEquals("hello", eval("\"hello\".lower()"));
  }

  @Test
  void trimLeading() {
    assertEquals("hello", eval("\"   hello\".trim()"));
  }

  @Test
  void trimTrailing() {
    assertEquals("hello", eval("\"hello   \".trim()"));
  }

  @Test
  void trimBoth() {
    assertEquals("hello", eval("\"  hello  \".trim()"));
  }

  @Test
  void splitByComma() {
    assertEquals("[\"a\", \"b\", \"c\"]", eval("\"a,b,c\".split(\",\")"));
  }

  @Test
  void splitBySpace() {
    assertEquals("[\"hello\", \"world\"]", eval("\"hello world\".split(\" \")"));
  }

  @Test
  void length() {
    assertEquals("5", eval("\"hello\".length"));
  }

  @Test
  void lengthEmpty() {
    assertEquals("0", eval("\"\".length"));
  }

  @Test
  void lengthOnVariable() {
    assertEquals("3", eval("let s = \"abc\"\ns.length"));
  }

  @Test
  void methodChainUpperTrim() {
    assertEquals("HELLO", eval("\"  hello  \".trim().upper()"));
  }

  @Test
  void upperOnLiteralAfterConcat() {
    assertEquals("HELLO WORLD", eval("(\"hello\" + \" world\").upper()"));
  }

  @Test
  void splitSingleElement() {
    assertEquals("[\"hello\"]", eval("\"hello\".split(\",\")"));
  }

  @Test
  void undefinedMethodThrows() {
    assertThrows(AetherRuntimeException.class, () -> eval("\"hello\".nonexistent()"));
  }
}
