# Anvil test harness (embedded via include_str!, written into the run's
# temp dir next to solution.py and cases.json).
#
# Protocol: one JSON line per case on stdout, prefixed with the sentinel
# "@@ANVIL@@" so user print() noise never breaks parsing:
#   @@ANVIL@@{"index": 1, "ok": true, "output": ..., "timeMs": 0.42}
#   @@ANVIL@@{"index": 2, "ok": false, "traceback": "..."}   (then stop)
#
# An optional meta.json next to cases.json configures entry-point
# resolution and execution mode (CONTENT_DESIGN.md §4–5):
#   { "entry_point": "Solution.twoSum",       # optional; absent = legacy solve
#     "mode": "call" | "in_place" | "design" | "any_valid",   # default "call"
#     "arg_index": 0,                          # in_place: which arg to emit
#     "validator_file": "validator.py" }       # any_valid: pack-shipped code
# No meta.json at all = the original behavior, byte for byte.
import copy
import inspect
import json
import os
import re
import sys
import time
import traceback

# Bundled pure-Python libs shipped with the app (e.g. sortedcontainers, which
# leetcode.com provides). The Rust runner sets ANVIL_PYLIB to the resource dir;
# prepend it so the prelude's optional imports resolve to the bundled copy.
_pylib = os.environ.get("ANVIL_PYLIB")
if _pylib and os.path.isdir(_pylib) and _pylib not in sys.path:
    sys.path.insert(0, _pylib)

SENTINEL = "@@ANVIL@@"
IDENT = r"[A-Za-z_][A-Za-z0-9_]*"


def emit(obj):
    sys.stdout.write(SENTINEL + json.dumps(obj) + "\n")
    sys.stdout.flush()


def clean_traceback(tb):
    # Drop harness frames (the "File .../harness.py" line and its source
    # line) so the user only sees their own code in the error panel.
    out = []
    skip_source = False
    for line in tb.splitlines():
        stripped = line.strip()
        if stripped.startswith('File "') and "harness.py" in stripped:
            skip_source = True
            continue
        if skip_source:
            skip_source = False
            if line.startswith((" ", "\t")) and not stripped.startswith('File "'):
                continue
        out.append(line)
    return "\n".join(out)


def fail(index, message):
    emit({"index": index, "ok": False, "traceback": message})


def resolve_legacy(module):
    # Prefer a function named `solve` (the shipped signature convention);
    # otherwise the first function defined at the top level of solution.py.
    functions = [
        value
        for value in vars(module).values()
        if inspect.isfunction(value) and value.__module__ == module.__name__
    ]
    for fn in functions:
        if fn.__name__ == "solve":
            return fn
    return functions[0] if functions else None


def resolve_entry(module, entry):
    # Returns a zero-state factory: calling it gives a fresh callable per
    # case ("Class.method" instantiates the class each time so no state
    # leaks between cases). Raises LookupError with a friendly message.
    if entry is None:
        fn = resolve_legacy(module)
        if fn is None:
            raise LookupError(
                "No function found in solution.py — define your solution as a function."
            )
        return lambda: fn

    if re.fullmatch(IDENT + r"\." + IDENT, entry):
        cls_name, method = entry.split(".")
        cls = getattr(module, cls_name, None)
        if not inspect.isclass(cls):
            raise LookupError(
                "Entry point '%s' not found — expected a class named '%s' in your code."
                % (entry, cls_name)
            )
        if not callable(getattr(cls, method, None)):
            raise LookupError(
                "Entry point '%s' not found — class '%s' has no method '%s'."
                % (entry, cls_name, method)
            )
        return lambda: getattr(cls(), method)

    if re.fullmatch(IDENT, entry):
        fn = getattr(module, entry, None)
        if not callable(fn):
            raise LookupError(
                "Entry point '%s' not found — define a function named '%s'." % (entry, entry)
            )
        return lambda: fn

    raise LookupError("Invalid entry point %r." % entry)


def resolve_codec(module, entry, cfg):
    # Round-trip codecs come in two shapes: a class (entry names it; LeetCode's
    # `Codec`) whose instance carries the encode/decode methods, or two
    # top-level functions. A fresh instance is made per case so no state leaks.
    enc_name = cfg.get("encode", "encode")
    dec_name = cfg.get("decode", "decode")
    # The pack's entry point may be "Codec" or the derived "Codec.serialize" —
    # only the class part matters here.
    entry = (entry or "").split(".")[0]
    cls = getattr(module, entry, None) if entry and re.fullmatch(IDENT, entry) else None
    if inspect.isclass(cls):
        def make():
            inst = cls()
            return getattr(inst, enc_name), getattr(inst, dec_name)
        # Validate the methods exist up front for a friendly message.
        probe = cls()
        for name in (enc_name, dec_name):
            if not callable(getattr(probe, name, None)):
                raise LookupError("Class '%s' has no method '%s'." % (entry, name))
        return make
    enc = getattr(module, enc_name, None)
    dec = getattr(module, dec_name, None)
    if callable(enc) and callable(dec):
        return lambda: (enc, dec)
    raise LookupError(
        "Codec not found — define a class '%s' with %s/%s methods, or top-level "
        "functions named '%s' and '%s'." % (entry, enc_name, dec_name, enc_name, dec_name)
    )


