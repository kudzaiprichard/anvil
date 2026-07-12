// Anvil test harness (embedded via include_str!, written into the run's
// temp dir next to solution.js and cases.json). Mirror of harness.py:
// same sentinel-line JSON protocol, same stop-on-first-error behavior.
//
//   @@ANVIL@@{"index": 1, "ok": true, "output": ..., "timeMs": 0.42}
//   @@ANVIL@@{"index": 2, "ok": false, "traceback": "..."}   (then stop)
//
// An optional meta.json configures entry-point resolution and execution
// mode (CONTENT_DESIGN.md §4–5); absent = the original behavior:
//   { "entry_point": "twoSum", "mode": "call" | "in_place" | "design" |
//     "any_valid", "arg_index": 0, "validator_file": "validator.js" }
"use strict";

const fs = require("fs");
const path = require("path");

const SENTINEL = "@@ANVIL@@";
const IDENT = /^[A-Za-z_$][A-Za-z0-9_$]*$/;
const CLASS_METHOD = /^[A-Za-z_$][A-Za-z0-9_$]*\.[A-Za-z_$][A-Za-z0-9_$]*$/;

function emit(obj) {
  process.stdout.write(SENTINEL + JSON.stringify(obj) + "\n");
}

// --- Node I/O adapter (CONTENT_DESIGN.md §5, task 0003) -------------------
// Mirror of harness.py: when meta.json declares `io_types`, (de)serialize
// ListNode/TreeNode params and return at the call boundary so a LeetCode stub
// runs unmodified. Wire form: array (linked list) / BFS level-order array with
// nulls (binary tree). The same definitions are injected into the solution
// scope (NODE_PRELUDE) so the user's code can `new ListNode(...)` etc.
function ListNode(val, next) {
  this.val = val === undefined ? 0 : val;
  this.next = next === undefined ? null : next;
}
function TreeNode(val, left, right) {
  this.val = val === undefined ? 0 : val;
  this.left = left === undefined ? null : left;
  this.right = right === undefined ? null : right;
}
const NODE_PRELUDE =
  "function ListNode(val, next){ this.val = (val===undefined?0:val); this.next = (next===undefined?null:next); }\n" +
  "function TreeNode(val, left, right){ this.val = (val===undefined?0:val); this.left = (left===undefined?null:left); this.right = (right===undefined?null:right); }\n";

function listKind(t) {
  return t && typeof t === "object" && "list_of" in t ? t.list_of : null;
}
function deserialize(value, t) {
  const inner = listKind(t);
  if (inner !== null) return (value || []).map((v) => deserialize(v, inner));
  if (t === "linked_list") {
    let head = null;
    const a = value || [];
    for (let i = a.length - 1; i >= 0; i--) head = new ListNode(a[i], head);
    return head;
  }
  if (t === "tree") {
    if (!value || value.length === 0) return null;
    const root = new TreeNode(value[0]);
    const queue = [root];
    let i = 1;
    while (queue.length && i < value.length) {
      const node = queue.shift();
      if (i < value.length) {
        if (value[i] !== null) { node.left = new TreeNode(value[i]); queue.push(node.left); }
        i++;
      }
      if (i < value.length) {
        if (value[i] !== null) { node.right = new TreeNode(value[i]); queue.push(node.right); }
        i++;
      }
    }
    return root;
  }
  return value;
}
function serialize(value, t) {
  const inner = listKind(t);
  if (inner !== null) return (value || []).map((v) => serialize(v, inner));
  if (t === "linked_list") {
    const out = [];
    let node = value, seen = 0;
    while (node !== null && node !== undefined) {
      out.push(node.val);
      node = node.next;
      if (++seen > 1000000) throw new Error("linked list too long (cycle?)");
    }
    return out;
  }
  if (t === "tree") {
    if (value === null || value === undefined) return [];
    const out = [];
    const queue = [value];
    while (queue.length) {
      const node = queue.shift();
      if (node === null || node === undefined) { out.push(null); continue; }
      out.push(node.val);
      queue.push(node.left === undefined ? null : node.left);
      queue.push(node.right === undefined ? null : node.right);
    }
    while (out.length && out[out.length - 1] === null) out.pop();
    return out;
  }
  return value;
}

// Drop harness frames so the user only sees their own code in the panel.
function cleanStack(err) {
  const stack = err && err.stack ? String(err.stack) : String(err);
  return stack
    .split("\n")
    .filter((line) => !line.includes("harness.js"))
    .join("\n");
}

function fail(index, message) {
  emit({ index, ok: false, traceback: message });
}

