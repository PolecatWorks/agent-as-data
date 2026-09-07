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
