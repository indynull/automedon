// Multi-turn coding task: Grok Build
// Binary: grok | Auth: Grok login | Stream: streaming-json
// Writes under examples/automedon_demo/ (gitignored).
//
//   automedon run examples/harnesses/grok_coding.rhai --print

let s = launch("grok", #{
    yolo: true,
    max_turns: 12,
    multi_turn: true,
    timeout_ms: 300_000
});

s.prompt(`You are working in this Automedon repo.

Create a tiny Python module (stdlib only):

1. examples/automedon_demo/__init__.py — re-export fib from .fib
2. examples/automedon_demo/fib.py — def fib(n: int) -> int for n >= 0
   (0-indexed: fib(0)=0, fib(1)=1, fib(10)=55). Raise ValueError if n < 0.

Use the file tools to write the files. No extra dependencies, no network.
When finished, end your reply with a single line: DONE:fib`);

s.expect(timeout_ms(tool_any(), 240_000));
s.expect(timeout_ms(text("DONE:fib"), 240_000));
s.await_turn();
print(`turn1 ok session=${s.session_id()} tools=${s.tool_names()}`);

s.prompt(`Continue in the same session.

Add examples/automedon_demo/test_fib.py that:
- imports fib from the package (or via path hack if needed)
- asserts fib(0)==0, fib(1)==1, fib(10)==55
- can be run with: python examples/automedon_demo/test_fib.py

Then run that command with the shell tool and show the exit status.
End your final reply with PASS if tests succeeded, else FAIL.`);

s.expect(timeout_ms(tool_any(), 240_000));
s.await_turn();

let t = s.text();
print(`turn2 ok turns=${s.turn()}`);
print("--- transcript tail ---");
print(t);

assert_true(s.turn() >= 2);
assert_contains(t, "DONE:fib");
assert_true(t.contains("PASS") || t.contains("fib(10)") || t.contains("test_fib"));

s.close();
"GROK_DEMO_OK"
