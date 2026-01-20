import React, { useCallback, useEffect, useReducer, useRef, useState } from 'react';
import { ChevronDown, LogOut, Plus, Settings, Trash2, User } from 'lucide-react';
import { useAuth } from '../auth/AuthContext.jsx';
import ChatPanel from '../chat/ChatPanel.jsx';
import SettingsModal from './SettingsModal.jsx';
import { fetchSessions, saveSessions, fetchAgentDefaults, saveAgentDefault as saveAgentDefaultApi } from '../../lib/agentApi.js';

const AGENT_LABELS = {
  claude_code: 'Claude Code',
  codex: 'Codex',
  droid: 'Droid',
  opencode: 'OpenCode',
  kimi_cli: 'Kimi-Cli',
  augment: 'Augment',
  amp: 'AMP',
  mock: 'Mock',
};

function agentLabel(id) {
  return AGENT_LABELS[id] || id || 'Agent';
}

function newSession() {
  const id = `s_${Math.random().toString(16).slice(2)}`;
  return { id, title: 'New Agent', messages: [], agent_type: 'claude_code', model: '', env: {}, extra_args: [] };
}

function agentReducer(state, action) {
  switch (action.type) {
    case 'REPLACE_ALL': {
      return { ...state, sessions: action.sessions, hiddenSessions: action.hiddenSessions };
    }
    case 'ADD': {
      return { ...state, sessions: [newSession(), ...state.sessions] };
    }
    case 'HIDE': {
      const s = state.sessions.find((x) => x.id === action.id);
      if (!s) return state;
      const nextSessions = state.sessions.filter((x) => x.id !== action.id);
      const sessions = nextSessions.length ? nextSessions : [newSession()];
      if (state.hiddenSessions.some((x) => x.id === action.id)) return { ...state, sessions };
      return { ...state, sessions, hiddenSessions: [s, ...state.hiddenSessions] };
    }
    case 'RESTORE': {
      const s = state.hiddenSessions.find((x) => x.id === action.id);
      if (!s) return state;
      if (state.sessions.some((x) => x.id === action.id)) {
        return { ...state, hiddenSessions: state.hiddenSessions.filter((x) => x.id !== action.id) };
      }
      return {
        ...state,
        sessions: [s, ...state.sessions],
        hiddenSessions: state.hiddenSessions.filter((x) => x.id !== action.id),
      };
    }
    case 'CLOSE_HIDDEN': {
      return { ...state, hiddenSessions: state.hiddenSessions.filter((x) => x.id !== action.id) };
    }
    case 'UPDATE_SESSION': {
      return {
        ...state,
        sessions: state.sessions.map((s) => (s.id === action.id ? action.updater(s) : s)),
      };
    }
    default:
      return state;
  }
}

