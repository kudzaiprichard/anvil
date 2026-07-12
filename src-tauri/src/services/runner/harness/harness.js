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

// LeetCode names every special node class `Node`, so exactly one variant can
// be injected per problem — chosen from the io types the meta declares
// (closing-the-48 Phase B). Signatures match leetcode.com's stubs.
const VARIANT_NODE_PRELUDES = {
  graph:
    "function Node(val, neighbors){ this.val = (val===undefined?0:val); this.neighbors = (neighbors===undefined?[]:neighbors); }\n",
  n_ary_tree:
    "function Node(val, children){ this.val = (val===undefined?null:val); this.children = (children===undefined?[]:children); }\n",
  quad_tree:
    "function Node(val, isLeaf, topLeft, topRight, bottomLeft, bottomRight){ this.val = (val===undefined?false:val); this.isLeaf = (isLeaf===undefined?false:isLeaf); this.topLeft = (topLeft===undefined?null:topLeft); this.topRight = (topRight===undefined?null:topRight); this.bottomLeft = (bottomLeft===undefined?null:bottomLeft); this.bottomRight = (bottomRight===undefined?null:bottomRight); }\n",
  next_tree:
    "function Node(val, left, right, next){ this.val = (val===undefined?0:val); this.left = (left===undefined?null:left); this.right = (right===undefined?null:right); this.next = (next===undefined?null:next); }\n",
  random_list:
    "function Node(val, next, random){ this.val = (val===undefined?0:val); this.next = (next===undefined?null:next); this.random = (random===undefined?null:random); }\n",
  multilevel_list:
    "function Node(val, prev, next, child){ this.val = (val===undefined?0:val); this.prev = (prev===undefined?null:prev); this.next = (next===undefined?null:next); this.child = (child===undefined?null:child); }\n",
};

function leafTypes(t, out) {
  if (t && typeof t === "object") {
    if ("list_of" in t) leafTypes(t.list_of, out);
    else if ("ctx_only" in t) leafTypes(t.ctx_only, out);
    return;
  }
  if (typeof t === "string") out.add(t);
}

function variantPrelude(meta) {
  // The extra `Node` constructor for this problem's solution scope, "" when
  // none of the declared io types needs one. Throws on a conflicting mix.
  const leaves = new Set();
  const io = meta.io_types || {};
  for (const t of io.params || []) leafTypes(t, leaves);
  leafTypes(io.returns === undefined ? "json" : io.returns, leaves);
  const dio = meta.design_io || {};
  for (const t of dio.ctor || []) leafTypes(t, leaves);
  for (const name of Object.keys(dio.methods || {})) {
    const mio = dio.methods[name] || {};
    for (const t of mio.params || []) leafTypes(t, leaves);
    if (mio.returns !== undefined) leafTypes(mio.returns, leaves);
  }
  const picks = Object.keys(VARIANT_NODE_PRELUDES).filter((k) => leaves.has(k));
  if (picks.length > 1) {
    throw new Error("conflicting Node class variants declared: " + picks.sort().join(", "));
  }
  return picks.length ? VARIANT_NODE_PRELUDES[picks[0]] : "";
}

// Harness-internal constructors for the closing-the-48 wire types; the
// solution scope gets its own `Node` via variantPrelude(). Everything is
// duck-typed on attribute names so constructor identity never matters.
function GraphNode(val) { this.val = val; this.neighbors = []; }
function NAryNode(val) { this.val = val; this.children = []; }
function QuadNode(val, isLeaf) {
  this.val = val;
  this.isLeaf = isLeaf;
  this.topLeft = null;
  this.topRight = null;
  this.bottomLeft = null;
  this.bottomRight = null;
}
function NextNode(val) { this.val = val; this.left = null; this.right = null; this.next = null; }
function RandomNode(val) { this.val = val; this.next = null; this.random = null; }
function MultilevelNode(val) { this.val = val; this.prev = null; this.next = null; this.child = null; }

function listKind(t) {
  return t && typeof t === "object" && "list_of" in t ? t.list_of : null;
}

function refParam(t, key) {
  if (t && typeof t === "object" && key in t) {
    const spec = t[key];
    if (spec && typeof spec === "object" && Number.isInteger(spec.param)) return spec.param;
  }
  return null;
}

function isCtxOnly(t) {
  return Boolean(t && typeof t === "object" && "ctx_only" in t);
}

