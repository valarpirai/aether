package com.aether.interpreter;

import com.aether.exception.AetherRuntimeException;
import com.aether.parser.ast.BinaryOp;
import com.aether.parser.ast.Expr;
import com.aether.parser.ast.Stmt;
import com.aether.parser.ast.UnaryOp;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * Tree-walking interpreter for Aether.
 *
 * <p>Evaluates AST nodes produced by {@link com.aether.parser.Parser} against a mutable {@link
 * Environment}.
 */
public final class Evaluator {

  /** Internal signal used to implement return / break / continue control flow. */
  private sealed interface ControlFlow
      permits ControlFlow.None,
          ControlFlow.Return,
          ControlFlow.Break,
          ControlFlow.Continue {

    record None() implements ControlFlow {}

    record Return(Value value) implements ControlFlow {}

    record Break() implements ControlFlow {}

    record Continue() implements ControlFlow {}
  }

  private static final int DEFAULT_MAX_CALL_DEPTH = 100;

  private Environment environment;
  private int callDepth = 0;
  private final int maxCallDepth;
  private final Map<String, Environment> moduleCache = new HashMap<>();
  private final java.util.ArrayDeque<String> loadingStack = new java.util.ArrayDeque<>();

  /** Create an evaluator with builtins pre-registered and stdlib loaded. */
  public static Evaluator withStdlib() {
    Evaluator ev = new Evaluator(new Environment(), DEFAULT_MAX_CALL_DEPTH);
    ev.registerBuiltins();
    ev.loadStdlib();
    return ev;
  }

  /** Create an evaluator with only builtins (no stdlib — faster for tests). */
  public static Evaluator withoutStdlib() {
    Evaluator ev = new Evaluator(new Environment(), DEFAULT_MAX_CALL_DEPTH);
    ev.registerBuiltins();
    return ev;
  }

  private Evaluator(Environment env, int maxCallDepth) {
    this.environment = env;
    this.maxCallDepth = maxCallDepth;
  }

  // ── Public API ───────────────────────────────────────────────────────────────

  /**
   * Evaluate an expression and return its value.
   *
   * @throws AetherRuntimeException on runtime errors
   */
  public Value evalExpr(Expr expr) {
    return switch (expr) {
      case Expr.IntLiteral(long v) -> new Value.IntVal(v);
      case Expr.FloatLiteral(double v) -> new Value.FloatVal(v);
      case Expr.StringLiteral(String v) -> new Value.Str(v);
      case Expr.BoolLiteral(boolean v) -> new Value.Bool(v);
      case Expr.NullLiteral() -> Value.Null.INSTANCE;
      case Expr.Identifier(String name) -> environment.get(name);
      case Expr.Unary(UnaryOp op, Expr operand) -> evalUnary(op, operand);
      case Expr.Binary(Expr left, BinaryOp op, Expr right) -> evalBinary(left, op, right);
      case Expr.Call(Expr callee, List<Expr> args) -> evalCall(callee, args);
      case Expr.Array(List<Expr> elements) -> evalArray(elements);
      case Expr.Dict(List<Expr.DictEntry> entries) -> evalDict(entries);
      case Expr.Index(Expr object, Expr index) -> evalIndex(object, index);
      case Expr.Slice(Expr object, Expr start, Expr end) -> evalSlice(object, start, end);
      case Expr.Member(Expr object, String member) -> evalMember(object, member);
      case Expr.FunctionExpr(List<String> params, Stmt body) ->
          new Value.AetherFunction(params, body, environment);
      case Expr.StringInterp(List<Expr> parts) -> evalStringInterp(parts);
      case Expr.StructInit(String name, List<Expr.FieldInit> fields) ->
          evalStructInit(name, fields);
      case Expr.Spread ignored ->
          throw new AetherRuntimeException.InvalidOperation(
              "spread operator is only valid inside array literals");
    };
  }

  /**
   * Execute a statement.
   *
   * @throws AetherRuntimeException on runtime errors
   */
  public void execStmt(Stmt stmt) {
    execStmtInternal(stmt);
  }

  /**
   * Execute a list of statements (a program).
   *
   * @throws AetherRuntimeException on runtime errors
   */
  public void execute(List<Stmt> statements) {
    for (Stmt stmt : statements) {
      execStmtInternal(stmt);
    }
  }

  /** Return the current top-level environment (for REPL inspection). */
  public Environment environment() {
    return environment;
  }

  // ── Internal execution ───────────────────────────────────────────────────────

