import React, { useState } from 'react';

function getBase() {
  return localStorage.getItem('run-agent.apiBase') || '';
}

export default function SettingsModal({ open, onClose }) {
  const [apiBase, setApiBase] = useState(getBase());

  if (!open) return null;

  return (
    <div
      onClick={onClose}
      style={{
        position: 'fixed',
        inset: 0,
        background: 'rgba(0,0,0,0.25)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 20,
      }}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          width: 520,
          background: '#fff',
          borderRadius: 16,
          boxShadow: '0 12px 40px rgba(0,0,0,0.2)',
          overflow: 'hidden',
        }}
      >
        <div style={{ padding: '14px 16px', borderBottom: '1px solid #E5E5EA', background: '#FAFAFA' }}>
          <div style={{ fontSize: 15, fontWeight: 700 }}>Settings</div>
        </div>

        <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 10 }}>
          <label style={{ fontSize: 12, color: '#666' }}>Backend API Base URL（可选）</label>
          <input
            value={apiBase}
            onChange={(e) => setApiBase(e.target.value)}
            placeholder="例如 https://api.example.com"
            style={{ padding: '10px 12px', border: '1px solid #D1D5DB', borderRadius: 10, fontSize: 14 }}
          />
          <div style={{ fontSize: 12, color: '#666' }}>
            留空则使用同域 /api（建议在反向代理或网关里转发到你的后端）。
          </div>
        </div>

        <div style={{ padding: 16, borderTop: '1px solid #E5E5EA', display: 'flex', justifyContent: 'flex-end', gap: 8 }}>
          <button onClick={onClose} style={{ height: 34, padding: '0 12px', borderRadius: 10, border: '1px solid #D1D5DB', background: '#fff', cursor: 'pointer' }}>
            Cancel
          </button>
          <button
            onClick={() => {
              localStorage.setItem('run-agent.apiBase', apiBase.trim());
              onClose();
            }}
            style={{ height: 34, padding: '0 12px', borderRadius: 10, border: 'none', background: '#007AFF', color: '#fff', fontWeight: 700, cursor: 'pointer' }}
          >
            Save
          </button>
        </div>
      </div>
    </div>
  );
}