function walkNodes(root) {
  // Generic identity-ordered walk over any node structure (list, tree,
  // graph, multilevel), cycle-safe. Returns each node exactly once.
  const seen = new Set();
  const out = [];
  const queue = [root];
  const LINK_ATTRS = ["next", "left", "right", "child", "random",
    "topLeft", "topRight", "bottomLeft", "bottomRight"];
  while (queue.length) {
    const node = queue.shift();
    if (node === null || node === undefined || seen.has(node)) continue;
    seen.add(node);
    out.push(node);
    for (const attr of LINK_ATTRS) {
      const c = node[attr];
      if (c !== null && c !== undefined && typeof c === "object") queue.push(c);
    }
    const group = node.neighbors || node.children || [];
    for (const n of group) if (n !== null && n !== undefined) queue.push(n);
  }
  return out;
}

function findNode(root, val) {
  for (const node of walkNodes(root)) {
    if (node.val === val) return node;
  }
  throw new Error("node_ref: no node with value " + JSON.stringify(val) + " in the referenced argument");
}

function chainNodes(head) {
  // The nodes of a (possibly cyclic) list in order, stopping at a revisit.
  const out = [];
  const seen = new Set();
  let node = head;
  while (node !== null && node !== undefined && !seen.has(node)) {
    seen.add(node);
    out.push(node);
    node = node.next;
  }
  return out;
}

function cloneStructure(node, memo) {
  // Structure-preserving deep copy of a binary tree.
  if (node === null || node === undefined) return null;
  memo = memo || new Map();
  if (memo.has(node)) return memo.get(node);
  const copy = new TreeNode(node.val);
  memo.set(node, copy);
  copy.left = cloneStructure(node.left, memo);
  copy.right = cloneStructure(node.right, memo);
  return copy;
}

function collectInputNodes(ctx) {
  const nodes = new Set();
  for (const arg of ctx || []) {
    if (arg !== null && typeof arg === "object" && "val" in arg) {
      for (const node of walkNodes(arg)) nodes.add(node);
    }
  }
  return nodes;
}

function checkFresh(value, ctx, what) {
  // Copy problems (clone-graph, copy-list-with-random-pointer) must return
  // entirely new nodes — returning any input node is the classic cheat.
  const inputNodes = collectInputNodes(ctx);
  for (const node of walkNodes(value)) {
    if (inputNodes.has(node)) {
      throw new Error("the returned " + what + " reuses a node from the input — return a deep copy");
    }
  }
}