  private ControlFlow execStmtInternal(Stmt stmt) {
    return switch (stmt) {
      case Stmt.ExprStmt(Expr expr) -> {
        evalExpr(expr);
        yield new ControlFlow.None();
      }
      case Stmt.Let(String name, Expr init) -> {
        environment.define(name, evalExpr(init));
        yield new ControlFlow.None();
      }
      case Stmt.Assign(Expr target, Expr value) -> {
        assignTarget(target, evalExpr(value));
        yield new ControlFlow.None();
      }
      case Stmt.CompoundAssign(Expr target, BinaryOp op, Expr value) -> {
        Value current = evalExpr(target);
        Value rhs = evalExpr(value);
        assignTarget(target, evalBinaryValues(current, op, rhs));
        yield new ControlFlow.None();
      }
      case Stmt.Block(List<Stmt> statements) -> {
        Environment blockEnv = new Environment(environment);
        Environment saved = environment;
        environment = blockEnv;
        try {
          for (Stmt s : statements) {
            ControlFlow cf = execStmtInternal(s);
            if (!(cf instanceof ControlFlow.None)) {
              yield cf;
            }
          }
          yield new ControlFlow.None();
        } finally {
          environment = saved;
        }
      }
      case Stmt.If(Expr condition, Stmt thenBranch, Stmt elseBranch) -> {
        if (evalExpr(condition).isTruthy()) {
          yield execStmtInternal(thenBranch);
        } else if (elseBranch != null) {
          yield execStmtInternal(elseBranch);
        } else {
          yield new ControlFlow.None();
        }
      }
      case Stmt.While(Expr condition, Stmt body) -> {
        while (evalExpr(condition).isTruthy()) {
          ControlFlow cf = execStmtInternal(body);
          if (cf instanceof ControlFlow.Break) break;
          if (cf instanceof ControlFlow.Return r) yield r;
          // Continue: proceed to next iteration
        }
        yield new ControlFlow.None();
      }
      case Stmt.For(String varName, Expr iterable, Stmt body) -> {
        Value iterVal = evalExpr(iterable);
        if (!(iterVal instanceof Value.Array arr)) {
          throw new AetherRuntimeException.TypeError("array", iterVal.typeName());
        }
        for (Value element : arr.elements()) {
          Environment saved = environment;
          environment = new Environment(saved);
          environment.define(varName, element);
          ControlFlow cf = execStmtInternal(body);
          environment = saved;
          if (cf instanceof ControlFlow.Break) break;
          if (cf instanceof ControlFlow.Return r) yield r;
        }
        yield new ControlFlow.None();
      }
      case Stmt.Return(Expr value) -> {
        Value v = value != null ? evalExpr(value) : Value.Null.INSTANCE;
        yield new ControlFlow.Return(v);
      }
      case Stmt.Break() -> new ControlFlow.Break();
      case Stmt.Continue() -> new ControlFlow.Continue();
      case Stmt.Function(String name, List<String> params, Stmt body) -> {
        Value.AetherFunction func = new Value.AetherFunction(params, body, environment);
        environment.define(name, func);
        yield new ControlFlow.None();
      }
      case Stmt.TryCatch(Stmt tryBody, String errorVar, Stmt catchBody) -> {
        try {
          yield execStmtInternal(tryBody);
        } catch (AetherRuntimeException e) {
          environment.define(errorVar, new Value.Str(e.getMessage()));
          yield execStmtInternal(catchBody);
        }
      }
      case Stmt.Throw(Expr value) -> {
        String msg = Builtins.display(evalExpr(value));
        throw new AetherRuntimeException.Thrown(msg);
      }
      case Stmt.StructDecl(String name, List<String> fields, List<Stmt.MethodDecl> methods) -> {
        Map<String, Value.MethodEntry> methodMap = new HashMap<>();
        for (Stmt.MethodDecl m : methods) {
          methodMap.put(m.name(), new Value.MethodEntry(m.params(), m.body()));
        }
        environment.define(name, new Value.StructDef(name, fields, methodMap));
        yield new ControlFlow.None();
      }
      case Stmt.Import(String moduleName) -> {
        loadModule(moduleName);
        yield new ControlFlow.None();
      }
      case Stmt.ImportAs(String moduleName, String alias) -> {
        loadModuleAs(moduleName, alias);
        yield new ControlFlow.None();
      }
      case Stmt.FromImport(String moduleName, List<String> items) -> {
        fromImport(moduleName, items);
        yield new ControlFlow.None();
      }
      case Stmt.FromImportAs(String moduleName, List<Stmt.ImportAlias> items) -> {
        fromImportAs(moduleName, items);
        yield new ControlFlow.None();
      }
    };
  }

  // ── Expression evaluation helpers ────────────────────────────────────────────

  private Value evalUnary(UnaryOp op, Expr operand) {
    Value val = evalExpr(operand);
    return switch (op) {
      case NEGATE -> switch (val) {
        case Value.IntVal(long n) -> new Value.IntVal(-n);
        case Value.FloatVal(double f) -> new Value.FloatVal(-f);
        default -> throw new AetherRuntimeException.TypeError("number", val.typeName());
      };
      case NOT -> new Value.Bool(!val.isTruthy());
    };
  }

  private Value evalBinary(Expr leftExpr, BinaryOp op, Expr rightExpr) {
    // Short-circuit logical operators
    if (op == BinaryOp.AND) {
      Value left = evalExpr(leftExpr);
      return left.isTruthy() ? evalExpr(rightExpr) : left;
    }
    if (op == BinaryOp.OR) {
      Value left = evalExpr(leftExpr);
      return left.isTruthy() ? left : evalExpr(rightExpr);
    }
    return evalBinaryValues(evalExpr(leftExpr), op, evalExpr(rightExpr));
  }

