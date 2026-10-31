# Anvil complexity probe (Phase 5) — written into the run's temp dir next to
# solution.py, harness.py and probe.json.
#
# Runs the learner's entry function on a ladder of generated input sizes under
# a Python-line counter (sys.settrace) and emits one sentinel line per size:
#   @@ANVIL@@{"size": 100, "ops": 812, "depth": 1, "time_ns": 41000,
#             "heap_bytes": 920, "ok": true}
# `depth` = max recursion depth of solution frames (call-stack space),
# `time_ns` = fastest untraced wall time (exposes work hidden in C built-ins
# that the op counter can't see, e.g. `x in list`, slicing, `s += c`), and
# `heap_bytes` = tracemalloc peak minus the newly allocated return value
# (auxiliary heap space). The last two are best-effort and may be absent.
#   @@ANVIL@@{"size": 200, "ok": false, "error": "..."}   (that size skipped)
# A single line with no "size" and "ok": false means the whole probe failed to
# start (solution didn't load / no entry point / generator broken).
#
# It measures *Python-level* operations, so work inside C built-ins (sorted,
# set, Counter) is under-counted — the UI says so. That's the right lens for
# catching an O(n^2) nested scan you could have made O(n).
#
# Reuses harness.py's prelude loader / entry resolver / node (de)serializer by
# importing it — harness.py only auto-runs under `__main__`, so importing it is
# side-effect free.
import builtins
import copy
import gc
import json
import math
import random
import sys
import threading
import time
import tracemalloc
import traceback

import harness as H

SENTINEL = "@@ANVIL@@"

# Sequence-consuming built-ins whose real cost is O(k)/O(k log k) in the size k
# of their first argument but which run in C, so sys.settrace never sees them —
# left uncharged, an `sorted()` (O(n log n)) or `sum()` over the input would read
# as flat. We wrap them to charge that work into the op count. Only *functions*
# are wrapped, never type constructors (list/dict/set/tuple/Counter): shadowing a
# type would break `isinstance(x, list)` and annotations in the learner's code.
# Container *methods* (.sort(), .index(), .insert(), …) are charged separately
# via sys.setprofile (make_profiler). What neither hook can see — membership in
# a list (x in [...]), slicing, container construction — is caught by the
# untraced timing pass: wall time outgrowing the op count flags hidden work.
_LINEAR_BUILTINS = ("sum", "min", "max", "any", "all")


def _instrument_builtins(ns, counter):
    """Shadow the wrapped built-ins inside the solution's own globals so the
    learner's code resolves to the charging versions; each call adds the size
    of its first argument (times log for `sorted`) to the op counter."""

    def size_of(x):
        try:
            return len(x)
        except TypeError:
            return 0

    def linear(fn):
        def wrapped(*args, **kwargs):
            if args:
                counter[0] += size_of(args[0])
            return fn(*args, **kwargs)

        return wrapped

    def sortlike(fn):
        def wrapped(*args, **kwargs):
            if args:
                n = size_of(args[0])
                if n > 1:
                    counter[0] += n * max(1, math.ceil(math.log2(n)))
            return fn(*args, **kwargs)

        return wrapped

    for name in _LINEAR_BUILTINS:
        ns[name] = linear(getattr(builtins, name))
    ns["sorted"] = sortlike(builtins.sorted)


def emit(obj):
    sys.stdout.write(SENTINEL + json.dumps(obj) + "\n")
    sys.stdout.flush()


SOLFILE = "solution.py"

# Method calls on containers whose real cost scales with the container, charged
# via sys.setprofile's `c_call` event (it hands us the bound C method, so
# `__self__` is the container). Only methods that are O(n) on EVERY call are
# charged: `remove` always searches then shifts the tail (n total); `count`,
# `copy`, `reverse` and whole-string transforms touch everything. Methods that
# can stop early or cost less (`index`, `find`, `insert`/`pop` near the end,
# `strip`) are left uncharged rather than over-charged — if they really scan,
# the untraced timing pass catches it. Matched with isinstance, so subclasses
# (Counter, defaultdict, OrderedDict) are charged too.
_LINEAR_METHODS = (
    (list, ("count", "remove", "copy", "reverse")),
    (str, ("count", "replace", "split", "upper", "lower")),
    (dict, ("copy",)),
    (set, ("copy",)),
)
_SORT_METHODS = ((list, ("sort",)),)


