import React, { useEffect, useMemo, useRef, useState } from 'react';
import { ArrowUp, ChevronsUpDown, Hash, Image as ImageIcon, ListChecks, Settings, X } from 'lucide-react';
import { sendMessage } from '../../lib/agentApi.js';
import { Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '../components/ui/dialog.jsx';
import { Select, SelectOption } from '../components/ui/select.jsx';
import { Input, Textarea } from '../components/ui/input.jsx';
import { Button } from '../components/ui/button.jsx';
import { Label, Hint, Warning } from '../components/ui/label.jsx';

// Generate a random project name
function generateRandomProjectName() {
  const adjectives = ['quick', 'bright', 'calm', 'swift', 'bold', 'cool', 'sharp', 'smart'];
  const nouns = ['fox', 'wolf', 'hawk', 'bear', 'lion', 'tiger', 'eagle', 'falcon'];
  const adj = adjectives[Math.floor(Math.random() * adjectives.length)];
  const noun = nouns[Math.floor(Math.random() * nouns.length)];
  const num = Math.floor(Math.random() * 1000);
  return `${adj}-${noun}-${num}`;
}

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

function argsToText(args) {
  return Array.isArray(args) ? args.join('\n') : '';
}

function textToArgs(text) {
  return String(text || '')
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean);
}

function agentHints(agentType) {
  switch (agentType) {
    case 'kimi_cli':
      return {
        modelPlaceholder: 'e.g. kimi-k2 / kimi-k2-thinking',
        envPlaceholder: `KIMI_API_KEY=sk-...
KIMI_BASE_URL=https://api.kimi.com/coding/v1`,
        envHelp: 'Kimi CLI uses ~/.kimi/config.toml by default; you can override via --config-file.',
        argsPlaceholder: `--config-file
/path/to/kimi.toml`,
        argsHelp: 'Use this to point to a config file created from the UI.',
      };
    case 'codex':
      return {
        modelPlaceholder: 'e.g. gpt-4.1 / o4-mini',
        envPlaceholder: `OPENAI_API_KEY=sk-...
OPENAI_BASE_URL=https://api.openai.com/v1`,
        envHelp: 'OpenAI-compatible endpoints can be set via BASE_URL.',
        argsPlaceholder: `--model
gpt-4.1`,
        argsHelp: 'Args are appended to the codex CLI command.',
      };
    case 'opencode':
      return {
        modelPlaceholder: 'e.g. kat-coder-pro-v1',
        envPlaceholder: `OPENCODE_BASE_URL=https://api.example.com/v1
OPENCODE_API_KEY=your-api-key
OPENCODE_PROVIDER=custom`,
        envHelp: 'Set OPENCODE_BASE_URL, OPENCODE_API_KEY, OPENCODE_PROVIDER for custom providers. Settings are saved as defaults for this agent type.',
        argsPlaceholder: ``,
        argsHelp: 'Args are appended to the opencode CLI command.',
      };
    case 'claude_code':
      return {
        modelPlaceholder: 'e.g. claude-3-7-sonnet',
        envPlaceholder: `ANTHROPIC_API_KEY=sk-ant-...
ANTHROPIC_BASE_URL=https://api.anthropic.com`,
        envHelp: 'Set Anthropic API key and optional base URL.',
        argsPlaceholder: `--model
claude-3-7-sonnet`,
        argsHelp: 'Args are appended to the claude CLI command.',
      };
    default:
      return {
        modelPlaceholder: 'e.g. model-name',
        envPlaceholder: `API_KEY=sk-...
BASE_URL=https://api.example.com`,
        envHelp: 'Pass provider credentials as needed.',
        argsPlaceholder: `--config-file
/path/to/agent.toml`,
        argsHelp: 'Args are appended to the agent CLI command.',
      };
  }
}



// Default no-op functions for when props are not provided
const defaultGetAgentDefault = () => ({ model: '', env: {}, extra_args: [] });
const defaultSetAgentDefault = async () => {};
const defaultOnCreateProject = async () => {};

