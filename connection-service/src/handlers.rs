//! Handler模块

use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use common::errors::AppError;
use common::models::connection::{ConnectionItem, CreateConnectionRequest};
use common::response::ApiResponse;
use crate::service::ConnectionService;
use crate::state::AppState;

/// 列出所有已保存的数据库连接
#[utoipa::path(
    get,
    path = "/api/connections",
    tag = "connections",
    responses(
        (status = 200, description = "连接列表", body = ApiResponse<Vec<ConnectionItem>>)
    )
)]
pub async fn list_connections(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ConnectionItem>>>, AppError> {
    let service = ConnectionService::new(state.pool_manager);
    let data = service.list().await;
    Ok(Json(ApiResponse::ok_with_service(data, "connection-service")))
}

/// 创建新的数据库连接
#[utoipa::path(
    post,
    path = "/api/connections",
    tag = "connections",
    request_body = CreateConnectionRequest,
    responses(
        (status = 200, description = "连接已创建", body = ApiResponse<ConnectionItem>)
    )
)]
pub async fn create_connection(
    State(state): State<AppState>,
    Json(req): Json<CreateConnectionRequest>,
) -> Result<Json<ApiResponse<ConnectionItem>>, AppError> {
    let service = ConnectionService::new(state.pool_manager);
    let data = service.create(req).await?;
    Ok(Json(ApiResponse::ok_with_service(data, "connection-service")))
}

/// 根据 ID 获取连接
#[utoipa::path(
    get,
    path = "/api/connections/{id}",
    tag = "connections",
    params(
        ("id" = String, Path, description = "连接 ID")
    ),
    responses(
        (status = 200, description = "连接详情", body = ApiResponse<ConnectionItem>),
        (status = 404, description = "连接未找到")
    )
)]
pub async fn get_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ConnectionItem>>, AppError> {
    let service = ConnectionService::new(state.pool_manager);
    let data = service.get(&id).await?;
    Ok(Json(ApiResponse::ok_with_service(data, "connection-service")))
}

/// 根据 ID 删除数据库连接
#[utoipa::path(
    delete,
    path = "/api/connections/{id}",
    tag = "connections",
    params(
        ("id" = String, Path, description = "连接 ID")
    ),
    responses(
        (status = 200, description = "连接已删除", body = ApiResponse<bool>),
        (status = 404, description = "连接未找到")
    )
)]
pub async fn delete_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    let service = ConnectionService::new(state.pool_manager);
    service.delete(&id).await?;
    Ok(Json(ApiResponse::ok_with_service(true, "connection-service")))
}

/// 测试数据库连接
#[utoipa::path(
    get,
    path = "/api/connections/{id}/test",
    tag = "connections",
    params(
        ("id" = String, Path, description = "连接 ID")
    ),
    responses(
        (status = 200, description = "连接测试结果", body = ApiResponse<ConnectionTestResult>),
        (status = 404, description = "连接未找到")
    )
)]
pub async fn test_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ConnectionTestResult>>, AppError> {
    let service = ConnectionService::new(state.pool_manager);
    match service.test(&id).await {
        Ok(latency_ms) => Ok(Json(ApiResponse::ok_with_service(
            ConnectionTestResult {
                id,
                success: true,
                latency_ms: Some(latency_ms),
                error: None,
            },
            "connection-service",
        ))),
        Err(e) => Ok(Json(ApiResponse::ok_with_service(
            ConnectionTestResult {
                id,
                success: false,
                latency_ms: None,
                error: Some(e.to_string()),
            },
            "connection-service",
        ))),
    }
}

/// 健康检查端点
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "health",
    responses(
        (status = 200, description = "服务运行正常", body = HealthResponse)
    )
)]
pub async fn health_check(
    State(state): State<AppState>,
) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "connection-service".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now(),
        connections: state.pool_manager.connection_count().await,
    })
}

