//! Background worker for debate processing
//!
//! This module provides the background worker task that polls for active debates
//! and processes them asynchronously. The worker:
//!
//! - Polls the database at configurable intervals for pending/active debates
//! - Processes debates with concurrency limits using a semaphore
//! - Handles errors gracefully without stopping the worker
//! - Tracks metrics for monitoring
//!
//! # Architecture
//!
//! The worker runs as a separate tokio task spawned during server initialization.
//! It uses a tokio interval timer for periodic polling and a semaphore to limit
//! concurrent debate processing.
//!
//! # Example
//!
//! ```rust,ignore
//! use lemmy_debate::worker::debate_worker_task;
//!
//! // Spawn worker during server startup
//! tokio::spawn(debate_worker_task(context));
//! ```

use crate::{context::DebateContext, models::DebateState, orchestrator::DebateOrchestrator, openrouter::OpenRouterClient};
use lemmy_utils::{error::LemmyResult, settings::SETTINGS};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Semaphore, time::interval};
use tracing::{debug, error, info, warn};

/// Statistics for worker monitoring
#[derive(Debug, Clone, Default)]
pub struct WorkerStats {
  pub debates_processed: u64,
  pub debates_failed: u64,
  pub total_processing_time_ms: u64,
  pub active_debates: u64,
  pub last_poll_time: Option<std::time::Instant>,
  pub total_polls: u64,
}

impl WorkerStats {
  /// Create a new WorkerStats instance
  pub fn new() -> Self {
    Self::default()
  }

  /// Record a successful debate processing
  pub fn record_success(&mut self, processing_time_ms: u64) {
    self.debates_processed += 1;
    self.total_processing_time_ms += processing_time_ms;
  }

  /// Record a failed debate processing
  pub fn record_failure(&mut self) {
    self.debates_failed += 1;
  }

  /// Update active debates count
  pub fn set_active_debates(&mut self, count: u64) {
    self.active_debates = count;
  }

  /// Record a poll
  pub fn record_poll(&mut self) {
    self.last_poll_time = Some(std::time::Instant::now());
    self.total_polls += 1;
  }

  /// Get average processing time in milliseconds
  pub fn average_processing_time_ms(&self) -> u64 {
    if self.debates_processed == 0 {
      0
    } else {
      self.total_processing_time_ms / self.debates_processed
    }
  }

  /// Get success rate as a percentage
  pub fn success_rate(&self) -> f64 {
    let total = self.debates_processed + self.debates_failed;
    if total == 0 {
      0.0
    } else {
      (self.debates_processed as f64 / total as f64) * 100.0
    }
  }

  /// Get time since last poll in seconds
  pub fn seconds_since_last_poll(&self) -> Option<u64> {
    self
      .last_poll_time
      .map(|t| t.elapsed().as_secs())
  }
}

/// Global worker stats for health checks
static WORKER_STATS: std::sync::OnceLock<Arc<tokio::sync::Mutex<WorkerStats>>> =
  std::sync::OnceLock::new();

/// Get current worker statistics
///
/// This function can be called from health check endpoints to monitor worker status.
pub async fn get_worker_stats() -> WorkerStats {
  if let Some(stats) = WORKER_STATS.get() {
    stats.lock().await.clone()
  } else {
    WorkerStats::new()
  }
}

