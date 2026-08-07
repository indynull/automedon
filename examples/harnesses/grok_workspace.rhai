// Real-world multi-turn: Grok Build coding task in-repo.
//
// Binary:     grok
// Auth:       Grok Build login
// Stream:     -p --output-format streaming-json
// Multi-turn: --resume / --continue
// Workspace:  examples/automedon_demo/
//
// Live QA entry for Grok (coding + tools + multi-turn):
//   medon run examples/harnesses/grok_workspace.rhai --print
//
// (grok_coding.rhai is the same scenario class; this script is the documented
//  live entry with stable markers GROK_WS_* .)
//
// Success markers: DONE:fib, GROK_WS_OK (or PASS / test evidence)

let s = launch("grok", #{
    yolo: true,
    max_turns: 12,
    multi_turn: true,
    timeout_ms: 300_000
});

s.prompt(`You are working in this Automedon repo (current workspace).

Create a tiny Python module (stdlib only):

1. examples/automedon_demo/__init__.py — re-export fib from .fib
2. examples/automedon_demo/fib.py — def fib(n: int) -> int for n >= 0
   (0-indexed: fib(0)=0, fib(1)=1, fib(10)=55). Raise ValueError if n < 0.

Use the file tools to write the files. No extra dependencies, no network.
When finished, end your reply with a single line: DONE:fib`);

s.expect(timeout_ms(tool_any(), 240_000));
s.expect(timeout_ms(text("DONE:fib"), 240_000));
s.await_turn();
print(`grok turn1 session=${s.session_id()} tools=${s.tool_names()}`);
assert_contains(s.text(), "DONE:fib");

s.prompt(`Continue in the same session.

Add examples/automedon_demo/test_fib.py that:
- imports fib from the package (or via path hack if needed)
- asserts fib(0)==0, fib(1)==1, fib(10)==55
- can be run with: python examples/automedon_demo/test_fib.py

Then run that command with the shell tool and show the exit status.
End your final reply with a single line: GROK_WS_OK if tests succeeded, else FAIL.`);

s.expect(timeout_ms(tool_any(), 240_000));
s.expect(timeout_ms(text("GROK_WS_OK"), 240_000));
s.await_turn();

let t = s.text();
print(`grok turn2 session=${s.session_id()} turn=${s.turn()}`);
print("--- transcript tail ---");
print(t);

assert_true(s.turn() >= 2);
assert_contains(t, "DONE:fib");
assert_true(
    t.contains("GROK_WS_OK") || t.contains("PASS") || t.contains("fib(10)") || t.contains("test_fib")
);

s.close();
print("GROK_WS_OK");
"GROK_WS_OK"