  private Value evalBinaryValues(Value left, BinaryOp op, Value right) {
    return switch (op) {
      case ADD -> evalAdd(left, right);
      case SUBTRACT -> evalArithmetic(left, right, (a, b) -> a - b, (a, b) -> a - b);
      case MULTIPLY -> evalArithmetic(left, right, (a, b) -> a * b, (a, b) -> a * b);
      case DIVIDE -> evalDivide(left, right);
      case MODULO -> evalModulo(left, right);
      case EQUAL -> new Value.Bool(valuesEqual(left, right));
      case NOT_EQUAL -> new Value.Bool(!valuesEqual(left, right));
      case LESS -> evalComparison(left, right, (a, b) -> a < b, (a, b) -> a < b);
      case GREATER -> evalComparison(left, right, (a, b) -> a > b, (a, b) -> a > b);
      case LESS_EQUAL -> evalComparison(left, right, (a, b) -> a <= b, (a, b) -> a <= b);
      case GREATER_EQUAL -> evalComparison(left, right, (a, b) -> a >= b, (a, b) -> a >= b);
      case AND, OR ->
          throw new AetherRuntimeException.InvalidOperation(
              "AND/OR must use short-circuit evaluation");
    };
  }

  @FunctionalInterface
  private interface LongBinaryOp {
    long apply(long a, long b);
  }

  @FunctionalInterface
  private interface DoubleBinaryOp {
    double apply(double a, double b);
  }

  @FunctionalInterface
  private interface LongToBoolOp {
    boolean apply(long a, long b);
  }

  @FunctionalInterface
  private interface DoubleToBoolOp {
    boolean apply(double a, double b);
  }

  private Value evalAdd(Value left, Value right) {
    return switch (left) {
      case Value.IntVal(long a) -> switch (right) {
        case Value.IntVal(long b) -> new Value.IntVal(a + b);
        case Value.FloatVal(double b) -> new Value.FloatVal(a + b);
        default -> throw new AetherRuntimeException.TypeError("number", right.typeName());
      };
      case Value.FloatVal(double a) -> switch (right) {
        case Value.IntVal(long b) -> new Value.FloatVal(a + b);
        case Value.FloatVal(double b) -> new Value.FloatVal(a + b);
        default -> throw new AetherRuntimeException.TypeError("number", right.typeName());
      };
      case Value.Str(String a) -> switch (right) {
        case Value.Str(String b) -> new Value.Str(a + b);
        default ->
            throw new AetherRuntimeException.TypeError(
                "string", left.typeName() + " + " + right.typeName());
      };
      default ->
          throw new AetherRuntimeException.TypeError(
              "number or string", left.typeName() + " + " + right.typeName());
    };
  }

  private Value evalArithmetic(Value left, Value right, LongBinaryOp intOp, DoubleBinaryOp floatOp) {
    return switch (left) {
      case Value.IntVal(long a) -> switch (right) {
        case Value.IntVal(long b) -> new Value.IntVal(intOp.apply(a, b));
        case Value.FloatVal(double b) -> new Value.FloatVal(floatOp.apply(a, b));
        default -> throw new AetherRuntimeException.TypeError("number", right.typeName());
      };
      case Value.FloatVal(double a) -> switch (right) {
        case Value.IntVal(long b) -> new Value.FloatVal(floatOp.apply(a, b));
        case Value.FloatVal(double b) -> new Value.FloatVal(floatOp.apply(a, b));
        default -> throw new AetherRuntimeException.TypeError("number", right.typeName());
      };
      default -> throw new AetherRuntimeException.TypeError("number", left.typeName());
    };
  }

  private Value evalDivide(Value left, Value right) {
    return switch (left) {
      case Value.IntVal(long a) -> switch (right) {
        case Value.IntVal(long b) -> {
          if (b == 0) throw new AetherRuntimeException.DivisionByZero();
          yield new Value.IntVal(a / b);
        }
        case Value.FloatVal(double b) -> new Value.FloatVal(a / b);
        default -> throw new AetherRuntimeException.TypeError("number", right.typeName());
      };
      case Value.FloatVal(double a) -> switch (right) {
        case Value.IntVal(long b) -> new Value.FloatVal(a / b);
        case Value.FloatVal(double b) -> new Value.FloatVal(a / b);
        default -> throw new AetherRuntimeException.TypeError("number", right.typeName());
      };
      default -> throw new AetherRuntimeException.TypeError("number", left.typeName());
    };
  }

  private Value evalModulo(Value left, Value right) {
    return switch (left) {
      case Value.IntVal(long a) -> switch (right) {
        case Value.IntVal(long b) -> {
          if (b == 0) throw new AetherRuntimeException.DivisionByZero();
          yield new Value.IntVal(a % b);
        }
        case Value.FloatVal(double b) -> {
          if (b == 0.0) throw new AetherRuntimeException.DivisionByZero();
          yield new Value.FloatVal(a % b);
        }
        default -> throw new AetherRuntimeException.TypeError("number", right.typeName());
      };
      case Value.FloatVal(double a) -> switch (right) {
        case Value.IntVal(long b) -> {
          if (b == 0) throw new AetherRuntimeException.DivisionByZero();
          yield new Value.FloatVal(a % b);
        }
        case Value.FloatVal(double b) -> {
          if (b == 0.0) throw new AetherRuntimeException.DivisionByZero();
          yield new Value.FloatVal(a % b);
        }
        default -> throw new AetherRuntimeException.TypeError("number", right.typeName());
      };
      default -> throw new AetherRuntimeException.TypeError("number", left.typeName());
    };
  }

