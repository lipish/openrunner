# OpenRunner 架构文档

## 概述

OpenRunner 是一个轻量级的 Agent 执行环境，基于 Rust + tokio channels 实现。支持多种 AI 编码助手（Claude Code、Codex、OpenCode）的统一调度和流式输出。

## 核心设计

### 轻量 Actor 模式

使用 tokio channels 模拟 Actor 行为，而非重量级 Actor 框架：

```
┌─────────────┐     mpsc::channel     ┌─────────────┐
│   Client    │ ──────────────────▶  │ AgentHandle │
│  (HTTP API) │                       │  (Actor)    │
└─────────────┘                       └──────┬──────┘
                                             │
                                             ▼
                                      ┌─────────────┐
                                      │    Agent    │
                                      │ (子进程执行) │
                                      └─────────────┘
```

### 模块结构

```
src/
├── agent/              # Agent 抽象层
│   ├── traits.rs       # Agent trait 定义
│   ├── handle.rs       # AgentHandle - 轻量 Actor 封装
│   ├── claude_code.rs  # Claude Code CLI 适配
│   ├── codex.rs        # OpenAI Codex CLI 适配
│   └── opencode.rs     # OpenCode CLI 适配
│
├── api/                # HTTP API 层
│   ├── handlers.rs     # 请求处理函数
│   └── router.rs       # 路由配置 + AppState
│
├── auth/               # 认证模块
│   ├── mod.rs          # 用户验证
│   └── jwt.rs          # JWT Token 处理
│
├── run/                # Run 生命周期管理
│   ├── store.rs        # 并发状态存储 (DashMap)
│   ├── events.rs       # SSE 事件类型定义
│   └── manager.rs      # Run 创建/执行/订阅
│
├── types.rs            # 公共类型定义
├── lib.rs              # 库入口
└── main.rs             # 服务入口
```

## 数据流

### 创建 Run 流程

```
1. POST /api/runs
   │
   ▼
2. RunManager.create_run()
   │  - 生成 run_id
   │  - 存储到 RunStore
   │
   ▼
3. RunManager.start_run()
   │  - 创建 Agent
   │  - spawn AgentHandle
   │  - 启动子进程
   │
   ▼
4. 返回 { run_id }
```

### SSE 事件流

```
GET /api/runs/:id/events
        │
        ▼
   subscribe(run_id)
        │
        ▼
   mpsc::Receiver<RunEvent>
        │
        ▼
   ┌────────────────────────────┐
   │  event: message_delta      │
   │  data: {"delta": "..."}    │
   ├────────────────────────────┤
   │  event: run_completed      │
   │  data: {"message": {...}}  │
   └────────────────────────────┘
```

## Agent 适配

每个 Agent 实现 `Agent` trait：

```rust
#[async_trait]
pub trait Agent: Send + Sync + 'static {
    fn name(&self) -> &str;
    async fn run(&self, prompt: String, tx: mpsc::Sender<StreamEvent>) -> Result<()>;
    async fn health_check(&self) -> Result<()>;
}
```

### CLI 适配方式

| Agent | 命令 | 非交互参数 |
|-------|------|-----------|
| claude_code | `claude` | `-p --dangerously-skip-permissions` |
| codex | `codex exec` | `--full-auto --skip-git-repo-check` |
| opencode | `opencode` | `-p` + `TERM=dumb` |

## 状态管理

使用 `DashMap` 实现并发安全的 Run 存储：

```rust
pub struct RunStore {
    runs: Arc<DashMap<String, Run>>,
}
```

Run 状态机：

```
Pending → Running → Completed
                 ↘ Failed
                 ↘ Cancelled
```

## 认证

- JWT Bearer Token
- SSE 端点使用 query param: `?access_token=xxx`
- 默认用户：`admin/admin`, `user/user`

## 扩展

### 添加新 Agent

1. 创建 `src/agent/my_agent.rs`
2. 实现 `Agent` trait
3. 在 `src/agent/mod.rs` 注册
4. 更新 `create_agent()` 工厂函数

### 未来扩展

- DAG 编排（类似 Airflow）
- Agent 进程池
- 持久化存储
- 分布式执行
