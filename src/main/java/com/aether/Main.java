package com.aether;

import com.aether.interpreter.Evaluator;
import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
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
      // TODO: call main() function if defined (matching Rust run_file behaviour)
    } catch (Exception e) {
      System.err.println("Runtime error: " + e.getMessage());
      System.exit(1);
    }
  }
}
