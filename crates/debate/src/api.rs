use crate::{DebateStatus, DebateStyle};
use lemmy_db_schema::newtypes::PostId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// ============================================================================
// Create Debate Request/Response
// ============================================================================

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Request to create a new debate
pub struct CreateDebateRequest {
  pub post_id: PostId,
  pub ai_models: Option<Vec<String>>,
  pub debate_style: Option<DebateStyle>,
  pub max_rounds: Option<i32>,
  pub max_tokens_per_response: Option<i32>,
  pub custom_system_prompt: Option<String>,
  pub include_human_comments: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Response after creating a debate
pub struct CreateDebateResponse {
  pub debate_id: i32,
  pub status: DebateStatus,
  pub message: String,
}

// ============================================================================
// Debate Status Request/Response
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Information about a participating AI agent
pub struct ParticipatingAgent {
  pub model: String,
  pub display_name: String,
  pub comment_count: i64,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response with debate status information
pub struct DebateStatusResponse {
  pub post_id: PostId,
  pub status: DebateStatus,
  pub current_round: i32,
  pub total_rounds: i32,
  pub participating_agents: Vec<ParticipatingAgent>,
  pub last_activity: chrono::DateTime<chrono::Utc>,
  pub error_message: Option<String>,
}

// ============================================================================
// Debate Metadata (for embedding in post responses)
// ============================================================================

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Debate metadata to include in post responses
pub struct DebateMetadata {
  pub status: DebateStatus,
  pub current_round: i32,
  pub total_rounds: i32,
  pub participating_agents: Vec<ParticipatingAgent>,
  pub can_control: bool,
}

// ============================================================================
// Debate Control Request/Response
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Actions that can be performed on a debate
pub enum DebateControlAction {
  Pause,
  Resume,
  Stop,
  AddRounds,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Request to control a debate
pub struct DebateControlRequest {
  pub action: DebateControlAction,
  pub additional_rounds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Response after controlling a debate
pub struct DebateControlResponse {
  pub status: DebateStatus,
  pub message: String,
}