// Entry resolution: module.exports if it is a function, else an exported
// `solve`, else the first exported function. The Rust side also appends a
// `module.exports` shim to solution.js so the shipped starter
// (`function solve() {}`, no export) works without the user adding one.
function resolveLegacy(mod) {
  if (typeof mod === "function") return mod;
  if (mod && typeof mod === "object") {
    if (typeof mod.solve === "function") return mod.solve;
    for (const value of Object.values(mod)) {
      if (typeof value === "function") return value;
    }
  }
  return null;
}

// Evaluates solution.js inside a function scope and returns a lookup for
// its top-level bindings, so `var f = function…`, `function f(…)`,
// `const f = (…) =>`, and `class C {}` forms all resolve without the user
// adding an export. Names are validated against IDENT before reaching eval.
function loadSolutionScope(prelude) {
  const source = (prelude || "") + fs.readFileSync("solution.js", "utf8");
  const factory = new Function(
    "require",
    "module",
    "exports",
    source +
      "\n;return function (__anvilName) {" +
      "  try { return eval(__anvilName); } catch (e) { return undefined; }" +
      "};"
  );
  const mod = { exports: {} };
  const lookup = factory(require, mod, mod.exports);
  return { lookup, mod };
}

// Returns a per-case callable factory (mirrors harness.py: "Class.method"
// instantiates fresh per case). Throws Error with a friendly message.
function resolveEntry(scope, entry) {
  if (entry === null || entry === undefined) {
    const fn = resolveLegacy(scope.mod.exports);
    if (!fn) {
      throw new Error(
        "No function found in solution.js — define your solution as a function."
      );
    }
    return () => fn;
  }
  if (CLASS_METHOD.test(entry)) {
    const [clsName, method] = entry.split(".");
    const cls = scope.lookup(clsName);
    if (typeof cls !== "function") {
      throw new Error(
        "Entry point '" +
          entry +
          "' not found — expected a class named '" +
          clsName +
          "' in your code."
      );
    }
    if (
      typeof cls.prototype !== "object" ||
      typeof cls.prototype[method] !== "function"
    ) {
      throw new Error(
        "Entry point '" +
          entry +
          "' not found — class '" +
          clsName +
          "' has no method '" +
          method +
          "'."
      );
    }
    return () => {
      const instance = new cls();
      return instance[method].bind(instance);
    };
  }
  if (IDENT.test(entry)) {
    const fn = scope.lookup(entry);
    if (typeof fn !== "function") {
      throw new Error(
        "Entry point '" +
          entry +
          "' not found — define a function named '" +
          entry +
          "'."
      );
    }
    return () => fn;
  }
  throw new Error("Invalid entry point '" + entry + "'.");
}

// LeetCode wire format: args = [ops, argLists]. ops[0] names the class;
// instantiate with argLists[0], apply remaining ops, collect outputs
// (null for the constructor and void methods). Returns null after fail().
function applyParamTypes(values, types) {
  // Positional deserialization; params beyond the declared list stay JSON.
  return values.map((v, i) => deserialize(v, i < types.length ? types[i] : "json"));
}

function runDesign(scope, index, args, designIo) {
  // `designIo` (closing-the-48 Phase A) node-types the ctor/method boundary:
  // {ctor: [types], methods: {name: {params: [types], returns: type}}}; ops
  // absent from the map run all-JSON, exactly as before.
  if (
    !Array.isArray(args) ||
    args.length !== 2 ||
    !Array.isArray(args[0]) ||
    !Array.isArray(args[1])
  ) {
    fail(index, "Design case input must be [operations, argument_lists].");
    return null;
  }
  const [ops, argLists] = args;
  if (ops.length === 0 || ops.length !== argLists.length) {
    fail(index, "Design case operations and argument lists must align.");
    return null;
  }
  const ctorTypes = (designIo && designIo.ctor) || [];
  const methodIo = (designIo && designIo.methods) || {};
  const clsName = String(ops[0]);
  const cls = IDENT.test(clsName) ? scope.lookup(clsName) : undefined;
  if (typeof cls !== "function") {
    fail(index, "Class '" + clsName + "' not found in your code.");
    return null;
  }
  let instance;
  try {
    instance = new cls(...applyParamTypes(argLists[0], ctorTypes));
  } catch (err) {
    fail(
      index,
      "op 0 (" + clsName + " constructor) raised:\n" + cleanStack(err)
    );
    return null;
  }
  const outputs = [null];
  for (let i = 1; i < ops.length; i++) {
    const method = instance[String(ops[i])];
    if (typeof method !== "function") {
      fail(
        index,
        "op " + i + ": class '" + clsName + "' has no method '" + ops[i] + "'."
      );
      return null;
    }
    const io = methodIo[String(ops[i])] || {};
    let value;
    try {
      value = method.apply(instance, applyParamTypes(argLists[i], io.params || []));
    } catch (err) {
      fail(index, "op " + i + " (" + ops[i] + ") raised:\n" + cleanStack(err));
      return null;
    }
    if (io.returns) {
      try {
        value = serialize(value, io.returns);
      } catch (err) {
        fail(
          index,
          "op " + i + " (" + ops[i] + "): failed to serialize node output:\n" + cleanStack(err)
        );
        return null;
      }
    }
    outputs.push(value === undefined ? null : value);
  }
  return outputs;
}

