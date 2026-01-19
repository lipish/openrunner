import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../auth/AuthContext.jsx';
import { loginWithPassword } from '../../lib/agentApi.js';
import { Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '../components/ui/dialog.jsx';

export default function LoginPage() {
  const { setToken } = useAuth();
  const navigate = useNavigate();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [isRegisterMode, setIsRegisterMode] = useState(false);
  const [registerSuccessOpen, setRegisterSuccessOpen] = useState(false);

  async function onSubmit(e) {
    e.preventDefault();
    setError('');
    
    if (isRegisterMode) {
      // 注册模式
      if (!username.trim()) {
        setError('请输入邮箱');
        return;
      }
      if (!/^\S+@\S+\.\S+$/.test(username.trim())) {
        setError('请输入有效的邮箱');
        return;
      }
      if (!password) {
        setError('请输入密码');
        return;
      }
      if (password !== confirmPassword) {
        setError('两次输入的密码不一致');
        return;
      }
      
      setLoading(true);
      try {
        // 调用注册接口
        const response = await fetch(`/api/auth/register`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ username: username.trim(), password })
        });
        
        if (!response.ok) {
          const errorData = await response.json().catch(() => ({ message: '注册失败' }));
          throw new Error(errorData.error || errorData.message || '注册失败');
        }
        
        // 注册成功，切换到登录模式
        setError('');
        setIsRegisterMode(false);
        setPassword('');
        setConfirmPassword('');
        setRegisterSuccessOpen(true);
      } catch (e2) {
        setError(String(e2?.message || e2));
      } finally {
        setLoading(false);
      }
    } else {
      // 登录模式
      if (!username.trim()) {
        setError('请输入邮箱');
        return;
      }
      if (!/^\S+@\S+\.\S+$/.test(username.trim())) {
        setError('请输入有效的邮箱');
        return;
      }
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
  }

  function toggleMode() {
    setIsRegisterMode(!isRegisterMode);
    setError('');
    setPassword('');
    setConfirmPassword('');
  }

  return (
    <div style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', background: '#F5F5F7' }}>
      <Dialog open={registerSuccessOpen} onOpenChange={setRegisterSuccessOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>注册成功</DialogTitle>
          </DialogHeader>
          <DialogBody>注册成功！请使用邮箱和密码登录。</DialogBody>
          <DialogFooter>
            <button
              type="button"
              onClick={() => setRegisterSuccessOpen(false)}
              style={{ padding: '10px 12px', borderRadius: 10, border: 'none', background: '#007AFF', color: '#fff', fontWeight: 600, cursor: 'pointer' }}
            >
              知道了
            </button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
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
        <h1 style={{ margin: 0, fontSize: 17, fontWeight: 600 }}>OpenRunner {isRegisterMode ? '注册' : '登录'}</h1>
      </div>
      <form onSubmit={onSubmit} style={{ padding: 20, display: 'flex', flexDirection: 'column', gap: 12 }}>
        <label style={{ fontSize: 13, color: '#666' }}>邮箱</label>
        <input
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          placeholder="name@example.com"
          autoComplete="email"
          inputMode="email"
          required
          style={{ padding: '10px 12px', border: '1px solid #D1D5DB', borderRadius: 10, fontSize: 14 }}
        />

        <label style={{ fontSize: 13, color: '#666' }}>密码</label>
        <input
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          type="password"
          autoComplete={isRegisterMode ? 'new-password' : 'current-password'}
          required
          style={{ padding: '10px 12px', border: '1px solid #D1D5DB', borderRadius: 10, fontSize: 14 }}
        />

        {isRegisterMode && (
          <>
            <label style={{ fontSize: 13, color: '#666' }}>确认密码</label>
            <input
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              type="password"
              autoComplete="new-password"
              required
              style={{ padding: '10px 12px', border: '1px solid #D1D5DB', borderRadius: 10, fontSize: 14 }}
            />
          </>
        )}

        {error ? <div style={{ color: '#B00020', fontSize: 12 }}>{error}</div> : null}

        <button
          type="submit"
          disabled={loading}
          style={{ padding: '10px 12px', borderRadius: 10, border: 'none', background: '#007AFF', color: '#fff', fontWeight: 600, cursor: loading ? 'not-allowed' : 'pointer', opacity: loading ? 0.7 : 1 }}
        >
          {loading ? (isRegisterMode ? '注册中…' : '登录中…') : (isRegisterMode ? '注册' : '登录')}
        </button>

        <div style={{ textAlign: 'center', marginTop: 8 }}>
          <button
            type="button"
            onClick={toggleMode}
            style={{ background: 'none', border: 'none', color: '#007AFF', fontSize: 13, cursor: 'pointer', textDecoration: 'underline' }}
          >
            {isRegisterMode ? '已有账号？立即登录' : '没有账号？立即注册'}
          </button>
        </div>
      </form>
      </div>
    </div>
  );
}