def run_round_trip(index, args, cfg, make_codec):
    # The solver invents the format, so the only checkable contract is
    # decode(encode(x)) == x: emit the canonical serialization of the
    # round-tripped structure and let the exact compare do the rest. The
    # intermediate encoding never leaves this process.
    io = cfg.get("io", "json")
    try:
        x = deserialize(args[0] if args else None, io)
    except BaseException:
        fail(index, "Failed to build node input:\n" + clean_traceback(traceback.format_exc()))
        return None
    encode_fn, decode_fn = make_codec()
    start = time.perf_counter()
    try:
        decoded = decode_fn(encode_fn(x))
    except BaseException:
        fail(index, clean_traceback(traceback.format_exc()))
        return None
    elapsed_ms = (time.perf_counter() - start) * 1000.0
    try:
        result = serialize(decoded, io)
    except BaseException:
        fail(index, "Failed to serialize node output:\n" + clean_traceback(traceback.format_exc()))
        return None
    return result, elapsed_ms


def _apply_param_types(values, types):
    # Positional deserialization; params beyond the declared list stay JSON.
    return [
        deserialize(v, types[i] if i < len(types) else "json")
        for i, v in enumerate(values)
    ]


def run_design(module, index, args, design_io=None):
    # LeetCode wire format: args = [ops, argLists]. ops[0] names the class;
    # instantiate with argLists[0], apply remaining ops, collect outputs
    # (None for the constructor and void methods). `design_io` (closing-the-48
    # Phase A) node-types the ctor/method boundary: {"ctor": [types],
    # "methods": {name: {"params": [types], "returns": type}}}; ops absent
    # from the map run all-JSON, exactly as before.
    if len(args) != 2 or not isinstance(args[0], list) or not isinstance(args[1], list):
        fail(index, "Design case input must be [operations, argument_lists].")
        return None
    ops, arg_lists = args
    if not ops or len(ops) != len(arg_lists):
        fail(index, "Design case operations and argument lists must align.")
        return None
    ctor_types = (design_io or {}).get("ctor") or []
    method_io = (design_io or {}).get("methods") or {}
    cls_name = ops[0]
    cls = getattr(module, cls_name, None) if re.fullmatch(IDENT, str(cls_name)) else None
    if not inspect.isclass(cls):
        fail(index, "Class '%s' not found in your code." % cls_name)
        return None
    try:
        instance = cls(*_apply_param_types(arg_lists[0], ctor_types))
    except BaseException:
        fail(
            index,
            "op 0 (%s constructor) raised:\n%s"
            % (cls_name, clean_traceback(traceback.format_exc())),
        )
        return None
    outputs = [None]
    for i in range(1, len(ops)):
        method = getattr(instance, str(ops[i]), None)
        if not callable(method):
            fail(index, "op %d: class '%s' has no method '%s'." % (i, cls_name, ops[i]))
            return None
        io = method_io.get(str(ops[i])) or {}
        try:
            result = method(*_apply_param_types(arg_lists[i], io.get("params") or []))
        except BaseException:
            fail(
                index,
                "op %d (%s) raised:\n%s"
                % (i, ops[i], clean_traceback(traceback.format_exc())),
            )
            return None
        if io.get("returns"):
            try:
                result = serialize(result, io["returns"])
            except BaseException:
                fail(
                    index,
                    "op %d (%s): failed to serialize node output:\n%s"
                    % (i, ops[i], clean_traceback(traceback.format_exc())),
                )
                return None
        outputs.append(result)
    return outputs


# --- Node I/O adapter (CONTENT_DESIGN.md §5, task 0003) -------------------
# When meta.json declares `io_types`, the harness (de)serializes ListNode /
# TreeNode parameters and return values at the call boundary, so a LeetCode
# stub referencing those classes runs unmodified. The canonical wire form is an
# array (linked list) / BFS level-order array with nulls (binary tree).
# Serialization is duck-typed (.val/.next, .val/.left/.right) so it does not
# matter which class object the solution constructed.
# Injected before EVERY solution (not just node problems) so LeetCode-style
# stubs run unmodified — their signatures use `List[int]`, `Optional[...]`, etc.
# and their bodies reach for `collections`, `math`, `heapq`, … without imports,
# exactly as the leetcode.com judge environment provides them. ListNode/TreeNode
# are defined here too so node stubs resolve those names at definition time.
NODE_PRELUDE = """
import sys as _sys
_sys.setrecursionlimit(100000)
from typing import *
import collections
from collections import Counter, defaultdict, deque, OrderedDict
import math
import heapq
import bisect
import itertools
import functools
import operator
import re
import string
import random
# sortedcontainers is available on leetcode.com; expose it when installed so
# idiomatic SortedList/SortedDict/SortedSet solutions run, without hard-failing
# the (stdlib-only) sandbox when it is absent.
try:
    from sortedcontainers import SortedList, SortedDict, SortedSet
except ImportError:
    pass


class ListNode:
    def __init__(self, val=0, next=None):
        self.val = val
        self.next = next


class TreeNode:
    def __init__(self, val=0, left=None, right=None):
        self.val = val
        self.left = left
        self.right = right
"""

