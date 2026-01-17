# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-01-17

### Added

- 初始版本发布
- 基于 tokio channels 的轻量 Actor 架构
- 支持三种 Agent：Claude Code、Codex、OpenCode
- HTTP API with SSE 流式输出
- JWT 认证
- Run 生命周期管理
- Agent 健康检查接口
- 一键安装脚本

### API Endpoints

- `GET /health` - 健康检查
- `GET /health/agents` - Agent 可用性检查
- `POST /api/auth/login` - 登录
- `POST /api/runs` - 创建 Run
- `GET /api/runs/:id/events` - SSE 事件流
- `POST /api/chat` - 非流式聊天（降级）

### Agent Support

- **claude_code**: Claude Code CLI (`claude -p`)
- **codex**: OpenAI Codex CLI (`codex exec --full-auto`)
- **opencode**: OpenCode CLI (`opencode -p`)