  private Value evalComparison(
      Value left, Value right, LongToBoolOp intOp, DoubleToBoolOp floatOp) {
    return switch (left) {
      case Value.IntVal(long a) -> switch (right) {
        case Value.IntVal(long b) -> new Value.Bool(intOp.apply(a, b));
        case Value.FloatVal(double b) -> new Value.Bool(floatOp.apply(a, b));
        default -> throw new AetherRuntimeException.TypeError("number", right.typeName());
      };
      case Value.FloatVal(double a) -> switch (right) {
        case Value.IntVal(long b) -> new Value.Bool(floatOp.apply(a, b));
        case Value.FloatVal(double b) -> new Value.Bool(floatOp.apply(a, b));
        default -> throw new AetherRuntimeException.TypeError("number", right.typeName());
      };
      case Value.Str(String a) -> switch (right) {
        case Value.Str(String b) -> new Value.Bool(intOp.apply(a.compareTo(b), 0));
        default -> throw new AetherRuntimeException.TypeError("string", right.typeName());
      };
      default -> throw new AetherRuntimeException.TypeError("comparable value", left.typeName());
    };
  }

  private boolean valuesEqual(Value left, Value right) {
    return switch (left) {
      case Value.IntVal(long a) -> switch (right) {
        case Value.IntVal(long b) -> a == b;
        case Value.FloatVal(double b) -> a == b;
        default -> false;
      };
      case Value.FloatVal(double a) -> switch (right) {
        case Value.IntVal(long b) -> a == b;
        case Value.FloatVal(double b) -> a == b;
        default -> false;
      };
      case Value.Str(String a) -> right instanceof Value.Str(String b) && a.equals(b);
      case Value.Bool(boolean a) -> right instanceof Value.Bool(boolean b) && a == b;
      case Value.Null ignored -> right instanceof Value.Null;
      default -> false;
    };
  }

  private Value evalArray(List<Expr> elements) {
    List<Value> values = new ArrayList<>();
    for (Expr elem : elements) {
      if (elem instanceof Expr.Spread(Expr inner)) {
        Value spread = evalExpr(inner);
        if (!(spread instanceof Value.Array arr)) {
          throw new AetherRuntimeException.InvalidOperation(
              "spread operator requires an array, got " + spread.typeName());
        }
        values.addAll(arr.elements());
      } else {
        values.add(evalExpr(elem));
      }
    }
    return new Value.Array(values);
  }

  private Value evalDict(List<Expr.DictEntry> entries) {
    LinkedHashMap<Value, Value> map = new LinkedHashMap<>();
    for (Expr.DictEntry entry : entries) {
      map.put(evalExpr(entry.key()), evalExpr(entry.value()));
    }
    return new Value.Dict(map);
  }

  private Value evalIndex(Expr objectExpr, Expr indexExpr) {
    Value object = evalExpr(objectExpr);
    Value index = evalExpr(indexExpr);

    return switch (object) {
      case Value.Array arr -> {
        if (!(index instanceof Value.IntVal(long idx))) {
          throw new AetherRuntimeException.TypeError("int", index.typeName());
        }
        List<Value> elements = arr.elements();
        long resolved = idx < 0 ? elements.size() + idx : idx;
        if (resolved < 0 || resolved >= elements.size()) {
          throw new AetherRuntimeException.IndexOutOfBounds(idx, elements.size());
        }
        yield elements.get((int) resolved);
      }
      case Value.Str(String s) -> {
        if (!(index instanceof Value.IntVal(long idx))) {
          throw new AetherRuntimeException.TypeError("int", index.typeName());
        }
        List<Character> chars = new ArrayList<>();
        for (char c : s.toCharArray()) chars.add(c);
        long resolved = idx < 0 ? chars.size() + idx : idx;
        if (resolved < 0 || resolved >= chars.size()) {
          throw new AetherRuntimeException.IndexOutOfBounds(idx, chars.size());
        }
        yield new Value.Str(String.valueOf(chars.get((int) resolved)));
      }
      case Value.Dict dict -> {
        Value v = dict.entries().get(index);
        if (v == null) {
          throw new AetherRuntimeException.InvalidOperation(
              "Key " + Builtins.display(index) + " not found in dict");
        }
        yield v;
      }
      default ->
          throw new AetherRuntimeException.TypeError(
              "array, string, or dict", object.typeName());
    };
  }

  private Value evalSlice(Expr objectExpr, Expr startExpr, Expr endExpr) {
    Value object = evalExpr(objectExpr);
    Long startIdx = startExpr != null ? toLong(evalExpr(startExpr)) : null;
    Long endIdx = endExpr != null ? toLong(evalExpr(endExpr)) : null;

    return switch (object) {
      case Value.Array arr -> {
        List<Value> elements = arr.elements();
        int len = elements.size();
        int s = startIdx == null ? 0 : resolveIndex(startIdx, len);
        int e = endIdx == null ? len : resolveIndex(endIdx, len);
        yield new Value.Array(s >= e ? List.of() : elements.subList(s, e));
      }
      case Value.Str(String str) -> {
        int len = str.length();
        int s = startIdx == null ? 0 : resolveIndex(startIdx, len);
        int e = endIdx == null ? len : resolveIndex(endIdx, len);
        yield new Value.Str(s >= e ? "" : str.substring(s, e));
      }
      default ->
          throw new AetherRuntimeException.InvalidOperation(
              "slice not supported on " + object.typeName());
    };
  }

  private long toLong(Value v) {
    if (v instanceof Value.IntVal(long n)) return n;
    throw new AetherRuntimeException.TypeError("int", v.typeName());
  }