export default function ChatPanel({
  session,
  onSessionChange,
  showHeader = true,
  getAgentDefault = defaultGetAgentDefault,
  setAgentDefault = defaultSetAgentDefault,
  projects = [],
  onCreateProject = defaultOnCreateProject,
}) {
  const [input, setInput] = useState('');
  const [attachments, setAttachments] = useState([]);
  const [agentPickerOpen, setAgentPickerOpen] = useState(false);
  const [agentQuery, setAgentQuery] = useState('');
  const agentPickerRef = useRef(null);
  const agentSearchRef = useRef(null);

  const [agentSettingsOpen, setAgentSettingsOpen] = useState(false);
  const [modelDraft, setModelDraft] = useState(session?.model || '');
  const [envDraft, setEnvDraft] = useState(envToText(session?.env));
  const [extraArgsDraft, setExtraArgsDraft] = useState(argsToText(session?.extra_args));
  const [projectDraft, setProjectDraft] = useState(session?.project_id || '');
  const [newProjectName, setNewProjectName] = useState('');
  const [creatingProject, setCreatingProject] = useState(false);

  const agentType = session?.agent_type || 'claude_code';

  // Sync draft states when session changes (e.g., switching tabs or loading from server)
  // If session has empty settings, use defaults for this agent type
  useEffect(() => {
    const defaults = getAgentDefault(agentType);
    const sessionModel = session?.model || '';
    const sessionEnv = session?.env || {};
    const sessionArgs = session?.extra_args || [];

    // Use session values if they exist, otherwise use defaults
    const hasSessionConfig = sessionModel || Object.keys(sessionEnv).length > 0 || sessionArgs.length > 0;

    if (hasSessionConfig) {
      setModelDraft(sessionModel);
      setEnvDraft(envToText(sessionEnv));
      setExtraArgsDraft(argsToText(sessionArgs));
    } else {
      // Apply defaults for this agent type
      setModelDraft(defaults.model || '');
      setEnvDraft(envToText(defaults.env || {}));
      setExtraArgsDraft(argsToText(defaults.extra_args || []));
    }

    // Sync project
    setProjectDraft(session?.project_id || '');
  }, [session?.id, session?.model, session?.env, session?.extra_args, session?.project_id, agentType, getAgentDefault]);

  const model = session?.model || undefined;
  const env = session?.env || undefined;
  const extraArgs = session?.extra_args || undefined;
  const hints = agentHints(agentType);

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
      // Auto-create project if not set
      let projectId = session?.project_id || null;
      if (!projectId && onCreateProject) {
        const randomName = generateRandomProjectName();
        console.log(`[ChatPanel] Auto-creating project: ${randomName}`);
        try {
          const newProject = await onCreateProject(randomName);
          projectId = newProject.id;
          // Update session with the new project
          onSessionChange((s) => ({ ...s, project_id: projectId, _autoCreatedProject: randomName }));
        } catch (e) {
          console.error('Failed to auto-create project:', e);
        }
      }
      const result = await sendMessage({ message: text, sessionId: session.id, onDelta: appendDelta, model, agentType: agentType, env, extraArgs, attachments: attachmentMeta, projectId });
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
            setExtraArgsDraft(argsToText(session?.extra_args));
            setProjectDraft(session?.project_id || '');
            setNewProjectName('');
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Agent Settings</DialogTitle>
          </DialogHeader>
          <DialogBody>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
              <div>
                <Label>Project (Codebase)</Label>
                <Select value={projectDraft} onChange={(e) => setProjectDraft(e.target.value)}>
                  <SelectOption value="">-- No Project (auto-create) --</SelectOption>
                  {projects.map((p) => (
                    <SelectOption key={p.id} value={p.id}>
                      {p.name}
                    </SelectOption>
                  ))}
                </Select>
                <div style={{ marginTop: 8, display: 'flex', gap: 8, alignItems: 'center' }}>
                  <Input
                    value={newProjectName}
                    onChange={(e) => setNewProjectName(e.target.value)}
                    placeholder="New project name..."
                    style={{ flex: 1, padding: '8px 10px', fontSize: 13 }}
                  />
                  <Button
                    size="sm"
                    disabled={!newProjectName.trim() || creatingProject}
                    onClick={async () => {
                      if (!newProjectName.trim()) return;
                      setCreatingProject(true);
                      try {
                        const project = await onCreateProject(newProjectName.trim());
                        setProjectDraft(project.id);
                        setNewProjectName('');
                      } catch (e) {
                        console.error('Failed to create project:', e);
                      } finally {
                        setCreatingProject(false);
                      }
                    }}
                  >
                    {creatingProject ? '...' : 'Create'}
                  </Button>
                </div>
                {!projectDraft && (
                  <Warning>No project selected. A random project will be created when you send a message.</Warning>
                )}
                {session?._autoCreatedProject && (
                  <Warning>Auto-created project "{session._autoCreatedProject}". You can rename it in Settings.</Warning>
                )}
                <Hint>Agent will run in the project directory. Select or create a project for your codebase.</Hint>
              </div>
              <div>
                <Label>Model (optional)</Label>
                <Input
                  value={modelDraft}
                  onChange={(e) => setModelDraft(e.target.value)}
                  placeholder={hints.modelPlaceholder}
                />
              </div>
              <div>
                <Label>Environment variables (KEY=VALUE per line)</Label>
                <Textarea
                  value={envDraft}
                  onChange={(e) => setEnvDraft(e.target.value)}
                  rows={6}
                  spellCheck={false}
                  placeholder={hints.envPlaceholder}
                />
                <Hint>{hints.envHelp}</Hint>
              </div>
              <div>
                <Label>Extra CLI args (one per line)</Label>
                <Textarea
                  value={extraArgsDraft}
                  onChange={(e) => setExtraArgsDraft(e.target.value)}
                  rows={4}
                  spellCheck={false}
                  placeholder={hints.argsPlaceholder}
                />
                <Hint>{hints.argsHelp}</Hint>
              </div>
            </div>
          </DialogBody>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAgentSettingsOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={() => {
                const nextModel = modelDraft.trim();
                const nextEnv = textToEnv(envDraft);
                const nextArgs = textToArgs(extraArgsDraft);
                const nextProjectId = projectDraft || null;
                // Save to current session
                onSessionChange((s) => ({ ...s, model: nextModel || '', env: nextEnv, extra_args: nextArgs, project_id: nextProjectId, _autoCreatedProject: null }));
                // Also save as default for this agent type
                setAgentDefault(agentType, { model: nextModel || '', env: nextEnv, extra_args: nextArgs });
                console.log(`[AgentSettings] Saved settings for ${agentType}:`, { model: nextModel, env: nextEnv, extra_args: nextArgs, project_id: nextProjectId });
                setAgentSettingsOpen(false);
              }}
            >
              Save
            </Button>
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
                            const defaults = getAgentDefault(a.id);
                            onSessionChange((s) => ({
                              ...s,
                              agent_type: a.id,
                              model: defaults.model || '',
                              env: defaults.env || {},
                              extra_args: defaults.extra_args || []
                            }));
                            setAgentPickerOpen(false);
                          }
                        }}
                        placeholder="Search agent…"
                        style={{ width: '100%', padding: '8px 10px', border: '1px solid #D1D5DB', borderRadius: 10, fontSize: 13 }}
                      />
                      <div className="ra-agent-scroll" style={{ marginTop: 8, display: 'flex', flexDirection: 'column', gap: 4, maxHeight: 240, overflow: 'auto' }}>
                        {filteredAgents.map((a) => (
                          <button
                            key={a.id}
                            type="button"
                            className={`ra-agent-item${a.id === agentType ? ' is-selected' : ''}`}
                            onClick={() => {
                              const defaults = getAgentDefault(a.id);
                              console.log(`[AgentPicker] Switching to ${a.id}, defaults:`, defaults);
                              onSessionChange((s) => ({
                                ...s,
                                agent_type: a.id,
                                model: defaults.model || '',
                                env: defaults.env || {},
                                extra_args: defaults.extra_args || []
                              }));
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
