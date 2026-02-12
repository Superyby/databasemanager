//! 连接管理服务模块
//!
//! 使用 Trait 模式实现，支持多种实现方式（真实实现、Mock实现等）

use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use common::errors::{AppError, AppResult};
use common::models::connection::{ConnectionItem, CreateConnectionRequest};
use crate::pool_manager::PoolManager;

// ============================================================
// 1️⃣ 定义 Trait（类似 Java 的 Service 接口）
// ============================================================

/// 连接服务 Trait - 定义连接管理的所有能力
/// 
/// 这就像 Java 中的 `interface ConnectionService`
/// 任何实现了这个 trait 的类型，都可以作为连接服务使用
#[async_trait]
pub trait ConnectionServiceTrait: Send + Sync {
    /// 列出所有连接
    async fn list(&self) -> Vec<ConnectionItem>;
    
    /// 创建新连接
    async fn create(&self, req: CreateConnectionRequest) -> AppResult<ConnectionItem>;
    
    /// 根据 ID 获取连接
    async fn get(&self, id: &str) -> AppResult<ConnectionItem>;
    
    /// 根据 ID 删除连接
    async fn delete(&self, id: &str) -> AppResult<()>;
    
    /// 测试连接
    async fn test(&self, id: &str) -> AppResult<u64>;
}

// ============================================================
// 2️⃣ 真实实现（类似 Java 的 ServiceImpl）
// ============================================================

/// 数据库连接管理服务 - 真实实现
/// 
/// 这就像 Java 中的 `class ConnectionServiceImpl implements ConnectionService`
pub struct ConnectionService {
    pool_manager: Arc<PoolManager>,
}

impl ConnectionService {
    /// 创建新的连接服务实例
    pub fn new(pool_manager: Arc<PoolManager>) -> Self {
        Self { pool_manager }
    }
}

/// 为 ConnectionService 实现 Trait
#[async_trait]
impl ConnectionServiceTrait for ConnectionService {
    async fn list(&self) -> Vec<ConnectionItem> {
        self.pool_manager
            .list_connections()
            .await
            .into_iter()
            .map(ConnectionItem::from)
            .collect()
    }

    async fn create(&self, req: CreateConnectionRequest) -> AppResult<ConnectionItem> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        let config = req.into_config(id.clone(), created_at);

        // 添加到连接池管理器（会进行验证并建立连接）
        self.pool_manager.add_connection(config.clone()).await?;

        tracing::info!(id = %id, name = %config.name, "连接已创建");
        Ok(ConnectionItem::from(config))
    }

    async fn get(&self, id: &str) -> AppResult<ConnectionItem> {
        self.pool_manager
            .get_connection(id)
            .await
            .map(ConnectionItem::from)
            .ok_or_else(|| AppError::ConnectionNotFound(id.to_string()))
    }

    async fn delete(&self, id: &str) -> AppResult<()> {
        self.pool_manager.remove_connection(id).await?;
        tracing::info!(id = %id, "连接已删除");
        Ok(())
    }

    async fn test(&self, id: &str) -> AppResult<u64> {
        let latency = self.pool_manager.test_connection(id).await?;
        Ok(latency.as_millis() as u64)
    }
}

// ============================================================
// 3️⃣ Mock 实现（用于测试或演示）
// ============================================================

/// Mock 连接服务 - 返回假数据，用于测试
/// 
/// 这就像 Java 中的 `class MockConnectionService implements ConnectionService`
/// 可以在单元测试中使用，不需要真正的数据库连接
pub struct MockConnectionService {
    /// 模拟的连接列表
    connections: Vec<ConnectionItem>,
}

impl MockConnectionService {
    /// 创建空的 Mock 服务
    pub fn new() -> Self {
        Self { connections: vec![] }
    }
    
    /// 创建带有预设数据的 Mock 服务
    pub fn with_data(connections: Vec<ConnectionItem>) -> Self {
        Self { connections }
    }
}

impl Default for MockConnectionService {
    fn default() -> Self {
        Self::new()
    }
}

/// 为 MockConnectionService 实现同样的 Trait
#[async_trait]
impl ConnectionServiceTrait for MockConnectionService {
    async fn list(&self) -> Vec<ConnectionItem> {
        // 返回预设的数据
        self.connections.clone()
    }

    async fn create(&self, req: CreateConnectionRequest) -> AppResult<ConnectionItem> {
        // Mock 实现：直接返回一个假的连接
        let id = Uuid::new_v4().to_string();
        Ok(ConnectionItem {
            id,
            name: req.name,
            db_type: req.db_type,
            host: req.host,
            port: req.port,
            database: req.database,
            username: req.username,
            file_path: req.file_path,
            created_at: Utc::now().to_rfc3339(),
        })
    }

    async fn get(&self, id: &str) -> AppResult<ConnectionItem> {
        // Mock 实现：从预设数据中查找
        self.connections
            .iter()
            .find(|c| c.id == id)
            .cloned()
            .ok_or_else(|| AppError::ConnectionNotFound(id.to_string()))
    }

    async fn delete(&self, id: &str) -> AppResult<()> {
        // Mock 实现：检查是否存在
        if self.connections.iter().any(|c| c.id == id) {
            Ok(())
        } else {
            Err(AppError::ConnectionNotFound(id.to_string()))
        }
    }

    async fn test(&self, _id: &str) -> AppResult<u64> {
        // Mock 实现：直接返回一个假的延迟值
        Ok(10) // 假装延迟 10ms
    }
}

// ============================================================
// 4️⃣ 演示：使用 Trait 统一调用
// ============================================================

/// 演示函数：展示如何使用 Trait 进行统一调用
/// 
/// 这个函数接受任何实现了 ConnectionServiceTrait 的类型
/// 不管是真实实现还是 Mock 实现，调用方式完全一样
pub async fn demo_trait_usage<S: ConnectionServiceTrait>(service: &S) {
    // 获取所有连接
    let connections = service.list().await;
    println!("连接数量: {}", connections.len());
    
    // 遍历连接
    for conn in &connections {
        println!("  - {} ({})", conn.name, conn.id);
    }
}
