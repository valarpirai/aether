package com.aether.parser.ast;

import java.util.List;

/**
 * Sealed hierarchy of all statement node types in the Aether AST.
 *
 * <p>Use {@code switch} with pattern matching to exhaustively handle every variant.
 */
public sealed interface Stmt
    permits Stmt.ExprStmt,
        Stmt.Let,
        Stmt.Assign,
        Stmt.CompoundAssign,
        Stmt.Block,
        Stmt.If,
        Stmt.While,
        Stmt.For,
        Stmt.Return,
        Stmt.Break,
        Stmt.Continue,
        Stmt.Function,
        Stmt.Import,
        Stmt.ImportAs,
        Stmt.FromImport,
        Stmt.FromImportAs,
        Stmt.TryCatch,
        Stmt.Throw,
        Stmt.StructDecl {

  /** Expression used as a statement (result discarded). */
  record ExprStmt(Expr expr) implements Stmt {}

  /** Variable declaration: {@code let name = initializer}. */
  record Let(String name, Expr initializer) implements Stmt {}

  /** Assignment: {@code target = value}. */
  record Assign(Expr target, Expr value) implements Stmt {}

  /** Compound assignment: {@code target op= value}. */
  record CompoundAssign(Expr target, BinaryOp op, Expr value) implements Stmt {}

  /** Block of statements: {@code \{ stmts \}}. */
  record Block(List<Stmt> statements) implements Stmt {}

  /** If/else: {@code if (cond) thenBranch [else elseBranch]}. */
  record If(Expr condition, Stmt thenBranch, Stmt elseBranch) implements Stmt {}

  /** While loop: {@code while (cond) body}. */
  record While(Expr condition, Stmt body) implements Stmt {}

  /** For-each loop: {@code for varName in iterable body}. */
  record For(String varName, Expr iterable, Stmt body) implements Stmt {}

  /** Return statement: {@code return [value]}. */
  record Return(Expr value) implements Stmt {}

  /** Break statement. */
  record Break() implements Stmt {}

  /** Continue statement. */
  record Continue() implements Stmt {}

  /** Named function declaration: {@code fn name(params) \{ body \}}. */
  record Function(String name, List<String> params, Stmt body) implements Stmt {}

  /** Module import: {@code import moduleName}. */
  record Import(String moduleName) implements Stmt {}

  /** Aliased import: {@code import moduleName as alias}. */
  record ImportAs(String moduleName, String alias) implements Stmt {}

  /** Selective import: {@code from moduleName import item1, item2}. */
  record FromImport(String moduleName, List<String> items) implements Stmt {}

  /** Selective import with aliases: {@code from moduleName import item as alias, …}. */
  record FromImportAs(String moduleName, List<ImportAlias> items) implements Stmt {}

  /** A single import alias pair. */
  record ImportAlias(String item, String alias) {}

  /** Try/catch block. */
  record TryCatch(Stmt tryBody, String errorVar, Stmt catchBody) implements Stmt {}

  /** Throw statement: {@code throw expr}. */
  record Throw(Expr value) implements Stmt {}

  /** Struct declaration: {@code struct Name \{ fields; methods \}}. */
  record StructDecl(String name, List<String> fields, List<MethodDecl> methods) implements Stmt {}

  /** A method declared inside a struct body. */
  record MethodDecl(String name, List<String> params, Stmt body) {}
}
