#!/bin/bash
set -e

echo "=== OpenRunner Agent Installer ==="
echo ""

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

check_command() {
    if command -v "$1" &> /dev/null; then
        echo -e "${GREEN}✓${NC} $1 已安装"
        return 0
    else
        echo -e "${RED}✗${NC} $1 未安装"
        return 1
    fi
}

install_claude_code() {
    echo ""
    echo -e "${YELLOW}[1/3] Claude Code${NC}"
    if check_command "claude"; then
        return
    fi
    
    if ! check_command "npm"; then
        echo "  需要先安装 Node.js: https://nodejs.org/"
        return 1
    fi
    
    echo "  安装中..."
    npm install -g @anthropic-ai/claude-code
    echo -e "  ${GREEN}✓ Claude Code 安装完成${NC}"
    echo "  配置: export ANTHROPIC_API_KEY=your_key"
}

install_codex() {
    echo ""
    echo -e "${YELLOW}[2/3] OpenAI Codex${NC}"
    if check_command "codex"; then
        return
    fi
    
    if ! check_command "npm"; then
        echo "  需要先安装 Node.js: https://nodejs.org/"
        return 1
    fi
    
    echo "  安装中..."
    npm install -g @openai/codex
    echo -e "  ${GREEN}✓ Codex 安装完成${NC}"
    echo "  配置: export OPENAI_API_KEY=your_key"
}

install_opencode() {
    echo ""
    echo -e "${YELLOW}[3/3] OpenCode${NC}"
    if check_command "opencode"; then
        return
    fi
    
    # 尝试 brew
    if check_command "brew"; then
        echo "  通过 Homebrew 安装..."
        brew install opencode-ai/tap/opencode || {
            echo "  Homebrew 安装失败，尝试 go install..."
            install_opencode_go
        }
        return
    fi
    
    install_opencode_go
}

install_opencode_go() {
    if ! check_command "go"; then
        echo "  需要先安装 Go: https://go.dev/dl/"
        return 1
    fi
    
    echo "  通过 go install 安装..."
    go install github.com/opencode-ai/opencode@latest
    echo -e "  ${GREEN}✓ OpenCode 安装完成${NC}"
}

# 解析参数
INSTALL_ALL=true
INSTALL_CLAUDE=false
INSTALL_CODEX=false
INSTALL_OPENCODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --claude-code)
            INSTALL_ALL=false
            INSTALL_CLAUDE=true
            shift
            ;;
        --codex)
            INSTALL_ALL=false
            INSTALL_CODEX=true
            shift
            ;;
        --opencode)
            INSTALL_ALL=false
            INSTALL_OPENCODE=true
            shift
            ;;
        --help|-h)
            echo "用法: $0 [选项]"
            echo ""
            echo "选项:"
            echo "  --claude-code  只安装 Claude Code"
            echo "  --codex        只安装 OpenAI Codex"
            echo "  --opencode     只安装 OpenCode"
            echo "  --help         显示帮助"
            echo ""
            echo "不带参数时安装所有 agent"
            exit 0
            ;;
        *)
            echo "未知选项: $1"
            exit 1
            ;;
    esac
done

# 执行安装
if $INSTALL_ALL || $INSTALL_CLAUDE; then
    install_claude_code
fi

if $INSTALL_ALL || $INSTALL_CODEX; then
    install_codex
fi

if $INSTALL_ALL || $INSTALL_OPENCODE; then
    install_opencode
fi

echo ""
echo "=== 安装完成 ==="
echo ""
echo "验证安装:"
echo "  curl http://localhost:3000/health/agents"