  private int resolveIndex(long n, int len) {
    return (int) (n < 0 ? Math.max(0, len + n) : Math.min(n, len));
  }

  private Value evalMember(Expr objectExpr, String member) {
    Value object = evalExpr(objectExpr);

    return switch (object) {
      case Value.Array arr when member.equals("length") -> new Value.IntVal(arr.elements().size());
      case Value.Str(String s) when member.equals("length") -> new Value.IntVal(s.length());
      case Value.Dict dict -> {
        if (member.equals("length")) yield new Value.IntVal(dict.entries().size());
        Value v = dict.entries().get(new Value.Str(member));
        if (v == null) {
          throw new AetherRuntimeException.InvalidOperation(
              "Key '" + member + "' not found in dict");
        }
        yield v;
      }
      case Value.Module(String name, Map<String, Value> members) -> {
        Value v = members.get(member);
        if (v == null) {
          throw new AetherRuntimeException.InvalidOperation(
              "Module '" + name + "' has no member '" + member + "'");
        }
        yield v;
      }
      case Value.Instance inst -> {
        Value v = inst.fields().get(member);
        if (v == null) {
          throw new AetherRuntimeException.InvalidOperation(
              "Field '" + member + "' does not exist on '" + inst.typeName() + "'");
        }
        yield v;
      }
      default ->
          throw new AetherRuntimeException.InvalidOperation(
              "Property '" + member + "' does not exist on type '" + object.typeName() + "'");
    };
  }

  private Value evalStringInterp(List<Expr> parts) {
    StringBuilder sb = new StringBuilder();
    for (Expr part : parts) {
      sb.append(Builtins.display(evalExpr(part)));
    }
    return new Value.Str(sb.toString());
  }

  private Value evalStructInit(String name, List<Expr.FieldInit> fields) {
    Value def = environment.get(name);
    if (!(def instanceof Value.StructDef sd)) {
      throw new AetherRuntimeException.InvalidOperation(
          "'" + name + "' is not a struct (got " + def.typeName() + ")");
    }
    Map<String, Value> fieldMap = new HashMap<>();
    for (String f : sd.fields()) {
      fieldMap.put(f, Value.Null.INSTANCE);
    }
    for (Expr.FieldInit fi : fields) {
      if (!sd.fields().contains(fi.name())) {
        throw new AetherRuntimeException.InvalidOperation(
            "Struct '" + name + "' has no field '" + fi.name() + "'");
      }
      fieldMap.put(fi.name(), evalExpr(fi.value()));
    }
    return new Value.Instance(name, fieldMap, sd.methods());
  }

  private Value evalCall(Expr calleeExpr, List<Expr> argExprs) {
    // Method call: obj.method(args)
    if (calleeExpr instanceof Expr.Member(Expr object, String method)) {
      return evalMethodCall(object, method, argExprs);
    }

    // Save function name for recursion support
    String funcName = calleeExpr instanceof Expr.Identifier(String n) ? n : null;

    Value funcVal = evalExpr(calleeExpr);
    return callValue(funcVal, funcName, argExprs);
  }

  private Value callValue(Value funcVal, String funcName, List<Expr> argExprs) {
    return switch (funcVal) {
      case Value.AetherFunction(List<String> params, Stmt body, Environment closure) -> {
        callDepth++;
        if (callDepth > maxCallDepth) {
          callDepth--;
          throw new AetherRuntimeException.StackOverflow(callDepth + 1, maxCallDepth);
        }
        if (argExprs.size() > params.size()) {
          callDepth--;
          throw new AetherRuntimeException.ArityMismatch(params.size(), argExprs.size());
        }
        List<Value> argVals = new ArrayList<>();
        for (Expr arg : argExprs) {
          argVals.add(evalExpr(arg));
        }
        while (argVals.size() < params.size()) {
          argVals.add(Value.Null.INSTANCE);
        }

        Environment savedEnv = environment;
        environment = new Environment(closure);
        // Re-bind function name for recursion
        if (funcName != null) {
          environment.define(funcName, funcVal);
        }
        for (int i = 0; i < params.size(); i++) {
          environment.define(params.get(i), argVals.get(i));
        }
        try {
          ControlFlow cf = execStmtInternal(body);
          yield cf instanceof ControlFlow.Return(Value v) ? v : Value.Null.INSTANCE;
        } finally {
          environment = savedEnv;
          callDepth--;
        }
      }
      case Value.Builtin(String name, int arity, java.util.function.Function<List<Value>, Value> impl) -> {
        if (arity != -1 && arity != argExprs.size()) {
          throw new AetherRuntimeException.ArityMismatch(arity, argExprs.size());
        }
        List<Value> argVals = new ArrayList<>();
        for (Expr arg : argExprs) {
          argVals.add(evalExpr(arg));
        }
        yield impl.apply(argVals);
      }
      default ->
          throw new AetherRuntimeException.InvalidOperation(
              "Cannot call value of type '" + funcVal.typeName() + "'");
    };
  }

