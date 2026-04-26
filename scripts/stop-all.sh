#!/bin/bash
# CEX DEX 服务停止脚本
# 用法: ./scripts/stop-all.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

main() {
    log_info "=== Stopping CEX DEX Services ==="

    # 停止所有 cargo run 进程
    pkill -f "cargo run" 2>/dev/null || true

    # 停止所有已知的 Rust 服务进程
    for service in user-service wallet-service portfolio-service order-service \
                   risk-service market-data-service matching-engine \
                   prediction-market-service ws-market-data \
                   ws-order ws-prediction api-gateway; do
        pkill -f "$service" 2>/dev/null || true
    done

    # 清理 PID 文件
    rm -f "$PROJECT_DIR"/logs/*.pid 2>/dev/null || true

    sleep 1

    log_info "All services stopped"
}

main "$@"
