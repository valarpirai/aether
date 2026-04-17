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

/**
 * Comprehensive tests for string indexing: s[i].
 * Note: Java uses UTF-16 char units; BMP characters (like CJK) index correctly,
 * but supplementary plane characters (emoji) are split into surrogate pairs.
 */
class StringIndexingTest {

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
  void firstChar() {
    assertEquals("h", eval("let s = \"hello\"\ns[0]"));
  }

  @Test
  void middleChar() {
    assertEquals("l", eval("let s = \"hello\"\ns[2]"));
  }

  @Test
  void lastChar() {
    assertEquals("o", eval("let s = \"hello\"\ns[4]"));
  }

  @Test
  void literal() {
    assertEquals("o", eval("\"world\"[1]"));
  }

  @Test
  void utf8BmpChars() {
    // BMP CJK characters index correctly in Java (single char unit each)
    assertEquals("\u4e16", eval("let s = \"\u4e16\u754c\"\ns[0]"));
  }

  @Test
  void negativeIndex() {
    // Java wraps negative indices (s[-1] returns last char)
    assertEquals("o", eval("let s = \"hello\"\ns[-1]"));
  }

  @Test
  void outOfBoundsThrows() {
    assertThrows(AetherRuntimeException.IndexOutOfBounds.class,
        () -> eval("let s = \"hello\"\ns[10]"));
  }

  @Test
  void multipleAccess() {
    assertEquals("abc", eval("let s = \"abc\"\ns[0] + s[1] + s[2]"));
  }

  @Test
  void comparison() {
    assertEquals("true", eval("let s = \"hello\"\ns[0] == \"h\""));
  }

  @Test
  void concatenation() {
    assertEquals("wo",
        eval("let s = \"world\"\nlet first = s[0]\nlet last = s[4]\nfirst + s[1]"));
  }

  @Test
  void emptyStringThrows() {
    assertThrows(AetherRuntimeException.IndexOutOfBounds.class,
        () -> eval("let s = \"\"\ns[0]"));
  }

  @Test
  void singleChar() {
    assertEquals("x", eval("let s = \"x\"\ns[0]"));
  }

  @Test
  void withVariables() {
    assertEquals("s", eval("let s = \"testing\"\nlet idx = 2\ns[idx]"));
  }

  @Test
  void withExpression() {
    assertEquals("a", eval("let s = \"example\"\ns[1 + 1]"));
  }

  @Test
  void multipleIndexes() {
    assertEquals("code", eval("let s = \"code\"\ns[0] + s[1] + s[2] + s[3]"));
  }
}
