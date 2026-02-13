//! 路由模块

use axum::{
    routing::{get, post},
    Router,
};
use crate::handlers;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/query", post(handlers::execute_query))
        .route("/api/health", get(handlers::health_check))
        .route("/api/test", get(handlers::hello_test))

        // Trait 演示接口
        .route("/api/demo/trait/real", get(handlers::demo_trait_real))
        .route("/api/demo/trait/mock", get(handlers::demo_trait_mock))
}
