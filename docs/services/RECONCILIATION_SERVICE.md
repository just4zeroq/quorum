# Reconciliation Service - 对账服务

## 1. 服务概述

| 项目 | 值 |
|------|---|
| 端口 | 50014 |
| 协议 | gRPC |
| 数据库 | 独立 SQLite/PostgreSQL |
| 依赖 | Order Service, Trade Service, Account Service |

### 1.1 核心职责

| 职责 | 说明 |
|------|------|
| 订单对账 | 订单与成交一致性 |
| 余额对账 | 账本与账户余额一致性 |
| 定时对账 | 定时任务对账 |

## 2. 数据模型

### 2.1 数据库表

**reconciliation_tasks 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| task_type | VARCHAR(30) | NOT NULL | 任务类型 |
| status | VARCHAR(20) | NOT NULL | 状态 |
| start_time | BIGINT | NOT NULL | 开始时间 |
| end_time | BIGINT | | 结束时间 |
| total_count | INT | DEFAULT 0 | 总数 |
| consistent_count | INT | DEFAULT 0 | 一致数 |
| inconsistent_count | INT | DEFAULT 0 | 不一致数 |
| created_at | BIGINT | NOT NULL | 创建时间 |

**reconciliation_diffs 表**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | BIGSERIAL | PRIMARY KEY | ID |
| task_id | BIGINT | NOT NULL, FK | 任务ID |
| diff_type | VARCHAR(30) | NOT NULL | 差异类型 |
| entity_type | VARCHAR(20) | | 实体类型 |
| entity_id | VARCHAR(64) | | 实体ID |
| expected_value | TEXT | | 期望值 |
| actual_value | TEXT | | 实际值 |
| description | TEXT | | 描述 |
| resolved | BOOLEAN | DEFAULT FALSE | 是否解决 |
| created_at | BIGINT | NOT NULL | 创建时间 |

## 3. Proto 接口

```protobuf
syntax = "proto3";

package reconciliation;

service ReconciliationService {
    rpc RunReconciliation(RunReconciliationRequest) returns (RunReconciliationResponse);
    rpc GetTask(GetTaskRequest) returns (GetTaskResponse);
    rpc ListTasks(ListTasksRequest) returns (ListTasksResponse);
    rpc GetDiffs(GetDiffsRequest) returns (GetDiffsResponse);
}

message RunReconciliationRequest {
    string task_type = 1;
}

message RunReconciliationResponse {
    bool success = 1;
    string task_id = 2;
}

message GetTaskRequest {
    int64 task_id = 1;
}

message GetTaskResponse {
    int64 task_id = 1;
    string task_type = 2;
    string status = 3;
    int64 total_count = 4;
    int64 consistent_count = 5;
    int64 inconsistent_count = 6;
}

message ListTasksRequest {
    string task_type = 1;
    int32 page = 2;
    int32 page_size = 3;
}

message ListTasksResponse {
    repeated TaskSummary tasks = 1;
}

message TaskSummary {
    int64 task_id = 1;
    string task_type = 2;
    string status = 3;
    int64 total_count = 4;
    int64 created_at = 5;
}

message GetDiffsRequest {
    int64 task_id = 1;
}

message GetDiffsResponse {
    repeated DiffSummary diffs = 1;
}

message DiffSummary {
    int64 diff_id = 1;
    string diff_type = 2;
    string entity_type = 3;
    string entity_id = 4;
    string expected_value = 5;
    string actual_value = 6;
}
```

## 4. 配置

```yaml
service:
  port: 50014
database:
  driver: "sqlite"
  url: "sqlite:./data/reconciliation.db"

reconciliation:
  # 对账任务间隔 (秒)
  interval: 3600
```

## 5. 目录结构

```
crates/reconciliation-service/
├── Cargo.toml, build.rs, config/, src/
```
