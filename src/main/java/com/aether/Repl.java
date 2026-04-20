package com.aether;

import com.aether.interpreter.Builtins;
import com.aether.interpreter.Evaluator;
import com.aether.interpreter.Value;
import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Stmt;
import java.util.List;
import java.util.Map;
import org.jline.reader.EndOfFileException;
import org.jline.reader.LineReader;
import org.jline.reader.LineReaderBuilder;
import org.jline.reader.UserInterruptException;
import org.jline.terminal.Terminal;
import org.jline.terminal.TerminalBuilder;

/**
 * Interactive REPL (Read-Eval-Print Loop) for Aether.
 *
 * <p>Uses JLine 3 for line editing and history (mirrors the Rust rustyline-based REPL).
 */
public final class Repl {

  private static final String PROMPT = ">> ";
  private static final String VERSION = "0.1.0";

  private final Evaluator evaluator = Evaluator.withStdlib();

  /** Start the interactive REPL loop. Exits on Ctrl+D. */
  public void run() {
    System.out.println("Aether " + VERSION + " REPL — type '_help' for commands, Ctrl+D to exit");

    try (Terminal terminal = TerminalBuilder.builder().system(true).build()) {
      LineReader reader =
          LineReaderBuilder.builder().terminal(terminal).variable(LineReader.HISTORY_SIZE, 500)
              .build();

      while (true) {
        String line;
        try {
          line = reader.readLine(PROMPT);
        } catch (UserInterruptException e) {
          continue;
        } catch (EndOfFileException e) {
          System.out.println("\nGoodbye!");
          break;
        }

        if (line == null || line.isBlank()) {
          continue;
        }

        String trimmed = line.trim();
        if (handleCommand(trimmed)) {
          continue;
        }

        evalLine(trimmed);
      }
    } catch (Exception e) {
      System.err.println("REPL error: " + e.getMessage());
    }
  }

  /**
   * Handle built-in REPL commands.
   *
   * @return true if the line was a command and should not be evaluated
   */
  private boolean handleCommand(String line) {
    return switch (line) {
      case "_help" -> {
        System.out.println("Commands:");
        System.out.println("  _help  — show this message");
        System.out.println("  _env   — list all defined variables");
        System.out.println("  _exit  — exit the REPL");
        System.out.println("  Ctrl+D — exit");
        yield true;
      }
      case "_env" -> {
        Map<String, Value> bindings = evaluator.environment().localBindings();
        if (bindings.isEmpty()) {
          System.out.println("(no variables defined)");
        } else {
          bindings.forEach((name, val) ->
              System.out.println(name + " = " + Builtins.display(val)));
        }
        yield true;
      }
      case "_exit" -> {
        System.out.println("Goodbye!");
        System.exit(0);
        yield true;
      }
      default -> false;
    };
  }

  private void evalLine(String source) {
    try {
      List<Stmt> stmts = new Parser(new Scanner(source).scanTokens()).parse();
      if (stmts.isEmpty()) {
        return;
      }

      // Execute all statements but the last
      for (int i = 0; i < stmts.size() - 1; i++) {
        evaluator.execStmt(stmts.get(i));
      }

      // If the last statement is an expression, print its value
      Stmt last = stmts.getLast();
      if (last instanceof Stmt.ExprStmt es) {
        Value result = evaluator.evalExpr(es.expr());
        if (!(result instanceof Value.Null)) {
          System.out.println(Builtins.display(result));
        }
      } else {
        evaluator.execStmt(last);
      }
    } catch (Exception e) {
      System.err.println("Error: " + e.getMessage());
    }
  }
}