# LeetCode names every special node class `Node`, so exactly one variant can
# be injected per problem — chosen from the io types the meta declares
# (closing-the-48 Phase B). Signatures match leetcode.com's stubs.
VARIANT_NODE_PRELUDES = {
    "graph": """
class Node:
    def __init__(self, val=0, neighbors=None):
        self.val = val
        self.neighbors = neighbors if neighbors is not None else []
""",
    "n_ary_tree": """
class Node:
    def __init__(self, val=None, children=None):
        self.val = val
        self.children = children if children is not None else []
""",
    "quad_tree": """
class Node:
    def __init__(self, val=False, isLeaf=False, topLeft=None, topRight=None, bottomLeft=None, bottomRight=None):
        self.val = val
        self.isLeaf = isLeaf
        self.topLeft = topLeft
        self.topRight = topRight
        self.bottomLeft = bottomLeft
        self.bottomRight = bottomRight
""",
    "next_tree": """
class Node:
    def __init__(self, val=0, left=None, right=None, next=None):
        self.val = val
        self.left = left
        self.right = right
        self.next = next
""",
    "random_list": """
class Node:
    def __init__(self, x=0, next=None, random=None):
        self.val = x
        self.next = next
        self.random = random
""",
    "multilevel_list": """
class Node:
    def __init__(self, val=0, prev=None, next=None, child=None):
        self.val = val
        self.prev = prev
        self.next = next
        self.child = child
""",
}


def _leaf_types(t, out):
    if isinstance(t, dict):
        if "list_of" in t:
            _leaf_types(t["list_of"], out)
        elif "ctx_only" in t:
            _leaf_types(t["ctx_only"], out)
        return
    if isinstance(t, str):
        out.add(t)


def variant_prelude(meta):
    # The extra `class Node` source for this problem's solution scope, "" when
    # none of the declared io types needs one. Raises on a conflicting mix
    # (an authoring error — one problem never uses two Node shapes).
    leaves = set()
    io = meta.get("io_types") or {}
    for t in io.get("params") or []:
        _leaf_types(t, leaves)
    _leaf_types(io.get("returns", "json"), leaves)
    dio = meta.get("design_io") or {}
    for t in dio.get("ctor") or []:
        _leaf_types(t, leaves)
    for mio in (dio.get("methods") or {}).values():
        for t in mio.get("params") or []:
            _leaf_types(t, leaves)
        _leaf_types(mio.get("returns", "json"), leaves)
    rt = meta.get("round_trip") or {}
    if "io" in rt:
        _leaf_types(rt["io"], leaves)
    picks = [k for k in VARIANT_NODE_PRELUDES if k in leaves]
    if len(picks) > 1:
        raise RuntimeError("conflicting Node class variants declared: %s" % ", ".join(sorted(picks)))
    return VARIANT_NODE_PRELUDES[picks[0]] if picks else ""


# The harness's own node classes, used by deserialize() to build inputs. The
# solution gets its own copies via NODE_PRELUDE; serialize() is duck-typed
# (.val/.next, .val/.left/.right) so the two class identities never matter.
class ListNode:
    def __init__(self, val=0, next=None):
        self.val = val
        self.next = next


class TreeNode:
    def __init__(self, val=0, left=None, right=None):
        self.val = val
        self.left = left
        self.right = right


# Harness-internal node classes for the closing-the-48 wire types. The
# solution scope gets its own `Node` class via _variant_prelude(); everything
# here is duck-typed on attribute names so class identity never matters.
class GraphNode:
    def __init__(self, val=0, neighbors=None):
        self.val = val
        self.neighbors = neighbors if neighbors is not None else []


class NAryNode:
    def __init__(self, val=None, children=None):
        self.val = val
        self.children = children if children is not None else []


class QuadNode:
    def __init__(self, val=False, isLeaf=False, topLeft=None, topRight=None, bottomLeft=None, bottomRight=None):
        self.val = val
        self.isLeaf = isLeaf
        self.topLeft = topLeft
        self.topRight = topRight
        self.bottomLeft = bottomLeft
        self.bottomRight = bottomRight


class NextNode:
    def __init__(self, val=0, left=None, right=None, next=None):
        self.val = val
        self.left = left
        self.right = right
        self.next = next


class RandomNode:
    def __init__(self, val=0, next=None, random=None):
        self.val = val
        self.next = next
        self.random = random


class MultilevelNode:
    def __init__(self, val=0, prev=None, next=None, child=None):
        self.val = val
        self.prev = prev
        self.next = next
        self.child = child


