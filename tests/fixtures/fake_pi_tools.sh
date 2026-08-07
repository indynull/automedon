#!/usr/bin/env bash
# Pi-shaped NDJSON: one tool_execution_start line expands to HookStarted+ToolCall.
set -euo pipefail
echo '{"type":"session","id":"fake-pi-1"}'
echo '{"type":"agent_start"}'
echo '{"type":"turn_start"}'
echo '{"type":"tool_execution_start","toolCallId":"call-1","toolName":"bash","args":{"command":"echo hi"}}'
echo '{"type":"tool_execution_end","toolCallId":"call-1","toolName":"bash","result":{"content":[{"type":"text","text":"hi\n"}]},"isError":false}'
echo '{"type":"message_update","assistantMessageEvent":{"type":"text_delta","delta":"hooks_done"}}'
echo '{"type":"turn_end"}'
echo '{"type":"agent_settled"}'
