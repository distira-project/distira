use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;
use tower_http::cors::{Any, CorsLayer};

/// ── Shared application state ──────────────────────────
///
/// `MetricsCollector` accumulates real metrics from every
/// `/v1/compile` call.  The SSE endpoint reads a snapshot
/// every 2 s and pushes it to connected dashboards.
#[derive(Debug, Clone, Serialize)]
struct MetricsSnapshot {
    ts: u64,
    total_requests: u64,
    raw_tokens: usize,
    compiled_tokens: usize,
    memory_reused_tokens: usize,
    efficiency_score: f32,
    local_ratio: f32,
    cache_hits: u64,
    cache_misses: u64,
    /// Rolling history – last 24 data‑points (one per snapshot)
    history_raw: Vec<usize>,
    history_compiled: Vec<usize>,
    history_reused: Vec<usize>,
    /// Per‑provider counters
    routes_local: u64,
    routes_cloud: u64,
    routes_midtier: u64,
}

#[derive(Debug)]
struct MetricsCollector {
    snapshot: MetricsSnapshot,
    sem_cache: cache::SemanticCache,
}

impl MetricsCollector {
    fn new() -> Self {
        Self {
            snapshot: MetricsSnapshot {
                ts: now_epoch(),
                total_requests: 0,
                raw_tokens: 0,
                compiled_tokens: 0,
                memory_reused_tokens: 0,
                efficiency_score: 0.0,
                local_ratio: 0.0,
                cache_hits: 0,
                cache_misses: 0,
                history_raw: Vec::with_capacity(24),
                history_compiled: Vec::with_capacity(24),
                history_reused: Vec::with_capacity(24),
                routes_local: 0,
                routes_cloud: 0,
                routes_midtier: 0,
            },
            sem_cache: cache::SemanticCache::new(),
        }
    }

    /// Record one compile pass.
    fn record(
        &mut self,
        raw: usize,
        compiled: usize,
        reused: usize,
        provider: &str,
        cache_hit: bool,
    ) {
        let s = &mut self.snapshot;
        s.total_requests += 1;
        s.raw_tokens += raw;
        s.compiled_tokens += compiled;
        s.memory_reused_tokens += reused;

        if cache_hit {
            s.cache_hits += 1;
        } else {
            s.cache_misses += 1;
        }

        match provider {
            "ollama-local" => s.routes_local += 1,
            "mistral-cloud" => s.routes_midtier += 1,
            _ => s.routes_cloud += 1,
        }

        // efficiency = avoided / raw (cumulative)
        let avoided = s.raw_tokens.saturating_sub(s.compiled_tokens);
        s.efficiency_score = if s.raw_tokens == 0 {
            0.0
        } else {
            (avoided as f32 / s.raw_tokens as f32) * 100.0
        };

        let total_routes = s.routes_local + s.routes_cloud + s.routes_midtier;
        s.local_ratio = if total_routes == 0 {
            0.0
        } else {
            (s.routes_local as f32 / total_routes as f32) * 100.0
        };

        // Push rolling history (cap 24)
        s.history_raw.push(s.raw_tokens);
        s.history_compiled.push(s.compiled_tokens);
        s.history_reused.push(s.memory_reused_tokens);
        if s.history_raw.len() > 24 {
            s.history_raw.remove(0);
            s.history_compiled.remove(0);
            s.history_reused.remove(0);
        }

        s.ts = now_epoch();
    }

    fn snapshot(&self) -> &MetricsSnapshot {
        &self.snapshot
    }
}

type SharedState = Arc<Mutex<MetricsCollector>>;

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// ── Handlers ──────────────────────────────────────────

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "katara-core",
        "version": "7.0.0"
    }))
}

async fn version() -> Json<serde_json::Value> {
    Json(json!({
        "version": "7.0.0",
        "product": "KATARA"
    }))
}

#[derive(Deserialize)]
struct CompileRequest {
    context: Option<String>,
    sensitive: Option<bool>,
}

async fn compile(
    State(state): State<SharedState>,
    Json(payload): Json<CompileRequest>,
) -> Json<serde_json::Value> {
    let raw = payload.context.as_deref().unwrap_or("");
    let sensitive = payload.sensitive.unwrap_or(false);

    // 1. Fingerprint
    let fp = fingerprint::fingerprint(raw);

    // 2. Cache lookup
    let mut collector = state.lock().unwrap();
    let cache_hit = collector.sem_cache.get(fp).is_some();

    // 3. Compile
    let result = compiler::compile_context(raw);

    // 4. Memory
    let mem = memory::summarize_memory(result.raw_tokens_estimate);

    // 5. Route
    let route = router::choose_provider(&result.intent, sensitive);

    // 6. Efficiency
    let efficiency = metrics::compute(
        result.raw_tokens_estimate,
        result.compiled_tokens_estimate,
        mem.reused_tokens,
    );

    // 7. Update cache
    if !cache_hit {
        collector.sem_cache.insert(fp, result.summary.clone());
    }

    // 8. Record metrics
    collector.record(
        result.raw_tokens_estimate,
        result.compiled_tokens_estimate,
        mem.reused_tokens,
        &route.provider,
        cache_hit,
    );

    drop(collector); // release lock before response serialization

    Json(json!({
        "fingerprint": fp.to_string(),
        "cache_hit": cache_hit,
        "intent": result.intent,
        "raw_tokens": result.raw_tokens_estimate,
        "compiled_tokens": result.compiled_tokens_estimate,
        "memory_reused_tokens": mem.reused_tokens,
        "context_reuse_ratio": mem.context_reuse_ratio,
        "provider": route.provider,
        "routing_reason": route.reason,
        "token_avoidance_ratio": efficiency.token_avoidance_ratio
    }))
}

/// Snapshot endpoint — one‑shot JSON for dashboards that poll.
async fn metrics_snapshot(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let collector = state.lock().unwrap();
    Json(serde_json::to_value(collector.snapshot()).unwrap_or_default())
}

/// SSE stream — pushes a `MetricsSnapshot` event every 2 seconds.
async fn metrics_stream(
    State(state): State<SharedState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let interval = tokio::time::interval(std::time::Duration::from_secs(2));
    let stream = IntervalStream::new(interval).map(move |_| {
        let collector = state.lock().unwrap();
        let data = serde_json::to_string(collector.snapshot()).unwrap_or_default();
        Ok(Event::default().event("metrics").data(data))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// ── Main ──────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let state: SharedState = Arc::new(Mutex::new(MetricsCollector::new()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/version", get(version))
        .route("/v1/compile", post(compile))
        .route("/v1/metrics", get(metrics_snapshot))
        .route("/v1/metrics/stream", get(metrics_stream))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("KATARA core listening on {addr}");
    println!("  POST /v1/compile          — compile context");
    println!("  GET  /v1/metrics          — JSON snapshot");
    println!("  GET  /v1/metrics/stream   — SSE live stream");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