function deserialize(value, t, ctx) {
  const inner = listKind(t);
  if (inner !== null) return (value || []).map((v) => deserialize(v, inner, ctx));
  if (t && typeof t === "object") {
    if ("ctx_only" in t) return deserialize(value, t.ctx_only, ctx);
    let p = refParam(t, "node_ref");
    if (p !== null) return value === null ? null : findNode((ctx || [])[p], value);
    p = refParam(t, "clone_of");
    if (p !== null) return cloneStructure((ctx || [])[p]);
    p = refParam(t, "tail_of");
    if (p !== null) {
      // {"values": [...], "attach": idx|null}: a fresh list whose last node
      // links to node #idx of the referenced argument's chain.
      const values = (value && value.values) || [];
      const attach = value ? value.attach : null;
      let head = null;
      if (attach !== null && attach !== undefined) {
        head = chainNodes((ctx || [])[p])[attach];
      }
      for (let i = values.length - 1; i >= 0; i--) head = new ListNode(values[i], head);
      return head;
    }
    return value;
  }
  if (t === "linked_list") {
    let head = null;
    const a = value || [];
    for (let i = a.length - 1; i >= 0; i--) head = new ListNode(a[i], head);
    return head;
  }
  if (t === "cyclic_list") {
    // {"values": [...], "pos": k|null}: tail links back to node #k.
    const values = (value && value.values) || [];
    const pos = value ? value.pos : null;
    const nodes = values.map((v) => new ListNode(v, null));
    for (let i = 0; i + 1 < nodes.length; i++) nodes[i].next = nodes[i + 1];
    if (pos !== null && pos !== undefined && nodes.length) {
      nodes[nodes.length - 1].next = nodes[pos];
    }
    return nodes.length ? nodes[0] : null;
  }
  if (t === "random_list") {
    // [[val, randomIdx|null], ...] — LeetCode's exact wire form.
    const pairs = value || [];
    const nodes = pairs.map((p) => new RandomNode(p[0]));
    for (let i = 0; i + 1 < nodes.length; i++) nodes[i].next = nodes[i + 1];
    pairs.forEach((p, i) => {
      if (p[1] !== null && p[1] !== undefined) nodes[i].random = nodes[p[1]];
    });
    return nodes.length ? nodes[0] : null;
  }
  if (t === "graph") {
    // Adjacency list; node i (0-based) has val i+1 (LeetCode convention).
    const adj = value || [];
    if (!adj.length) return null;
    const nodes = adj.map((_, i) => new GraphNode(i + 1));
    adj.forEach((neighbors, i) => {
      nodes[i].neighbors = neighbors.map((v) => nodes[v - 1]);
    });
    return nodes[0];
  }
  if (t === "tree" || t === "next_tree") {
    if (!value || value.length === 0) return null;
    const make = t === "tree" ? (v) => new TreeNode(v) : (v) => new NextNode(v);
    const root = make(value[0]);
    const queue = [root];
    let i = 1;
    while (queue.length && i < value.length) {
      const node = queue.shift();
      if (i < value.length) {
        if (value[i] !== null) { node.left = make(value[i]); queue.push(node.left); }
        i++;
      }
      if (i < value.length) {
        if (value[i] !== null) { node.right = make(value[i]); queue.push(node.right); }
        i++;
      }
    }
    return root;
  }
  if (t === "n_ary_tree") {
    // LeetCode level order with null group separators: [1,null,3,2,4,null,5,6].
    const vals = value || [];
    if (!vals.length) return null;
    const root = new NAryNode(vals[0]);
    const queue = [root];
    let i = 2; // skip the null right after the root
    while (queue.length && i < vals.length) {
      const node = queue.shift();
      while (i < vals.length && vals[i] !== null) {
        const child = new NAryNode(vals[i]);
        node.children.push(child);
        queue.push(child);
        i++;
      }
      i++; // the null closing this node's group
    }
    return root;
  }
  if (t === "quad_tree") {
    // Level order of [isLeaf, val] pairs with nulls, children tl/tr/bl/br.
    const vals = value || [];
    if (!vals.length || vals[0] === null) return null;
    const root = new QuadNode(Boolean(vals[0][1]), Boolean(vals[0][0]));
    const queue = [root];
    let i = 1;
    const attrs = ["topLeft", "topRight", "bottomLeft", "bottomRight"];
    while (queue.length && i < vals.length) {
      const node = queue.shift();
      for (const attr of attrs) {
        if (i >= vals.length) break;
        if (vals[i] !== null) {
          const child = new QuadNode(Boolean(vals[i][1]), Boolean(vals[i][0]));
          node[attr] = child;
          queue.push(child);
        }
        i++;
      }
    }
    return root;
  }
  if (t === "multilevel_list") {
    // Segments with a global-index parent: [{"values": [...], "parent":
    // null|int}, ...]. Nodes are numbered in segment order, left to right.
    const segments = value || [];
    const allNodes = [];
    const heads = [];
    for (const seg of segments) {
      const nodes = (seg.values || []).map((v) => new MultilevelNode(v));
      for (let i = 0; i + 1 < nodes.length; i++) {
        nodes[i].next = nodes[i + 1];
        nodes[i + 1].prev = nodes[i];
      }
      heads.push(nodes.length ? nodes[0] : null);
      for (const n of nodes) allNodes.push(n);
    }
    segments.forEach((seg, i) => {
      if (seg.parent !== null && seg.parent !== undefined && heads[i] !== null) {
        allNodes[seg.parent].child = heads[i];
      }
    });
    return heads.length ? heads[0] : null;
  }
  return value;
}

