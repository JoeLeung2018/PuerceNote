# PuerceNote 数据存储架构 - SQLite向量数据支持

**日期**: 2026-04-20  
**版本**: v1.0  
**范围**: 普通数据 + 向量数据存储方案

---

## 📊 当前数据存储架构概览

### **三层存储系统**

```
┌─────────────────────────────────────────────────────────────┐
│                     PuerceNote 数据架构                      │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Layer 1: 用户交互层                                          │
│  ├─ Flutter Frontend (appflowy_flutter/)                     │
│  └─ Dart API Client                                          │
│                                                               │
│  Layer 2: 应用服务层                                          │
│  ├─ flowy-core (核心业务逻辑)                                 │
│  ├─ flowy-document (文档管理)                                 │
│  ├─ flowy-folder (文件夹管理)                                 │
│  ├─ flowy-database2 (数据库视图)                              │
│  ├─ flowy-search (搜索服务)                                   │
│  ├─ flowy-ai (AI功能)                                         │
│  └─ flowy-subscription (订阅计费) ← Phase 2B                  │
│                                                               │
│  Layer 3: 数据持久化层                                        │
│  ├─ flowy-sqlite (普通数据)                                   │
│  │  └─ flowy-database.db                                     │
│  │     ├─ user_subscription                                  │
│  │     ├─ payment_orders                                     │
│  │     ├─ payment_webhooks                                   │
│  │     ├─ ai_usage_log ← AI计费                             │
│  │     ├─ workspace_members                                  │
│  │     ├─ documents                                          │
│  │     └─ ...其他业务表                                      │
│  │                                                            │
│  └─ flowy-sqlite-vec (向量数据)                              │
│     └─ vector.db                                             │
│        ├─ af_collab_embeddings (虚拟表)                      │
│        └─ af_pending_index_collab (待索引)                   │
│                                                               │
│  Layer 4: 外部服务                                            │
│  ├─ Lemon Squeezy (支付处理)                                 │
│  ├─ Paddle (本地支付)                                        │
│  ├─ PayPal (全球支付)                                        │
│  ├─ Polygon (Web3支付)                                       │
│  └─ vLLM (AI推理)                                            │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 💾 数据存储详解

### **1. 普通数据 (flowy-sqlite)**

**数据库文件**: `flowy-database.db`  
**存储类型**: SQLite关系数据库  
**ORM方案**: Diesel (类型安全的SQL查询构建器)  

**主要表**:

```sql
-- 用户和工作区
users (uid, email, name, password_hash, ...)
user_workspace (user_id, workspace_id, role, ...)
workspace_members (email, workspace_id, role, ...)

-- 文档和内容
documents (oid, workspace_id, name, type, ...)
document_snapshots (doc_id, version, snapshot_data, ...)

-- 订阅和支付 (Phase 2B)
user_subscription (id, user_id, plan_type, status, ...)
payment_orders (id, user_id, amount_cents, status, ...)
payment_webhooks (id, provider, event_type, payload, ...)
ai_usage_log (id, user_id, feature_type, tokens_used, ...)

-- 协作数据
collab_table (oid, workspace_id, type, data, ...)
```

**特点**:
- ✅ ACID事务支持
- ✅ 关系完整性约束
- ✅ 复杂查询支持
- ✅ 索引加速查询
- ❌ 不支持向量相似度搜索

### **2. 向量数据 (flowy-sqlite-vec)**

**数据库文件**: `vector.db`  
**存储类型**: SQLite虚拟表 (vec0)  
**向量扩展**: sqlite-vec (Rust绑定)  
**向量维度**: 768维 (默认)  

**关键表**:

```sql
-- 虚拟向量表 (使用vec0扩展)
CREATE VIRTUAL TABLE af_collab_embeddings 
USING vec0(
  workspace_id    TEXT NOT NULL,
  object_id       TEXT NOT NULL,
  fragment_id     TEXT NOT NULL,
  content_type    INTEGER,
  content         TEXT,
  metadata        TEXT,
  fragment_index  INTEGER,
  embedder_type   INTEGER,
  embedding       float[768]  ← 向量列
);

