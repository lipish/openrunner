import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../auth/AuthContext.jsx';
import { loginWithPassword } from '../../lib/agentApi.js';

export default function LoginPage() {
  const { setToken } = useAuth();
  const navigate = useNavigate();
  const [apiBase, setApiBase] = useState(() => localStorage.getItem('run-agent.apiBase') || '');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  async function onSubmit(e) {
    e.preventDefault();
    setError('');
    setLoading(true);
    try {
      const { access_token } = await loginWithPassword({ username: username.trim(), password });
      if (!access_token) throw new Error('Missing access_token in login response');
      setToken(access_token);
      navigate('/', { replace: true });
    } catch (e2) {
      setError(String(e2?.message || e2));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <div
        style={{
          width: 520,
          background: '#fff',
          borderRadius: 16,
          boxShadow: '0 8px 24px rgba(0,0,0,0.12)',
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
      <div style={{ padding: '16px 20px', borderBottom: '1px solid #E5E5EA', background: '#FAFAFA' }}>
        <h1 style={{ margin: 0, fontSize: 17, fontWeight: 600 }}>run-agent 登录</h1>
      </div>
      <form onSubmit={onSubmit} style={{ padding: 20, display: 'flex', flexDirection: 'column', gap: 12 }}>
        <label style={{ fontSize: 13, color: '#666' }}>Backend API Base URL（可选）</label>
        <input
          value={apiBase}
          onChange={(e) => {
            const v = e.target.value;
            setApiBase(v);
            localStorage.setItem('run-agent.apiBase', v.trim());
          }}
          placeholder="例如 http://127.0.0.1:8080"
          style={{ padding: '10px 12px', border: '1px solid #D1D5DB', borderRadius: 10, fontSize: 14 }}
        />

        <label style={{ fontSize: 13, color: '#666' }}>用户名</label>
        <input
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          autoComplete="username"
          style={{ padding: '10px 12px', border: '1px solid #D1D5DB', borderRadius: 10, fontSize: 14 }}
        />

        <label style={{ fontSize: 13, color: '#666' }}>密码</label>
        <input
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          type="password"
          autoComplete="current-password"
          style={{ padding: '10px 12px', border: '1px solid #D1D5DB', borderRadius: 10, fontSize: 14 }}
        />

        {error ? <div style={{ color: '#B00020', fontSize: 12 }}>{error}</div> : null}

        <button
          type="submit"
          disabled={loading}
          style={{ padding: '10px 12px', borderRadius: 10, border: 'none', background: '#007AFF', color: '#fff', fontWeight: 600, cursor: loading ? 'not-allowed' : 'pointer', opacity: loading ? 0.7 : 1 }}
        >
          {loading ? '登录中…' : '登录'}
        </button>

        <div style={{ fontSize: 12, color: '#666' }}>后端需实现 POST /api/auth/login → {'{"access_token":"..."}'}</div>
      </form>
      </div>
    </div>
  );
}
