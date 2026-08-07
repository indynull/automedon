// One-turn Grok Build smoke.
// Requires: `grok` on PATH and product authentication.
//
//   medon run examples/harnesses/grok_smoke.rhai --print

let s = launch("grok", #{
    yolo: true,
    timeout_ms: 120_000
});

s.prompt("Reply with exactly one line: AUTOMEDON_SMOKE");
s.expect(timeout_ms(text("AUTOMEDON_SMOKE"), 90_000));
s.await_turn();

print(`harness=${s.harness()} session=${s.session_id()} text=${s.text()}`);
assert_contains(s.text(), "AUTOMEDON_SMOKE");
s.close();
"SMOKE_OK"
