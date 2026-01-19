import React, { useEffect, useMemo, useRef, useState } from 'react';
import { ArrowUp, ChevronsUpDown, Hash, Image as ImageIcon, ListChecks, Settings, X } from 'lucide-react';
import { sendMessage } from '../../lib/agentApi.js';
import { Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '../components/ui/dialog.jsx';

function now() {
  return new Date();
}

const AGENTS = [
  { id: 'claude_code', label: 'Claude Code' },
  { id: 'codex', label: 'Codex' },
  { id: 'droid', label: 'Droid' },
  { id: 'opencode', label: 'OpenCode' },
  { id: 'kimi_cli', label: 'Kimi-Cli' },
  { id: 'augment', label: 'Augment' },
  { id: 'amp', label: 'AMP' },
  { id: 'mock', label: 'Mock' },
];

function agentLabel(id) {
  return AGENTS.find((a) => a.id === id)?.label || id || 'Agent';
}

function envToText(env) {
  if (!env || typeof env !== 'object') return '';
  return Object.entries(env)
    .map(([k, v]) => `${k}=${v}`)
    .join('\n');
}

function textToEnv(text) {
  const env = {};
  String(text || '')
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean)
    .forEach((line) => {
      const idx = line.indexOf('=');
      if (idx <= 0) return;
      const k = line.slice(0, idx).trim();
      const v = line.slice(idx + 1).trim();
      if (k) env[k] = v;
    });
  return env;
}