# --- Injected-object shims (closing-the-48 Phase D) -----------------------
# A fixed menu of harness-owned wrappers around per-case hidden data, declared
# as the param type {"shim": {"kind": "<name>"}}. Three kinds are installed as
# globals in the solution scope instead of being passed positionally (their
# param slot only carries the hidden data): is_bad_version, guess_oracle,
# rand7. Shims count calls and enforce the problem's stated budgets.
SHIM_GLOBAL_NAMES = {
    "is_bad_version": "isBadVersion",
    "guess_oracle": "guess",
    "rand7": "rand7",
}


class _MountainArray:
    def __init__(self, arr, budget):
        self._arr = list(arr)
        self._budget = budget
        self._gets = 0

    def get(self, k):
        self._gets += 1
        if self._budget is not None and self._gets > self._budget:
            raise RuntimeError(
                "MountainArray.get call budget (%d) exceeded" % self._budget
            )
        return self._arr[k]

    def length(self):
        return len(self._arr)


class _Master:
    def __init__(self, secret, words, budget):
        self._secret = secret
        self._words = set(words)
        self._budget = budget
        self._calls = 0
        self._correct = False

    def guess(self, word):
        self._calls += 1
        if self._budget is not None and self._calls > self._budget:
            raise RuntimeError("Master.guess call budget (%d) exceeded" % self._budget)
        if word == self._secret:
            self._correct = True
        if word not in self._words:
            return -1
        return sum(a == b for a, b in zip(word, self._secret))

    def _anvil_verdict(self):
        # The case's real output: was the secret guessed within budget?
        return self._correct


class _NestedInteger:
    def __init__(self, value):
        if isinstance(value, list):
            self._int = None
            self._list = [_NestedInteger(v) for v in value]
        else:
            self._int = value
            self._list = None

    def isInteger(self):
        return self._int is not None

    def getInteger(self):
        return self._int

    def getList(self):
        return self._list


class _Iterator:
    def __init__(self, data):
        self._data = list(data)
        self._i = 0

    def hasNext(self):
        return self._i < len(self._data)

    def next(self):
        v = self._data[self._i]
        self._i += 1
        return v


# f(x, y) formulas for the CustomFunction shim, keyed by function_id — all
# strictly increasing in x and y (the problem's contract). Identical in
# harness.js.
CUSTOM_FUNCTIONS = {
    1: lambda x, y: x + y,
    2: lambda x, y: x * y,
    3: lambda x, y: x * x + y,
    4: lambda x, y: x + y * y,
    5: lambda x, y: x * x + y * y,
    6: lambda x, y: x * x * x + y,
    7: lambda x, y: x + y * y * y,
    8: lambda x, y: 2 * x + y,
    9: lambda x, y: x + 2 * y,
}


class _CustomFunction:
    def __init__(self, function_id):
        if function_id not in CUSTOM_FUNCTIONS:
            raise RuntimeError("unknown custom_function id %r" % (function_id,))
        self._fn = CUSTOM_FUNCTIONS[function_id]

    def f(self, x, y):
        return self._fn(x, y)


def _build_shim(spec, value):
    kind = spec.get("kind")
    if kind == "is_bad_version":
        bad = value

        def isBadVersion(version):
            return version >= bad

        return isBadVersion
    if kind == "guess_oracle":
        pick = value

        def guess(num):
            if num == pick:
                return 0
            return -1 if num > pick else 1

        return guess
    if kind == "rand7":
        # Deterministic Park-Miller LCG (identical in harness.js — the
        # multiplier keeps products under 2^53 so JS doubles stay exact) so
        # everything that is NOT property-judged cross-checks byte-identically.
        state = [int(value or 1) % 2147483647 or 1]

        def rand7():
            state[0] = (state[0] * 48271) % 2147483647
            return state[0] % 7 + 1

        return rand7
    if kind == "mountain_array":
        v = value or {}
        return _MountainArray(v.get("arr") or [], v.get("budget"))
    if kind == "master_guess":
        v = value or {}
        return _Master(v.get("secret", ""), v.get("words") or [], v.get("budget"))
    if kind == "custom_function":
        return _CustomFunction(value)
    if kind == "iterator":
        return _Iterator(value or [])
    if kind == "nested_integer":
        # Top-level wire value is a list => a List[NestedInteger] argument.
        if isinstance(value, list):
            return [_NestedInteger(v) for v in value]
        return _NestedInteger(value)
    raise RuntimeError("unknown shim kind %r" % (kind,))


def _shim_spec(t):
    if isinstance(t, dict) and "shim" in t and isinstance(t["shim"], dict):
        return t["shim"]
    return None


def _list_kind(t):
    # A composite list type is {"list_of": <inner>}; everything else is a leaf.
    return t.get("list_of") if isinstance(t, dict) else None


def _ref_param(t, key):
    # {"node_ref": {"param": i}} and friends → i, else None.
    if isinstance(t, dict) and key in t:
        spec = t[key]
        if isinstance(spec, dict):
            return spec.get("param")
    return None


