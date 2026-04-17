package com.aether.parser;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.aether.exception.ParseException;
import com.aether.lexer.Scanner;
import com.aether.parser.ast.BinaryOp;
import com.aether.parser.ast.Expr;
import com.aether.parser.ast.Stmt;
import com.aether.parser.ast.UnaryOp;
import java.util.List;
import org.junit.jupiter.api.Test;

/** Unit tests for {@link Parser}. */
class ParserTest {

  private List<Stmt> parse(String source) {
    return new Parser(new Scanner(source).scanTokens()).parse();
  }

  private Expr parseExpr(String source) {
    return ((Stmt.ExprStmt) parse(source).get(0)).expr();
  }

  @Test
  void emptySourceProducesEmptyList() {
    assertTrue(parse("").isEmpty());
  }

  @Test
  void integerLiteral() {
    assertEquals(42L, ((Expr.IntLiteral) parseExpr("42")).value());
  }

  @Test
  void floatLiteral() {
    assertEquals(3.14, ((Expr.FloatLiteral) parseExpr("3.14")).value(), 0.001);
  }

  @Test
  void stringLiteral() {
    assertEquals("hello", ((Expr.StringLiteral) parseExpr("\"hello\"")).value());
  }

  @Test
  void boolLiterals() {
    assertEquals(true, ((Expr.BoolLiteral) parseExpr("true")).value());
    assertEquals(false, ((Expr.BoolLiteral) parseExpr("false")).value());
  }

  @Test
  void nullLiteral() {
    assertInstanceOf(Expr.NullLiteral.class, parseExpr("null"));
  }

  @Test
  void identifier() {
    assertEquals("x", ((Expr.Identifier) parseExpr("x")).name());
  }

  @Test
  void additionPrecedence() {
    Expr.Binary bin = (Expr.Binary) parseExpr("1 + 2 * 3");
    assertEquals(BinaryOp.ADD, bin.op());
    assertEquals(BinaryOp.MULTIPLY, ((Expr.Binary) bin.right()).op());
  }

  @Test
  void unaryNegate() {
    Expr.Unary u = (Expr.Unary) parseExpr("-5");
    assertEquals(UnaryOp.NEGATE, u.op());
    assertEquals(5L, ((Expr.IntLiteral) u.operand()).value());
  }

  @Test
  void functionCall() {
    Expr.Call call = (Expr.Call) parseExpr("f(1, 2)");
    assertEquals("f", ((Expr.Identifier) call.callee()).name());
    assertEquals(2, call.args().size());
  }

  @Test
  void arrayLiteral() {
    assertEquals(3, ((Expr.Array) parseExpr("[1, 2, 3]")).elements().size());
  }

  @Test
  void dictLiteral() {
    // Dict literals must appear in expression context, not as standalone statements
    Stmt.Let let = (Stmt.Let) parse("let d = {\"a\": 1}").get(0);
    Expr.Dict dict = (Expr.Dict) let.initializer();
    assertEquals("a", ((Expr.StringLiteral) dict.entries().get(0).key()).value());
  }

  @Test
  void memberAccess() {
    assertEquals("field", ((Expr.Member) parseExpr("obj.field")).member());
  }

  @Test
  void indexAccess() {
    assertEquals(0L, ((Expr.IntLiteral) ((Expr.Index) parseExpr("arr[0]")).index()).value());
  }

  @Test
  void sliceAccess() {
    Expr.Slice slice = (Expr.Slice) parseExpr("arr[1:3]");
    assertEquals(1L, ((Expr.IntLiteral) slice.start()).value());
    assertEquals(3L, ((Expr.IntLiteral) slice.end()).value());
  }

  @Test
  void sliceNoEnd() {
    assertNull(((Expr.Slice) parseExpr("arr[1:]")).end());
  }

  @Test
  void spreadInArray() {
    Expr.Array arr = (Expr.Array) parseExpr("[...xs, 4]");
    assertInstanceOf(Expr.Spread.class, arr.elements().get(0));
  }

