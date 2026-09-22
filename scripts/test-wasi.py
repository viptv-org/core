#!/usr/bin/env python3
"""Exercise the JSON-lines bridge: test-wasi.py <native binary or wasmtime command>."""
import json
from pathlib import Path
import selectors
import statistics
import subprocess
import sys
import time

if len(sys.argv) < 2:
    raise SystemExit("usage: test-wasi.py <bridge command> [arguments ...]")

root = Path(__file__).resolve().parent.parent
vectors = json.loads((root / "tests/bridge-vectors.json").read_text())
process = subprocess.Popen(sys.argv[1:], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
selector = selectors.DefaultSelector()
selector.register(process.stdout, selectors.EVENT_READ)


def ask(request, *, valid=True):
    line = request if isinstance(request, str) else json.dumps(request, separators=(",", ":"))
    process.stdin.write(line + "\n")
    process.stdin.flush()
    assert selector.select(30), "bridge did not flush a response within 30 seconds"
    result = json.loads(process.stdout.readline())
    assert result["ok"] is valid, result
    return result.get("result")


try:
    assert ask({"op": "view"})["phase"] == "Starting"
    for scenario in vectors:
        assert ask({"op": "reset"})["phase"] == "Starting"
        effects = []
        for step in scenario["steps"]:
            if "event" in step:
                effects = ask({"op": "process_event", "event": step["event"]})
            else:
                effect = next(item for item in effects if step["effect"] in item["effect"])
                effects = ask({"op": "handle_response", "id": effect["id"], "result": step["output"]})
            view = ask({"op": "view"})
            assert view["phase"] == step["phase"], scenario["name"]
            if "profileId" in step:
                assert view["selectedProfileId"] == step["profileId"], scenario["name"]

    before = ask({"op": "view"})
    for invalid in ["{", {}, {"op": "unknown"}, {"op": "update"},
                    {"op": "resolve", "id": 4294967296, "result": {}},
                    {"op": "normalize", "kind": "catalogs"}]:
        ask(invalid, valid=False)
    assert ask({"op": "view"}) == before
    assert ask({"op": "reset"})["phase"] == "Starting"
    effects = ask({"op": "update", "event": {"Begin": {
        "origin": "https://example.test", "allowInsecurePreview": False}}})
    storage = next(item for item in effects if "Storage" in item["effect"])
    ask({"op": "resolve", "id": storage["id"], "result": {"Ok": None}})
    assert ask({"op": "view"})["phase"] == "Pairing"

    assert ask({"op": "normalize", "kind": "catalogs", "input": []}) == []
    assert ask({"op": "normalize", "kind": "clean", "input": None}) is None
    assert ask(r'{"o\u0070":"view"}') == ask({"op": "view"})
    ask({"op": "normalize", "kind": "clean", "input": "x" * (2 * 1024 * 1024)}, valid=False)
    portrait = {"id": "movie", "type": "movie", "name": "Movie", "poster": "portrait.jpg",
                "position": 20, "duration": 100}
    presentation = ask({"op": "normalize", "kind": "presentation", "input": portrait})
    assert presentation["heroImage"] is None
    assert presentation["progress"] == 0.2
    ask({"op": "normalize", "kind": "playback", "input": {
        "id": "s", "url": "https://user:pass@example.test/media/s"}}, valid=False)
    assert ask({"op": "vizio_platform_support", "platform": "roku"}) is not None

    request = {"op": "normalize", "kind": "catalogs", "input": [
        {"id": str(index), "type": "movie", "name": "Movies"} for index in range(32)]}
    timings = []
    for _ in range(30):
        start = time.perf_counter()
        assert len(ask(request)) == 32
        timings.append((time.perf_counter() - start) * 1000)
    print(f"WASI bridge: {len(vectors)} shared state vectors, aliases, reset, malformed requests, "
          f"pure ops and persistent requests passed; 32 catalogs median={statistics.median(timings):.3f}ms "
          f"p95={sorted(timings)[int(len(timings) * .95)]:.3f}ms")
    process.stdin.close()
    assert process.wait(timeout=30) == 0
    oversized = subprocess.run(
        sys.argv[1:], input=" " * (12 * 1024 * 1024 + 1),
        text=True, capture_output=True, timeout=30, check=True,
    )
    assert json.loads(oversized.stdout)["ok"] is False
    print("WASI bridge: oversized request rejected with a bounded read")
finally:
    selector.close()
    if process.poll() is None:
        process.kill()
        process.wait()