def _walk_nodes(root):
    # Generic identity-ordered walk over any node structure (list, tree,
    # graph, multilevel), cycle-safe. Yields each node exactly once.
    seen = set()
    stack = [root]
    while stack:
        node = stack.pop(0)
        if node is None or id(node) in seen:
            continue
        seen.add(id(node))
        yield node
        for attr in ("next", "left", "right", "child", "random",
                     "topLeft", "topRight", "bottomLeft", "bottomRight"):
            child = getattr(node, attr, None)
            if child is not None:
                stack.append(child)
        for n in getattr(node, "neighbors", None) or getattr(node, "children", None) or []:
            if n is not None:
                stack.append(n)


def _find_node(root, val):
    for node in _walk_nodes(root):
        if node.val == val:
            return node
    raise RuntimeError("node_ref: no node with value %r in the referenced argument" % (val,))


def _chain_nodes(head):
    # The nodes of a (possibly cyclic) list in order, stopping at a revisit.
    out, seen = [], set()
    node = head
    while node is not None and id(node) not in seen:
        seen.add(id(node))
        out.append(node)
        node = node.next
    return out


def _clone_structure(node, memo=None):
    # Structure-preserving deep copy of a binary tree (find-corresponding-
    # node's clone argument).
    if node is None:
        return None
    if memo is None:
        memo = {}
    if id(node) in memo:
        return memo[id(node)]
    copy_node = TreeNode(node.val)
    memo[id(node)] = copy_node
    copy_node.left = _clone_structure(getattr(node, "left", None), memo)
    copy_node.right = _clone_structure(getattr(node, "right", None), memo)
    return copy_node


def deserialize(value, t, ctx=None):
    inner = _list_kind(t)
    if inner is not None:
        return [deserialize(v, inner, ctx) for v in value]
    if isinstance(t, dict):
        if "ctx_only" in t:
            # ctx_only wraps a real type: built into the context, not passed.
            return deserialize(value, t["ctx_only"], ctx)
        spec = _shim_spec(t)
        if spec is not None:
            return _build_shim(spec, value)
        p = _ref_param(t, "node_ref")
        if p is not None:
            return None if value is None else _find_node((ctx or [])[p], value)
        p = _ref_param(t, "clone_of")
        if p is not None:
            return _clone_structure((ctx or [])[p])
        p = _ref_param(t, "tail_of")
        if p is not None:
            # {"values": [...], "attach": idx|null}: a fresh list whose last
            # node links to node #idx of the referenced argument's chain.
            values = (value or {}).get("values", [])
            attach = (value or {}).get("attach")
            tail = None
            if attach is not None:
                chain = _chain_nodes((ctx or [])[p])
                tail = chain[attach]
            head = tail
            for v in reversed(values):
                head = ListNode(v, head)
            return head
        return value
    if t == "linked_list":
        head = None
        for v in reversed(value or []):
            head = ListNode(v, head)
        return head
    if t == "cyclic_list":
        # {"values": [...], "pos": k|null}: tail links back to node #k.
        values = (value or {}).get("values", [])
        pos = (value or {}).get("pos")
        nodes = [ListNode(v) for v in values]
        for a, b in zip(nodes, nodes[1:]):
            a.next = b
        if pos is not None and nodes:
            nodes[-1].next = nodes[pos]
        return nodes[0] if nodes else None
    if t == "random_list":
        # [[val, randomIdx|null], ...] — LeetCode's exact wire form.
        pairs = value or []
        nodes = [RandomNode(p[0]) for p in pairs]
        for a, b in zip(nodes, nodes[1:]):
            a.next = b
        for node, p in zip(nodes, pairs):
            if p[1] is not None:
                node.random = nodes[p[1]]
        return nodes[0] if nodes else None
    if t == "graph":
        # Adjacency list; node i (0-based) has val i+1 (LeetCode convention).
        adj = value or []
        if not adj:
            return None
        nodes = [GraphNode(i + 1) for i in range(len(adj))]
        for node, neighbors in zip(nodes, adj):
            node.neighbors = [nodes[v - 1] for v in neighbors]
        return nodes[0]
    if t == "tree" or t == "next_tree":
        if not value:
            return None
        make = TreeNode if t == "tree" else NextNode
        root = make(value[0])
        queue = [root]
        i = 1
        while queue and i < len(value):
            node = queue.pop(0)
            if i < len(value):
                if value[i] is not None:
                    node.left = make(value[i])
                    queue.append(node.left)
                i += 1
            if i < len(value):
                if value[i] is not None:
                    node.right = make(value[i])
                    queue.append(node.right)
                i += 1
        return root
    if t == "n_ary_tree":
        # LeetCode level order with null group separators: [1,null,3,2,4,null,5,6].
        vals = value or []
        if not vals:
            return None
        root = NAryNode(vals[0])
        queue = [root]
        i = 2  # skip the null right after the root
        while queue and i < len(vals):
            node = queue.pop(0)
            while i < len(vals) and vals[i] is not None:
                child = NAryNode(vals[i])
                node.children.append(child)
                queue.append(child)
                i += 1
            i += 1  # the null closing this node's group
        return root
    if t == "quad_tree":
        # Level order of [isLeaf, val] pairs with nulls, children tl/tr/bl/br.
        vals = value or []
        if not vals or vals[0] is None:
            return None
        root = QuadNode(val=bool(vals[0][1]), isLeaf=bool(vals[0][0]))
        queue = [root]
        i = 1
        attrs = ("topLeft", "topRight", "bottomLeft", "bottomRight")
        while queue and i < len(vals):
            node = queue.pop(0)
            for attr in attrs:
                if i >= len(vals):
                    break
                if vals[i] is not None:
                    child = QuadNode(val=bool(vals[i][1]), isLeaf=bool(vals[i][0]))
                    setattr(node, attr, child)
                    queue.append(child)
                i += 1
        return root
    if t == "multilevel_list":
        # Segments with a global-index parent: [{"values": [...], "parent":
        # null|int}, ...]. Nodes are numbered in segment order, left to right.
        segments = value or []
        all_nodes = []
        heads = []
        for seg in segments:
            nodes = [MultilevelNode(v) for v in seg.get("values", [])]
            for a, b in zip(nodes, nodes[1:]):
                a.next = b
                b.prev = a
            heads.append(nodes[0] if nodes else None)
            all_nodes.extend(nodes)
        for seg, head in zip(segments, heads):
            parent = seg.get("parent")
            if parent is not None and head is not None:
                all_nodes[parent].child = head
        return heads[0] if heads else None
    return value  # "json"


