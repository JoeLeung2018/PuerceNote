# PuerceNote - AppFlowy Commercial Edition v1.0

## 改动记录与追踪

**项目**: PuerceNote - AI驱动的笔记应用商业版  
**基础**: AppFlowy 官方仓库（AGPL v3.0）  
**分支**: `feature/commercial-edition-v1.0`  
**启动时间**: 2026-04-20  
**目标版本**: v1.0 (发布时间: 2025年3月)

---

## 📋 第一阶段改动（基础准备与架构）

### M1.1: 项目名称更改

**时间**: 2026-04-20  
**分类**: Configuration

| 文件 | 改动 | 说明 |
|------|------|------|
| `README.md` | 项目标题改为PuerceNote | 更新品牌信息 |
| `frontend/appflowy_flutter/pubspec.yaml` | 项目名称: appflowy → puercenote | Flutter包名称 |

**Git提交**: 待提交

---

### M1.2: Git远程配置

**时间**: 2026-04-20  
**分类**: Git Configuration

```bash
# 修改前
origin  https://github.com/AppFlowy-IO/AppFlowy.git (fetch)
origin  https://github.com/AppFlowy-IO/AppFlowy.git (push)

# 修改后
origin    https://github.com/JoeLeung2018/AppFlowy.git (fetch)
origin    https://github.com/JoeLeung2018/AppFlowy.git (push)
upstream  https://github.com/AppFlowy-IO/AppFlowy.git (fetch)
upstream  https://github.com/AppFlowy-IO/AppFlowy.git (push)
```

**目的**: 支持向JoeLeung2018账户推送，同时保持与上游仓库的连接

---

### M1.3: 数据库迁移脚本

**时间**: 2026-04-20  
**分类**: Database  
**优先级**: HIGH

| 文件 | 改动 | 说明 |
|------|------|------|
| `frontend/rust-lib/flowy-user/src/migrations/payment_tables_v1.rs` | 新增 | 支付系统数据库表 |
| `frontend/rust-lib/flowy-user/src/migrations/mod.rs` | 更新 | 导入payment_tables_v1模块 |
| `.env.example` | 新增 | 完整的环境变量配置模板 |

**数据库表**：
- `payment_orders`: Lemon Squeezy订单追踪（ID、金额、状态、时间戳）
- `payment_products`: 产品缓存（支持多个变体定价）
- `web3_transactions`: Polygon区块链交易日志（txhash、地址、token、确认数）
- `user_subscription`: 用户订阅状态（计划、周期、功能开关）
- `payment_webhooks`: Webhook事件审计日志（用于合规性）
- `ai_usage_logs`: AI Token使用追踪（用户、token数、成本、功能类型）
- `user_ai_quota`: 月度Token配额（限额、已用、成本）

**数据库触发器**：
- `init_ai_quota_on_subscription`: 创建订阅时自动初始化配额
- `reset_monthly_quota_on_usage`: 跨月份时自动重置配额

**Git提交**: `ee0b16035`

---

### M1.4: 环境变量配置

**时间**: 2026-04-20  
**分类**: Configuration

创建 `.env.example` 模板，包含：
- vLLM + frpc隧道配置
- Token配额设置
- Lemon Squeezy API密钥
- Web3/Polygon RPC配置
- WalletConnect配置
- 数据库和日志配置
- 安全和功能开关

**指导**：复制 `.env.example` → `.env`，填入实际的API密钥和RPC地址

**Git提交**: `ee0b16035`

---

## 📦 计划改动（待实施）

### Phase 1.5: 创建flowy-subscription crate

**计划完成日期**: 2026-04-21

```
□ 创建flowy-subscription新crate
□ 定义Protobuf消息格式（payment.proto）
□ 配置Cargo.toml依赖
□ 编写模块结构（payment/, event_handler/, repository/)
```

### Phase 2A: Lemon Squeezy支付集成

**计划完成日期**: 2026-04-25

```
□ 实现Lemon Squeezy客户端
  - POST /checkouts 创建支付页面
  - HMAC签名验证
  - Webhook处理
□ 添加webhook路由和处理
□ Flutter支付UI开发
  - 支付页面（WebView）
  - 订阅管理页面
  - 成功/失败处理
```

### Phase 2B: 自建Token计费系统

**计划完成日期**: 2026-05-01

```
□ CloudAIService实现（Llama-2 9B + vLLM）
  - vLLM API集成
  - frpc隧道支持
  - 重试机制
□ 本地Token计数算法（启发式）
□ Token配额检查和管理
□ 月度配额重置逻辑
□ 成本计算和追踪
```