-- 待索引的内容
af_pending_index_collab (
  oid TEXT PRIMARY KEY,
  workspace_id TEXT,
  content TEXT,
  collab_type SMALLINT,
  updated_at TIMESTAMP,
  indexed_at TIMESTAMP
);
```

**特点**:
- ✅ 向量相似度搜索 (KNN)
- ✅ 语义搜索
- ✅ HNSW索引支持
- ✅ 单独数据库不阻塞主库
- ❌ 不支持事务（虚拟表限制）

---

## 🔍 SQLite向量存储实现细节

### **向量表特性**

```
af_collab_embeddings (向量虚拟表):

列名              | 类型         | 用途
──────────────────┼──────────────┼────────────────────
workspace_id      | TEXT         | 工作区ID (分片)
object_id         | TEXT         | 文档/笔记ID
fragment_id       | TEXT         | 文本片段ID (段落)
content_type      | INTEGER      | 内容类型 (1=doc, 2=note)
content           | TEXT         | 原始内容文本
metadata          | TEXT (JSON)  | 元数据 (链接、标签等)
fragment_index    | INTEGER      | 片段序号
embedder_type     | INTEGER      | 嵌入模型 (1=OpenAI, 2=开源)
embedding         | float[768]   | ← 向量数据

虚拟表优势:
✓ 自动HNSW索引化
✓ 快速向量相似度搜索
✓ 低内存占用
✓ 对大规模向量优化
```

### **向量工作流**

```
1. 内容生成 (文档编辑)
   ↓
2. 等待索引 (af_pending_index_collab)
   ↓
3. 调用AI模型生成向量
   ↓
4. 存入向量表 (af_collab_embeddings)
   ↓
5. 用户搜索请求
   ↓
6. 查询向量表 (相似度搜索)
   ↓
7. 返回结果 (排序)
```

---

## 📁 存储位置对应关系

### **文件系统位置**

```
~/Library/Application Support/FlowySandbox/  (macOS)
├─ storage/                    ← 本地存储根目录
│  ├─ flowy-database.db        ← 普通数据库 (重要)
│  ├─ flowy-database.db-shm    ← WAL日志
│  ├─ flowy-database.db-wal    ← WAL检查点
│  ├─ vector.db                ← 向量数据库 (新增)
│  ├─ vector.db-shm            ← WAL日志
│  ├─ vector.db-wal            ← WAL检查点
│  ├─ files/                   ← 上传的文件
│  └─ ...
```

### **Rust代码中的初始化**

```rust
// flowy-sqlite 初始化
let db = Database::new(
    storage_path,
    "flowy-database.db",
    pool_config
)?;

// flowy-sqlite-vec 初始化 (Phase 2B扩展)
let vec_db = VectorSqliteDB::new(storage_path.into())?;
```

---

## 🚀 在Phase 2B中的应用

### **AI使用计费的向量支持**

**场景**: 用户使用AI功能 (文档总结、标签生成、搜索)

**流程图**:

```
用户请求AI功能
    ↓
┌─ 文档总结
├─ 标签生成
├─ 语义搜索  ← 需要向量
└─ 内容推荐

    ↓
    
[生成嵌入向量]
    ↓
存入向量表:
  af_collab_embeddings {
    workspace_id: "ws_123",
    object_id: "doc_456",
    fragment_id: "para_789",
    content: "这是文档内容...",
    embedding: [0.1, 0.2, ..., 0.768]  ← 768维向量
  }

    ↓

[记录AI使用]
    ↓
存入计费表:
  ai_usage_log {
    user_id: "user_123",
    feature_type: "semantic_search",
    tokens_used: 150,
    cost_usd: 0.0015,
    created_at: NOW()
  }

    ↓

[更新订阅配额]
    ↓
user_subscription {
  id: "sub_123",
  user_id: "user_123",
  tokens_used: 150,  ← 累计
  monthly_limit: 100000,
  status: "active"
}
```

### **向量搜索计费**

```sql
-- Pro用户搜索成本
SELECT 
  SUM(tokens_used) as total_tokens,
  SUM(cost_usd) as total_cost,
  COUNT(*) as search_count
FROM ai_usage_log
WHERE 
  user_id = 'user_123'
  AND feature_type = 'semantic_search'
  AND DATE(created_at) = CURRENT_DATE;

