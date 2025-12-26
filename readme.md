# Penlai - 企业级大模型异步上下文管理系统

Penlai 是一个企业级的大模型上下文管理系统，支持异步处理、智能上下文选择和实时网络搜索功能。

## 功能特性

- **异步上下文管理**：支持高并发的上下文创建、检索和更新
- **智能上下文选择**：基于领域分类和相似度算法的智能上下文选择
- **实时网络搜索**：支持智能路由的网络搜索功能
- **多搜索引擎支持**：自动根据查询类型选择合适的搜索引擎
- **上下文优先级管理**：支持上下文优先级设置和版本控制
- **监控与统计**：内置监控系统跟踪上下文使用情况

## 智能搜索功能

系统集成了智能搜索功能，能够：

- **自动分类查询**：识别代码/技术查询与一般查询
- **智能路由**：
  - 代码/技术查询 → GitHub 搜索 API
  - 一般查询 → Bing 搜索 API
- **结果聚合**：自动去重和相关性排序
- **实时上下文**：将搜索结果直接整合到上下文管理中

## 安装要求

- Rust 1.70+
- Cargo

## 配置

系统使用 `.env` 文件进行配置：

```bash
# API Configuration
BING_API_KEY=your_bing_api_key_here
BING_SEARCH_URL=https://api.bing.microsoft.com/v7.0/search

# GitHub API Configuration (optional, for higher rate limits)
GITHUB_API_KEY=your_github_personal_access_token_here

# AI Configuration
AI_BASE_URL=http://103.203.140.12:7578/v1
AI_MODEL=qwen3-8b-union
AI_TEMPERATURE=0.7
AI_MAX_TOKENS=100
```

## 快速开始

```bash
# 构建项目
cargo build

# 运行测试
cargo test

# 运行示例
cargo run --example ai_example
cargo run --example web_search_example
cargo run --example intelligent_search_example
```

## 使用示例

### 智能搜索示例

```rust
use penlai::context::context_management::ContextManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context_manager = ContextManager::new_with_web_search()?;
    
    // 执行智能搜索（自动路由）
    let results = context_manager.intelligent_search("Rust async programming").await?;
    
    // 从搜索结果创建上下文
    let context = context_manager.create_context_from_intelligent_search(
        "machine learning frameworks 2025", 
        "technology"
    ).await?;
    
    Ok(())
}
```

### 基础上下文管理

```rust
use penlai::context::context_management::ContextManager;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context_manager = ContextManager::new();
    
    let context = Context {
        id: Uuid::new_v4(),
        domain: "medical".to_string(),
        content: "Medical context information".to_string(),
        tags: vec!["health".to_string(), "medicine".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 1,
        priority: 5,
        metadata: HashMap::new(),
    };

    context_manager.add_context(context).await?;
    
    Ok(())
}
```

## 项目结构

- `src/context/` - 上下文管理核心模块
- `src/utils/` - 工具模块（包括搜索功能）
- `src/selection/` - 上下文选择算法
- `src/processing/` - 并发处理模块
- `src/monitoring/` - 监控系统
- `src/cache/` - 缓存系统
- `src/domain/` - 领域分类器

## API 支持

- **GitHub API**：用于代码和技术资源搜索
- **Bing Search API**：用于一般网络搜索
- **自定义 AI API**：支持各种大模型服务

## 许可证

MIT License