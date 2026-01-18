import React, { useState } from 'react';
import { LogOut, Plus, Settings, X } from 'lucide-react';
import { useAuth } from '../auth/AuthContext.jsx';
import ChatPanel from '../chat/ChatPanel.jsx';
import SettingsModal from './SettingsModal.jsx';

function newSession() {
  const id = `s_${Math.random().toString(16).slice(2)}`;
  return { id, title: 'New Agent', messages: [] };
}

export default function Shell() {
  const { logout } = useAuth();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [sessions, setSessions] = useState(() => [newSession()]);

  function addSession() {
    const s = newSession();
    setSessions((prev) => [s, ...prev]);
  }

  function removeSession(id) {
    setSessions((prev) => {
      const next = prev.filter((s) => s.id !== id);
      return next.length ? next : [newSession()];
    });
  }

  function updateSession(id, updater) {
    setSessions((prev) => prev.map((s) => (s.id === id ? updater(s) : s)));
  }

  return (
    <>
      <div className="ra-topbar">
        <div style={{ fontWeight: 800 }}>run-agent</div>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <button className="ra-icon-btn" onClick={addSession} title="New Agent">
            <Plus size={16} />
          </button>
          <button className="ra-icon-btn" onClick={() => setSettingsOpen(true)} title="Settings">
            <Settings size={16} />
          </button>
          <button className="ra-icon-btn" onClick={logout} title="Logout">
            <LogOut size={16} />
          </button>
        </div>
      </div>

      <div className="ra-grid">
        {sessions.map((s) => (
          <div key={s.id} className="ra-card">
            <div className="ra-card-header">
              <div style={{ minWidth: 0 }}>
                <div style={{ fontSize: 13, fontWeight: 800, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{s.title}</div>
                <div style={{ fontSize: 11, color: '#666' }}>{s.id}</div>
              </div>
              <button className="ra-icon-btn" onClick={() => removeSession(s.id)} title="Close">
                <X size={16} />
              </button>
            </div>

            <div style={{ flex: 1, minHeight: 0 }}>
              <ChatPanel session={s} onSessionChange={(updater) => updateSession(s.id, updater)} showHeader={false} />
            </div>
          </div>
        ))}
      </div>

      <SettingsModal open={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </>
  );
}
