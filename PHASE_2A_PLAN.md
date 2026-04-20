# Phase 2A - Lemon Squeezy 完整支付集成

**开始时间**: 2026-04-20  
**目标完成**: 2026-04-25  
**优先级**: 🔴 Critical Path

---

## 📋 任务分解

### Task 2A.1: 完善 Lemon Squeezy 客户端

#### 文件: `frontend/rust-lib/flowy-subscription/src/payment/lemon_squeezy.rs`

**需要实现的功能**:

```rust
// 1. 完整的列表查询
pub async fn list_checkouts(&self, store_id: u32) -> SubscriptionResult<Vec<CheckoutData>>
pub async fn list_orders(&self) -> SubscriptionResult<Vec<Order>>
pub async fn list_subscriptions(&self) -> SubscriptionResult<Vec<Subscription>>

// 2. 订阅管理
pub async fn create_subscription(
    &self, 
    variant_id: u32, 
    customer_email: &str
) -> SubscriptionResult<SubscriptionData>

pub async fn update_subscription(
    &self, 
    subscription_id: &str, 
    action: &str  // pause, unpause, cancel
) -> SubscriptionResult<SubscriptionData>

pub async fn get_subscription(&self, subscription_id: &str) -> SubscriptionResult<SubscriptionData>

// 3. 客户管理
pub async fn create_customer(
    &self,
    email: &str,
    name: Option<String>
) -> SubscriptionResult<CustomerData>

// 4. Webhook 事件类型扩展
pub async fn list_webhook_events(
    &self
) -> SubscriptionResult<Vec<WebhookLog>>

pub async fn verify_and_process_webhook(
    &self,
    payload: &[u8],
    signature: &str
) -> SubscriptionResult<ProcessedWebhookEvent>
```

