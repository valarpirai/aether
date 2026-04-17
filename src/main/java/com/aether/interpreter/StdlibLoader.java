package com.aether.interpreter;

import com.aether.lexer.Scanner;
import com.aether.parser.Parser;
import com.aether.parser.ast.Stmt;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;

/**
 * Loads the embedded Aether standard library into an {@link Evaluator}.
 *
 * <p>The stdlib {@code .ae} files are bundled under {@code src/main/resources/stdlib/} and loaded
 * from the classpath at runtime — the same approach as the Rust implementation's
 * {@code include_str!()}.
 *
 * <p>Each module is parsed and executed in a fresh child evaluator so that closures only capture
 * the small bootstrap environment, not the ever-growing main env. Only the top-level bindings are
 * then copied into the provided environment.
 *
 * <p>TODO: implement {@link #load(Environment)} — currently a no-op stub.
 */
public final class StdlibLoader {

  /** Ordered list of stdlib module names to load (order matters for inter-module deps). */
  private static final List<String> STDLIB_MODULES =
      List.of("core", "collections", "math", "string", "testing");

  private StdlibLoader() {}

  /**
   * Load all stdlib modules into the given environment.
   *
   * @param env the global environment to populate
   */
  public static void load(Environment env) {
    for (String module : STDLIB_MODULES) {
      String source = readResource("stdlib/" + module + ".ae");
      if (source == null) {
        System.err.println("Warning: stdlib module '" + module + "' not found in resources");
        continue;
      }
      // TODO: parse source → execute in child evaluator → copy bindings to env
    }
  }

  /**
   * Read a classpath resource as a UTF-8 string.
   *
   * @return the resource content, or {@code null} if not found
   */
  private static String readResource(String path) {
    try (InputStream is = StdlibLoader.class.getClassLoader().getResourceAsStream(path)) {
      if (is == null) {
        return null;
      }
      return new String(is.readAllBytes(), StandardCharsets.UTF_8);
    } catch (IOException e) {
      return null;
    }
  }
}
