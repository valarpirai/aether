package com.aether.interpreter;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.aether.exception.AetherRuntimeException;
import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Stmt;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/** Tests for I/O builtins (read_file, write_file) and time builtins (clock, sleep). */
class IoAndTimeTest {

  @TempDir Path tmpDir;

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

  // ── read_file / write_file ────────────────────────────────────────────────────

  @Test
  void writeAndReadRoundTrip() throws IOException {
    Path file = tmpDir.resolve("hello.txt");
    String path = file.toString().replace("\\", "/");
    assertEquals("hello from aether",
        eval("write_file(\"" + path + "\", \"hello from aether\")\nread_file(\"" + path + "\")"));
  }

  @Test
  void writeFileReturnsNull() throws IOException {
    Path file = tmpDir.resolve("out.txt");
    String path = file.toString().replace("\\", "/");
    assertEquals("null", eval("write_file(\"" + path + "\", \"content\")"));
  }

  @Test
  void writeFileCreatesFile() throws IOException {
    Path file = tmpDir.resolve("created.txt");
    String path = file.toString().replace("\\", "/");
    eval("write_file(\"" + path + "\", \"data\")");
    assertTrue(Files.exists(file));
  }

  @Test
  void writeFileCreatesWithContent() throws IOException {
    Path file = tmpDir.resolve("content.txt");
    String path = file.toString().replace("\\", "/");
    eval("write_file(\"" + path + "\", \"line1\nline2\")");
    assertEquals("line1\nline2", Files.readString(file));
  }

  @Test
  void readFileNotFoundThrows() {
    assertThrows(AetherRuntimeException.InvalidOperation.class,
        () -> eval("read_file(\"/tmp/aether_nonexistent_file_xyz_12345.txt\")"));
  }

  @Test
  void readFileErrorCaughtByTryCatch() {
    assertEquals("error caught",
        eval("let r = \"\"\n"
            + "try { r = read_file(\"/tmp/aether_no_such_file.txt\") }\n"
            + "catch(e) { r = \"error caught\" }\nr"));
  }

  @Test
  void overwriteExistingFile() throws IOException {
    Path file = tmpDir.resolve("overwrite.txt");
    String path = file.toString().replace("\\", "/");
    eval("write_file(\"" + path + "\", \"first\")\nwrite_file(\"" + path + "\", \"second\")");
    assertEquals("second", Files.readString(file));
  }

  // ── clock / sleep ─────────────────────────────────────────────────────────────

  @Test
  void clockReturnsFloat() {
    Value result = evaluator.evalExpr(
        new com.aether.parser.ast.Expr.Call(
            new com.aether.parser.ast.Expr.Identifier("clock"),
            List.of()));
    assertInstanceOf(Value.FloatVal.class, result);
  }

  @Test
  void clockIsPositive() {
    Value result = evaluator.evalExpr(
        new com.aether.parser.ast.Expr.Call(
            new com.aether.parser.ast.Expr.Identifier("clock"),
            List.of()));
    assertTrue(((Value.FloatVal) result).value() > 0);
  }

  @Test
  void clockAdvances() {
    Value t1 = evaluator.evalExpr(
        new com.aether.parser.ast.Expr.Call(
            new com.aether.parser.ast.Expr.Identifier("clock"),
            List.of()));
    eval("sleep(0.05)");
    Value t2 = evaluator.evalExpr(
        new com.aether.parser.ast.Expr.Call(
            new com.aether.parser.ast.Expr.Identifier("clock"),
            List.of()));
    assertTrue(((Value.FloatVal) t2).value() >= ((Value.FloatVal) t1).value());
  }

  @Test
  void sleepIntegerReturnsNull() {
    assertEquals("null", eval("sleep(0)"));
  }

  @Test
  void sleepFloatReturnsNull() {
    assertEquals("null", eval("sleep(0.01)"));
  }

  @Test
  void sleepTypeErrorOnString() {
    assertThrows(AetherRuntimeException.TypeError.class, () -> eval("sleep(\"long\")"));
  }
}
