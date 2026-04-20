# PuerceNote 开发指南

## 🚀 快速开始

### 环境要求

```bash
# 检查Rust版本
rustc --version   # >= 1.70

# 检查Flutter版本
flutter --version # >= 3.27.4

# 检查Node.js版本
node --version    # >= 18
```

### 首次设置

```bash
cd /opt_projects/sync_working/puerce_note/src/AppFlowy

# 1. 创建开发分支
git checkout -b feature/commercial-edition-v1.0

# 2. 检查依赖（Rust）
cd frontend/rust-lib
cargo check

# 3. 检查依赖（Flutter）
cd ../appflowy_flutter
flutter pub get

# 4. 同步上游更新
git fetch upstream
git rebase upstream/main
```

---

## 📂 项目结构

```
AppFlowy/
├── README.md                          # 项目说明（已改为PuerceNote）
├── MODIFICATIONS.md                   # 改动记录（本文件）
├── LICENSE                            # AGPL v3.0许可证
│
├── frontend/
│   ├── appflowy_flutter/
│   │   ├── pubspec.yaml              # Flutter项目配置（已改为puercenote）
│   │   ├── lib/
│   │   │   ├── main.dart
│   │   │   ├── workspace/
│   │   │   │   └── presentation/
│   │   │   │       └── settings/
│   │   │   │           ├── billing/  # 💰 新增：支付UI (Lemon Squeezy)
│   │   │   │           └── ai/       # 🤖 新增：AI配额管理UI
│   │   │   └── wallet/               # 💳 新增：Web3钱包集成
│   │   │
│   │   └── packages/
│   │       ├── appflowy_backend/     # Dart FFI后端接口
│   │       └── ...
│   │
│   └── rust-lib/                      # Rust后端核心
│       ├── Cargo.toml                 # 工作空间定义
│       ├── flowy-subscription/        # 🆕 新增crate：支付+订阅
│       ├── flowy-ai/                  # 🤖 AI服务（修改）
│       │   ├── src/
│       │   │   ├── cloud_ai.rs       # vLLM集成+本地计费
│       │   │   ├── privacy_filter.rs # TinyLlama隐私检测
│       │   │   ├── semantic_search.rs# 语义搜索增强
│       │   │   └── quota_manager.rs  # Token配额管理
│       │   └── Cargo.toml
│       │
│       ├── flowy-user/                # 用户管理（修改）
│       │   ├── migrations/
│       │   │   ├── 2026-04-20-create-payment-tables.sql
│       │   │   └── ...
│       │   └── src/
│       │
│       └── ... 其他既有crates
│
└── docs/
    ├── CONTRIBUTING.md
    └── ...
```

---

## 💡 开发工作流

### 第一步：查看当前分支状态

```bash
cd /opt_projects/sync_working/puerce_note/src/AppFlowy

# 确认在正确分支
git branch -v

# 查看改动
git status
git log --oneline -10
```

### 第二步：创建功能分支

```bash
# 从 feature/commercial-edition-v1.0 创建子分支
git checkout -b feature/payment-system

# 开发...

# 提交改动
git add .
git commit -m "[Integration] Payment: Implement Lemon Squeezy integration

Add Lemon Squeezy payment client with webhook handling.

Modified files:
- rust-lib/flowy-subscription/src/payment/lemon_squeezy.rs
- frontend/appflowy_flutter/lib/workspace/presentation/settings/billing/

Database changes: None
API changes: Added /api/webhook/lemon-squeezy endpoint
Configuration changes: Added LEMON_SQUEEZY_API_KEY to .env"

# 推送
git push origin feature/payment-system

# 创建Pull Request到 feature/commercial-edition-v1.0
```

### 第三步：整合到主分支

```bash
# 切换到主分支
git checkout feature/commercial-edition-v1.0

# 合并功能分支
git merge feature/payment-system

# 更新MODIFICATIONS.md
# 编辑文件...
git add MODIFICATIONS.md
git commit -m "[Documentation] Update MODIFICATIONS.md with payment integration"

# 推送到个人仓库
git push origin feature/commercial-edition-v1.0
```

---

## 🔧 构建与测试

### Rust编译检查

```bash
# 检查代码
cd frontend/rust-lib
cargo check

# 运行格式检查
cargo fmt --check

# 运行lint
cargo clippy -- -D warnings

# 运行测试
cargo test --lib
```

