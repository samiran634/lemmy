use actix_web::{web, HttpResponse};
use lemmy_db_schema::source::local_user::LocalUser;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  LemmyContext,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDebateMetricsResponse {
  pub active_debates: usize,
  pub total_debates_started: u64,
  pub total_debates_completed: u64,
  pub total_debates_failed: u64,
  pub total_rounds_executed: u64,
  pub total_api_calls: u64,
  pub total_api_errors: u64,
  pub total_tokens_used: u64,
  pub api_error_rate_percent: f64,
}

/// Get debate system metrics (admin only)
///
/// Returns comprehensive metrics about the debate system including:
/// - Active debate count
/// - Total debates started/completed/failed
/// - API call statistics
/// - Token usage
///
/// # Authentication
/// Requires admin privileges
pub async fn get_debate_metrics(
  context: web::Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<HttpResponse> {
  // Verify user is admin
  if !local_user_view.local_user.admin {
    return Err(LemmyErrorType::NotAnAdmin.into());
  }

  // Get metrics from orchestrator
  let metrics = context.debate_orchestrator().metrics();

  let response = GetDebateMetricsResponse {
    active_debates: metrics.get_active_debates(),
    total_debates_started: metrics.get_total_debates_started(),
    total_debates_completed: metrics.get_total_debates_completed(),
    total_debates_failed: metrics.get_total_debates_failed(),
    total_rounds_executed: metrics.get_total_rounds_executed(),
    total_api_calls: metrics.get_total_api_calls(),
    total_api_errors: metrics.get_total_api_errors(),
    total_tokens_used: metrics.get_total_tokens_used(),
    api_error_rate_percent: metrics.get_api_error_rate(),
  };

  Ok(HttpResponse::Ok().json(response))
}