/// Main debate worker task
///
/// This function runs indefinitely, polling for active debates and processing them
/// with concurrency limits. It should be spawned as a tokio task during server startup.
///
/// # Arguments
/// * `context` - The Lemmy context containing database pool and other resources
///
/// # Errors
/// Returns error only if the worker cannot be initialized. Processing errors for
/// individual debates are logged but do not stop the worker.
pub async fn debate_worker_task(context: DebateContext) -> LemmyResult<()> {
  info!("Starting debate worker task");

  // Get configuration from settings
  let poll_interval_secs = SETTINGS
    .debate
    .as_ref()
    .and_then(|d| d.poll_interval_seconds)
    .unwrap_or(10);

  let max_concurrent = SETTINGS
    .debate
    .as_ref()
    .and_then(|d| d.max_concurrent_debates)
    .unwrap_or(5);

  info!(
    "Debate worker configured: poll_interval={}s, max_concurrent={}",
    poll_interval_secs, max_concurrent
  );

  // Create interval timer
  let mut poll_interval = interval(Duration::from_secs(poll_interval_secs as u64));

  // Create semaphore for concurrency control
  let semaphore = Arc::new(Semaphore::new(max_concurrent as usize));

  // Initialize global worker stats
  let stats = WORKER_STATS.get_or_init(|| Arc::new(tokio::sync::Mutex::new(WorkerStats::new())));
  let stats = stats.clone();

  loop {
    poll_interval.tick().await;

    // Record poll
    {
      let mut stats_guard = stats.lock().await;
      stats_guard.record_poll();
    }

    debug!("Polling for active debates");

    // Find pending or active debates
    let debates = match DebateState::find_active_debates(&mut context.pool()).await {
      Ok(debates) => debates,
      Err(e) => {
        error!("Failed to query active debates: {}", e);
        continue;
      }
    };

    // Update active debates count
    {
      let mut stats_guard = stats.lock().await;
      stats_guard.set_active_debates(debates.len() as u64);
    }

    if debates.is_empty() {
      debug!("No active debates found");
      continue;
    }

    info!("Found {} active debate(s) to process", debates.len());

    // Process each debate with concurrency limit
    for debate in debates {
      let permit = match semaphore.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
          warn!(
            "Concurrency limit reached, skipping debate for post {} (will retry next poll)",
            debate.post_id
          );
          continue;
        }
      };

      let context_clone = context.clone();
      let stats_clone = stats.clone();
      let post_id = debate.post_id;

      tokio::spawn(async move {
        let _permit = permit; // Hold permit until task completes

        let start_time = std::time::Instant::now();

        match process_debate(context_clone, debate).await {
          Ok(_) => {
            let processing_time_ms = start_time.elapsed().as_millis() as u64;
            let mut stats = stats_clone.lock().await;
            stats.record_success(processing_time_ms);
            info!(
              "Debate for post {} completed in {}ms",
              post_id, processing_time_ms
            );
          }
          Err(e) => {
            let mut stats = stats_clone.lock().await;
            stats.record_failure();
            error!("Debate for post {} failed: {}", post_id, e);
          }
        }
      });
    }

    // Log comprehensive stats periodically (every 10 polls)
    let stats_snapshot = stats.lock().await.clone();
    if stats_snapshot.total_polls % 10 == 0 {
      info!(
        "Worker stats: polls={}, active={}, processed={}, failed={}, success_rate={:.1}%, avg_time={}ms",
        stats_snapshot.total_polls,
        stats_snapshot.active_debates,
        stats_snapshot.debates_processed,
        stats_snapshot.debates_failed,
        stats_snapshot.success_rate(),
        stats_snapshot.average_processing_time_ms()
      );
    }
  }
}

/// Process a single debate
///
/// This function handles the processing of one debate round. It creates an orchestrator
/// and executes the next round for the debate.
///
/// # Arguments
/// * `context` - The debate context
/// * `debate` - The debate state to process
///
/// # Errors
/// Returns error if debate processing fails
async fn process_debate(context: DebateContext, debate: DebateState) -> LemmyResult<()> {
  info!(
    "Processing debate for post {} (status: {}, round: {}/{})",
    debate.post_id, debate.status, debate.current_round, debate.total_rounds
  );

  // Create orchestrator for this debate
  let client = OpenRouterClient::new(
    context.settings().debate.as_ref()
      .and_then(|d| d.openrouter_api_key.clone())
      .unwrap_or_default(),
    context.settings().debate.as_ref()
      .and_then(|d| d.openrouter_base_url.clone()),
  );
  
  let orchestrator = DebateOrchestrator::new(client);

  // Execute round
  orchestrator
    .execute_round(&mut context.pool().clone(), debate.post_id)
    .await?;

  info!(
    "Successfully processed debate round for post {}",
    debate.post_id
  );

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_worker_stats() {
    let mut stats = WorkerStats::new();
    assert_eq!(stats.debates_processed, 0);
    assert_eq!(stats.debates_failed, 0);
    assert_eq!(stats.average_processing_time_ms(), 0);
    assert_eq!(stats.success_rate(), 0.0);

    stats.record_success(100);
    assert_eq!(stats.debates_processed, 1);
    assert_eq!(stats.average_processing_time_ms(), 100);
    assert_eq!(stats.success_rate(), 100.0);

    stats.record_success(200);
    assert_eq!(stats.debates_processed, 2);
    assert_eq!(stats.average_processing_time_ms(), 150);
    assert_eq!(stats.success_rate(), 100.0);

    stats.record_failure();
    assert_eq!(stats.debates_failed, 1);
    assert!((stats.success_rate() - 66.66).abs() < 0.1);

    stats.set_active_debates(5);
    assert_eq!(stats.active_debates, 5);

    stats.record_poll();
    assert_eq!(stats.total_polls, 1);
    assert!(stats.last_poll_time.is_some());
  }
}