def make_profiler(counter):
    """A sys.setprofile hook charging container-method work done in C (see
    _LINEAR_METHODS) when it's called from the learner's own code."""

    def prof(frame, event, arg):
        if event != "c_call" or frame.f_code.co_filename != SOLFILE:
            return
        owner = getattr(arg, "__self__", None)
        name = getattr(arg, "__name__", "")
        if any(isinstance(owner, t) and name in names for t, names in _LINEAR_METHODS):
            counter[0] += len(owner)
        elif any(isinstance(owner, t) and name in names for t, names in _SORT_METHODS):
            n = len(owner)
            if n > 1:
                counter[0] += n * max(1, math.ceil(math.log2(n)))

    return prof


def make_tracer(counter, depth):
    # Count only 'line' events inside the user's solution.py. The global tracer
    # is invoked on every call; returning the local tracer for solution frames
    # enables line events there, returning None everywhere else keeps overhead
    # (and the count) confined to the learner's own code. `depth` = [current,
    # max] nesting of solution frames — recursion depth is call-stack space.
    def local(frame, event, arg):
        if event == "line":
            counter[0] += 1
        elif event == "return":
            depth[0] -= 1
        return local

    def glob(frame, event, arg):
        if frame.f_code.co_filename == SOLFILE:
            depth[0] += 1
            if depth[0] > depth[1]:
                depth[1] = depth[0]
            return local
        return None

    return glob


def deep_size(obj, seen=None, keep=None):
    """Approximate bytes retained by a returned value (containers, strings,
    numbers, ListNode/TreeNode-style objects) — subtracted from the heap peak
    so the space measurement is *auxiliary* space, as interviews count it.
    `keep`, when given, collects every visited object so the caller can hold
    references (keeps ids from being reused while they're being compared)."""
    if seen is None:
        seen = set()
    if id(obj) in seen:
        return 0
    seen.add(id(obj))
    if keep is not None:
        keep.append(obj)
    size = sys.getsizeof(obj, 0)
    if isinstance(obj, dict):
        size += sum(
            deep_size(k, seen, keep) + deep_size(v, seen, keep) for k, v in obj.items()
        )
    elif isinstance(obj, (list, tuple, set, frozenset)):
        size += sum(deep_size(x, seen, keep) for x in obj)
    elif hasattr(obj, "__dict__") and not isinstance(obj, type):
        size += sum(deep_size(v, seen, keep) for v in vars(obj).values())
    return size


# Timing budget per size: repeat the untraced call until this much wall time
# has passed (or MAX_REPS), keeping the fastest run — min is the least noisy
# estimator of the work itself.
TIME_BUDGET_NS = 40_000_000
MAX_REPS = 9


def time_call(get_callable, args):
    best = None
    spent = 0
    reps = 0
    gc_was = gc.isenabled()
    gc.disable()
    try:
        while reps < MAX_REPS and (reps < 2 or spent < TIME_BUDGET_NS):
            call_args = copy.deepcopy(args)
            fn = get_callable()
            t0 = time.perf_counter_ns()
            fn(*call_args)
            dt = time.perf_counter_ns() - t0
            spent += dt
            reps += 1
            best = dt if best is None else min(best, dt)
    finally:
        if gc_was:
            gc.enable()
    return best


def measure_aux_heap(get_callable, args):
    """Peak heap bytes allocated during one call, minus the *newly allocated*
    part of the returned value. Objects reachable from the inputs (an argument
    handed back after in-place work, input ints reused in an output list) were
    allocated before the measurement started, so they're excluded from both.
    `get_callable` should come from a freshly loaded module so module-level
    memo caches start cold. Input objects are kept alive for the whole
    measurement so a freed input's id can't be reused by a new result object."""
    call_args = copy.deepcopy(args)
    input_ids = set()
    inputs_alive = []
    deep_size(call_args, input_ids, inputs_alive)  # ids + live refs of inputs
    fn = get_callable()
    gc.collect()
    tracemalloc.start()
    try:
        base = tracemalloc.get_traced_memory()[0]
        tracemalloc.reset_peak()
        result = fn(*call_args)
        peak = tracemalloc.get_traced_memory()[1]
    finally:
        tracemalloc.stop()
    out = deep_size(result, input_ids)
    del inputs_alive
    return max(0, peak - base - out)


