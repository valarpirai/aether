package com.aether.interpreter;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import com.aether.exception.AetherRuntimeException;
import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Stmt;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;

/** Tests for array/string slice syntax and the spread operator. */
class SliceAndSpreadTest {

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

  // ── Array slices ──────────────────────────────────────────────────────────────

  @Nested
  class ArraySlice {

    @Test
    void startAndEnd() {
      assertEquals("[2, 3]", eval("[1, 2, 3, 4, 5][1:3]"));
    }

    @Test
    void fromStart() {
      assertEquals("[1, 2, 3]", eval("[1, 2, 3, 4, 5][:3]"));
    }

    @Test
    void toEnd() {
      assertEquals("[3, 4, 5]", eval("[1, 2, 3, 4, 5][2:]"));
    }

    @Test
    void fullCopy() {
      assertEquals("[1, 2, 3]", eval("[1, 2, 3][:]"));
    }

    @Test
    void emptyResult() {
      assertEquals("[]", eval("[1, 2, 3][2:1]"));
    }

    @Test
    void outOfBoundsClamped() {
      assertEquals("[1, 2, 3]", eval("[1, 2, 3][0:100]"));
    }

    @Test
    void viaVariable() {
      assertEquals("[20, 30, 40]", eval("let a = [10, 20, 30, 40, 50]\na[1:4]"));
    }

    @Test
    void singleElement() {
      assertEquals("[3]", eval("[1, 2, 3, 4][2:3]"));
    }

    @Test
    void sliceOfSlice() {
      assertEquals("[3, 4]", eval("let a = [1, 2, 3, 4, 5]\na[1:4][1:3]"));
    }
  }

  // ── String slices ─────────────────────────────────────────────────────────────

  @Nested
  class StringSlice {

    @Test
    void startAndEnd() {
      assertEquals("el", eval("\"hello\"[1:3]"));
    }

    @Test
    void fromStart() {
      assertEquals("hel", eval("\"hello\"[:3]"));
    }

    @Test
    void toEnd() {
      assertEquals("llo", eval("\"hello\"[2:]"));
    }

    @Test
    void fullCopy() {
      assertEquals("hello", eval("\"hello\"[:]"));
    }

    @Test
    void singleChar() {
      assertEquals("l", eval("\"hello\"[2:3]"));
    }

    @Test
    void outOfBoundsClamped() {
      assertEquals("hello", eval("\"hello\"[0:100]"));
    }

    @Test
    void emptyResult() {
      assertEquals("", eval("\"hello\"[3:1]"));
    }
  }

  // ── String indexing ───────────────────────────────────────────────────────────

  @Nested
  class StringIndex {

    @Test
    void firstChar() {
      assertEquals("h", eval("\"hello\"[0]"));
    }

    @Test
    void lastChar() {
      assertEquals("o", eval("\"hello\"[4]"));
    }

    @Test
    void middleChar() {
      assertEquals("l", eval("\"hello\"[2]"));
    }

    @Test
    void singleCharString() {
      assertEquals("x", eval("\"x\"[0]"));
    }

    @Test
    void indexOutOfBoundsThrows() {
      assertThrows(AetherRuntimeException.IndexOutOfBounds.class, () -> eval("\"hi\"[5]"));
    }

    @Test
    void indexInLoop() {
      assertEquals("hello", eval(
          "let s = \"hello\"\nlet r = \"\"\nlet i = 0\n"
          + "while (i < len(s)) { r = r + s[i]\ni += 1 }\nr"));
    }

    @Test
    void indexUsedInInterpolation() {
      assertEquals("first: h", eval("let s = \"hello\"\n\"first: ${s[0]}\""));
    }
  }

  // ── Spread operator ───────────────────────────────────────────────────────────

  @Nested
  class Spread {

    @Test
    void twoArrays() {
      assertEquals("[1, 2, 3, 4, 5, 6]",
          eval("let a = [1, 2, 3]\nlet b = [4, 5, 6]\n[...a, ...b]"));
    }

    @Test
    void leadingElements() {
      assertEquals("[1, 2, 3, 4]", eval("let rest = [2, 3, 4]\n[1, ...rest]"));
    }

    @Test
    void trailingElements() {
      assertEquals("[1, 2, 3, 4]", eval("let head = [1, 2, 3]\n[...head, 4]"));
    }

    @Test
    void mixedPositions() {
      assertEquals("[1, 2, 3, 4, 5]", eval("let mid = [2, 3]\n[1, ...mid, 4, 5]"));
    }

    @Test
    void emptyArray() {
      assertEquals("[1, 2]", eval("let empty = []\n[1, ...empty, 2]"));
    }

    @Test
    void literalArray() {
      assertEquals("[1, 2, 3]", eval("[...[1, 2, 3]]"));
    }

    @Test
    void threeArrays() {
      assertEquals("[1, 2, 3, 4, 5, 6]",
          eval("let a = [1, 2]\nlet b = [3, 4]\nlet c = [5, 6]\n[...a, ...b, ...c]"));
    }

    @Test
    void doesNotMutateOriginal() {
      assertEquals("[1, 2, 3]", eval("let a = [1, 2, 3]\nlet b = [...a, 4]\na"));
    }

    @Test
    void spreadNonArrayThrows() {
      assertThrows(AetherRuntimeException.class, () -> eval("[...42]"));
    }
  }
}
