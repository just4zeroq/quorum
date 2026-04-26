#!/bin/bash
# CEX DEX 服务启动脚本
# 用法: ./scripts/start-all.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# 确保 cargo 和 rtk 可用
export PATH="/d/soft/rtk:/home/ubuntu/.cargo/bin:$PATH"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 停止已有进程
stop_services() {
    log_info "Stopping existing services..."
    pkill -f "cargo run" 2>/dev/null || true
    pkill -f "user_service" 2>/dev/null || true
    pkill -f "wallet_service" 2>/dev/null || true
    pkill -f "portfolio_service" 2>/dev/null || true
    pkill -f "order_service" 2>/dev/null || true
    pkill -f "risk_service" 2>/dev/null || true
    pkill -f "market_data_service" 2>/dev/null || true
    pkill -f "matching_engine" 2>/dev/null || true
    pkill -f "prediction_market_service" 2>/dev/null || true
    pkill -f "ws_market_data" 2>/dev/null || true
    pkill -f "api_gateway" 2>/dev/null || true
    sleep 1
}

# 启动函数 - 直接运行二进制文件
start_service() {
    local name=$1
    local binary=$2
    local port=$3

    log_info "Starting $name on port $port..."

    # 后台运行服务
    ./target/release/$binary > "logs/${name}.log" 2>&1 &
    local pid=$!

    # 等待服务启动
    sleep 2

    # 检查进程是否还在运行
    if kill -0 $pid 2>/dev/null; then
        log_info "$name started (PID: $pid)"
        echo $pid > "logs/${name}.pid"
    else
        log_error "$name failed to start. Check logs/${name}.log"
    fi
}

# 主流程
main() {
    log_info "=== CEX DEX Services Starting ==="

    # 创建日志目录
    mkdir -p logs

    # 停止已有服务
    stop_services

    # 先编译所有服务
    log_info "Building all services..."
    cargo build --release

    log_info "Starting gRPC services..."

    # 启动所有 gRPC 服务 (直接运行二进制文件避免 cargo 锁)
    start_service "user-service" "user_service" "50001" &
    start_service "wallet-service" "wallet_service" "50002" &
    start_service "portfolio-service" "portfolio_service" "50003" &
    start_service "order-service" "order_service" "50004" &
    start_service "risk-service" "risk_service" "50005" &
    start_service "market-data-service" "market_data_service" "50006" &
    start_service "matching-engine" "matching_engine" "50007" &
    start_service "prediction-market-service" "prediction_market_service" "50008" &
    start_service "ws-market-data" "ws_market_data" "50016" &

    # 等待所有 gRPC 服务启动完成
    wait

    sleep 2

    # 启动 API Gateway (最后启动，作为统一入口)
    start_service "api-gateway" "api_gateway" "8080"

    log_info "=== All services started ==="
    log_info "API Gateway: http://localhost:8080"
    log_info ""
    log_info "Logs are in: logs/"
    log_info "To stop all services: ./scripts/stop-all.sh"
}

main "$@"
