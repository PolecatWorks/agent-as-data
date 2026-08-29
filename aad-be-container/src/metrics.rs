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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::sync::Arc;
    use axum_prometheus::metrics_exporter_prometheus::PrometheusBuilder;
    use crate::config::AppConfig;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn test_prometheus_response_mystate_and_free() {
        let handle = PrometheusBuilder::new().install_recorder().unwrap_or_else(|_| {
            // If already installed in this process, builder error is ignored
            PrometheusBuilder::new().build_recorder().handle()
        });

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
}
