# Anvil complexity probe (Phase 5) — written into the run's temp dir next to
# solution.py, harness.py and probe.json.
#
# Runs the learner's entry function on a ladder of generated input sizes under
# a Python-line counter (sys.settrace) and emits one sentinel line per size:
#   @@ANVIL@@{"size": 100, "ops": 812, "ok": true}
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
import copy
import json
import random
import sys
import threading
import traceback

import harness as H

SENTINEL = "@@ANVIL@@"


def emit(obj):
    sys.stdout.write(SENTINEL + json.dumps(obj) + "\n")
    sys.stdout.flush()


def make_tracer(counter):
    # Count only 'line' events inside the user's solution.py. The global tracer
    # is invoked on every call; returning the local tracer for solution frames
    # enables line events there, returning None everywhere else keeps overhead
    # (and the count) confined to the learner's own code.
    solfile = "solution.py"

    def local(frame, event, arg):
        if event == "line":
            counter[0] += 1
        return local

    def glob(frame, event, arg):
        if frame.f_code.co_filename == solfile:
            return local
        return None

    return glob


def run():
    with open("probe.json", "r", encoding="utf-8") as f:
        cfg = json.load(f)
    sizes = cfg.get("sizes", [])
    seed = cfg.get("seed", 0)
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
    tracer = make_tracer(counter)

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

        fn = get_callable()
        call_args = copy.deepcopy(args)
        counter[0] = 0
        sys.settrace(tracer)
        try:
            fn(*call_args)
        except BaseException:
            sys.settrace(None)
            emit(
                {
                    "size": size,
                    "ok": False,
                    "error": H.clean_traceback(traceback.format_exc()),
                }
            )
            continue
        sys.settrace(None)
        emit({"size": size, "ops": counter[0], "ok": True})


def _run_with_big_stack():
    # Deep recursion (DFS on the larger sizes) needs a big C stack, same as the
    # main harness. settrace applies to the calling thread, so setting it inside
    # run() on this worker thread is correct.
    try:
        threading.stack_size(256 * 1024 * 1024)
    except (ValueError, OverflowError, RuntimeError):
        run()
        return
    t = threading.Thread(target=run)
    t.start()
    t.join()


if __name__ == "__main__":
    _run_with_big_stack()
