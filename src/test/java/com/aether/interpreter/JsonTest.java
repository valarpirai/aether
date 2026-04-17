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

/** Comprehensive tests for json_parse() and json_stringify() builtins. */
class JsonTest {

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

  // ── json_parse ────────────────────────────────────────────────────────────────

  @Test
  void parseNull() {
    assertEquals("null", eval("json_parse(\"null\")"));
  }

  @Test
  void parseBoolTrue() {
    assertEquals("true", eval("json_parse(\"true\")"));
  }

  @Test
  void parseBoolFalse() {
    assertEquals("false", eval("json_parse(\"false\")"));
  }

  @Test
  void parseInteger() {
    assertEquals("42", eval("json_parse(\"42\")"));
  }

  @Test
  void parseNegativeInteger() {
    assertEquals("-7", eval("json_parse(\"-7\")"));
  }

  @Test
  void parseFloat() {
    assertEquals("3.14", eval("json_parse(\"3.14\")"));
  }

  @Test
  void parseString() {
    assertEquals("hello", eval("json_parse(\"\\\"hello\\\"\")"));
  }

  @Test
  void parseArray() {
    assertEquals("[1, 2, 3]", eval("json_parse(\"[1,2,3]\")"));
  }

  @Test
  void parseNestedArray() {
    assertEquals("[[1, 2], [3, 4]]", eval("json_parse(\"[[1,2],[3,4]]\")"));
  }

  @Test
  void parseObject() {
    assertEquals("Alice", eval("let d = json_parse(\"{\\\"name\\\":\\\"Alice\\\",\\\"age\\\":30}\")\nd[\"name\"]"));
  }

  @Test
  void parseObjectIntValue() {
    assertEquals("42", eval("let d = json_parse(\"{\\\"x\\\":42}\")\nd[\"x\"]"));
  }

  @Test
  void parseWhitespace() {
    assertEquals("42", eval("json_parse(\"  42  \")"));
  }

  @Test
  void parseInvalidErrors() {
    assertThrows(AetherRuntimeException.class, () -> eval("json_parse(\"not json\")"));
  }

  // ── json_stringify ────────────────────────────────────────────────────────────

  @Test
  void stringifyNull() {
    assertEquals("null", eval("json_stringify(null)"));
  }

  @Test
  void stringifyBoolTrue() {
    assertEquals("true", eval("json_stringify(true)"));
  }

  @Test
  void stringifyBoolFalse() {
    assertEquals("false", eval("json_stringify(false)"));
  }

  @Test
  void stringifyInteger() {
    assertEquals("42", eval("json_stringify(42)"));
  }

  @Test
  void stringifyFloat() {
    assertEquals("3.14", eval("json_stringify(3.14)"));
  }

  @Test
  void stringifyString() {
    assertEquals("\"hello\"", eval("json_stringify(\"hello\")"));
  }

  @Test
  void stringifyStringWithQuotes() {
    assertEquals("\"say \\\"hi\\\"\"", eval("json_stringify(\"say \\\"hi\\\"\")"));
  }

  @Test
  void stringifyArray() {
    assertEquals("[1,2,3]", eval("json_stringify([1, 2, 3])"));
  }

  @Test
  void stringifyNestedArray() {
    assertEquals("[[1,2],[3,4]]", eval("json_stringify([[1, 2], [3, 4]])"));
  }

  @Test
  void stringifyFunctionErrors() {
    assertThrows(AetherRuntimeException.class,
        () -> eval("fn f() { return 1 }\njson_stringify(f)"));
  }

  // ── round-trip ────────────────────────────────────────────────────────────────

  @Test
  void roundTripArray() {
    assertEquals("[1, 2, 3]",
        eval("let arr = [1, 2, 3]\nlet s = json_stringify(arr)\njson_parse(s)"));
  }

  @Test
  void roundTripString() {
    assertEquals("hello world",
        eval("let s = json_stringify(\"hello world\")\njson_parse(s)"));
  }
}