export default function ChatPanel({ session, onSessionChange, showHeader = true }) {
  const [input, setInput] = useState('');
  const [attachments, setAttachments] = useState([]);
  const [agentPickerOpen, setAgentPickerOpen] = useState(false);
  const [agentQuery, setAgentQuery] = useState('');
  const agentPickerRef = useRef(null);
  const agentSearchRef = useRef(null);

  const [agentSettingsOpen, setAgentSettingsOpen] = useState(false);
  const [modelDraft, setModelDraft] = useState(session?.model || '');
  const [envDraft, setEnvDraft] = useState(envToText(session?.env));

  const agentType = session?.agent_type || 'claude_code';
  const model = session?.model || undefined;
  const env = session?.env || undefined;

  useEffect(() => {
    if (!agentPickerOpen) return;

    // wait for the popover to render
    setTimeout(() => agentSearchRef.current?.focus(), 0);

    const onMouseDown = (e) => {
      if (!agentPickerRef.current) return;
      if (agentPickerRef.current.contains(e.target)) return;
      setAgentPickerOpen(false);
    };
    const onKeyDown = (e) => {
      if (e.key === 'Escape') setAgentPickerOpen(false);
    };
    window.addEventListener('mousedown', onMouseDown);
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('mousedown', onMouseDown);
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [agentPickerOpen]);

  const listRef = useRef(null);
  const textareaRef = useRef(null);
  const fileInputRef = useRef(null);
  const messages = session?.messages || [];

  const scrollToBottom = () => {
    const el = listRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  };

  function adjustTextareaHeight() {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = 'auto';

    const style = getComputedStyle(el);
    const lineHeight = parseFloat(style.lineHeight);
    const paddingTop = parseFloat(style.paddingTop);
    const paddingBottom = parseFloat(style.paddingBottom);
    const maxHeight = lineHeight * 3 + paddingTop + paddingBottom;

    const sh = el.scrollHeight;
    if (sh > maxHeight) {
      el.style.height = `${maxHeight}px`;
      el.style.overflowY = 'auto';
    } else {
      el.style.height = `${sh}px`;
      el.style.overflowY = 'hidden';
    }
  }

  async function onSend() {
    const text = input.trim();
    if (!text && attachments.length === 0) return;

    const attachmentMeta = attachments.map((f) => ({ name: f.name, type: f.type, size: f.size }));

    const user = { id: `${Date.now()}_u`, role: 'user', content: text, attachments: attachmentMeta, model, agent_type: agentType, timestamp: now() };
    const assistantId = `${Date.now()}_a`;
    const assistant = { id: assistantId, role: 'assistant', content: '', status: 'running', timestamp: now() };

    onSessionChange((s) => ({ ...s, messages: [...s.messages, user, assistant], title: s.title === 'New Agent' ? (text || 'New Agent').slice(0, 18) : s.title }));
    setInput('');
    setAttachments([]);
    setTimeout(() => {
      adjustTextareaHeight();
      scrollToBottom();
    }, 0);

    const appendDelta = (delta) => {
      onSessionChange((s) => ({
        ...s,
        messages: s.messages.map((m) => (m.id === assistantId ? { ...m, content: (m.content || '') + delta } : m)),
      }));
      setTimeout(scrollToBottom, 0);
    };

    try {
      const result = await sendMessage({ message: text, sessionId: session.id, onDelta: appendDelta, model, agentType: agentType, env, attachments: attachmentMeta });
      onSessionChange((s) => ({
        ...s,
        messages: s.messages.map((m) =>
          m.id === assistantId
            ? { ...m, role: result.role || 'assistant', content: result.content || m.content, status: 'done', timestamp: result.timestamp ? new Date(result.timestamp) : now() }
            : m
        ),
      }));
    } catch (e) {
      onSessionChange((s) => ({
        ...s,
        messages: s.messages.map((m) => (m.id === assistantId ? { ...m, status: 'error', content: `Backend error: ${e?.message || e}` } : m)),
      }));
    }
  }

  const canSend = useMemo(() => input.trim().length > 0 || attachments.length > 0, [input, attachments.length]);

  const filteredAgents = useMemo(() => {
    const q = agentQuery.trim().toLowerCase();
    if (!q) return AGENTS;
    return AGENTS.filter((a) => a.label.toLowerCase().includes(q) || a.id.toLowerCase().includes(q));
  }, [agentQuery]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <Dialog
        open={agentSettingsOpen}
        onOpenChange={(v) => {
          setAgentSettingsOpen(v);
          if (v) {
            setModelDraft(session?.model || '');
            setEnvDraft(envToText(session?.env));
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Agent Settings</DialogTitle>
          </DialogHeader>
          <DialogBody>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
              <div>
                <div style={{ fontSize: 12, color: '#666', marginBottom: 6 }}>Model (optional)</div>
                <input
                  value={modelDraft}
                  onChange={(e) => setModelDraft(e.target.value)}
                  placeholder="e.g. glm-4.7 / gpt-4.1 / claude-sonnet"
                  style={{ width: '100%', padding: '10px 12px', border: '1px solid #D1D5DB', borderRadius: 10, fontSize: 14 }}
                />
              </div>
              <div>
                <div style={{ fontSize: 12, color: '#666', marginBottom: 6 }}>Environment variables (KEY=VALUE per line)</div>
                <textarea
                  value={envDraft}
                  onChange={(e) => setEnvDraft(e.target.value)}
                  rows={8}
                  spellCheck={false}
                  placeholder="ANTHROPIC_BASE_URL=https://...\nANTHROPIC_API_KEY=..."
                  style={{ width: '100%', padding: '10px 12px', border: '1px solid #D1D5DB', borderRadius: 10, fontSize: 13, fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace' }}
                />
              </div>
            </div>
          </DialogBody>
          <DialogFooter>
            <button
              type="button"
              onClick={() => setAgentSettingsOpen(false)}
              style={{ height: 34, padding: '0 12px', borderRadius: 10, border: '1px solid #D1D5DB', background: '#fff', cursor: 'pointer' }}
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={() => {
                const nextModel = modelDraft.trim();
                const nextEnv = textToEnv(envDraft);
                onSessionChange((s) => ({ ...s, model: nextModel || '', env: nextEnv }));
                setAgentSettingsOpen(false);
              }}
              style={{ height: 34, padding: '0 12px', borderRadius: 10, border: 'none', background: '#007AFF', color: '#fff', fontWeight: 700, cursor: 'pointer' }}
            >
              Save
            </button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      {showHeader ? (
        <div style={{ padding: '12px 14px', borderBottom: '1px solid #E5E5EA', background: '#FAFAFA', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div style={{ margin: 0, fontSize: 14, fontWeight: 700 }}>{session?.title || 'Agent'}</div>
          <div style={{ fontSize: 12, color: '#666' }}>{session?.id}</div>
        </div>
      ) : null}

      <div ref={listRef} style={{ flex: 1, overflowY: 'auto', padding: '20px 16px', background: '#fff' }}>
        {messages.length === 0 ? (
          <div style={{ height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#8E8E93' }}>Start a conversation with your AI assistant</div>
        ) : (
          messages.map((m) => (
            <div key={m.id} style={{ marginBottom: 20, display: 'flex', flexDirection: 'column', alignItems: m.role === 'user' ? 'flex-end' : 'flex-start' }}>
              <div
                style={{
                  maxWidth: '85%',
                  padding: '10px 14px',
                  borderRadius: 16,
                  wordWrap: 'break-word',
                  background: m.role === 'user' ? '#007AFF' : '#F2F2F7',
                  color: m.role === 'user' ? '#fff' : '#000',
                }}
              >
                <div style={{ fontSize: 15, lineHeight: 1.4, whiteSpace: 'pre-wrap' }}>
                  {m.content}
                  {m.status === 'running' ? '…' : ''}
                </div>
                {Array.isArray(m.attachments) && m.attachments.length > 0 ? (
                  <div style={{ marginTop: 8, display: 'flex', flexDirection: 'column', gap: 6 }}>
                    {m.attachments.map((a) => (
                      <div key={a.name} style={{ fontSize: 12, opacity: 0.9 }}>
                        📎 {a.name}
                      </div>
                    ))}
                  </div>
                ) : null}
              </div>
              <div style={{ fontSize: 11, color: '#8E8E93', marginTop: 4, padding: '0 4px' }}>{new Date(m.timestamp).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })}</div>
            </div>
          ))
        )}
      </div>

      <div style={{ padding: 16, background: '#F0F4F9', borderTop: '1px solid #D1D5DB' }}>
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            gap: 8,
            padding: 12,
            background: '#fff',
            border: '1px solid #D1D5DB',
            borderRadius: 12,
            boxShadow: '0 1px 2px rgba(0,0,0,0.05)',
          }}
        >
          <div style={{ width: '100%', display: 'flex', alignItems: 'stretch', minHeight: 'calc(1.5em * 3 + 12px)' }}>
            <textarea
              ref={textareaRef}
              value={input}
              onChange={(e) => {
                setInput(e.target.value);
              }}
              onInput={adjustTextareaHeight}
              placeholder="请输入..."
              rows={3}
              spellCheck={false}
              style={{ width: '100%', boxSizing: 'border-box', padding: '8px 12px', fontSize: 15, lineHeight: 1.5, border: 'none', background: 'transparent', outline: 'none', color: '#202124', resize: 'none', minHeight: 'calc(1.5em * 3)', overflow: 'hidden', textAlign: 'left' }}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault();
                  onSend();
                }
              }}
            />
          </div>

          {attachments.length > 0 ? (
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
              {attachments.map((f) => (
                <div key={f.name} style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, background: '#F1F3F4', color: '#3C4043', padding: '4px 8px', borderRadius: 10 }}>
                  <span style={{ maxWidth: 220, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{f.name}</span>
                  <button
                    onClick={() => setAttachments((prev) => prev.filter((x) => x !== f))}
                    title="移除"
                    style={{ width: 18, height: 18, border: 'none', background: 'transparent', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#5F6368' }}
                  >
                    <X size={14} />
                  </button>
                </div>
              ))}
            </div>
          ) : null}

          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
              <button title="引用上下文" style={{ width: 32, height: 32, border: 'none', borderRadius: 8, background: 'transparent', color: '#5F6368', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 0 }}>
                <Hash size={16} />
              </button>
              <button
                className="action-button"
                onClick={() => fileInputRef.current?.click()}
                title="上传图片"
                style={{ width: 32, height: 32, border: 'none', borderRadius: 8, background: 'transparent', color: '#5F6368', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 0 }}
              >
                <ImageIcon size={16} />
              </button>
              <button title="任务列表" style={{ width: 32, height: 32, border: 'none', borderRadius: 8, background: 'transparent', color: '#5F6368', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 0 }}>
                <ListChecks size={16} />
              </button>
              <input
                ref={fileInputRef}
                type="file"
                accept="image/*"
                multiple
                style={{ display: 'none' }}
                onChange={(e) => {
                  const files = Array.from(e.target.files || []);
                  setAttachments((prev) => [...prev, ...files]);
                  e.target.value = '';
                }}
              />
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
              <div style={{ marginRight: 8, height: 32, display: 'flex', alignItems: 'center', gap: 6, background: '#F8F9FA', padding: '4px 8px', borderRadius: 8 }}>
                <div style={{ position: 'relative' }} ref={agentPickerRef}>
                  <button
                    type="button"
                    onClick={() => {
                      setAgentPickerOpen((v) => !v);
                      setAgentQuery('');
                    }}
                    title="Select agent"
                    style={{ height: 28, border: 'none', outline: 'none', background: 'transparent', fontSize: 12, color: '#3C4043', cursor: 'pointer', display: 'inline-flex', alignItems: 'center', gap: 6, padding: '0 2px' }}
                  >
                    <span style={{ maxWidth: 140, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{agentLabel(agentType)}</span>
                    <ChevronsUpDown size={16} style={{ color: '#5F6368' }} />
                  </button>

                  {agentPickerOpen ? (
                    <div
                      style={{
                        position: 'absolute',
                        right: 0,
                        bottom: 'calc(100% + 8px)',
                        width: 260,
                        background: '#fff',
                        border: '1px solid #E5E5EA',
                        borderRadius: 12,
                        boxShadow: '0 12px 30px rgba(0,0,0,0.14)',
                        padding: 8,
                        zIndex: 200,
                      }}
                    >
                      <input
                        ref={agentSearchRef}
                        value={agentQuery}
                        onChange={(e) => setAgentQuery(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter' && agentQuery.trim() && filteredAgents.length > 0) {
                            e.preventDefault();
                            const a = filteredAgents[0];
                            onSessionChange((s) => ({ ...s, agent_type: a.id }));
                            setAgentPickerOpen(false);
                          }
                        }}
                        placeholder="Search agent…"
                        style={{ width: '100%', padding: '8px 10px', border: '1px solid #D1D5DB', borderRadius: 10, fontSize: 13 }}
                      />
                      <div style={{ marginTop: 8, display: 'flex', flexDirection: 'column', gap: 4, maxHeight: 240, overflow: 'auto' }}>
                        {filteredAgents.map((a) => (
                          <button
                            key={a.id}
                            type="button"
                            className={`ra-agent-item${a.id === agentType ? ' is-selected' : ''}`}
                            onClick={() => {
                              onSessionChange((s) => ({ ...s, agent_type: a.id }));
                              setAgentPickerOpen(false);
                            }}
                          >
                            <span>{a.label}</span>
                            <span className="ra-agent-item-hint">{a.id}</span>
                          </button>
                        ))}
                      </div>
                    </div>
                  ) : null}
                </div>

                <button
                  type="button"
                  onClick={() => setAgentSettingsOpen(true)}
                  title="Agent settings"
                  style={{ width: 28, height: 28, border: 'none', borderRadius: 8, background: 'transparent', color: '#5F6368', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 0 }}
                >
                  <Settings size={16} />
                </button>
              </div>

              <button
                onClick={onSend}
                disabled={!canSend}
                title="发送"
                style={{ width: 32, height: 32, borderRadius: 8, border: 'none', background: canSend ? '#1A73E8' : '#EEE', color: canSend ? '#fff' : '#999', cursor: canSend ? 'pointer' : 'not-allowed', display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 0 }}
              >
                <ArrowUp size={16} />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
