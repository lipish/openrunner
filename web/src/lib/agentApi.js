function getApiBase() {
  const raw = localStorage.getItem('run-agent.apiBase') || '';
  return raw.trim().replace(/\/$/, '');
}

function getToken() {
  return localStorage.getItem('run-agent.access_token') || localStorage.getItem('run-agent.token') || '';
}

function apiUrl(path) {
  const base = getApiBase();
  return base ? `${base}${path}` : path;
}

function authHeaders() {
  const token = getToken();
  return token ? { authorization: `Bearer ${token}` } : {};
}

export async function loginWithPassword({ username, password }) {
  const res = await fetch(apiUrl('/api/auth/login'), {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ username, password }),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  const data = await res.json();
  return {
    access_token: data.access_token,
    token_type: data.token_type || 'Bearer',
    expires_in: data.expires_in,
    refresh_token: data.refresh_token,
    user: data.user || null,
  };
}

export async function fetchSessions() {
  const res = await fetch(apiUrl('/api/sessions'), {
    method: 'GET',
    headers: { 'content-type': 'application/json', ...authHeaders() },
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  const data = await res.json();
  return data.sessions || [];
}

export async function saveSessions(sessions) {
  const res = await fetch(apiUrl('/api/sessions'), {
    method: 'POST',
    headers: { 'content-type': 'application/json', ...authHeaders() },
    body: JSON.stringify({ sessions }),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return await res.json();
}

export async function chatOnce(message, opts = {}) {
  const { model, attachments, agentType, env, extraArgs } = opts;
  const res = await fetch(apiUrl('/api/chat'), {
    method: 'POST',
    headers: { 'content-type': 'application/json', ...authHeaders() },
    body: JSON.stringify({ message, model, attachments, agent_type: agentType, env, extra_args: extraArgs })
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return await res.json();
}

export async function createRun(message, sessionId, opts = {}) {
  const { model, attachments, agentType, env, extraArgs } = opts;
  const res = await fetch(apiUrl('/api/runs'), {
    method: 'POST',
    headers: { 'content-type': 'application/json', ...authHeaders() },
    body: JSON.stringify({
      input: { text: message, attachments },
      session_id: sessionId || null,
      metadata: { client: 'web', model, agent_type: agentType, env, extra_args: extraArgs }
    })
  });

  if (!res.ok) {
    const err = new Error(`HTTP ${res.status}`);
    err.status = res.status;
    throw err;
  }

  return await res.json();
}

export function streamRun(runId, { onDelta, onCompleted, onError }) {
  const token = getToken();
  const qs = token ? `?access_token=${encodeURIComponent(token)}` : '';
  const es = new EventSource(apiUrl(`/api/runs/${encodeURIComponent(runId)}/events${qs}`));

  function handleData(raw) {
    try {
      const data = JSON.parse(raw);
      if (data?.delta) onDelta?.(data.delta);
      if (data?.message?.content) onCompleted?.(data.message);
      if (data?.error) onError?.(new Error(data.error));
    } catch {
      if (raw) onDelta?.(raw);
    }
  }

  es.addEventListener('message_delta', (e) => handleData(e.data));
  es.addEventListener('run_completed', (e) => handleData(e.data));
  es.addEventListener('run_failed', (e) => handleData(e.data));
  es.onmessage = (e) => handleData(e.data);

  es.onerror = (e) => {
    es.close();
    onError?.(e instanceof Error ? e : new Error('stream error'));
  };

  return () => es.close();
}

export async function sendMessage({ message, sessionId, onDelta, model, attachments, agentType, env, extraArgs }) {
  const opts = { model, attachments, agentType, env, extraArgs };

  try {
    const { run_id } = await createRun(message, sessionId, opts);

    return await new Promise((resolve, reject) => {
      const close = streamRun(run_id, {
        onDelta,
        onCompleted: (msg) => {
          close();
          resolve(msg);
        },
        onError: (err) => {
          close();
          reject(err);
        }
      });
    });
  } catch (e) {
    if (e?.status === 404 || e?.status === 501) {
      return await chatOnce(message, opts);
    }
    throw e;
  }
}
