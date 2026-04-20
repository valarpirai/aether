package com.aether;

import com.aether.exception.AetherRuntimeException;
import com.aether.interpreter.Evaluator;
import com.aether.interpreter.Value;
import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Expr;
import com.aether.parser.ast.Stmt;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

/**
 * Entry point for the Aether interpreter.
 *
 * <ul>
 *   <li>No arguments: start the interactive REPL
 *   <li>One argument: execute the given {@code .ae} file
 * </ul>
 */
public final class Main {

  private Main() {}

  /** Main entry point. */
  public static void main(String[] args) {
    if (args.length == 0) {
      new Repl().run();
    } else if (args.length == 1) {
      runFile(Path.of(args[0]));
    } else {
      System.err.println("Usage: aether [file.ae]");
      System.exit(1);
    }
  }

  private static void runFile(Path path) {
    String source;
    try {
      source = Files.readString(path);
    } catch (IOException e) {
      System.err.println("Error reading file '" + path + "': " + e.getMessage());
      System.exit(1);
      return;
    }

    try {
      Scanner scanner = new Scanner(source);
      List<Stmt> statements = new Parser(scanner.scanTokens()).parse();
      Evaluator evaluator = Evaluator.withStdlib();
      evaluator.execute(statements);
      try {
        Value main = evaluator.environment().get("main");
        if (main instanceof Value.AetherFunction || main instanceof Value.Builtin) {
          evaluator.evalExpr(new Expr.Call(new Expr.Identifier("main"), List.of()));
        }
      } catch (AetherRuntimeException.UndefinedVariable e) {
        System.err.println(
            "No main() function defined. Every Aether program must have a main() function.");
        System.exit(1);
      }
    } catch (Exception e) {
      System.err.println("Runtime error: " + e.getMessage());
      System.exit(1);
    }
  }
}