### Flutter编译

```bash
# 检查代码
cd frontend/appflowy_flutter
dart analyze

# 格式化
dart format .

# 运行集成测试（可选）
flutter test integration_test/
```

### 完整编译（仅在提交重要改动时）

```bash
# 这会很慢，需要15-30分钟
cd frontend/appflowy_flutter
flutter build apk --release  # Android
# 或
flutter build ios --release  # iOS
# 或
flutter build linux --release  # Linux
```

---

## 📝 编码规范

### Rust代码

```rust
// ✅ 好的例子
/// 生成文档摘要
/// 
/// 仅Pro/Team用户可用，将从配额中扣除token
#[instrument(skip_all)]
pub async fn generate_summary(
    &self,
    user_id: &str,
    content: &str,
) -> Result<String, CloudAIError> {
    // 实现...
}

// ❌ 不好的例子
// 函数注释不清楚
fn gen_sum(uid: &str, c: &str) -> Result<String, Error> {
```

### Dart代码

```dart
// ✅ 好的例子
class PaymentService {
  /// 初始化Lemon Squeezy支付
  Future<void> initPayment() async {
    // ...
  }
}

// ❌ 不好的例子
class PaymentService {
  void init() {  // 缺少async关键字和文档
  }
}
```

---

## 🔀 与上游同步

定期与AppFlowy官方仓库同步，避免大量冲突：

```bash
# 1. 检查是否有新更新
git fetch upstream

# 2. 查看差异
git log --oneline main..upstream/main

# 3. 变基到最新的上游main
git checkout feature/commercial-edition-v1.0
git rebase upstream/main

# 如果有冲突，解决冲突后：
git rebase --continue

# 4. 推送到个人仓库
git push -f origin feature/commercial-edition-v1.0
```

---

## 📊 改动检查清单

在提交改动前，使用此清单：

```
代码质量:
  ☐ Rust: cargo fmt && cargo clippy
  ☐ Dart: dart format && dart analyze
  ☐ 函数/类有文档注释
  ☐ 代码无TODO或FIXME标记

功能完整性:
  ☐ 核心功能已实现
  ☐ 错误处理完整
  ☐ 日志记录充分（tracing/log）
  ☐ 单元测试已添加（至少50%覆盖率）

配置管理:
  ☐ .env.example已更新
  ☐ Cargo.toml依赖版本明确
  ☐ pubspec.yaml依赖已检查

文档更新:
  ☐ MODIFICATIONS.md已更新
  ☐ README.md已更新（如需）
  ☐ 数据库迁移脚本已记录

Git整洁:
  ☐ 提交信息清晰
  ☐ 无意外的大文件
  ☐ 无敏感信息（密钥、密码）
  ☐ 与上游无冲突
```

---

## 🐛 常见问题排查

### Q: Rust编译失败

```bash
# 清除编译缓存
cargo clean

# 更新依赖
cargo update

# 查看完整错误
RUST_BACKTRACE=1 cargo build
```

### Q: Flutter依赖冲突

```bash
# 清除缓存
flutter clean
rm pubspec.lock

# 重新获取依赖
flutter pub get

# 尝试升级
flutter pub upgrade
```

### Q: Git rebase冲突

```bash
# 查看冲突文件
git status

# 手动编辑冲突文件
# 使用 <<<<<<< ======= >>>>>>>>> 标记

# 解决后标记为已解决
git add conflicted_file.rs

# 继续rebase
git rebase --continue
```

---

## 📞 帮助资源

- **AppFlowy官方文档**: https://docs.appflowy.io
- **Rust异步编程**: https://tokio.rs
- **Flutter官方文档**: https://flutter.dev/docs
- **vLLM文档**: https://docs.vllm.ai
- **Polygon文档**: https://polygon.technology/develop

---

## 🎯 下一步行动

按优先级：

1. ✅ **已完成**: 项目重命名，Git配置
2. ⏳ **接下来**: 创建 `flowy-subscription` crate（Phase 1.3）
3. ⏳ **然后**: 数据库迁移脚本（Phase 1.3）
4. ⏳ **最后**: 编译验证整个项目

---

**Last Updated**: 2026-04-20  
**Target**: v1.0 Release (2025-03-31)
