# OpenRunner

轻量级 Agent 执行环境，基于 Rust + tokio channels 实现。

## Agent 依赖安装

使用脚本一键安装所有 agent：

```bash
./scripts/install-agents.sh
```

或单独安装：

```bash
./scripts/install-agents.sh --claude-code  # 只安装 Claude Code
./scripts/install-agents.sh --codex        # 只安装 Codex
./scripts/install-agents.sh --opencode     # 只安装 OpenCode
```

手动安装：

```bash
# Claude Code
npm install -g @anthropic-ai/claude-code

# OpenAI Codex (https://github.com/openai/codex)
npm install -g @openai/codex

# OpenCode (https://github.com/opencode-ai/opencode)
go install github.com/opencode-ai/opencode@latest
# 或
brew install opencode-ai/tap/opencode
```

检查 agent 可用状态：

```bash
curl http://localhost:3000/health/agents
```

## 特性

- 流式输出（SSE）
- 支持多种 Agent：claude_code、codex、opencode
- 简单的 HTTP API
- 可扩展架构，方便添加新 Agent

## 快速开始

```bash
# 编译
cargo build --release

# 运行（默认端口 3000）
cargo run

# 指定端口
OPENRUNNER_ADDR=0.0.0.0:8090 cargo run
```

## API 接口

### 健康检查
```bash
curl http://localhost:8090/health
```

### 检查 Agent 可用性
```bash
curl http://localhost:8090/health/agents
```

### 登录获取 Token
```bash
curl -X POST http://localhost:8090/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin"}'
```

默认用户：`admin/admin`, `user/user`

### 创建 Run（流式）
```bash
# 1. 创建 run
curl -X POST http://localhost:8090/api/runs \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "input": {"text": "create hello.py"},
    "metadata": {"agent_type": "claude_code", "cwd": "/tmp/test"}
  }'

# 返回: {"run_id": "run_abc123"}

# 2. 订阅事件流 (SSE)
curl -N "http://localhost:8090/api/runs/run_abc123/events?access_token=<token>"
```

### 非流式聊天（降级方案）
```bash
curl -X POST http://localhost:8090/api/chat \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{"message": "hello", "model": "claude"}'
```

## API 端点

| 端点 | 方法 | 说明 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/health/agents` | GET | 检查 agent 可用性 |
| `/api/auth/login` | POST | 登录获取 token |
| `/api/runs` | POST | 创建 run |
| `/api/runs/:id/events` | GET | SSE 事件流 |
| `/api/chat` | POST | 非流式聊天 |

## 配置选项

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| agent_type | string | claude_code | Agent 类型 |
| working_dir / cwd | string | null | 工作目录 |
| timeout_secs | number | 300 | 超时时间（秒）|
| model | string | null | 模型名称 |
| extra_args | array | [] | 额外命令行参数 |

## 架构

```
src/
├── agent/           # Agent 抽象和实现
│   ├── traits.rs    # Agent trait 定义
│   ├── handle.rs    # AgentHandle（轻量 Actor）
│   ├── claude_code.rs
│   ├── codex.rs
│   └── opencode.rs
├── api/             # HTTP API
│   ├── handlers.rs  # 请求处理
│   └── router.rs    # 路由配置
├── auth/            # JWT 认证
│   ├── mod.rs
│   └── jwt.rs
├── run/             # Run 管理
│   ├── store.rs     # 状态存储
│   ├── events.rs    # SSE 事件类型
│   └── manager.rs   # Run 生命周期管理
├── types.rs         # 公共类型
├── lib.rs
└── main.rs
```

## 添加新 Agent

1. 在 `src/agent/` 下创建新文件
2. 实现 `Agent` trait
3. 在 `src/agent/mod.rs` 中注册

```rust
use async_trait::async_trait;
use crate::types::StreamEvent;

pub struct MyAgent { /* ... */ }

#[async_trait]
impl Agent for MyAgent {
    fn name(&self) -> &str { "my_agent" }
    
    async fn run(&self, prompt: String, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        // 实现执行逻辑，通过 tx 发送流式输出
        tx.send(StreamEvent::Token { content: "Hello".into() }).await?;
        Ok(())
    }
}
```