结果:
total_tokens: 15000 (150次搜索 × 100 tokens)
total_cost: $0.15 (0.01 per 1K tokens)
search_count: 150
```

---

## 🛠️ Phase 2B中添加向量计费支持

### **Step 1: 扩展AI使用日志**

```sql
-- 在flowy-sqlite中添加向量相关字段
ALTER TABLE ai_usage_log ADD COLUMN (
  vector_dimension INTEGER DEFAULT 768,
  embedding_model TEXT DEFAULT 'default',
  similarity_threshold REAL DEFAULT 0.75,
  retrieval_count INTEGER DEFAULT 0  -- 检索到的相似文档数
);

-- 创建索引加快计费查询
CREATE INDEX idx_ai_usage_feature_date 
  ON ai_usage_log(user_id, feature_type, DATE(created_at));
```

### **Step 2: 记录向量操作**

```rust
// flowy-subscription/src/billing/vector_billing.rs (新增)

pub struct VectorUsageRecord {
    pub user_id: String,
    pub operation: VectorOperation,  // search, embed, batch
    pub tokens_used: u32,            // tokens消耗
    pub embedding_dim: u16,          // 768
    pub collection_size: u32,        // 向量表大小
    pub query_latency_ms: u64,       // 性能指标
}

pub enum VectorOperation {
    SemanticSearch,
    EmbeddingGeneration,
    BatchIndexing,
}

// 计费逻辑
impl VectorUsageRecord {
    pub fn calculate_cost(&self) -> f64 {
        match self.operation {
            VectorOperation::SemanticSearch => {
                // $0.01 per 1K 向量查询操作
                (self.tokens_used as f64) * 0.00001
            },
            VectorOperation::EmbeddingGeneration => {
                // $0.01 per 1K token 嵌入生成
                (self.tokens_used as f64) * 0.00001
            },
            VectorOperation::BatchIndexing => {
                // 批量免费或折扣
                0.0
            }
        }
    }
}
```

### **Step 3: 整合向量搜索到计费系统**

```rust
// flowy-subscription/src/billing/mod.rs

pub async fn track_vector_usage(
    db: &Database,
    vec_db: &VectorSqliteDB,
    user_id: &str,
    operation: VectorOperation,
) -> SubscriptionResult<()> {
    // 1. 执行向量操作
    let start = Instant::now();
    let results = vec_db.semantic_search(
        user_id,
        query_vector.as_slice(),
        top_k: 10
    ).await?;
    let latency = start.elapsed().as_millis() as u64;
    
    // 2. 创建使用记录
    let usage = VectorUsageRecord {
        user_id: user_id.to_string(),
        operation: VectorOperation::SemanticSearch,
        tokens_used: query_tokens,
        embedding_dim: 768,
        collection_size: results.len(),
        query_latency_ms: latency,
    };
    
    // 3. 计算成本
    let cost = usage.calculate_cost();
    
    // 4. 记录到数据库
    db.log_ai_usage(
        user_id,
        "semantic_search",
        query_tokens,
        cost
    ).await?;
    
    // 5. 检查配额
    check_user_quota(db, user_id, query_tokens).await?;
    
    Ok(())
}
```

---

## 📊 数据库架构总结

| 特性 | flowy-sqlite | flowy-sqlite-vec |
|------|-------------|------------------|
| **用途** | 业务数据 | 向量搜索 |
| **表类型** | 标准关系表 | 虚拟表 (vec0) |
| **文件** | flowy-database.db | vector.db |
| **大小** | ~100MB (一般) | ~1GB (百万向量) |
| **查询方式** | SQL JOIN | 向量相似度 |
| **事务支持** | ✅ ACID | ❌ 虚拟表限制 |
| **索引** | B-tree | HNSW |
| **写入** | 行级 | 向量级 |
| **场景** | 存储结构化数据 | 语义搜索/推荐 |

---

## 🎯 后续集成步骤 (Phase 2B.X)

### **当前完成**:
- ✅ Task 2B.Token+Price (定价优化)
- ✅ Task 2B.SQL (基础Schema)
- ⏳ Task 2B.Webhook (支付事件)

### **下一步 (Phase 2B.Vector)**:

```
Priority 1 (周一完成):
  1. 扩展ai_usage_log表添加向量字段
  2. 实现vector_billing.rs模块
  3. 添加向量操作计费逻辑
  4. 集成到webhook处理 (记录使用)

