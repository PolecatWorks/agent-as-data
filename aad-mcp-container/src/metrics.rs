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

    let axum_string = state.prometheus_handle.render();

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
    use std::sync::OnceLock;
    use axum_prometheus::metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

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

    #[test]
    fn test_init_startup_metrics() {
        let handle = get_test_handle();

        init_startup_metrics("aad-mcp", "0.1.0");

        let rendered = handle.render();
        assert!(!rendered.is_empty(), "Rendered metrics should not be empty after startup initialization");
        assert!(rendered.contains("app_info"), "Rendered metrics should include app_info");
        assert!(rendered.contains("aad-mcp"), "Rendered metrics should include application name");
        assert!(rendered.contains("0.1.0"), "Rendered metrics should include version");
    }

    #[tokio::test]
    async fn test_tokio_metrics_reporting() {
        let handle = get_test_handle();

        let reporter_task = tokio::task::spawn(
            tokio_metrics::RuntimeMetricsReporterBuilder::default()
                .with_interval(std::time::Duration::from_millis(50))
                .describe_and_run(),
        );

        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        let rendered = handle.render();
        assert!(
            rendered.contains("tokio_workers_count"),
            "Rendered metrics should contain tokio_workers_count from tokio-metrics reporter"
        );

        reporter_task.abort();
    }
}