### Phase 2C: Web3集成

**计划完成日期**: 2026-05-10

```
□ 智能合约开发（Solidity）
□ Polygon网络集成
□ 钱包连接（WalletConnect 2.0）
□ Web3支付流程
```

---

## 🔄 改动分类体系

### 分类标记说明

- **Configuration**: 项目配置更改（名称、版本等）
- **Feature**: 新功能实现
- **Database**: 数据库架构更改
- **Backend**: Rust后端代码
- **Frontend**: Flutter前端代码
- **Integration**: 第三方服务集成
- **Bugfix**: 缺陷修复
- **Refactor**: 代码重构

### 优先级标记

- **CRITICAL**: 阻挡其他任务，必须立即处理
- **HIGH**: 重要功能，需优先完成
- **MEDIUM**: 普通功能
- **LOW**: 可选改进

---

## 📊 版本历史

### v1.0 (计划 2025-03-31)

**功能集**:
- ✅ Lemon Squeezy支付（信用卡）
- ✅ Web3支付（USDC/USDT，Polygon）
- ✅ Llama-2 9B云端AI推理
- ✅ TinyLlama隐私过滤
- ✅ 语义搜索增强
- ✅ 本地Token计费和配额管理
- ✅ 三层订阅模式（Free/Pro/Team）

**已完成**:
- ✅ Git配置
- ✅ 项目命名
- ✅ 数据库架构设计
- ✅ 环境变量配置

**进行中**:
- ⏳ flowy-subscription crate创建

**未开始**:
- Lemon Squeezy集成
- Web3开发
- AI服务部署

---

## 🔀 开发分支管理

**当前分支**: `feature/commercial-edition-v1.0`

**最近提交**：
1. `ee0b16035` - [Database] Payment: Add database migrations for payment system
2. `f10b9a91f` - [Configuration] Project: Rename AppFlowy to PuerceNote

```
main (官方主分支，仅sync)
  ├─ feature/commercial-edition-v1.0 (活跃开发) ← 当前位置
  │   ├─ feature/payment-system (计划)
  │   ├─ feature/ai-service (计划)
  │   └─ feature/web3-integration (计划)
  └─ release/v1.0 (待创建)
```

---

## 📝 改动提交模板

每个Git提交应遵循以下格式：

```
[CATEGORY] Module: 简短描述

Detailed explanation if needed.

Related changes:
- File: /path/to/file
- Lines: 10-25
- Type: [Feature|Bugfix|Refactor]

Modified files:
- src/file1.rs
- frontend/file2.dart

Database changes: None | 描述
API changes: None | 描述
Configuration changes: None | 描述

Issue: #123 (如果有)
```

### 示例提交

```
[Configuration] Project: Rename AppFlowy to PuerceNote

Updated project name across configuration files to establish
PuerceNote as the commercial distribution brand.

Modified files:
- README.md
- frontend/appflowy_flutter/pubspec.yaml

Database changes: None
API changes: None
Configuration changes: Brand name updated
```

---

## 🔍 改动审核清单

在提交每个重要改动前，确认：

- [ ] 代码遵循AppFlowy编码规范
- [ ] Rust代码通过 `cargo fmt` 和 `cargo clippy`
- [ ] Dart代码通过 `dart format` 和 `dart analyze`
- [ ] 所有关键路径的Git历史清晰
- [ ] MODIFICATIONS.md已更新
- [ ] 无意外的依赖或配置更改
- [ ] 与上游分支无冲突

---

## 📞 快速参考

**查看所有改动**:
```bash
git log --oneline feature/commercial-edition-v1.0 --not main
```

**与官方同步**:
```bash
git fetch upstream
git rebase upstream/main
```

**推送改动**:
```bash
git push origin feature/commercial-edition-v1.0
```

**对比官方版本**:
```bash
git diff upstream/main..feature/commercial-edition-v1.0
```

---

## 📌 重要约束

1. **AGPL合规**: 所有改动必须遵守AGPL v3.0许可
2. **上游同步**: 定期与AppFlowy官方仓库同步
3. **开源承诺**: 所有代码保持开源，不含专有扩展
4. **隐私优先**: 本地AI优先于云端，用户数据本地保有

---

**Last Updated**: 2026-04-20  
**Next Review**: 待定  
**Maintained By**: JoeLeung2018
