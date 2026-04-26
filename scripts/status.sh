#!/bin/bash
# CEX DEX 服务状态检查脚本
# 用法: ./scripts/status.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

main() {
    echo "=== CEX DEX Services Status ==="
    echo ""

    local total=0
    local running=0

    services=(
        "50001:User Service"
        "50002:Wallet Service"
        "50003:Portfolio Service"
        "50004:Order Service"
        "50005:Risk Service"
        "50006:Market Data Service"
        "50007:Matching Engine"
        "50008:Prediction Market Service"
        "50009:Auth Service"
        "50016:WS Market Data"
        "8080:API Gateway"
    )

    for entry in "${services[@]}"; do
        port="${entry%%:*}"
        name="${entry#*:}"
        total=$((total + 1))

        # 检查端口是否监听 (使用 bash 内置 /dev/tcp 或 nc)
        if timeout 1 bash -c "echo >/dev/tcp/localhost/$port" 2>/dev/null; then
            echo -e "${GREEN}✓${NC} $name (port $port)"
            running=$((running + 1))
        else
            echo -e "${RED}✗${NC} $name (port $port)"
        fi
    done

    echo ""
    echo "Status: $running/$total services running"

    # 检查日志中的错误
    if [ -d "$PROJECT_DIR/logs" ]; then
        error_count=$(grep -l "ERROR\|panic\|failed" "$PROJECT_DIR"/logs/*.log 2>/dev/null | wc -l || true)
        if [ "$error_count" -gt 0 ]; then
            echo -e "${YELLOW}Warning: $error_count log file(s) contain errors${NC}"
        fi
    fi
}

main "$@"
