use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use axum::{
    routing::{get, post},
    Router,
    extract::{State, Path},
    response::{IntoResponse, Response, Sse},
    Json,
    http::{StatusCode, header, HeaderMap},
    body::StreamBody,
};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use std::convert::Infallible;
use std::time::Duration;
use serde_json::json;
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
    compression::CompressionLayer,
    timeout::TimeoutLayer,
};
use codex_orchestrator::{AgentOrchestrator, OrchestrationRequest, OrchestrationResponse};
use uuid::Uuid;
use chrono;

struct AppState {
    orchestrator: Arc<AgentOrchestrator>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the orchestrator
    let orchestrator = Arc::new(AgentOrchestrator::new()?);
    
    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build our application with routes
    let app = Router::new()
        // Health check endpoints
        .route("/healthz", get(health_check))
        .route("/readyz", get(ready_check))
        // API v1 endpoints
        .route("/api/v1/orchestrate", post(handle_orchestration))
        .route("/api/v1/orchestrate/stream", get(handle_streaming_orchestration))
        .route("/api/v1/requests/:request_id", get(get_request_status))
        .route("/api/v1/agents", get(list_agents))
        .route("/api/v1/metrics", get(get_metrics))
        .with_state(Arc::new(AppState { orchestrator }))
        .layer(cors)
        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(30)));

    // Configure server
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // Log startup
    println!("🚀 Server running on http://{}/healthz", addr);
    println!("📚 API documentation available at http://{}/api-docs", addr);
    
    // Start server
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

// Health check endpoint
async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    let response = json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });
    (StatusCode::OK, Json(response))
}

// Readiness check endpoint
async fn ready_check() -> (StatusCode, Json<serde_json::Value>) {
    // Add your readiness checks here (e.g., database connection, external services)
    let response = json!({
        "status": "ready",
        "services": {
            "database": "ok",
            "cache": "ok",
            "model_provider": "ok"
        }
    });
    (StatusCode::OK, Json(response))
}

// Get request status
async fn get_request_status(
    Path(request_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // In a real implementation, you would fetch this from a database
    // For now, we'll return a mock response
    Ok(Json(json!({
        "request_id": request_id,
        "status": "completed",
        "created_at": "2024-03-20T12:00:00Z",
        "completed_at": "2024-03-20T12:00:30Z",
        "result": {}
    })))
}

// List available agents
async fn list_agents() -> Json<serde_json::Value> {
    Json(json!([
        {
            "id": "spec",
            "name": "Specification Agent",
            "description": "Generates requirements and specifications"
        },
        {
            "id": "code",
            "name": "Code Agent",
            "description": "Generates and modifies code"
        },
        {
            "id": "test",
            "name": "Test Generator",
            "description": "Generates test cases"
        },
        {
            "id": "reviewer",
            "name": "Code Reviewer",
            "description": "Reviews code and provides feedback"
        },
        {
            "id": "debug",
            "name": "Debug Agent",
            "description": "Helps debug issues in code"
        }
    ]))
}

// Get metrics
async fn get_metrics() -> Response {
    let metrics = format!(
        "# HELP codex_requests_total Total number of requests\n".to_owned() + 
        "# TYPE codex_requests_total counter\n" +
        "codex_requests_total{{status=\"success\"}} 42\n" +
        "codex_requests_total{{status=\"error\"}} 5\n" +
        "# HELP codex_request_duration_seconds Request duration in seconds\n" +
        "# TYPE codex_request_duration_seconds histogram\n" +
        "codex_request_duration_seconds_bucket{{le=\"0.1\"}} 30\n" +
        "codex_request_duration_seconds_bucket{{le=\"0.5\"}} 35\n" +
        "codex_request_duration_seconds_bucket{{le=\"1.0\"}} 40\n" +
        "codex_request_duration_seconds_bucket{{le=\"+Inf\"}} 47\n" +
        "codex_request_duration_seconds_sum 15.3\n" +
        "codex_request_duration_seconds_count 47"
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "text/plain; version=0.0.4".parse().unwrap(),
    );

    (headers, metrics).into_response()
}

// Graceful shutdown handler
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Shutting down gracefully...");
}

// Streaming orchestration endpoint
async fn handle_streaming_orchestration(
    State(state): State<Arc<AppState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    let stream = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
        .then(move |_| {
            let state = state.clone();
            async move {
                // Generate streaming content (this would be replaced with actual streaming from agents)
                let content = format!("Streaming content at {}", chrono::Utc::now().format("%H:%M:%S"));

                Ok(axum::response::sse::Event::default().data(content))
            }
        });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