export default function Shell() {
  const { logout, token } = useAuth();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [userMenuOpen, setUserMenuOpen] = useState(false);
  const userMenuRef = useRef(null);
  const [agentState, dispatch] = useReducer(agentReducer, null, () => ({
    sessions: [newSession(), newSession(), newSession()],
    hiddenSessions: [],
  }));
  const persistTimeout = useRef(null);

  // Agent defaults - shared across all ChatPanels
  const [agentDefaults, setAgentDefaults] = useState({});

  const sessions = agentState.sessions;
  const hiddenSessions = agentState.hiddenSessions;

  const getAgentDefault = useCallback((agentType) => {
    return agentDefaults[agentType] || { model: '', env: {}, extra_args: [] };
  }, [agentDefaults]);

  const setAgentDefault = useCallback(async (agentType, config) => {
    try {
      await saveAgentDefaultApi(agentType, config);
      setAgentDefaults((prev) => ({ ...prev, [agentType]: config }));
      console.log(`[Shell] Saved defaults for ${agentType}:`, config);
    } catch (e) {
      console.error('[Shell] Failed to save defaults:', e);
    }
  }, []);

  // Helper to apply defaults to sessions that have no config
  const applyDefaultsToSessions = useCallback((sessionList, defaults) => {
    return sessionList.map((s) => {
      const agentDefault = defaults[s.agent_type];
      if (!agentDefault) return s;

      // Check if session has any config
      const hasConfig = s.model || (s.env && Object.keys(s.env).length > 0) || (s.extra_args && s.extra_args.length > 0);
      if (hasConfig) return s;

      // Apply defaults
      return {
        ...s,
        model: agentDefault.model || '',
        env: agentDefault.env || {},
        extra_args: agentDefault.extra_args || [],
      };
    });
  }, []);

  useEffect(() => {
    let active = true;
    if (!token) {
      dispatch({ type: 'REPLACE_ALL', sessions: [newSession(), newSession(), newSession()], hiddenSessions: [] });
      setAgentDefaults({});
      return () => {
        active = false;
      };
    }
    // Load sessions and agent defaults in parallel
    Promise.all([fetchSessions(), fetchAgentDefaults()])
      .then(([loaded, defaults]) => {
        if (!active) return;
        const visible = loaded.filter((s) => !s.hidden);
        const hidden = loaded.filter((s) => s.hidden);

        // Apply defaults to sessions that have no config
        const visibleWithDefaults = applyDefaultsToSessions(visible.length ? visible : [newSession()], defaults);
        const hiddenWithDefaults = applyDefaultsToSessions(hidden, defaults);

        dispatch({
          type: 'REPLACE_ALL',
          sessions: visibleWithDefaults,
          hiddenSessions: hiddenWithDefaults,
        });
        setAgentDefaults(defaults);
        console.log('[Shell] Loaded agent defaults:', defaults);
      })
      .catch((e) => {
        console.error('[Shell] Failed to load:', e);
      });
    return () => {
      active = false;
    };
  }, [token, applyDefaultsToSessions]);

  useEffect(() => {
    if (persistTimeout.current) clearTimeout(persistTimeout.current);
    if (!token) return;
    persistTimeout.current = setTimeout(() => {
      const payload = [
        ...sessions.map((s) => ({ ...s, hidden: false })),
        ...hiddenSessions.map((s) => ({ ...s, hidden: true })),
      ];
      saveSessions(payload).catch(() => {});
    }, 400);
    return () => {
      if (persistTimeout.current) clearTimeout(persistTimeout.current);
    };
  }, [sessions, hiddenSessions]);

  useEffect(() => {
    if (!userMenuOpen) return;

    const onMouseDown = (e) => {
      if (!userMenuRef.current) return;
      if (userMenuRef.current.contains(e.target)) return;
      setUserMenuOpen(false);
    };

    const onKeyDown = (e) => {
      if (e.key === 'Escape') setUserMenuOpen(false);
    };

    window.addEventListener('mousedown', onMouseDown);
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('mousedown', onMouseDown);
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [userMenuOpen]);

  function addSession() {
    dispatch({ type: 'ADD' });
  }

  function hideSession(id) {
    dispatch({ type: 'HIDE', id });
  }

  function restoreSession(id) {
    dispatch({ type: 'RESTORE', id });
  }

  function closeHiddenSession(id) {
    dispatch({ type: 'CLOSE_HIDDEN', id });
  }

  function updateSession(id, updater) {
    dispatch({ type: 'UPDATE_SESSION', id, updater });
  }

  return (
    <>
      <div className="ra-topbar">
        <div className="ra-brand">OpenRunner</div>
        <div className="ra-topbar-actions" aria-label="Actions">
          <button className="ra-primary-btn" onClick={addSession} title="New Agent" aria-label="New Agent">
            <Plus size={16} />
            <span style={{ fontSize: 12, fontWeight: 700 }}>New Agent</span>
          </button>

          <div className="ra-user-menu" ref={userMenuRef}>
            <button
              className="ra-action-btn"
              onClick={() => setUserMenuOpen((v) => !v)}
              title="Account"
              aria-label="Account"
              aria-expanded={userMenuOpen}
            >
              <User size={16} />
              <ChevronDown size={14} />
            </button>

            {userMenuOpen ? (
              <div className="ra-menu" role="menu">
                {hiddenSessions.length ? (
                  <>
                    <div className="ra-menu-label">Hidden agents</div>
                    {hiddenSessions.slice(0, 8).map((s) => (
                      <button
                        key={s.id}
                        className="ra-menu-item"
                        role="menuitem"
                        onClick={() => {
                          restoreSession(s.id);
                          setUserMenuOpen(false);
                        }}
                      >
                        <span style={{ flex: 1, minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{s.title}</span>
                        <span className="ra-menu-hint">Restore</span>
                      </button>
                    ))}
                    <div className="ra-menu-sep" />
                  </>
                ) : null}

                <button
                  className="ra-menu-item"
                  role="menuitem"
                  onClick={() => {
                    setUserMenuOpen(false);
                    setSettingsOpen(true);
                  }}
                >
                  <Settings size={16} />
                  Settings
                </button>
                <button
                  className="ra-menu-item"
                  role="menuitem"
                  onClick={() => {
                    setUserMenuOpen(false);
                    logout();
                  }}
                >
                  <LogOut size={16} />
                  Logout
                </button>
              </div>
            ) : null}
          </div>
        </div>
      </div>

      {hiddenSessions.length ? (
        <div className="ra-hidden-bar" aria-label="Hidden agents">
          <div className="ra-hidden-label">Hidden</div>
          <div className="ra-hidden-list">
            {hiddenSessions.slice(0, 12).map((s) => (
              <div key={s.id} className="ra-hidden-pill" title={`Restore: ${s.title} (${agentLabel(s.agent_type)})`}>
                <button
                  type="button"
                  className="ra-hidden-pill-main"
                  onClick={() => restoreSession(s.id)}
                  aria-label={`Restore: ${s.title} (${agentLabel(s.agent_type)})`}
                >
                  {s.title} · {agentLabel(s.agent_type)}
                </button>
                <button
                  type="button"
                  className="ra-hidden-pill-close"
                  onClick={() => closeHiddenSession(s.id)}
                  title="Close permanently"
                  aria-label="Close permanently"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            ))}
          </div>
        </div>
      ) : null}

      <div className="ra-grid">
        {sessions.map((s) => (
          <div key={s.id} className="ra-card">
            <div className="ra-card-header">
              <div style={{ minWidth: 0 }}>
                <div style={{ fontSize: 13, fontWeight: 800, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{s.title}</div>
              </div>
              <button className="ra-close-btn" onClick={() => hideSession(s.id)} title="Hide" aria-label="Hide">
                <ChevronDown size={16} />
              </button>
            </div>

            <div style={{ flex: 1, minHeight: 0 }}>
              <ChatPanel
                session={s}
                onSessionChange={(updater) => updateSession(s.id, updater)}
                showHeader={false}
                getAgentDefault={getAgentDefault}
                setAgentDefault={setAgentDefault}
              />
            </div>
          </div>
        ))}
      </div>

      <SettingsModal open={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </>
  );
}