function toJsonValue(value) {
  // `undefined` returns must serialize as explicit null so the Rust-side
  // comparison stays uniform with Python's None.
  const json = JSON.stringify(value === undefined ? null : value);
  return json === undefined ? null : JSON.parse(json);
}

function main() {
  const cases = JSON.parse(fs.readFileSync("cases.json", "utf8"));
  const meta = fs.existsSync("meta.json")
    ? JSON.parse(fs.readFileSync("meta.json", "utf8"))
    : {};
  const mode = meta.mode || "call";
  const ioTypes = meta.io_types || null;
  const paramTypes = ioTypes ? ioTypes.params || [] : [];
  const returnType = ioTypes ? (ioTypes.returns || "json") : "json";

  let scope;
  try {
    // Node classes must be in scope whenever any boundary is node-typed —
    // including design packs whose only io declaration is `design_io`.
    scope = loadSolutionScope(ioTypes || meta.design_io ? NODE_PRELUDE : "");
  } catch (err) {
    fail(0, cleanStack(err));
    return;
  }

  let validate = null;
  if (mode === "any_valid") {
    // Pack-shipped validator (our code, written by the Rust side — never
    // from the imported file). Must define validate(args, output).
    try {
      const v = require(path.resolve(meta.validator_file || "validator.js"));
      validate = typeof v === "function" ? v : v.validate;
      if (typeof validate !== "function") {
        throw new Error("validator exports no validate function");
      }
    } catch (err) {
      fail(0, "Validator failed to load:\n" + cleanStack(err));
      return;
    }
  }

  let getCallable = null;
  if (mode !== "design") {
    try {
      getCallable = resolveEntry(scope, meta.entry_point);
    } catch (err) {
      fail(0, err.message);
      return;
    }
  }

  for (const { index, args } of cases) {
    let payload;
    if (mode === "design") {
      const start = process.hrtime.bigint();
      const outputs = runDesign(scope, index, args, meta.design_io || null);
      if (outputs === null) return;
      const timeMs = Number(process.hrtime.bigint() - start) / 1e6;
      payload = { index, ok: true, output: outputs, timeMs };
    } else {
      const fn = getCallable();
      // any_valid: the validator judges against the ORIGINAL input, so the
      // solution gets a deep copy in case it mutates its arguments.
      let callArgs =
        mode === "any_valid" ? JSON.parse(JSON.stringify(args)) : args;
      // io_types: deserialize array/level-order args into ListNode/TreeNode so
      // the user's stub runs unmodified (task 0003).
      if (ioTypes) {
        callArgs = callArgs.map((a, i) =>
          deserialize(a, i < paramTypes.length ? paramTypes[i] : "json")
        );
      }
      const start = process.hrtime.bigint();
      let result;
      try {
        result = fn(...callArgs);
      } catch (err) {
        fail(index, cleanStack(err));
        return;
      }
      const timeMs = Number(process.hrtime.bigint() - start) / 1e6;
      let outType = returnType;
      if (mode === "in_place") {
        const argIndex = Number.isInteger(meta.arg_index) ? meta.arg_index : 0;
        if (argIndex >= callArgs.length) {
          fail(
            index,
            "in_place judge: arg_index " + argIndex + " is out of range."
          );
          return;
        }
        result = callArgs[argIndex];
        if (ioTypes) outType = argIndex < paramTypes.length ? paramTypes[argIndex] : "json";
      }
      if (ioTypes) {
        try {
          result = serialize(result, outType);
        } catch (err) {
          fail(index, "Failed to serialize node output:\n" + cleanStack(err));
          return;
        }
      }
      payload = { index, ok: true, output: result, timeMs };
      if (mode === "any_valid") {
        try {
          payload.valid = Boolean(validate(args, result));
        } catch (err) {
          fail(index, "Validator raised:\n" + cleanStack(err));
          return;
        }
      }
    }

    try {
      payload.output = toJsonValue(payload.output);
    } catch (err) {
      fail(index, "Return value is not JSON-serializable.");
      return;
    }
    emit(payload);
  }
}

main();