Priority 2 (周二):
  5. 创建向量搜索费率配置
  6. 添加向量操作测试
  7. 实现Pro用户配额检查

Priority 3 (周三):
  8. 向量使用仪表板
  9. 使用量告警
  10. 成本预测模型
```

### **代码变更清单**:

```rust
// 新增文件
flowy-subscription/src/billing/vector_billing.rs       (150行)
flowy-subscription/src/billing/vector_quotas.rs        (120行)
flowy-subscription/tests/vector_billing_tests.rs       (200行)

// 修改文件
flowy-subscription/src/repository.rs                   (+50行)
flowy-subscription/src/billing/mod.rs                  (+30行)
flowy-sqlite/migrations/.../ai_usage_log_v2.sql       (新建)

// 配置更新
frontend/rust-lib/flowy-subscription/src/config.rs    (+10行)
```

---

## 💡 设计决策

### **为什么分离向量数据库?**

```
✓ 性能隔离: 向量查询不影响业务查询
✓ 扩展灵活: 可独立升级或迁移向量引擎
✓ 成本控制: 向量操作有单独的计费
✓ 维护简单: 向量和业务数据独立版本管理
✓ 安全性: 业务数据不依赖向量库
```

### **为什么选择SQLite向量?**

```
✓ 零部署: 无需额外服务 (Pinecone/Weaviate)
✓ 本地化: 用户数据完全本地存储
✓ 低成本: 无SaaS费用
✓ 隐私好: 用户向量不上传云端
✗ 规模限制: 单机向量库 (但足够100万+向量)
✗ 分布式: 不支持多机部署
```

### **何时考虑升级?**

```
当前: SQLite向量数据库
  用户数: < 10,000
  向量数: < 1,000,000
  查询延迟: < 100ms

升级触发点:
  → 用户数 > 50,000: 考虑Pinecone/Milvus
  → 向量数 > 10,000,000: 迁移Weaviate
  → 需要分布式: Elasticsearch + 向量插件
```

---

## 🔒 安全性考虑

### **向量数据隐私**

```
✓ 本地加密: vector.db遵循用户本地加密策略
✓ 工作区隔离: workspace_id作为向量分片键
✓ 访问控制: 向量搜索权限受document权限制约
✓ 审计: ai_usage_log记录所有向量操作
```

### **计费数据保护**

```
✓ 原子性: 向量操作+费用记录为单事务
✓ 防篡改: ai_usage_log使用签名验证
✓ 备份: 自动备份ai_usage_log到支付系统
✓ 对账: 定期与第三方支付网关对账
```

---

## 📈 性能预期

### **向量搜索性能**

```
集合大小          查询延迟 (768维)    吞吐量
────────────────────────────────────
10,000向量         < 1ms             10K QPS
100,000向量        < 5ms             2K QPS
1,000,000向量      < 50ms            200 QPS
10,000,000向量     < 500ms           20 QPS

存储需求:
768维float32向量 = 3,072字节
100万向量 = 3GB (含索引~1.5GB)
```

### **计费系统性能**

```
ai_usage_log 查询:
  单用户每日使用: < 5ms
  租户汇总: < 100ms
  全库统计: < 1s

写入性能:
  向量操作计费: < 10ms
  批量索引: 异步后台处理
```

---

## 📚 参考资源

- **sqlite-vec文档**: https://github.com/asg017/sqlite-vec
- **向量搜索算法**: HNSW (Hierarchical Navigable Small World)
- **PuerceNote AI模块**: `frontend/rust-lib/flowy-ai/`

---

## 📞 接下来的行动

### **立即 (今天)**:
1. ✅ 完成Phase 2B.Webhook (支付事件处理)
2. ✅ Phase 2B Day 1测试验证
3. 📋 准备Phase 2B.Vector设计文档

### **明天 (Day 2)**:
4. 📋 开始Web3集成 (WalletConnect + MetaMask)
5. 📋 支付方式集成 (PayPal + Paddle)

### **Phase 2B.Vector (下周)**:
6. 🔄 扩展ai_usage_log表
7. 🔄 实现向量计费模块
8. 🔄 向量配额管理

---

**问题?** 关于向量数据的存储、计费或架构有任何疑问？