  private Value callFunctionWithValues(Value funcVal, List<Value> args) {
    return switch (funcVal) {
      case Value.AetherFunction(List<String> params, Stmt body, Environment closure) -> {
        callDepth++;
        if (callDepth > maxCallDepth) {
          callDepth--;
          throw new AetherRuntimeException.StackOverflow(callDepth + 1, maxCallDepth);
        }
        Environment savedEnv = environment;
        environment = new Environment(closure);
        for (int i = 0; i < params.size(); i++) {
          environment.define(params.get(i), i < args.size() ? args.get(i) : Value.Null.INSTANCE);
        }
        try {
          ControlFlow cf = execStmtInternal(body);
          yield cf instanceof ControlFlow.Return(Value v) ? v : Value.Null.INSTANCE;
        } finally {
          environment = savedEnv;
          callDepth--;
        }
      }
      case Value.Builtin(String name, int arity, java.util.function.Function<List<Value>, Value> impl) ->
          impl.apply(args);
      default ->
          throw new AetherRuntimeException.InvalidOperation(
              "Cannot call value of type '" + funcVal.typeName() + "'");
    };
  }

  private Value evalMethodCall(Expr objectExpr, String method, List<Expr> argExprs) {
    Value object = evalExpr(objectExpr);

    return switch (object) {
      case Value.Array arr -> evalArrayMethod(arr, objectExpr, method, argExprs);
      case Value.Str(String s) -> evalStringMethod(s, method, argExprs);
      case Value.Dict dict -> evalDictMethod(dict, method, argExprs);
      case Value.Module(String modName, Map<String, Value> members) -> {
        Value func = members.get(method);
        if (func == null) {
          throw new AetherRuntimeException.InvalidOperation(
              "Module '" + modName + "' has no member '" + method + "'");
        }
        yield callValue(func, null, argExprs);
      }
      case Value.Instance inst -> evalInstanceMethod(inst, method, argExprs);
      default ->
          throw new AetherRuntimeException.InvalidOperation(
              "Method '" + method + "' does not exist on type '" + object.typeName() + "'");
    };
  }

  private Value evalArrayMethod(Value.Array arr, Expr objectExpr, String method, List<Expr> argExprs) {
    return switch (method) {
      case "push" -> {
        if (argExprs.size() != 1) {
          throw new AetherRuntimeException.ArityMismatch(1, argExprs.size());
        }
        Value item = evalExpr(argExprs.get(0));
        List<Value> newElements = new ArrayList<>(arr.elements());
        newElements.add(item);
        if (objectExpr instanceof Expr.Identifier(String name)) {
          environment.set(name, new Value.Array(newElements));
        } else {
          arr.setElements(newElements);
        }
        yield Value.Null.INSTANCE;
      }
      case "pop" -> {
        if (!argExprs.isEmpty()) {
          throw new AetherRuntimeException.ArityMismatch(0, argExprs.size());
        }
        List<Value> elements = arr.elements();
        if (elements.isEmpty()) yield Value.Null.INSTANCE;
        List<Value> newElements = new ArrayList<>(elements);
        Value popped = newElements.remove(newElements.size() - 1);
        if (objectExpr instanceof Expr.Identifier(String name)) {
          environment.set(name, new Value.Array(newElements));
        } else {
          arr.setElements(newElements);
        }
        yield popped;
      }
      case "sort" -> {
        List<Value> sorted = new ArrayList<>(arr.elements());
        if (!argExprs.isEmpty()) {
          Value comparatorVal = evalExpr(argExprs.get(0));
          sorted.sort((a, b) -> {
            if (callFunctionWithValues(comparatorVal, List.of(a, b)).isTruthy()) return -1;
            if (callFunctionWithValues(comparatorVal, List.of(b, a)).isTruthy()) return 1;
            return 0;
          });
        } else {
          sorted.sort(this::compareValues);
        }
        if (objectExpr instanceof Expr.Identifier(String name)) {
          environment.set(name, new Value.Array(sorted));
        } else {
          arr.setElements(sorted);
        }
        yield Value.Null.INSTANCE;
      }
      case "contains" -> {
        if (argExprs.size() != 1) {
          throw new AetherRuntimeException.ArityMismatch(1, argExprs.size());
        }
        Value target = evalExpr(argExprs.get(0));
        boolean found = arr.elements().stream()
            .anyMatch(el -> compareValues(el, target) == 0);
        yield new Value.Bool(found);
      }
      case "concat" -> {
        if (argExprs.size() != 1) {
          throw new AetherRuntimeException.ArityMismatch(1, argExprs.size());
        }
        Value other = evalExpr(argExprs.get(0));
        if (!(other instanceof Value.Array otherArr)) {
          throw new AetherRuntimeException.TypeError("array", other.typeName());
        }
        List<Value> combined = new ArrayList<>(arr.elements());
        combined.addAll(otherArr.elements());
        yield new Value.Array(combined);
      }
      default ->
          throw new AetherRuntimeException.InvalidOperation(
              "Method '" + method + "' does not exist on type 'array'");
    };
  }

  @SuppressWarnings("unchecked")
  private int compareValues(Value a, Value b) {
    if (a instanceof Value.IntVal(long x) && b instanceof Value.IntVal(long y)) {
      return Long.compare(x, y);
    }
    if (a instanceof Value.FloatVal(double x) && b instanceof Value.FloatVal(double y)) {
      return Double.compare(x, y);
    }
    if (a instanceof Value.Str(String x) && b instanceof Value.Str(String y)) {
      return x.compareTo(y);
    }
    throw new AetherRuntimeException.TypeError("comparable values", a.typeName() + " and " + b.typeName());
  }

