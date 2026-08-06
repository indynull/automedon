#!/usr/bin/env bash
# Minimal NDJSON harness for process-level tests (Grok-shaped lines).
# Understands Grok-like argv: takes the value after `-p` as the user prompt.
set -euo pipefail
prompt="demo"
prev=""
for a in "$@"; do
  if [[ "$prev" == "-p" || "$prev" == "--single" ]]; then
    prompt="$a"
  fi
  if [[ "$a" == "--stderr-note" ]]; then
    echo "stderr-note" >&2
  fi
  prev="$a"
done
json_escape() {
  printf '%s' "$1" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()), end="")' 2>/dev/null \
    || printf '"%s"' "${1//\"/\\\"}"
}
pjson=$(json_escape "$prompt")
echo '{"type":"text","data":"FAKE:"}'
echo "{\"type\":\"text\",\"data\":${pjson}}"
echo '{"type":"end","stopReason":"end_turn","sessionId":"fake-sess-1","num_turns":1,"usage":{"input_tokens":1,"output_tokens":2,"total_tokens":3},"total_cost_usd":0.0}'