  @Test
  void letDeclaration() {
    Stmt.Let let = (Stmt.Let) parse("let x = 5").get(0);
    assertEquals("x", let.name());
    assertEquals(5L, ((Expr.IntLiteral) let.initializer()).value());
  }

  @Test
  void assignStatement() {
    assertEquals("x", ((Expr.Identifier) ((Stmt.Assign) parse("x = 10").get(0)).target()).name());
  }

  @Test
  void compoundAssign() {
    assertEquals(BinaryOp.ADD, ((Stmt.CompoundAssign) parse("x += 3").get(0)).op());
  }

  @Test
  void ifStatement() {
    assertNull(((Stmt.If) parse("if (x) { 1 }").get(0)).elseBranch());
  }

  @Test
  void ifElseStatement() {
    assertInstanceOf(Stmt.Block.class, ((Stmt.If) parse("if (x) { 1 } else { 2 }").get(0)).elseBranch());
  }

  @Test
  void whileLoop() {
    Stmt.While w = (Stmt.While) parse("while (x > 0) { x = x - 1 }").get(0);
    assertInstanceOf(Expr.Binary.class, w.condition());
  }

  @Test
  void forLoop() {
    Stmt.For f = (Stmt.For) parse("for i in items { 1 }").get(0);
    assertEquals("i", f.varName());
    assertEquals("items", ((Expr.Identifier) f.iterable()).name());
  }

  @Test
  void returnVoid() {
    Stmt.Function fn = (Stmt.Function) parse("fn f() { return }").get(0);
    assertNull(((Stmt.Return) ((Stmt.Block) fn.body()).statements().get(0)).value());
  }

  @Test
  void functionDeclaration() {
    Stmt.Function fn = (Stmt.Function) parse("fn add(a, b) { a + b }").get(0);
    assertEquals("add", fn.name());
    assertEquals(List.of("a", "b"), fn.params());
  }

  @Test
  void functionExpression() {
    assertEquals(List.of("x"), ((Expr.FunctionExpr) parseExpr("fn(x) { x + 1 }")).params());
  }

  @Test
  void stringInterpolation() {
    Expr.StringInterp si = (Expr.StringInterp) parseExpr("\"hello ${name}!\"");
    assertEquals(3, si.parts().size());
    assertEquals("hello ", ((Expr.StringLiteral) si.parts().get(0)).value());
    assertEquals("name", ((Expr.Identifier) si.parts().get(1)).name());
    assertEquals("!", ((Expr.StringLiteral) si.parts().get(2)).value());
  }

  @Test
  void tryCatch() {
    assertEquals("e", ((Stmt.TryCatch) parse("try { 1 } catch(e) { 2 }").get(0)).errorVar());
  }

  @Test
  void throwStatement() {
    assertEquals("oops", ((Expr.StringLiteral) ((Stmt.Throw) parse("throw \"oops\"").get(0)).value()).value());
  }

  @Test
  void importStatement() {
    assertEquals("math", ((Stmt.Import) parse("import math").get(0)).moduleName());
  }

  @Test
  void importAs() {
    Stmt.ImportAs ia = (Stmt.ImportAs) parse("import math as m").get(0);
    assertEquals("math", ia.moduleName());
    assertEquals("m", ia.alias());
  }

  @Test
  void fromImport() {
    Stmt.FromImport fi = (Stmt.FromImport) parse("from math import sin, cos").get(0);
    assertEquals(List.of("sin", "cos"), fi.items());
  }

  @Test
  void structDeclaration() {
    Stmt.StructDecl sd =
        (Stmt.StructDecl) parse("struct Point { x, y fn dist(self) { 0 } }").get(0);
    assertEquals("Point", sd.name());
    assertEquals(List.of("x", "y"), sd.fields());
    assertEquals(1, sd.methods().size());
  }

  @Test
  void structInit() {
    Expr.StructInit si = (Expr.StructInit) parseExpr("Point { x: 1, y: 2 }");
    assertEquals("Point", si.name());
    assertEquals(2, si.fields().size());
    assertEquals("x", si.fields().get(0).name());
  }

  @Test
  void unexpectedTokenThrows() {
    assertThrows(ParseException.class, () -> parse(")"));
  }
}