/// 内部端点，供其他服务获取连接池信息
#[utoipa::path(
    get,
    path = "/internal/pools/{id}",
    tag = "internal",
    params(
        ("id" = String, Path, description = "连接 ID")
    ),
    responses(
        (status = 200, description = "连接池信息", body = ApiResponse<PoolInfo>),
        (status = 404, description = "连接未找到")
    )
)]
pub async fn get_pool_info(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<PoolInfo>>, AppError> {
    let service = ConnectionService::new(state.pool_manager.clone());
    let conn = service.get(&id).await?;
    
    Ok(Json(ApiResponse::ok(PoolInfo {
        id: conn.id,
        db_type: conn.db_type.to_string(),
        host: conn.host,
        port: conn.port,
        database: conn.database,
    })))
}

#[derive(Serialize, ToSchema)]
pub struct ConnectionTestResult {
    pub id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub connections: usize,
}

#[derive(Serialize, ToSchema)]
pub struct PoolInfo {
    pub id: String,
    pub db_type: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
}

// ============================================================
// Trait 演示接口
// ============================================================

use crate::service::{ConnectionServiceTrait, MockConnectionService};
use common::models::connection::DbType;

/// Trait 演示响应
#[derive(Serialize, ToSchema)]
pub struct TraitDemoResponse {
    /// 使用的实现类型
    pub impl_type: String,
    /// 连接数量
    pub connection_count: usize,
    /// 连接列表
    pub connections: Vec<ConnectionItem>,
    /// 说明
    pub description: String,
}

/// 演示 Trait 用法 - 使用真实实现
/// 
/// 这个接口使用真实的 ConnectionService 实现
#[utoipa::path(
    get,
    path = "/api/demo/trait/real",
    tag = "demo",
    responses(
        (status = 200, description = "真实实现演示", body = ApiResponse<TraitDemoResponse>)
    )
)]
pub async fn demo_trait_real(
    State(state): State<AppState>,
) -> Json<ApiResponse<TraitDemoResponse>> {
    // 使用真实实现
    let service = ConnectionService::new(state.pool_manager);
    
    // 通过 trait 方法调用
    let connections = service.list().await;
    
    Json(ApiResponse::ok(TraitDemoResponse {
        impl_type: "ConnectionService (真实实现)".to_string(),
        connection_count: connections.len(),
        connections,
        description: "这是使用真实数据库连接池的实现，数据来自实际存储".to_string(),
    }))
}

/// 演示 Trait 用法 - 使用 Mock 实现
/// 
/// 这个接口使用 MockConnectionService 实现，返回假数据
#[utoipa::path(
    get,
    path = "/api/demo/trait/mock",
    tag = "demo",
    responses(
        (status = 200, description = "Mock 实现演示", body = ApiResponse<TraitDemoResponse>)
    )
)]
pub async fn demo_trait_mock() -> Json<ApiResponse<TraitDemoResponse>> {
    // 创建带有假数据的 Mock 服务
    let fake_connections = vec![
        ConnectionItem {
            id: "mock-001".to_string(),
            name: "Mock MySQL 连接".to_string(),
            db_type: DbType::MySQL,
            host: Some("fake-mysql.example.com".to_string()),
            port: Some(3306),
            username: Some("mock_user".to_string()),
            database: Some("mock_db".to_string()),
            file_path: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        },
        ConnectionItem {
            id: "mock-002".to_string(),
            name: "Mock PostgreSQL 连接".to_string(),
            db_type: DbType::Postgres,
            host: Some("fake-postgres.example.com".to_string()),
            port: Some(5432),
            username: Some("mock_admin".to_string()),
            database: Some("mock_postgres".to_string()),
            file_path: None,
            created_at: "2026-01-02T00:00:00Z".to_string(),
        },
    ];
    
    // 使用 Mock 实现
    let service = MockConnectionService::with_data(fake_connections);
    
    // 通过同样的 trait 方法调用（接口一样，实现不同）
    let connections = service.list().await;
    
    Json(ApiResponse::ok(TraitDemoResponse {
        impl_type: "MockConnectionService (Mock 实现)".to_string(),
        connection_count: connections.len(),
        connections,
        description: "这是 Mock 实现，返回的是预设的假数据，不需要真实数据库".to_string(),
    }))
}

/// 演示泛型函数：接受任何实现了 Trait 的类型
async fn get_connection_count<S: ConnectionServiceTrait>(service: &S) -> usize {
    service.list().await.len()
}

/// 演示 Trait 泛型调用
#[utoipa::path(
    get,
    path = "/api/demo/trait/generic",
    tag = "demo",
    responses(
        (status = 200, description = "泛型调用演示", body = ApiResponse<serde_json::Value>)
    )
)]
pub async fn demo_trait_generic(
    State(state): State<AppState>,
) -> Json<ApiResponse<serde_json::Value>> {
    // 真实实现
    let real_service = ConnectionService::new(state.pool_manager);
    let real_count = get_connection_count(&real_service).await;
    
    // Mock 实现
    let mock_service = MockConnectionService::with_data(vec![
        ConnectionItem {
            id: "mock-1".to_string(),
            name: "Mock 1".to_string(),
            db_type: DbType::SQLite,
            host: None,
            port: None,
            username: None,
            database: None,
            file_path: Some("/tmp/mock.db".to_string()),
            created_at: "2026-01-01T00:00:00Z".to_string(),
        },
    ]);
    let mock_count = get_connection_count(&mock_service).await;
    
    // 同一个泛型函数，传入不同实现，结果不同
    Json(ApiResponse::ok(serde_json::json!({
        "说明": "同一个泛型函数 get_connection_count<S>()，传入不同的实现",
        "真实实现结果": {
            "类型": "ConnectionService",
            "连接数": real_count
        },
        "Mock实现结果": {
            "类型": "MockConnectionService", 
            "连接数": mock_count
        },
        "结论": "这就是 Trait 的作用：定义统一接口，允许不同实现"
    })))
}
