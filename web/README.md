# run-agent

Web UI only (React + Vite). Backend runtime lives in a separate project; this repo only contains the frontend.

## Start web UI

```bash
npm install
npm run dev
```

Open `http://localhost:5173`.

## Configure backend

- In UI: Settings → set **Backend API Base URL** (e.g. `http://127.0.0.1:8080`).
- Login API: `POST /api/auth/login` → `{ "access_token": "..." }`.
- Runs API (recommended): `POST /api/runs` + `GET /api/runs/:id/events` (SSE).
