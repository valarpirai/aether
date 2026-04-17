package com.aether;

import com.aether.interpreter.Evaluator;
import com.aether.interpreter.Value;
import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Stmt;
import java.util.List;
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
 *
 * <p>TODO: implement {@link #run()} — currently a no-op stub.
 */
public final class Repl {

  private static final String PROMPT = ">> ";
  private static final String VERSION = "0.1.0";

  private final Evaluator evaluator = Evaluator.withStdlib();

  /** Start the interactive REPL loop. Exits on Ctrl+D. */
  public void run() {
    System.out.println("Aether " + VERSION + " REPL — type 'help' for commands, Ctrl+D to exit");

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

        // TODO: handle special commands (help, env, etc.)
        // TODO: scan → parse → execute, printing last expression value
        evalLine(line.trim());
      }
    } catch (Exception e) {
      System.err.println("REPL error: " + e.getMessage());
    }
  }

  private void evalLine(String source) {
    try {
      Scanner scanner = new Scanner(source);
      List<Stmt> stmts = new Parser(scanner.scanTokens()).parse();
      if (stmts.isEmpty()) {
        return;
      }
      // TODO: if last stmt is ExprStmt, print its value
      evaluator.execute(stmts);
    } catch (Exception e) {
      System.err.println("Error: " + e.getMessage());
    }
  }
}