def run():
    with open("probe.json", "r", encoding="utf-8") as f:
        cfg = json.load(f)
    sizes = cfg.get("sizes", [])
    seed = cfg.get("seed", 0)
    # "time": untraced timing only — used to calibrate against the pack's
    # reference solution, which needs nothing else.
    timing_only = cfg.get("mode") == "time"
    io_types = cfg.get("io_types")
    param_types = io_types.get("params", []) if io_types else []

    try:
        solution = H.load_solution_with_prelude()
        get_callable = H.resolve_entry(solution, cfg.get("entry_point"))
    except BaseException:
        emit({"ok": False, "error": H.clean_traceback(traceback.format_exc())})
        return

    gen_ns = {}
    try:
        exec(compile(cfg["generator"], "<generator>", "exec"), gen_ns)
        gen = gen_ns["gen"]
    except BaseException:
        emit({"ok": False, "error": "generator failed to load"})
        return

    counter = [0]
    depth = [0, 0]
    # Charge C-level built-in work (sorted/sum/…) into the same counter so it
    # scales with the input, not just the Python lines the learner wrote.
    _instrument_builtins(solution.__dict__, counter)
    tracer = make_tracer(counter, depth)
    profiler = make_profiler(counter)

    for size in sizes:
        try:
            rng = random.Random(seed)
            args = list(gen(rng, size))
            if io_types:
                args = [
                    H.deserialize(a, param_types[i] if i < len(param_types) else "json")
                    for i, a in enumerate(args)
                ]
        except BaseException:
            emit({"size": size, "ok": False, "error": "input generation failed"})
            continue

        if timing_only:
            try:
                emit({"size": size, "ops": 0, "time_ns": time_call(get_callable, args), "ok": True})
            except BaseException:
                emit({"size": size, "ok": False, "error": H.clean_traceback(traceback.format_exc())})
            continue

        fn = get_callable()
        call_args = copy.deepcopy(args)
        counter[0] = 0
        depth[0] = depth[1] = 0
        sys.settrace(tracer)
        sys.setprofile(profiler)
        try:
            fn(*call_args)
        except BaseException:
            sys.settrace(None)
            sys.setprofile(None)
            emit(
                {
                    "size": size,
                    "ok": False,
                    "error": H.clean_traceback(traceback.format_exc()),
                }
            )
            continue
        sys.settrace(None)
        sys.setprofile(None)
        line = {"size": size, "ops": counter[0], "depth": depth[1], "ok": True}
        # Untraced wall time and auxiliary heap are best-effort extras: if
        # either measurement trips, the op count above still stands.
        try:
            line["time_ns"] = time_call(get_callable, args)
        except BaseException:
            pass
        try:
            # Fresh, uninstrumented module: cold module-level caches, and no
            # charging wrappers allocating during the measurement.
            fresh = H.resolve_entry(H.load_solution_with_prelude(), cfg.get("entry_point"))
            line["heap_bytes"] = measure_aux_heap(fresh, args)
        except BaseException:
            pass
        emit(line)


def _run_with_big_stack():
    # Deep recursion (DFS on the larger sizes) needs a big C stack, same as the
    # main harness. settrace applies to the calling thread, so setting it inside
    # run() on this worker thread is correct. Falls back to a direct call if
    # the platform rejects the requested stack size, or if the OS refuses to
    # actually spawn the thread (e.g. a thread/process-capped sandbox).
    try:
        threading.stack_size(256 * 1024 * 1024)
    except (ValueError, OverflowError, RuntimeError):
        run()
        return
    try:
        t = threading.Thread(target=run)
        t.start()
    except RuntimeError:
        run()
        return
    t.join()


if __name__ == "__main__":
    _run_with_big_stack()
