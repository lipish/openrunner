# run-agent backend API requirements

This document describes the **required backend contract** for the run-agent (React/Vite) web UI.

> The frontend stores `access_token` in localStorage under `run-agent.access_token`.

## Auth

### Login (required)

`POST /api/auth/login`

- Content-Type: `application/json`

Request:
```json
{ "username": "alice", "password": "***" }
```

Response (200):
```json
{
  "access_token": "...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "optional",
  "user": {
    "id": "u_123",
    "username": "alice",
    "display_name": "Alice",
    "roles": ["admin"]
  }
}
```

Error:
- `401` `{ "error": "invalid_credentials" }`

### Authentication for non-SSE endpoints

All non-SSE endpoints MUST accept:

`Authorization: Bearer <access_token>`

## Runs (recommended; enables streaming)

### Create run

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
  "session_id": "s_xxx",
  "metadata": {
    "client": "web",
    "model": "Gemini-2.5-Pro"
  }
}
```

Response:
```json
{ "run_id": "run_123" }
```

### Stream run events (SSE)

`GET /api/runs/:run_id/events?access_token=...`

- Response headers:
  - `Content-Type: text/event-stream`
  - `Cache-Control: no-cache`
  - `Connection: keep-alive`

Event types (`data:` is JSON):
- `message_delta`: `{ "delta": "..." }`
- `run_completed`: `{ "message": { "role": "assistant", "content": "...", "timestamp": "ISO-8601" } }`
- `run_failed`: `{ "error": "..." }`

Optional (future UI):
- `tool_call_started`
- `tool_call_finished`

## Fallback chat (optional but recommended)

If you do NOT implement runs/SSE initially, implement this fallback:

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

## Status codes expected by the frontend

- `401` unauthenticated/expired token
- `404` or `501` on `/api/runs` when not supported (frontend will fall back to `/api/chat` if implemented)
