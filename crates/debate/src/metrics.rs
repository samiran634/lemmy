use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Metrics for debate system monitoring
#[derive(Clone)]
pub struct DebateMetrics {
    inner: Arc<DebateMetricsInner>,
}

struct DebateMetricsInner {
    active_debates: AtomicUsize,
    total_debates_started: AtomicU64,
    total_debates_completed: AtomicU64,
    total_debates_failed: AtomicU64,
    total_rounds_executed: AtomicU64,
    total_api_calls: AtomicU64,
    total_api_errors: AtomicU64,
    total_tokens_used: AtomicU64,
}

impl Default for DebateMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl DebateMetrics {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DebateMetricsInner {
                active_debates: AtomicUsize::new(0),
                total_debates_started: AtomicU64::new(0),
                total_debates_completed: AtomicU64::new(0),
                total_debates_failed: AtomicU64::new(0),
                total_rounds_executed: AtomicU64::new(0),
                total_api_calls: AtomicU64::new(0),
                total_api_errors: AtomicU64::new(0),
                total_tokens_used: AtomicU64::new(0),
            }),
        }
    }

    pub fn increment_active_debates(&self) {
        self.inner.active_debates.fetch_add(1, Ordering::Relaxed);
        self.inner.total_debates_started.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_active_debates(&self) {
        self.inner.active_debates.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn increment_completed_debates(&self) {
        self.inner.total_debates_completed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_failed_debates(&self) {
        self.inner.total_debates_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_rounds_executed(&self) {
        self.inner.total_rounds_executed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_api_calls(&self) {
        self.inner.total_api_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_api_errors(&self) {
        self.inner.total_api_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_tokens_used(&self, tokens: u64) {
        self.inner.total_tokens_used.fetch_add(tokens, Ordering::Relaxed);
    }

    pub fn get_active_debates(&self) -> usize {
        self.inner.active_debates.load(Ordering::Relaxed)
    }

    pub fn get_total_debates_started(&self) -> u64 {
        self.inner.total_debates_started.load(Ordering::Relaxed)
    }

    pub fn get_total_debates_completed(&self) -> u64 {
        self.inner.total_debates_completed.load(Ordering::Relaxed)
    }

    pub fn get_total_debates_failed(&self) -> u64 {
        self.inner.total_debates_failed.load(Ordering::Relaxed)
    }

    pub fn get_total_rounds_executed(&self) -> u64 {
        self.inner.total_rounds_executed.load(Ordering::Relaxed)
    }

    pub fn get_total_api_calls(&self) -> u64 {
        self.inner.total_api_calls.load(Ordering::Relaxed)
    }

    pub fn get_total_api_errors(&self) -> u64 {
        self.inner.total_api_errors.load(Ordering::Relaxed)
    }

    pub fn get_total_tokens_used(&self) -> u64 {
        self.inner.total_tokens_used.load(Ordering::Relaxed)
    }

    pub fn get_api_error_rate(&self) -> f64 {
        let total = self.get_total_api_calls();
        if total == 0 {
            return 0.0;
        }
        let errors = self.get_total_api_errors();
        (errors as f64 / total as f64) * 100.0
    }
}

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
    operation: String,
}

impl Timer {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.into(),
        }
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.elapsed_ms();
        tracing::debug!(
            operation = %self.operation,
            duration_ms = elapsed,
            "Operation completed"
        );
    }
}