**关键数据结构**:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubscriptionData {
    pub id: String,
    pub customer_id: String,
    pub variant_id: u32,
    pub status: String,  // active, paused, cancelled
    pub current_period_start: String,
    pub current_period_end: String,
    pub next_billing_at: Option<String>,
    pub pause_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub customer_id: String,
    pub status: String,
    pub total_in_cents: u32,
    pub currency: String,
    pub created_at: String,
    pub paid_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerData {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub created_at: String,
}
```

---

### Task 2A.2: 数据库集成层

#### 文件: `frontend/rust-lib/flowy-subscription/src/repository.rs` (新建)

**功能**:
- 订单保存和查询
- 订阅状态更新
- Webhook 事件审计日志
- 支付历史查询

```rust
pub struct SubscriptionRepository {
    // database connection pool
}

impl SubscriptionRepository {
    // Orders
    pub async fn save_order(&self, order: PaymentOrder) -> SubscriptionResult<()>
    pub async fn get_order(&self, order_id: &str) -> SubscriptionResult<Option<PaymentOrder>>
    pub async fn list_user_orders(&self, user_id: &str) -> SubscriptionResult<Vec<PaymentOrder>>
    
    // Subscriptions
    pub async fn save_subscription(&self, sub: Subscription) -> SubscriptionResult<()>
    pub async fn get_user_subscription(&self, user_id: &str) -> SubscriptionResult<Option<Subscription>>
    pub async fn update_subscription_status(
        &self, 
        user_id: &str, 
        status: SubscriptionStatus
    ) -> SubscriptionResult<()>
    
    // Webhook audit
    pub async fn log_webhook_event(
        &self, 
        event: PaymentWebhookEvent
    ) -> SubscriptionResult<()>
}
```

---

### Task 2A.3: 事件处理完善

#### 文件: `frontend/rust-lib/flowy-subscription/src/event_handler.rs` (更新)

**实现完整的业务逻辑**:

```rust
pub async fn handle_webhook_order_completed(
    order_id: &str,
    customer_email: &str,
    amount_cents: u32,
    plan_type: &str
) -> SubscriptionResult<()> {
    // 1. 获取或创建用户
    let user = get_or_create_user(customer_email).await?;
    
    // 2. 根据plan_type创建订阅
    let plan = match plan_type {
        "pro" => SubscriptionPlan::Pro,
        "team" => SubscriptionPlan::Team,
        _ => return Err(SubscriptionError::InvalidRequest("Invalid plan".to_string())),
    };
    
    // 3. 保存订单
    let order = PaymentOrder {
        user_id: user.id,
        lemon_order_id: order_id.to_string(),
        amount_cents,
        status: "completed",
        webhook_verified_at: Some(chrono::Utc::now()),
        ..Default::default()
    };
    repository.save_order(order).await?;
    
    // 4. 创建订阅记录
    let subscription = Subscription {
        user_id: user.id,
        plan,
        status: SubscriptionStatus::Active,
        period_start: chrono::Utc::now(),
        period_end: chrono::Utc::now() + chrono::Duration::days(30),
        ..Default::default()
    };
    repository.save_subscription(subscription).await?;
    
    // 5. 初始化 AI 配额
    billing_service.init_ai_quota(&user.id, plan.token_limit()).await?;
    
    // 6. 发送激活邮件
    email_service.send_activation_email(customer_email).await?;
    
    Ok(())
}
```

---

### Task 2A.4: Protobuf 定义

#### 文件: `frontend/rust-lib/flowy-subscription/protos/payment.proto` (新建)

```protobuf
syntax = "proto3";

package puercenote.payment;

message PaymentOrder {
    string id = 1;
    string user_id = 2;
    string lemon_order_id = 3;
    int64 amount_cents = 4;
    string currency = 5;
    string status = 6;  // pending, completed, failed
    int64 created_at = 7;
    int64 webhook_verified_at = 8;
}

message SubscriptionStatus {
    string user_id = 1;
    string plan_type = 2;  // free, pro, team
    string status = 3;     // active, expired, cancelled
    int64 period_start = 4;
    int64 period_end = 5;
    bool ai_enabled = 6;
    bool semantic_search_enabled = 7;
}

message WebhookEvent {
    string event_type = 1;
    string provider = 2;  // lemon_squeezy, polygon
    string payload = 3;   // JSON
    string signature = 4;
    int64 received_at = 5;
}
```

---

### Task 2A.5: Flutter 支付 UI

#### 文件: `frontend/appflowy_flutter/lib/payment/` (新建目录)

**结构**:
```
lib/payment/
├── payment_service.dart          # Flutter支付服务
├── screens/
│   ├── subscription_screen.dart  # 订阅管理界面
│   ├── checkout_screen.dart      # 支付页面
│   └── success_screen.dart       # 支付成功
├── widgets/
│   ├── plan_card.dart            # 套餐卡片
│   ├── payment_button.dart       # 支付按钮
│   └── quota_display.dart        # 配额显示
└── models/
    └── payment_models.dart       # 数据模型
```

**关键 UI 流程**:
1. 订阅页面显示 3 个套餐卡片
2. 点击"升级"打开 WebView 进入 Lemon Squeezy checkout
3. 支付成功后回调本地应用
4. 显示激活成功提示
5. 更新用户 AI 功能状态

---

### Task 2A.6: API 端点完善

#### 文件: `frontend/rust-lib/flowy-subscription/src/api/` (新建)

**HTTP 路由**:
```
POST /api/subscription/checkout
  - 创建 checkout session
  - 返回 Lemon Squeezy checkout URL

GET /api/subscription/status/{user_id}
  - 获取用户订阅状态
  - 返回当前计划和配额

POST /api/webhooks/lemon-squeezy
  - Webhook 接收端点
  - 验证签名并处理事件

POST /api/subscription/upgrade
  - 升级订阅计划
  - 处理过期配额

DELETE /api/subscription/cancel
  - 取消订阅
  - 清除 AI 功能访问
```

---

## 🎯 完成标准

- [ ] Lemon Squeezy 客户端支持完整的 API 操作
- [ ] 订阅数据库层完整实现
- [ ] Webhook 事件处理业务逻辑完整
- [ ] Protobuf 定义和编译成功
- [ ] Flutter UI 界面完整实现
- [ ] API 端点全部可用并测试通过
- [ ] 支付流程端到端可运行

---

## 📝 实现步骤

### Step 1: 扩展 Lemon Squeezy 客户端 (2 小时)
- 添加列表查询接口
- 添加订阅创建/更新接口
- 完整 API 错误处理

### Step 2: 数据库集成层 (1.5 小时)
- 创建 repository.rs
- 实现 CRUD 操作
- 添加事务支持

### Step 3: 事件处理完善 (1.5 小时)
- 完整业务逻辑实现
- 邮件通知集成
- 错误恢复机制

### Step 4: Protobuf 定义 (1 小时)
- 定义所有消息类型
- 编译并验证

### Step 5: Flutter UI 实现 (3 小时)
- 订阅页面布局
- WebView 集成
- 支付回调处理

### Step 6: API 端点实现 (1.5 小时)
- RESTful 接口
- 请求验证
- 响应序列化

### Step 7: 集成测试 (1.5 小时)
- 端到端流程测试
- Webhook 测试
- 错误场景处理

**总估时: ~12 小时 (分 2-3 天完成)**

---

## 💰 支付流程架构

```
用户 UI
  ↓
flutter: 点击"升级Pro"
  ↓
flowy-subscription: POST /api/subscription/checkout
  ↓
LemonSqueezyClient: POST /v1/checkouts
  ↓
Lemon Squeezy: 生成支付链接
  ↓
flutter: WebView 打开支付页面
  ↓
用户: 输入支付信息
  ↓
Lemon Squeezy: 处理支付，发送 webhook
  ↓
PuerceNote: POST /api/webhooks/lemon-squeezy
  ↓
event_handler: 处理 order.completed 事件
  ↓
repository: 保存订单和订阅
  ↓
billing_service: 初始化 token 配额
  ↓
flutter: 显示"支付成功，AI功能已激活"
  ↓
用户: 开始使用 AI 功能
```

---

## 📋 检查清单

任务完成前确认以下项目:

- [ ] 所有 Rust 代码通过 `cargo check`
- [ ] 所有 Rust 代码通过 `cargo fmt`
- [ ] 单元测试通过 `cargo test`
- [ ] Dart 代码通过 `dart analyze`
- [ ] Dart 代码通过 `dart format`
- [ ] Protobuf 编译无错误
- [ ] Git commits 遵循 commit 规范
- [ ] 所有文件已添加到 git 并准备提交
- [ ] 没有 AppFlowy 或 "Commercial Edition" 的标识
- [ ] 文档已更新反映新功能

---

**状态**: 准备开始实现 ✅