function serialize(value, t, ctx) {
  const inner = listKind(t);
  if (inner !== null) return (value || []).map((v) => serialize(v, inner, ctx));
  if (t && typeof t === "object") {
    let p = refParam(t, "node_ref");
    if (p !== null) {
      if (value === null || value === undefined) return null;
      if (typeof value !== "object" || !("val" in value)) {
        throw new Error("expected a node return value, got " + JSON.stringify(value));
      }
      for (const node of walkNodes((ctx || [])[p])) {
        if (node === value) return value.val;
      }
      throw new Error("the returned node is not part of the referenced input structure");
    }
    p = refParam(t, "node_index_of");
    if (p !== null) {
      if (value === null || value === undefined) return null;
      const chain = chainNodes((ctx || [])[p]);
      for (let i = 0; i < chain.length; i++) {
        if (chain[i] === value) return i;
      }
      throw new Error("the returned node is not part of the referenced input list");
    }
    if ("ctx_only" in t) return serialize(value, t.ctx_only, ctx);
    return value;
  }
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
  if (t === "random_list") {
    if (ctx) checkFresh(value, ctx, "list");
    const chain = chainNodes(value);
    const index = new Map();
    chain.forEach((n, i) => index.set(n, i));
    return chain.map((node) => {
      const r = node.random === undefined ? null : node.random;
      if (r !== null && !index.has(r)) {
        throw new Error("a random pointer leaves the returned list");
      }
      return [node.val, r === null ? null : index.get(r)];
    });
  }
  if (t === "graph") {
    if (value === null || value === undefined) return [];
    if (ctx) checkFresh(value, ctx, "graph");
    const nodes = walkNodes(value);
    const byVal = new Map();
    for (const node of nodes) {
      if (!Number.isInteger(node.val) || byVal.has(node.val)) {
        throw new Error("graph nodes must carry the unique 1..n values of the input");
      }
      byVal.set(node.val, node);
    }
    const n = nodes.length;
    const out = [];
    for (let v = 1; v <= n; v++) {
      if (!byVal.has(v)) {
        throw new Error("graph nodes must carry the unique 1..n values of the input");
      }
      out.push((byVal.get(v).neighbors || []).map((nb) => nb.val).sort((a, b) => a - b));
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
  if (t === "next_tree") {
    // Serialized BY FOLLOWING the next pointers (null closes each level),
    // so unset/wrong pointers fail even when the tree shape is right.
    const out = [];
    let head = value === undefined ? null : value;
    let seen = 0;
    while (head !== null && head !== undefined) {
      let node = head;
      let nextHead = null;
      while (node !== null && node !== undefined) {
        out.push(node.val);
        if (nextHead === null) {
          nextHead = node.left !== null && node.left !== undefined ? node.left : node.right;
          if (nextHead === undefined) nextHead = null;
        }
        node = node.next;
        if (++seen > 1000000) throw new Error("next-pointer chain too long (cycle?)");
      }
      out.push(null);
      head = nextHead;
    }
    return out;
  }
  if (t === "n_ary_tree") {
    if (value === null || value === undefined) return [];
    const out = [value.val, null];
    const queue = [value];
    while (queue.length) {
      const node = queue.shift();
      for (const child of node.children || []) {
        out.push(child.val);
        queue.push(child);
      }
      out.push(null);
    }
    while (out.length && out[out.length - 1] === null) out.pop();
    return out;
  }
  if (t === "quad_tree") {
    if (value === null || value === undefined) return [];
    const out = [];
    const queue = [value];
    while (queue.length) {
      const node = queue.shift();
      if (node === null || node === undefined) { out.push(null); continue; }
      out.push([node.isLeaf ? 1 : 0, node.val ? 1 : 0]);
      queue.push(node.topLeft === undefined ? null : node.topLeft);
      queue.push(node.topRight === undefined ? null : node.topRight);
      queue.push(node.bottomLeft === undefined ? null : node.bottomLeft);
      queue.push(node.bottomRight === undefined ? null : node.bottomRight);
    }
    while (out.length && out[out.length - 1] === null) out.pop();
    return out;
  }
  if (t === "multilevel_list") {
    if (value === null || value === undefined) return [];
    const segments = [];
    const queue = [[value, null]];
    let counter = 0;
    let seen = 0;
    while (queue.length) {
      const [head, parent] = queue.shift();
      const values = [];
      let node = head;
      while (node !== null && node !== undefined) {
        values.push(node.val);
        if (node.child !== null && node.child !== undefined) {
          queue.push([node.child, counter]);
        }
        counter++;
        node = node.next;
        if (++seen > 1000000) throw new Error("multilevel list too long (cycle?)");
      }
      segments.push({ values, parent });
    }
    return segments;
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
    // including design packs whose only io declaration is `design_io`. The
    // per-problem `Node` variant (graph, n-ary, …) rides along when declared.
    scope = loadSolutionScope(
      ioTypes || meta.design_io ? NODE_PRELUDE + variantPrelude(meta) : ""
    );
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
      // io_types: deserialize wire args into live node structures (task 0003
      // + closing-the-48 Phase B). Args build left-to-right into `built` so a
      // later param can reference an earlier one (node_ref/clone_of/tail_of);
      // ctx_only params are built for judging but never passed.
      let built = null;
      if (ioTypes) {
        built = [];
        try {
          for (let i = 0; i < callArgs.length; i++) {
            built.push(
              deserialize(callArgs[i], i < paramTypes.length ? paramTypes[i] : "json", built)
            );
          }
        } catch (err) {
          fail(index, "Failed to build node input:\n" + cleanStack(err));
          return;
        }
        callArgs = built.filter(
          (_, i) => !(i < paramTypes.length && isCtxOnly(paramTypes[i]))
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
        const source = ioTypes ? built : callArgs;
        if (argIndex >= source.length) {
          fail(
            index,
            "in_place judge: arg_index " + argIndex + " is out of range."
          );
          return;
        }
        result = source[argIndex];
        if (ioTypes) outType = argIndex < paramTypes.length ? paramTypes[argIndex] : "json";
      }
      if (ioTypes) {
        try {
          result = serialize(result, outType, built);
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
