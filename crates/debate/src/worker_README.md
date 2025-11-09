# Debate Worker Implementation

## Overview

The debate worker is a background task that continuously polls for active debates and processes them asynchronously. It runs independently of the HTTP server and ensures debates progress through their rounds automatically.

## Architecture

### Components

1. **Worker Task** (`debate_worker_task`)
   - Main async function that runs indefinitely
   - Polls database at configurable intervals
   - Spawns concurrent tasks for debate processing
   - Handles graceful shutdown

2. **Worker Statistics** (`WorkerStats`)
   - Tracks processing metrics
   - Monitors active debates count
   - Calculates success rates and average processing times
   - Provides health check data

3. **Process Debate** (`process_debate`)
   - Handles individual debate round execution
   - Creates orchestrator with OpenRouter client
   - Executes debate rounds
   - Records metrics

## Configuration

The worker is configured through the `debate` section in Lemmy settings:

```hjson
{
  debate: {
    openrouter_api_key: "sk-or-v1-..."
    openrouter_base_url: "https://openrouter.ai/api/v1"  // optional
    max_concurrent_debates: 5
    poll_interval_seconds: 10
  }
}
```

### Configuration Options

- `openrouter_api_key`: Required API key for OpenRouter
- `openrouter_base_url`: Optional custom base URL (defaults to OpenRouter)
- `max_concurrent_debates`: Maximum number of debates to process simultaneously (default: 5)
- `poll_interval_seconds`: How often to poll for active debates (default: 10)

## Integration

### Server Startup

The worker is automatically started during server initialization if the debate configuration is present:

```rust
// In crates/server/src/lib.rs
let debate_worker_task = if SETTINGS.debate.is_some() {
  Some(tokio::task::spawn(lemmy_debate::debate_worker_task(context.clone())))
} else {
  None
};
```

### Graceful Shutdown

The worker is properly shut down when the server receives a shutdown signal:

```rust
if let Some(debate_worker) = debate_worker_task {
  debate_worker.abort();
}
```

## Monitoring

### Worker Statistics

The worker tracks comprehensive statistics accessible via `get_worker_stats()`:

- `debates_processed`: Total successful debate rounds processed
- `debates_failed`: Total failed debate rounds
- `active_debates`: Current number of active debates
- `total_polls`: Total number of database polls
- `last_poll_time`: Timestamp of last poll
- `total_processing_time_ms`: Cumulative processing time
- `average_processing_time_ms()`: Average time per debate
- `success_rate()`: Percentage of successful debates
- `seconds_since_last_poll()`: Time since last poll

### Logging

The worker provides comprehensive logging:

- **INFO**: Worker startup, debate processing, periodic stats
- **DEBUG**: Poll events, individual debate completions
- **WARN**: Concurrency limits reached
- **ERROR**: Database query failures, debate processing errors

Example log output:
```
INFO Starting debate worker task
INFO Debate worker configured: poll_interval=10s, max_concurrent=5
INFO Found 3 active debate(s) to process
INFO Debate for post 123 completed in 2500ms
INFO Worker stats: polls=10, active=2, processed=15, failed=1, success_rate=93.8%, avg_time=2300ms
```

## Concurrency Control

The worker uses a semaphore to limit concurrent debate processing:

1. Maximum concurrent debates is configurable
2. If limit is reached, debates are skipped and retried on next poll
3. Each debate holds a permit until processing completes
4. Prevents resource exhaustion and API rate limiting

## Error Handling

The worker is designed to be resilient:

1. **Database Errors**: Logged and skipped, worker continues
2. **Processing Errors**: Recorded in stats, worker continues
3. **API Failures**: Handled by orchestrator, marked in debate state
4. **Worker Crash**: Would need to be restarted by process manager

## Health Checks

To implement a health check endpoint, use `get_worker_stats()`:

```rust
use lemmy_debate::get_worker_stats;

async fn debate_health_check() -> Result<Json<WorkerStats>, Error> {
  let stats = get_worker_stats().await;
  
  // Check if worker is healthy
  if let Some(seconds) = stats.seconds_since_last_poll() {
    if seconds > 60 {
      // Worker may be stuck
      return Err(Error::WorkerUnhealthy);
    }
  }
  
  Ok(Json(stats))
}
```

## Performance Considerations

1. **Poll Interval**: Lower values provide faster response but increase database load
2. **Concurrency Limit**: Higher values process more debates but increase API usage
3. **Database Queries**: Worker queries only pending/active debates (indexed)
4. **Memory Usage**: Each concurrent debate spawns a tokio task

## Testing

The worker includes unit tests for statistics tracking:

```bash
cargo test --package lemmy_debate worker
```

## Future Enhancements

Potential improvements for the worker:

1. Dynamic poll interval based on active debates
2. Priority queue for urgent debates
3. Retry logic for failed debates
4. Metrics export to Prometheus
5. Admin API for worker control (pause/resume)
6. Distributed worker support for horizontal scaling