def _collect_input_node_ids(ctx):
    ids = set()
    for arg in ctx or []:
        if hasattr(arg, "val"):
            for node in _walk_nodes(arg):
                ids.add(id(node))
    return ids


def _check_fresh(value, ctx, what):
    # Copy problems (clone-graph, copy-list-with-random-pointer) must return
    # entirely new nodes — returning any input node is the classic cheat.
    input_ids = _collect_input_node_ids(ctx)
    for node in _walk_nodes(value):
        if id(node) in input_ids:
            raise RuntimeError(
                "the returned %s reuses a node from the input — return a deep copy" % what
            )


def serialize(value, t, ctx=None):
    inner = _list_kind(t)
    if inner is not None:
        return [serialize(v, inner, ctx) for v in value]
    if isinstance(t, dict):
        p = _ref_param(t, "node_ref")
        if p is not None:
            if value is None:
                return None
            if not hasattr(value, "val"):
                raise RuntimeError("expected a node return value, got %r" % (value,))
            for node in _walk_nodes((ctx or [])[p]):
                if node is value:
                    return value.val
            raise RuntimeError("the returned node is not part of the referenced input structure")
        p = _ref_param(t, "node_index_of")
        if p is not None:
            if value is None:
                return None
            chain = _chain_nodes((ctx or [])[p])
            for i, node in enumerate(chain):
                if node is value:
                    return i
            raise RuntimeError("the returned node is not part of the referenced input list")
        if "ctx_only" in t:
            return serialize(value, t["ctx_only"], ctx)
        return value
    if t == "linked_list":
        out = []
        node = value
        seen = 0
        while node is not None:
            out.append(node.val)
            node = node.next
            seen += 1
            if seen > 1_000_000:
                raise RuntimeError("linked list too long (cycle?)")
        return out
    if t == "random_list":
        if ctx is not None:
            _check_fresh(value, ctx, "list")
        chain = _chain_nodes(value)
        index = {id(n): i for i, n in enumerate(chain)}
        out = []
        for node in chain:
            r = getattr(node, "random", None)
            if r is not None and id(r) not in index:
                raise RuntimeError("a random pointer leaves the returned list")
            out.append([node.val, None if r is None else index[id(r)]])
        return out
    if t == "graph":
        if value is None:
            return []
        if ctx is not None:
            _check_fresh(value, ctx, "graph")
        nodes = list(_walk_nodes(value))
        by_val = {}
        for node in nodes:
            if not isinstance(node.val, int) or node.val in by_val:
                raise RuntimeError("graph nodes must carry the unique 1..n values of the input")
            by_val[node.val] = node
        n = len(nodes)
        if sorted(by_val) != list(range(1, n + 1)):
            raise RuntimeError("graph nodes must carry the unique 1..n values of the input")
        out = []
        for v in range(1, n + 1):
            out.append(sorted(nb.val for nb in by_val[v].neighbors or []))
        return out
    if t == "tree":
        if value is None:
            return []
        out = []
        queue = [value]
        while queue:
            node = queue.pop(0)
            if node is None:
                out.append(None)
                continue
            out.append(node.val)
            queue.append(node.left)
            queue.append(node.right)
        while out and out[-1] is None:
            out.pop()
        return out
    if t == "next_tree":
        # Serialized BY FOLLOWING the next pointers (null closes each level),
        # so unset/wrong pointers fail even when the tree shape is right.
        out = []
        head = value
        seen = 0
        while head is not None:
            node = head
            next_head = None
            while node is not None:
                out.append(node.val)
                if next_head is None:
                    next_head = node.left if node.left is not None else node.right
                node = node.next
                seen += 1
                if seen > 1_000_000:
                    raise RuntimeError("next-pointer chain too long (cycle?)")
            out.append(None)
            head = next_head
        return out
    if t == "n_ary_tree":
        if value is None:
            return []
        out = [value.val, None]
        queue = [value]
        while queue:
            node = queue.pop(0)
            for child in node.children or []:
                out.append(child.val)
                queue.append(child)
            out.append(None)
        while out and out[-1] is None:
            out.pop()
        return out
    if t == "quad_tree":
        if value is None:
            return []
        out = []
        queue = [value]
        while queue:
            node = queue.pop(0)
            if node is None:
                out.append(None)
                continue
            out.append([1 if node.isLeaf else 0, 1 if node.val else 0])
            queue.append(node.topLeft)
            queue.append(node.topRight)
            queue.append(node.bottomLeft)
            queue.append(node.bottomRight)
        while out and out[-1] is None:
            out.pop()
        return out
    if t == "multilevel_list":
        if value is None:
            return []
        segments = []
        queue = [(value, None)]
        numbered = {}
        counter = 0
        while queue:
            head, parent = queue.pop(0)
            values = []
            node = head
            seen = 0
            while node is not None:
                numbered[id(node)] = counter
                values.append(node.val)
                if node.child is not None:
                    queue.append((node.child, counter))
                counter += 1
                node = node.next
                seen += 1
                if seen > 1_000_000:
                    raise RuntimeError("multilevel list too long (cycle?)")
            segments.append({"values": values, "parent": parent})
        return segments
    return value  # "json"


