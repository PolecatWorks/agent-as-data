use serde::{Deserialize, Serialize};
use tokio::runtime::{self, Runtime};
use tracing::info;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ThreadRuntime {
    /// Number of worker threads (0 for current thread / single-threaded runtime).
    pub threads: usize,
    /// Thread stack size in bytes.
    pub stack_size: usize,
    /// Name prefix assigned to worker threads.
    pub name: String,
}

impl Default for ThreadRuntime {
    fn default() -> Self {
        ThreadRuntime {
            threads: 4,
            stack_size: 3_000_000,
            name: "aad-worker".into(),
        }
    }
}

pub fn create_tokio_runtime(runtime: &ThreadRuntime) -> Result<Runtime, String> {
    if runtime.threads == 0 {
        runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|e| format!("Failed to create single-threaded Tokio runtime: {}", e))
    } else {
        runtime::Builder::new_multi_thread()
            .worker_threads(runtime.threads)
            .thread_name(runtime.name.clone())
            .thread_stack_size(runtime.stack_size)
            .enable_io()
            .enable_time()
            .build()
            .map_err(|e| format!("Failed to create multi-threaded Tokio runtime: {}", e))
    }
}

pub fn run_in_tokio<F, T>(runtime: &ThreadRuntime, my_function: F) -> Result<T, String>
where
    F: std::future::Future<Output = Result<T, String>>,
{
    info!("Starting Tokio runtime instance: '{}' with {} worker threads", runtime.name, runtime.threads);
    let rt = create_tokio_runtime(runtime)?;
    rt.block_on(my_function)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_threaded_runtime() {
        let runtime = ThreadRuntime {
            threads: 0,
            stack_size: 2_000_000,
            name: "test-single".into(),
        };
        let rt = create_tokio_runtime(&runtime).unwrap();
        let result = rt.block_on(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_multi_threaded_runtime() {
        let runtime = ThreadRuntime {
            threads: 2,
            stack_size: 2_000_000,
            name: "test-multi".into(),
        };
        let rt = create_tokio_runtime(&runtime).unwrap();
        let result = rt.block_on(async { 100 });
        assert_eq!(result, 100);
    }
}