  private Value evalStringMethod(String s, String method, List<Expr> argExprs) {
    return switch (method) {
      case "upper" -> {
        if (!argExprs.isEmpty()) throw new AetherRuntimeException.ArityMismatch(0, argExprs.size());
        yield new Value.Str(s.toUpperCase());
      }
      case "lower" -> {
        if (!argExprs.isEmpty()) throw new AetherRuntimeException.ArityMismatch(0, argExprs.size());
        yield new Value.Str(s.toLowerCase());
      }
      case "trim" -> {
        if (!argExprs.isEmpty()) throw new AetherRuntimeException.ArityMismatch(0, argExprs.size());
        yield new Value.Str(s.trim());
      }
      case "split" -> {
        if (argExprs.size() != 1) throw new AetherRuntimeException.ArityMismatch(1, argExprs.size());
        Value delim = evalExpr(argExprs.get(0));
        if (!(delim instanceof Value.Str(String d))) {
          throw new AetherRuntimeException.TypeError("string", delim.typeName());
        }
        if (s.isEmpty()) yield new Value.Array(List.of());
        List<Value> parts = new ArrayList<>();
        for (String part : s.split(java.util.regex.Pattern.quote(d))) {
          parts.add(new Value.Str(part));
        }
        yield new Value.Array(parts);
      }
      default ->
          throw new AetherRuntimeException.InvalidOperation(
              "Method '" + method + "' does not exist on type 'string'");
    };
  }

  private Value evalDictMethod(Value.Dict dict, String method, List<Expr> argExprs) {
    return switch (method) {
      case "keys" -> {
        if (!argExprs.isEmpty()) throw new AetherRuntimeException.ArityMismatch(0, argExprs.size());
        yield new Value.Array(new ArrayList<>(dict.entries().keySet()));
      }
      case "values" -> {
        if (!argExprs.isEmpty()) throw new AetherRuntimeException.ArityMismatch(0, argExprs.size());
        yield new Value.Array(new ArrayList<>(dict.entries().values()));
      }
      case "contains" -> {
        if (argExprs.size() != 1) throw new AetherRuntimeException.ArityMismatch(1, argExprs.size());
        Value key = evalExpr(argExprs.get(0));
        yield new Value.Bool(dict.entries().containsKey(key));
      }
      default ->
          throw new AetherRuntimeException.InvalidOperation(
              "Method '" + method + "' does not exist on type 'dict'");
    };
  }

  private Value evalInstanceMethod(Value.Instance inst, String method, List<Expr> argExprs) {
    Value.MethodEntry entry = inst.methods().get(method);
    if (entry == null) {
      throw new AetherRuntimeException.InvalidOperation(
          "Method '" + method + "' does not exist on '" + inst.typeName() + "'");
    }
    List<String> params = entry.params();
    Stmt body = entry.body();

    // Evaluate arguments
    List<Value> argVals = new ArrayList<>();
    for (Expr arg : argExprs) {
      argVals.add(evalExpr(arg));
    }

    callDepth++;
    if (callDepth > maxCallDepth) {
      callDepth--;
      throw new AetherRuntimeException.StackOverflow(callDepth + 1, maxCallDepth);
    }
    Environment savedEnv = environment;
    environment = new Environment(environment);
    environment.define("self", inst);

    // Params excluding "self"
    List<String> userParams =
        params.isEmpty() || !params.get(0).equals("self") ? params : params.subList(1, params.size());
    while (argVals.size() < userParams.size()) argVals.add(Value.Null.INSTANCE);
    for (int i = 0; i < userParams.size(); i++) {
      environment.define(userParams.get(i), argVals.get(i));
    }

    try {
      ControlFlow cf = execStmtInternal(body);
      return cf instanceof ControlFlow.Return(Value v) ? v : Value.Null.INSTANCE;
    } finally {
      environment = savedEnv;
      callDepth--;
    }
  }

  // ── Assignment ───────────────────────────────────────────────────────────────

  private void assignTarget(Expr target, Value value) {
    switch (target) {
      case Expr.Identifier(String name) -> environment.set(name, value);
      case Expr.Index(Expr objectExpr, Expr indexExpr) -> {
        Value object = evalExpr(objectExpr);
        Value index = evalExpr(indexExpr);
        if (object instanceof Value.Dict dict) {
          dict.entries().put(index, value);
        } else if (object instanceof Value.Array arr && index instanceof Value.IntVal(long idx)) {
          List<Value> elements = arr.elements();
          if (idx < 0 || idx >= elements.size()) {
            throw new AetherRuntimeException.IndexOutOfBounds(idx, elements.size());
          }
          List<Value> newElements = new ArrayList<>(elements);
          newElements.set((int) idx, value);
          if (objectExpr instanceof Expr.Identifier(String name)) {
            environment.set(name, new Value.Array(newElements));
          } else {
            arr.setElements(newElements);
          }
        } else {
          throw new AetherRuntimeException.InvalidOperation(
              "Index assignment requires array[int] or dict[key]");
        }
      }
      case Expr.Member(Expr objectExpr, String member) -> {
        Value object = evalExpr(objectExpr);
        if (!(object instanceof Value.Instance inst)) {
          throw new AetherRuntimeException.InvalidOperation(
              "Cannot assign field on type '" + object.typeName() + "'");
        }
        if (!inst.fields().containsKey(member)) {
          throw new AetherRuntimeException.InvalidOperation(
              "Struct '" + inst.typeName() + "' has no field '" + member + "'");
        }
        inst.fields().put(member, value);
      }
      default ->
          throw new AetherRuntimeException.InvalidOperation("Invalid assignment target");
    }
  }

