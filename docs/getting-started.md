# 快速开始

## 环境要求

- Rust 1.70+
- 至少一个 Agent CLI（Claude Code / Codex / OpenCode）

## 安装

```bash
git clone https://github.com/lipish/openrunner.git
cd openrunner
cargo build --release
```

## 安装 Agent CLI

使用安装脚本：

```bash
./scripts/install-agents.sh
```

或手动安装：

```bash
# Claude Code
npm install -g @anthropic-ai/claude-code

# OpenAI Codex
npm install -g @openai/codex

# OpenCode
go install github.com/opencode-ai/opencode@latest
```

## 配置 API Key

```bash
# Claude Code
export ANTHROPIC_API_KEY="sk-ant-xxx"

# Codex
export OPENAI_API_KEY="sk-xxx"
```

## 启动服务

```bash
# 默认端口 8080
cargo run --release

# 自定义端口
OPENRUNNER_ADDR=0.0.0.0:3000 cargo run --release
```

## 验证安装

```bash
# 健康检查
curl http://localhost:8080/health

# 检查 Agent 可用性
curl http://localhost:8080/health/agents
```

## 基本使用

### 1. 登录获取 Token

```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin"}'
```

响应：
```json
{
  "access_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {"id": "u_admin", "username": "admin"}
}
```

### 2. 创建 Run

```bash
export TOKEN="eyJ..."

curl -X POST http://localhost:8080/api/runs \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "input": {"text": "create a hello world python script"},
    "metadata": {"agent_type": "codex", "cwd": "/tmp/demo"}
  }'
```

响应：
```json
{"run_id": "run_abc123def456"}
```

### 3. 订阅事件流

```bash
curl -N "http://localhost:8080/api/runs/run_abc123def456/events?access_token=$TOKEN"
```

SSE 输出：
```
event: message_delta
data: {"delta": "Creating hello.py...\n"}

event: message_delta
data: {"delta": "print('Hello World')\n"}

event: run_completed
data: {"message": {"role": "assistant", "content": "...", "timestamp": "2026-01-17T..."}}
```

### 4. 非流式调用（可选）

```bash
curl -X POST http://localhost:8080/api/chat \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"message": "explain what is rust"}'
```

## 下一步

- 查看 [API 文档](./run-agent-api.md)
- 了解 [架构设计](./architecture.md)
- 查看 [OpenAPI 规范](./run-agent-openapi.yaml)
