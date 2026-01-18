import React, { useMemo, useRef, useState } from 'react';
import { ArrowUp, ChevronDown, GitPullRequestCreate, Hash, Image as ImageIcon, ListChecks, X } from 'lucide-react';
import { sendMessage } from '../../lib/agentApi.js';

function now() {
  return new Date();
}

const MODELS = ['Gemini-2.5-Pro', 'GPT-4.1', 'Claude Sonnet 4.5'];

function loadModel() {
  return localStorage.getItem('run-agent.model') || MODELS[0];
}

export default function ChatPanel({ session, onSessionChange, showHeader = true }) {
  const [input, setInput] = useState('');
  const [model, setModel] = useState(loadModel());
  const [attachments, setAttachments] = useState([]);

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

    const user = { id: `${Date.now()}_u`, role: 'user', content: text, attachments: attachmentMeta, model, timestamp: now() };
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
      const result = await sendMessage({ message: text, sessionId: session.id, onDelta: appendDelta, model, attachments: attachmentMeta });
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

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
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
              <div style={{ marginRight: 8, height: 32, display: 'flex', alignItems: 'center', gap: 6, background: '#F8F9FA', padding: '4px 8px', borderRadius: 8, cursor: 'pointer' }}>
                <select
                  value={model}
                  onChange={(e) => {
                    const v = e.target.value;
                    setModel(v);
                    localStorage.setItem('run-agent.model', v);
                  }}
                  style={{ border: 'none', outline: 'none', background: 'transparent', fontSize: 12, color: '#3C4043', cursor: 'pointer' }}
                >
                  {MODELS.map((m) => (
                    <option key={m} value={m}>
                      {m}
                    </option>
                  ))}
                </select>
                <ChevronDown size={16} style={{ transform: 'translateY(1px)', color: '#5F6368' }} />
              </div>

              <button title="添加任务" style={{ width: 32, height: 32, border: 'none', borderRadius: 8, background: '#F8F9FA', color: '#5F6368', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 0 }}>
                <GitPullRequestCreate size={16} />
              </button>

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