  // ── Module loading ───────────────────────────────────────────────────────────

  private void loadModule(String moduleName) {
    Environment moduleEnv = resolveModule(moduleName);
    Map<String, Value> members = moduleEnv.localBindings();
    environment.define(moduleName, new Value.Module(moduleName, members));
  }

  private void loadModuleAs(String moduleName, String alias) {
    Environment moduleEnv = resolveModule(moduleName);
    Map<String, Value> members = moduleEnv.localBindings();
    environment.define(alias, new Value.Module(moduleName, members));
  }

  private void fromImport(String moduleName, List<String> items) {
    Environment moduleEnv = resolveModule(moduleName);
    for (String item : items) {
      environment.define(item, moduleEnv.get(item));
    }
  }

  private void fromImportAs(String moduleName, List<Stmt.ImportAlias> aliases) {
    Environment moduleEnv = resolveModule(moduleName);
    for (Stmt.ImportAlias alias : aliases) {
      environment.define(alias.alias(), moduleEnv.get(alias.item()));
    }
  }

  private Environment resolveModule(String moduleName) {
    if (moduleCache.containsKey(moduleName)) {
      return moduleCache.get(moduleName);
    }
    if (loadingStack.contains(moduleName)) {
      throw new AetherRuntimeException.InvalidOperation(
          "Circular dependency: " + moduleName);
    }
    loadingStack.push(moduleName);
    try {
      Environment moduleEnv = loadModuleFromFile(moduleName);
      moduleCache.put(moduleName, moduleEnv);
      return moduleEnv;
    } finally {
      loadingStack.pop();
    }
  }

  private Environment loadModuleFromFile(String moduleName) {
    // Try classpath (stdlib)
    String resourcePath = "stdlib/" + moduleName + ".ae";
    java.io.InputStream is =
        getClass().getClassLoader().getResourceAsStream(resourcePath);
    if (is != null) {
      return loadModuleFromSource(is, moduleName);
    }
    // Try filesystem
    java.nio.file.Path path = java.nio.file.Paths.get(moduleName + ".ae");
    if (java.nio.file.Files.exists(path)) {
      try {
        String source = java.nio.file.Files.readString(path);
        return executeInIsolation(source);
      } catch (java.io.IOException e) {
        throw new AetherRuntimeException.InvalidOperation(
            "Failed to read module '" + moduleName + "': " + e.getMessage());
      }
    }
    throw new AetherRuntimeException.InvalidOperation(
        "Module '" + moduleName + "' not found");
  }

  private Environment loadModuleFromSource(java.io.InputStream is, String moduleName) {
    try {
      String source = new String(is.readAllBytes(), java.nio.charset.StandardCharsets.UTF_8);
      return executeInIsolation(source);
    } catch (java.io.IOException e) {
      throw new AetherRuntimeException.InvalidOperation(
          "Failed to load module '" + moduleName + "': " + e.getMessage());
    }
  }

  private Environment executeInIsolation(String source) {
    com.aether.lexer.Scanner scanner = new com.aether.lexer.Scanner(source);
    com.aether.parser.Parser parser = new com.aether.parser.Parser(scanner.scanTokens());
    List<Stmt> stmts = parser.parse();

    Evaluator moduleEval = Evaluator.withoutStdlib();
    moduleEval.execute(stmts);
    return moduleEval.environment;
  }

  // ── Builtins registration ────────────────────────────────────────────────────

  private void registerBuiltins() {
    environment.define("print", Builtins.print());
    environment.define("println", Builtins.println());
    environment.define("input", Builtins.input());
    environment.define("read_file", Builtins.readFile());
    environment.define("write_file", Builtins.writeFile());
    environment.define("type", Builtins.type());
    environment.define("len", Builtins.len());
    environment.define("int", Builtins.toInt());
    environment.define("float", Builtins.toFloat());
    environment.define("str", Builtins.toStr());
    environment.define("bool", Builtins.toBool());
    environment.define("clock", Builtins.clock());
    environment.define("sleep", Builtins.sleep());
    environment.define("json_parse", Builtins.jsonParse());
    environment.define("json_stringify", Builtins.jsonStringify());
  }

  private void loadStdlib() {
    String[] modules = {"core", "collections", "math", "string", "testing"};
    for (String name : modules) {
      String resourcePath = "stdlib/" + name + ".ae";
      java.io.InputStream is =
          getClass().getClassLoader().getResourceAsStream(resourcePath);
      if (is == null) continue;
      try {
        String source = new String(is.readAllBytes(), java.nio.charset.StandardCharsets.UTF_8);
        List<Stmt> stmts =
            new com.aether.parser.Parser(new com.aether.lexer.Scanner(source).scanTokens()).parse();
        for (Stmt stmt : stmts) {
          ControlFlow cf = execStmtInternal(stmt);
          if (cf instanceof ControlFlow.Return) break;
        }
      } catch (java.io.IOException e) {
        System.err.println("Warning: Failed to load stdlib/" + name + ".ae: " + e.getMessage());
      }
    }
  }
}
