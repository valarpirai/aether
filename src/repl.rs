//! REPL (Read-Eval-Print Loop) for interactive Aether sessions

use crate::interpreter::Evaluator;
use crate::lexer::Scanner;
use crate::parser::Parser;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Editor, Helper};
use std::borrow::Cow;

// ── Keywords and built-ins for autocomplete ──────────────────────────────────

const KEYWORDS: &[&str] = &[
    "fn", "let", "if", "else", "while", "for", "in", "return", "break", "continue", "true",
    "false", "null", "and", "or", "not", "import", "from", "as", "struct", "try", "catch", "throw",
];

const BUILTINS: &[&str] = &[
    "print",
    "println",
    "input",
    "len",
    "type",
    "str",
    "int",
    "float",
    "bool",
    "range",
    "set",
    "map",
    "filter",
    "reduce",
    "find",
    "every",
    "some",
    "json_parse",
    "json_stringify",
    "http_get",
    "http_post",
    "clock",
    "sleep",
    "read_file",
    "write_file",
    "abs",
    "min",
    "max",
    "sum",
    "clamp",
    "sign",
    "join",
    "repeat",
    "reverse",
    "starts_with",
    "ends_with",
];

// ── Helper: autocomplete + multi-line validation ──────────────────────────────

struct AetherHelper {
    /// All known names for completion (keywords + builtins + user-defined)
    completions: Vec<String>,
}

impl AetherHelper {
    fn new() -> Self {
        let mut completions: Vec<String> = KEYWORDS
            .iter()
            .chain(BUILTINS.iter())
            .map(|s| s.to_string())
            .collect();
        completions.sort();
        Self { completions }
    }

    fn add_name(&mut self, name: &str) {
        if !self.completions.contains(&name.to_string()) {
            self.completions.push(name.to_string());
            self.completions.sort();
        }
    }
}

impl Helper for AetherHelper {}

impl Completer for AetherHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Find the start of the current word
        let start = line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &line[start..pos];

        let candidates: Vec<Pair> = self
            .completions
            .iter()
            .filter(|s| s.starts_with(prefix))
            .map(|s| Pair {
                display: s.clone(),
                replacement: s.clone(),
            })
            .collect();

        Ok((start, candidates))
    }
}

impl Hinter for AetherHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        if pos < line.len() {
            return None;
        }
        let start = line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &line[start..pos];
        if prefix.is_empty() {
            return None;
        }
        self.completions
            .iter()
            .find(|s| s.starts_with(prefix) && s.as_str() != prefix)
            .map(|s| s[prefix.len()..].to_string())
    }
}

impl Highlighter for AetherHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        // Render hints in dim grey
        Cow::Owned(format!("\x1b[2m{}\x1b[0m", hint))
    }
}

impl Validator for AetherHelper {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        // Multi-line accumulation is handled by the REPL loop with >> / .. prompts.
        Ok(ValidationResult::Valid(None))
    }
}

/// Returns true when the input has unclosed braces/parens or an unclosed triple-quoted string.
fn is_incomplete(input: &str) -> bool {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut depth = 0i32;
    let mut in_triple = false;
    let mut in_single = false;

    while i < len {
        if in_triple {
            if bytes[i] == b'\\' {
                i += 2;
                continue;
            }
            if i + 2 < len && bytes[i] == b'"' && bytes[i + 1] == b'"' && bytes[i + 2] == b'"' {
                in_triple = false;
                i += 3;
                continue;
            }
            i += 1;
            continue;
        }
        if in_single {
            if bytes[i] == b'\\' {
                i += 2;
                continue;
            }
            if bytes[i] == b'"' {
                in_single = false;
            }
            i += 1;
            continue;
        }
        // Outside any string — check for triple quote first
        if i + 2 < len && bytes[i] == b'"' && bytes[i + 1] == b'"' && bytes[i + 2] == b'"' {
            in_triple = true;
            i += 3;
            continue;
        }
        if bytes[i] == b'"' {
            in_single = true;
            i += 1;
            continue;
        }
        match bytes[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    depth > 0 || in_triple
}

// ── History file path ─────────────────────────────────────────────────────────

fn history_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| std::path::PathBuf::from(home).join(".aether_history"))
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("Aether v{}", env!("CARGO_PKG_VERSION"));
    println!("Type '_help' for commands, Tab to autocomplete, Ctrl+D to exit\n");

    let helper = AetherHelper::new();
    let config = rustyline::Config::builder()
        .history_ignore_space(true)
        .completion_type(rustyline::CompletionType::List)
        .edit_mode(rustyline::EditMode::Emacs)
        .build();

    let mut rl: Editor<AetherHelper, rustyline::history::FileHistory> =
        Editor::with_config(config)?;
    rl.set_helper(Some(helper));

    // Load persistent history
    if let Some(path) = history_path() {
        let _ = rl.load_history(&path);
    }

    let mut evaluator = Evaluator::new();

    'repl: loop {
        let mut buffer = String::new();

        // Accumulate lines until the block is syntactically complete.
        let input = loop {
            let prompt = if buffer.is_empty() { ">> " } else { ".. " };
            match rl.readline(prompt) {
                Ok(line) => {
                    if !buffer.is_empty() {
                        buffer.push('\n');
                    }
                    buffer.push_str(&line);
                    if !is_incomplete(&buffer) {
                        break buffer.clone();
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    if !buffer.is_empty() {
                        // Cancel the current multi-line block without exiting
                        println!("^C");
                        continue 'repl;
                    }
                    println!("^C");
                    continue 'repl;
                }
                Err(ReadlineError::Eof) => {
                    println!("Goodbye!");
                    break 'repl;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break 'repl;
                }
            }
        };

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        let _ = rl.add_history_entry(trimmed);

        if trimmed.starts_with('_') {
            handle_command(trimmed, &evaluator);
            continue;
        }

        match execute_input(trimmed, &mut evaluator) {
            Ok(Some(result)) => println!("{}", result),
            Ok(None) => {}
            Err(e) => eprintln!("Error: {}", e),
        }

        // Add newly defined names to autocomplete
        for name in evaluator.environment.bindings().keys() {
            if let Some(h) = rl.helper_mut() {
                h.add_name(name);
            }
        }
    }

    // Persist history on exit
    if let Some(path) = history_path() {
        let _ = rl.save_history(&path);
    }

    Ok(())
}

