package com.aether.interpreter;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;

import com.aether.exception.AetherRuntimeException;
import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Expr;
import com.aether.parser.ast.Stmt;
import java.util.List;
import org.junit.jupiter.api.Test;

class MainFunctionTest {

  private Evaluator runProgram(String source) {
    Evaluator evaluator = Evaluator.withoutStdlib();
    List<Stmt> stmts = new Parser(new Scanner(source).scanTokens()).parse();
    evaluator.execute(stmts);
    return evaluator;
  }

  @Test
  void fileMissingMainThrows() {
    Evaluator evaluator = runProgram("let x = 42");
    assertThrows(AetherRuntimeException.UndefinedVariable.class,
        () -> evaluator.environment().get("main"));
  }

  @Test
  void fileWithMainSucceeds() {
    Evaluator evaluator = runProgram("fn main() { let x = 1 }");
    assertDoesNotThrow(() -> {
      evaluator.environment().get("main");
      evaluator.evalExpr(new Expr.Call(new Expr.Identifier("main"), List.of()));
    });
  }
}
