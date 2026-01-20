# run-agent API (draft)

This document defines the **frontend ↔ backend** contract for the run-agent web UI.

See also:
- `backend-api-requirements.md`
- `run-agent-openapi.yaml`

## Base

- Default dev base URL: `http://127.0.0.1:8090`
- Frontend calls `/api/*` on the configured API base URL.

## Auth

### Login

`POST /api/auth/login`

Request:
```json
{ "username": "alice", "password": "***" }
```

Response:
```json
{
  "access_token": "...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "optional",
  "user": { "id": "u_123", "username": "alice" }
}
```

All non-SSE API calls MUST send:

`Authorization: Bearer <access_token>`

## Health

`GET /health`

Response:
```json
{ "ok": true }
```

## Create a run (recommended)

`POST /api/runs`

Request:
```json
{
  "input": {
    "text": "hello",
    "attachments": [
      { "name": "image.png", "type": "image/png", "size": 12345 }
    ]
  },
  "session_id": "optional-session-id",
  "metadata": {
    "cwd": "/path",
    "os": "darwin",
    "client": "web",
    "model": "Gemini-2.5-Pro"
  }
}
```

Response:
```json
{ "run_id": "run_123" }
```

## Stream run events (SSE)

`GET /api/runs/:run_id/events?access_token=...`

Server-Sent Events.

Event types (JSON `data:`):
- `message_delta`: `{ "delta": "..." }`
- `tool_call_started`: `{ "tool_call_id": "t1", "name": "bash", "input": {"command":"..."} }`
- `tool_call_finished`: `{ "tool_call_id": "t1", "output": "...", "ok": true }`
- `run_completed`: `{ "message": { "role": "assistant", "content": "...", "timestamp": "ISO-8601" } }`
- `run_failed`: `{ "error": "..." }`

## Non-streaming fallback

`POST /api/chat`

Request:
```json
{
  "message": "hello",
  "model": "Gemini-2.5-Pro",
  "attachments": [
    { "name": "image.png", "type": "image/png", "size": 12345 }
  ]
}
```

Response:
```json
{ "role": "assistant", "content": "...", "timestamp": "ISO-8601" }
```
