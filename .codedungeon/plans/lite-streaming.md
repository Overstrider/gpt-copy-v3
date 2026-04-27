# Lite Plan: Streaming Chat Polish

## Goal

Improve the generated ChatGPT copy so streaming responses feel production-ready end to end.

## Tasks

1. Backend streaming
   - Ensure `POST /api/chat/stream` streams assistant tokens incrementally.
   - Persist the final assistant message after the stream completes.
   - Return structured stream events for token, done, and error states.
   - Add tests for successful streaming and provider failure.

2. Frontend streaming
   - Consume the stream in the composer flow.
   - Render assistant text incrementally without layout jumps.
   - Disable duplicate sends while a stream is active.
   - Preserve the partial response if a recoverable stream error occurs.

3. Verification
   - Run backend formatting, build, and tests.
   - Run frontend lint, typecheck, tests, and build.
   - Update README instructions if streaming commands or behavior changed.

## Acceptance

- Streaming visibly updates the assistant message before completion.
- The final assistant message is persisted.
- Provider errors show a useful UI state.
- CodeDungeon final report shows COMPLETE with Verification: PASS.