def _is_ctx_only(t):
    if isinstance(t, dict) and "ctx_only" in t:
        return True
    # Global-installed shims occupy a param slot for their hidden data but
    # are never passed positionally.
    spec = _shim_spec(t)
    return spec is not None and spec.get("kind") in SHIM_GLOBAL_NAMES


def load_solution_with_prelude(extra_prelude=""):
    # Inject the prelude (typing names, common modules, ListNode/TreeNode, plus
    # the per-problem `Node` variant when one is declared) into a fresh
    # `solution` module namespace, THEN exec the user's code as its own
    # compilation unit so annotations like `List[int]` / `Optional[ListNode]`
    # resolve at definition time (as on leetcode.com) AND traceback line numbers
    # still point at the real line in solution.py (no prelude offset).
    import types as _types

    with open("solution.py", "r", encoding="utf-8") as f:
        src = f.read()
    mod = _types.ModuleType("solution")
    exec(compile(NODE_PRELUDE + extra_prelude, "<prelude>", "exec"), mod.__dict__)
    exec(compile(src, "solution.py", "exec"), mod.__dict__)
    return mod


def main():
    with open("cases.json", "r", encoding="utf-8") as f:
        cases = json.load(f)
    meta = {}
    if os.path.exists("meta.json"):
        with open("meta.json", "r", encoding="utf-8") as f:
            meta = json.load(f)
    mode = meta.get("mode", "call")
    io_types = meta.get("io_types")
    param_types = io_types.get("params", []) if io_types else []
    return_type = io_types.get("returns", "json") if io_types else "json"

    try:
        # Always load through the prelude so LeetCode-style annotations/imports
        # (List, Optional, collections, …) resolve for every problem, not just
        # node ones.
        solution = load_solution_with_prelude(variant_prelude(meta))
    except BaseException:
        fail(0, clean_traceback(traceback.format_exc()))
        return

    validate = None
    if mode in ("any_valid", "property"):
        # Pack-shipped validator (our code, written by the Rust side —
        # never from the imported file). Must define validate(args, output).
        import importlib.util

        spec = importlib.util.spec_from_file_location(
            "validator", meta.get("validator_file", "validator.py")
        )
        validator = importlib.util.module_from_spec(spec)
        try:
            spec.loader.exec_module(validator)
            validate = validator.validate
        except BaseException:
            fail(0, "Validator failed to load:\n" + clean_traceback(traceback.format_exc()))
            return

    # property packs execute either as an ops sequence ("design", the
    # default) or as a plain call — the validator judges either shape.
    design_like = mode == "design" or (
        mode == "property" and meta.get("exec", "design") == "design"
    )

    make_codec = None
    get_callable = None
    if mode == "round_trip":
        try:
            make_codec = resolve_codec(
                solution, meta.get("entry_point"), meta.get("round_trip") or {}
            )
        except LookupError as e:
            fail(0, str(e))
            return
        except BaseException:
            fail(0, clean_traceback(traceback.format_exc()))
            return
    elif not design_like:
        try:
            get_callable = resolve_entry(solution, meta.get("entry_point"))
        except LookupError as e:
            fail(0, str(e))
            return

    for case in cases:
        index = case["index"]
        args = case["args"]

        if design_like:
            start = time.perf_counter()
            outputs = run_design(solution, index, args, meta.get("design_io"))
            if outputs is None:
                return
            elapsed_ms = (time.perf_counter() - start) * 1000.0
            payload = {"index": index, "ok": True, "output": outputs, "timeMs": elapsed_ms}
            if mode == "property":
                try:
                    payload["valid"] = bool(validate(args, outputs))
                except BaseException:
                    fail(index, "Validator raised:\n" + clean_traceback(traceback.format_exc()))
                    return
        elif mode == "round_trip":
            round_tripped = run_round_trip(
                index, args, meta.get("round_trip") or {}, make_codec
            )
            if round_tripped is None:
                return
            result, elapsed_ms = round_tripped
            payload = {"index": index, "ok": True, "output": result, "timeMs": elapsed_ms}
        else:
            fn = get_callable()
            # any_valid: the validator judges against the ORIGINAL input, so
            # the solution gets a deep copy in case it mutates its arguments.
            call_args = copy.deepcopy(args) if mode == "any_valid" else args
            # io_types: deserialize wire args into live node structures so the
            # user's stub runs unmodified (task 0003 + closing-the-48 Phase B).
            # Args build left-to-right into `built` so a later param can
            # reference an earlier one (node_ref/clone_of/tail_of); ctx_only
            # params are built for judging but never passed to the solution.
            built = None
            if io_types:
                built = []
                try:
                    for i, a in enumerate(call_args):
                        built.append(
                            deserialize(a, param_types[i] if i < len(param_types) else "json", built)
                        )
                except BaseException:
                    fail(index, "Failed to build node input:\n" + clean_traceback(traceback.format_exc()))
                    return
                call_args = [
                    v
                    for i, v in enumerate(built)
                    if not (i < len(param_types) and _is_ctx_only(param_types[i]))
                ]
                # Global shims (isBadVersion, guess, rand7) are installed into
                # the solution module per case, hidden data and all.
                for i, t in enumerate(param_types):
                    spec = _shim_spec(t)
                    if spec is not None and spec.get("kind") in SHIM_GLOBAL_NAMES:
                        setattr(solution, SHIM_GLOBAL_NAMES[spec["kind"]], built[i])
            start = time.perf_counter()
            try:
                result = fn(*call_args)
            except BaseException:
                fail(index, clean_traceback(traceback.format_exc()))
                return
            elapsed_ms = (time.perf_counter() - start) * 1000.0
            # A shim with a verdict hook (Master.guess) IS the case's real
            # output — the solution itself returns nothing.
            if built is not None:
                for arg in built:
                    if hasattr(arg, "_anvil_verdict"):
                        result = arg._anvil_verdict()
                        break
            out_type = return_type
            if mode == "in_place":
                arg_index = meta.get("arg_index", 0)
                source = built if io_types else call_args
                if not isinstance(arg_index, int) or arg_index >= len(source):
                    fail(index, "in_place judge: arg_index %r is out of range." % arg_index)
                    return
                result = source[arg_index]
                if io_types:
                    out_type = param_types[arg_index] if arg_index < len(param_types) else "json"
            if io_types:
                try:
                    result = serialize(result, out_type, built)
                except BaseException:
                    fail(index, "Failed to serialize node output:\n" + clean_traceback(traceback.format_exc()))
                    return
            payload = {"index": index, "ok": True, "output": result, "timeMs": elapsed_ms}
            if mode in ("any_valid", "property"):
                try:
                    payload["valid"] = bool(validate(args, result))
                except BaseException:
                    fail(
                        index,
                        "Validator raised:\n" + clean_traceback(traceback.format_exc()),
                    )
                    return

        try:
            emit(payload)
        except (TypeError, ValueError):
            fail(index, "Return value is not JSON-serializable: %r" % (payload.get("output"),))
            return


def _run_with_big_stack():
    # Run on a thread with a large stack so deep recursion (DFS on 1e4–1e5
    # inputs) reaches the raised recursion limit instead of overflowing the C
    # stack (a segfault). Falls back to a direct call if the platform rejects
    # the requested stack size, or if the OS refuses to actually spawn the
    # thread (e.g. a thread/process-capped sandbox) — a solve must never
    # hard-fail just because the big-stack trick isn't available here.
    import threading

    try:
        threading.stack_size(256 * 1024 * 1024)
    except (ValueError, OverflowError, RuntimeError):
        main()
        return
    captured = {}

    def target():
        try:
            main()
        except BaseException:
            captured["tb"] = traceback.format_exc()

    try:
        t = threading.Thread(target=target)
        t.start()
    except RuntimeError:
        main()
        return
    t.join()
    if "tb" in captured:
        fail(0, clean_traceback(captured["tb"]))


# Only auto-run when invoked directly (`python harness.py`). The complexity
# probe (probe.py, Phase 5) imports this module to reuse
# load_solution_with_prelude / resolve_entry / deserialize without triggering a
# run.
if __name__ == "__main__":
    _run_with_big_stack()
