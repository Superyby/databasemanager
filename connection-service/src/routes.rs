//! 连接服务路由模块

use axum::{routing::get, Router};
use crate::handlers;
use crate::state::AppState;

/// 创建连接管理路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/connections", get(handlers::list_connections).post(handlers::create_connection))
        .route("/api/connections/{id}", get(handlers::get_connection).delete(handlers::delete_connection))
        .route("/api/connections/{id}/test", get(handlers::test_connection))
        .route("/api/health", get(handlers::health_check))
        .route("/internal/pools/{id}", get(handlers::get_pool_info))
        // Trait 演示接口
        .route("/api/demo/trait/real", get(handlers::demo_trait_real))
        .route("/api/demo/trait/mock", get(handlers::demo_trait_mock))
        .route("/api/demo/trait/generic", get(handlers::demo_trait_generic))
}
