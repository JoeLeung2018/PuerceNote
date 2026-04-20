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

## 📦 计划改动（待实施）

### Phase 2A: Lemon Squeezy支付集成

**计划完成日期**: 待定

```
□ 创建flowy-subscription新crate
□ 实现Lemon Squeezy客户端
□ 添加webhook处理
□ Flutter支付UI开发
```

### Phase 2B: 自建Token计费系统

**计划完成日期**: 待定

```
□ 数据库迁移脚本
□ CloudAIService实现（Llama-2 9B + vLLM）
□ 本地Token计数算法
□ Token配额管理
```

### Phase 2C: Web3集成

**计划完成日期**: 待定

```
□ 智能合约开发（Solidity）
□ Polygon集成
□ 钱包连接（WalletConnect 2.0）
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
- Git配置
- 项目命名

**进行中**:
- 基础架构建设

**未开始**:
- Lemon Squeezy集成
- Web3开发
- AI服务部署

---

## 🔀 开发分支管理

```
main (官方主分支，仅sync)
  ├─ feature/commercial-edition-v1.0 (活跃开发)
  │   ├─ feature/payment-system (支付系统)
  │   ├─ feature/ai-service (AI服务)
  │   └─ feature/web3-integration (Web3集成)
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