// ── Code execution ────────────────────────────────────────────────────────────

fn execute_input(source: &str, evaluator: &mut Evaluator) -> Result<Option<String>, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;

    if program.statements.is_empty() {
        return Ok(None);
    }

    // Execute all but the last statement
    let last = program.statements.last().unwrap();
    for stmt in &program.statements[..program.statements.len() - 1] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    // If last statement is a bare expression, print its value
    if let crate::parser::ast::Stmt::Expr(expr) = last {
        let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
        if !matches!(value, crate::interpreter::value::Value::Null) {
            return Ok(Some(format!("{}", value)));
        }
        return Ok(None);
    }

    evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    Ok(None)
}

// ── Special commands ──────────────────────────────────────────────────────────

fn handle_command(cmd: &str, evaluator: &Evaluator) {
    match cmd {
        "_help" => {
            println!("Special commands:");
            println!("  _help    Show this help");
            println!("  _env     Show all defined variables");
            println!("  _exit    Exit the REPL (or Ctrl+D)");
            println!();
            println!("Multi-line input:");
            println!("  Blocks with unclosed {{, (, [ or \"\"\" continue on the next line.");
            println!("  Prompt changes from >> to .. while a block is open.");
            println!("  Ctrl+C cancels the current block and returns to >>.");
            println!();
            println!("Shortcuts:");
            println!("  Tab      Autocomplete");
            println!("  ↑/↓      Browse history (persisted across sessions)");
            println!("  Ctrl+A   Move to start of line");
            println!("  Ctrl+E   Move to end of line");
            println!("  Ctrl+R   Search history");
            println!("  Ctrl+L   Clear screen");
        }
        "_exit" => std::process::exit(0),
        "_env" => {
            use crate::interpreter::value::Value;
            let bindings = evaluator.environment.bindings();
            if bindings.is_empty() {
                println!("(empty environment)");
            } else {
                // Separate into categories for readability
                let mut vars: Vec<(&String, &Value)> = Vec::new();
                let mut fns: Vec<(&String, &Value)> = Vec::new();
                let mut builtins: Vec<(&String, &Value)> = Vec::new();

                for (name, val) in bindings {
                    match val {
                        Value::BuiltinFn { .. } => builtins.push((name, val)),
                        Value::Function { .. } => fns.push((name, val)),
                        _ => vars.push((name, val)),
                    }
                }

                vars.sort_by_key(|(n, _)| n.as_str());
                fns.sort_by_key(|(n, _)| n.as_str());
                builtins.sort_by_key(|(n, _)| n.as_str());

                if !vars.is_empty() {
                    println!("Variables:");
                    for (name, val) in &vars {
                        println!("  {:<20} {}", name, val);
                    }
                }
                if !fns.is_empty() {
                    if !vars.is_empty() {
                        println!();
                    }
                    println!("Functions:");
                    for (name, val) in &fns {
                        if let Value::Function { params, .. } = val {
                            println!("  fn {}({})", name, params.join(", "));
                        }
                    }
                }
                if !builtins.is_empty() {
                    if !vars.is_empty() || !fns.is_empty() {
                        println!();
                    }
                    println!("Built-ins:");
                    let names: Vec<&str> = builtins.iter().map(|(n, _)| n.as_str()).collect();
                    // Print in columns (4 per row)
                    for chunk in names.chunks(4) {
                        print!(" ");
                        for name in chunk {
                            print!(" {:<18}", name);
                        }
                        println!();
                    }
                }
            }
        }
        _ => eprintln!("Unknown command: {}. Type '_help' for help.", cmd),
    }
}
