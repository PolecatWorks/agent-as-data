//! C-FFI compatibility callbacks for exporting Prometheus metrics.
//!
//! Provides memory-safe raw pointer handlers for bridging Tokio/Axum Prometheus metrics
//! with external C-FFI monitoring systems (e.g. HaMS).

use std::ffi::{c_char, c_void, CString};

use crate::state::AppState;

/// C-FFI callback function to render Prometheus metrics from a raw [`AppState`] pointer.
///
/// # Safety
///
/// `ptr` must be a valid non-null raw pointer to an [`AppState`].
/// The caller is responsible for freeing the returned string buffer using [`prometheus_response_free`].
#[unsafe(no_mangle)]
pub extern "C" fn prometheus_response_mystate(ptr: *const c_void) -> *mut c_char {
    let state = unsafe { &*(ptr as *const AppState) };

    let mut axum_string = state.prometheus_handle.render();

    let handle = &state.tokio_handle;
    let metrics = handle.metrics();
    axum_string.push_str("\n# HELP tokio_workers_count Number of worker threads\n");
    axum_string.push_str("# TYPE tokio_workers_count gauge\n");
    axum_string.push_str(&format!("tokio_workers_count {}\n", metrics.num_workers()));

    axum_string.push_str("# HELP tokio_alive_tasks Number of alive tasks\n");
    axum_string.push_str("# TYPE tokio_alive_tasks gauge\n");
    axum_string.push_str(&format!("tokio_alive_tasks {}\n", metrics.num_alive_tasks()));

    axum_string.push_str("# HELP tokio_blocking_threads Number of blocking threads\n");
    axum_string.push_str("# TYPE tokio_blocking_threads gauge\n");
    axum_string.push_str(&format!("tokio_blocking_threads {}\n", metrics.num_blocking_threads()));

    axum_string.push_str("# HELP tokio_idle_blocking_threads Number of idle blocking threads\n");
    axum_string.push_str("# TYPE tokio_idle_blocking_threads gauge\n");
    axum_string.push_str(&format!("tokio_idle_blocking_threads {}\n", metrics.num_idle_blocking_threads()));

    let buffer = axum_string.into_bytes();

    let prometheus = String::from_utf8(buffer).unwrap_or_default();
    let c_str_prometheus = std::ffi::CString::new(prometheus)
        .unwrap_or_else(|_| unsafe { CString::from_vec_unchecked(vec![]) });

    c_str_prometheus.into_raw()
}

/// C-FFI callback function to free string memory allocated by Prometheus response callbacks.
///
/// # Safety
///
/// `ptr` must point to a C-string previously allocated by [`prometheus_response_mystate`], or be null.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn prometheus_response_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    };
}

/// Initializes baseline application telemetry and registers initial static gauge metrics.
///
/// Guarantees that `/hams/metrics` immediately yields valid, non-empty Prometheus
/// exposition output upon container startup before any inbound API traffic is received.
pub fn init_startup_metrics(name: &str, version: &str) {


    metrics::describe_counter!(
        "skill_execution_total",
        "Total number of skill executions"
    );
    metrics::describe_counter!(
        "llm_tokens_total",
        "Total number of LLM tokens consumed"
    );

    metrics::describe_counter!(
        "agent_execution_total",
        "Total number of agent executions"
    );
    metrics::describe_counter!(
        "knowledge_ingestion_total",
        "Total number of knowledge ingestions"
    );

    metrics::describe_gauge!(
        "app_info",
        metrics::Unit::Count,
        "Application build and runtime metadata"
    );
    metrics::gauge!(
        "app_info",
        "name" => name.to_string(),
        "version" => version.to_string()
    )
    .set(1.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::sync::{Arc, OnceLock};
    use axum_prometheus::metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
    use crate::config::AppConfig;
    use sqlx::postgres::PgPoolOptions;

    static TEST_RECORDER_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

    fn get_test_handle() -> PrometheusHandle {
        TEST_RECORDER_HANDLE
            .get_or_init(|| {
                PrometheusBuilder::new()
                    .install_recorder()
                    .unwrap_or_else(|_| {
                        PrometheusBuilder::new().build_recorder().handle()
                    })
            })
            .clone()
    }

    #[tokio::test]
    async fn test_prometheus_response_mystate_and_free() {
        let handle = get_test_handle();

        let config = AppConfig {
            debugging: crate::config::DebuggingConfig {
                environment: "test".into(),
                log_level: "info".into(),
                fail_debug_delay: std::time::Duration::from_secs(0),
            },
            webservice: crate::config::WebServiceConfig {
                address: "127.0.0.1:8080".into(),
                api_prefix: "/api".into(),
            },
            llm: crate::config::LlmConfig {
                ollama_url: "http://localhost:11434".into(),
                model: "llama3".into(),
                timeout_secs: 30,
            },
            runtime: crate::tokio_tools::ThreadRuntime::default(),
            database: crate::config::DatabaseConfig {
                url: crate::config::UrlWithUsernamePassword {
                    url: url::Url::parse("postgres://localhost:5432/test").unwrap(),
                    username: Some("user".into()),
                    password: Some("pass".into()),
                },
                max_connections: 1,
            },
            hams: ::hams::hams::config::HamsConfig::default(),
        };

        // Create a dummy pool (does not connect)
        let pool = PgPoolOptions::new().connect_lazy("postgres://user:pass@localhost:5432/test").unwrap();

        let state = AppState {
            pool,
            config,
            prometheus_handle: Arc::new(handle),
            tokio_handle: tokio::runtime::Handle::current(),
        };

        let ptr = &state as *const _ as *const c_void;
        let c_char_ptr = prometheus_response_mystate(ptr);
        assert!(!c_char_ptr.is_null());

        let rendered = unsafe { CStr::from_ptr(c_char_ptr).to_str().unwrap() };
        let _ = rendered.len();

        prometheus_response_free(c_char_ptr);
        // Null pointer free safety
        prometheus_response_free(std::ptr::null_mut());
    }

    #[test]
    fn test_init_startup_metrics() {
        let handle = get_test_handle();

        init_startup_metrics("aad-be", "0.1.0");

        let rendered = handle.render();
        assert!(!rendered.is_empty(), "Rendered metrics should not be empty after startup initialization");
        assert!(rendered.contains("app_info"), "Rendered metrics should include app_info");
        assert!(rendered.contains("aad-be"), "Rendered metrics should include application name");
        assert!(rendered.contains("0.1.0"), "Rendered metrics should include version");
    }
}
