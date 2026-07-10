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


def run_design(module, index, args):
    # LeetCode wire format: args = [ops, argLists]. ops[0] names the class;
    # instantiate with argLists[0], apply remaining ops, collect outputs
    # (None for the constructor and void methods).
    if len(args) != 2 or not isinstance(args[0], list) or not isinstance(args[1], list):
        fail(index, "Design case input must be [operations, argument_lists].")
        return None
    ops, arg_lists = args
    if not ops or len(ops) != len(arg_lists):
        fail(index, "Design case operations and argument lists must align.")
        return None
    cls_name = ops[0]
    cls = getattr(module, cls_name, None) if re.fullmatch(IDENT, str(cls_name)) else None
    if not inspect.isclass(cls):
        fail(index, "Class '%s' not found in your code." % cls_name)
        return None
    try:
        instance = cls(*arg_lists[0])
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
        try:
            outputs.append(method(*arg_lists[i]))
        except BaseException:
            fail(
                index,
                "op %d (%s) raised:\n%s"
                % (i, ops[i], clean_traceback(traceback.format_exc())),
            )
            return None
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


def _list_kind(t):
    # A composite list type is {"list_of": <inner>}; everything else is a leaf.
    return t.get("list_of") if isinstance(t, dict) else None


def deserialize(value, t):
    inner = _list_kind(t)
    if inner is not None:
        return [deserialize(v, inner) for v in value]
    if t == "linked_list":
        head = None
        for v in reversed(value or []):
            head = ListNode(v, head)
        return head
    if t == "tree":
        if not value:
            return None
        root = TreeNode(value[0])
        queue = [root]
        i = 1
        while queue and i < len(value):
            node = queue.pop(0)
            if i < len(value):
                if value[i] is not None:
                    node.left = TreeNode(value[i])
                    queue.append(node.left)
                i += 1
            if i < len(value):
                if value[i] is not None:
                    node.right = TreeNode(value[i])
                    queue.append(node.right)
                i += 1
        return root
    return value  # "json"


def serialize(value, t):
    inner = _list_kind(t)
    if inner is not None:
        return [serialize(v, inner) for v in value]
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
    return value  # "json"


def load_solution_with_prelude():
    # Inject the prelude (typing names, common modules, ListNode/TreeNode) into a
    # fresh `solution` module namespace, THEN exec the user's code as its own
    # compilation unit so annotations like `List[int]` / `Optional[ListNode]`
    # resolve at definition time (as on leetcode.com) AND traceback line numbers
    # still point at the real line in solution.py (no prelude offset).
    import types as _types

    with open("solution.py", "r", encoding="utf-8") as f:
        src = f.read()
    mod = _types.ModuleType("solution")
    exec(compile(NODE_PRELUDE, "<prelude>", "exec"), mod.__dict__)
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
        solution = load_solution_with_prelude()
    except BaseException:
        fail(0, clean_traceback(traceback.format_exc()))
        return

    validate = None
    if mode == "any_valid":
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

    get_callable = None
    if mode != "design":
        try:
            get_callable = resolve_entry(solution, meta.get("entry_point"))
        except LookupError as e:
            fail(0, str(e))
            return

    for case in cases:
        index = case["index"]
        args = case["args"]

        if mode == "design":
            start = time.perf_counter()
            outputs = run_design(solution, index, args)
            if outputs is None:
                return
            elapsed_ms = (time.perf_counter() - start) * 1000.0
            payload = {"index": index, "ok": True, "output": outputs, "timeMs": elapsed_ms}
        else:
            fn = get_callable()
            # any_valid: the validator judges against the ORIGINAL input, so
            # the solution gets a deep copy in case it mutates its arguments.
            call_args = copy.deepcopy(args) if mode == "any_valid" else args
            # io_types: deserialize array/level-order args into ListNode/TreeNode
            # so the user's stub runs unmodified (task 0003).
            if io_types:
                call_args = [
                    deserialize(a, param_types[i] if i < len(param_types) else "json")
                    for i, a in enumerate(call_args)
                ]
            start = time.perf_counter()
            try:
                result = fn(*call_args)
            except BaseException:
                fail(index, clean_traceback(traceback.format_exc()))
                return
            elapsed_ms = (time.perf_counter() - start) * 1000.0
            out_type = return_type
            if mode == "in_place":
                arg_index = meta.get("arg_index", 0)
                if not isinstance(arg_index, int) or arg_index >= len(call_args):
                    fail(index, "in_place judge: arg_index %r is out of range." % arg_index)
                    return
                result = call_args[arg_index]
                if io_types:
                    out_type = param_types[arg_index] if arg_index < len(param_types) else "json"
            if io_types:
                try:
                    result = serialize(result, out_type)
                except BaseException:
                    fail(index, "Failed to serialize node output:\n" + clean_traceback(traceback.format_exc()))
                    return
            payload = {"index": index, "ok": True, "output": result, "timeMs": elapsed_ms}
            if mode == "any_valid":
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
    # the requested stack size.
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

    t = threading.Thread(target=target)
    t.start()
    t.join()
    if "tb" in captured:
        fail(0, clean_traceback(captured["tb"]))


# Only auto-run when invoked directly (`python harness.py`). The complexity
# probe (probe.py, Phase 5) imports this module to reuse
# load_solution_with_prelude / resolve_entry / deserialize without triggering a
# run.
if __name__ == "__main__":
    _run_with_big_stack()
