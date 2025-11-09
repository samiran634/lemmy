use chrono::{DateTime, Utc};
use lemmy_db_schema::newtypes::{CommentId, PersonId, PostId};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use diesel::prelude::*;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::{ai_agent, debate_config, debate_round, debate_state};

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The style of debate to conduct
pub enum DebateStyle {
  Formal,
  Casual,
  Adversarial,
  Collaborative,
  Socratic,
}

impl DebateStyle {
  pub fn as_str(&self) -> &'static str {
    match self {
      DebateStyle::Formal => "formal",
      DebateStyle::Casual => "casual",
      DebateStyle::Adversarial => "adversarial",
      DebateStyle::Collaborative => "collaborative",
      DebateStyle::Socratic => "socratic",
    }
  }

  pub fn from_str(s: &str) -> Option<Self> {
    match s {
      "formal" => Some(DebateStyle::Formal),
      "casual" => Some(DebateStyle::Casual),
      "adversarial" => Some(DebateStyle::Adversarial),
      "collaborative" => Some(DebateStyle::Collaborative),
      "socratic" => Some(DebateStyle::Socratic),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The current status of a debate
pub enum DebateStatus {
  Pending,
  Active,
  Paused,
  Completed,
  Error,
}

impl DebateStatus {
  pub fn as_str(&self) -> &'static str {
    match self {
      DebateStatus::Pending => "pending",
      DebateStatus::Active => "active",
      DebateStatus::Paused => "paused",
      DebateStatus::Completed => "completed",
      DebateStatus::Error => "error",
    }
  }

  pub fn from_str(s: &str) -> Option<Self> {
    match s {
      "pending" => Some(DebateStatus::Pending),
      "active" => Some(DebateStatus::Active),
      "paused" => Some(DebateStatus::Paused),
      "completed" => Some(DebateStatus::Completed),
      "error" => Some(DebateStatus::Error),
      _ => None,
    }
  }
}

// ============================================================================
// AI Agent
// ============================================================================

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = ai_agent))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// An AI agent that participates in debates
pub struct AiAgent {
  pub id: i32,
  pub person_id: PersonId,
  pub model_identifier: String,
  pub display_name: String,
  pub provider: String,
  pub model_version: Option<String>,
  pub avatar_url: Option<String>,
  pub is_active: bool,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = ai_agent))]
pub struct AiAgentInsertForm {
  pub person_id: PersonId,
  pub model_identifier: String,
  pub display_name: String,
  pub provider: String,
  #[new(default)]
  pub model_version: Option<String>,
  #[new(default)]
  pub avatar_url: Option<String>,
  #[new(default)]
  pub is_active: bool,
}

impl AiAgentInsertForm {
  pub fn with_model_version(mut self, model_version: Option<String>) -> Self {
    self.model_version = model_version;
    self
  }

  pub fn with_avatar_url(mut self, avatar_url: Option<String>) -> Self {
    self.avatar_url = avatar_url;
    self
  }

  pub fn with_is_active(mut self, is_active: bool) -> Self {
    self.is_active = is_active;
    self
  }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = ai_agent))]
pub struct AiAgentUpdateForm {
  pub display_name: Option<String>,
  pub provider: Option<String>,
  pub model_version: Option<Option<String>>,
  pub avatar_url: Option<Option<String>>,
  pub is_active: Option<bool>,
}

// ============================================================================
// Debate Config
// ============================================================================

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = debate_config))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Configuration for a debate
pub struct DebateConfig {
  pub id: i32,
  pub post_id: PostId,
  pub creator_id: PersonId,
  pub ai_models: Vec<String>,
  pub debate_style: String,
  pub max_rounds: i32,
  pub max_tokens_per_response: Option<i32>,
  pub custom_system_prompt: Option<String>,
  pub include_human_comments: bool,
  pub created_at: DateTime<Utc>,
}

impl DebateConfig {
  pub fn get_debate_style(&self) -> Option<DebateStyle> {
    DebateStyle::from_str(&self.debate_style)
  }
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = debate_config))]
pub struct DebateConfigInsertForm {
  pub post_id: PostId,
  pub creator_id: PersonId,
  pub ai_models: Vec<String>,
  pub debate_style: String,
  pub max_rounds: i32,
  #[new(default)]
  pub max_tokens_per_response: Option<i32>,
  #[new(default)]
  pub custom_system_prompt: Option<String>,
  #[new(default)]
  pub include_human_comments: bool,
}

impl DebateConfigInsertForm {
  pub fn with_max_tokens_per_response(mut self, max_tokens: Option<i32>) -> Self {
    self.max_tokens_per_response = max_tokens;
    self
  }

  pub fn with_custom_system_prompt(mut self, prompt: Option<String>) -> Self {
    self.custom_system_prompt = prompt;
    self
  }

  pub fn with_include_human_comments(mut self, include: bool) -> Self {
    self.include_human_comments = include;
    self
  }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = debate_config))]
pub struct DebateConfigUpdateForm {
  pub ai_models: Option<Vec<String>>,
  pub debate_style: Option<String>,
  pub max_rounds: Option<i32>,
  pub max_tokens_per_response: Option<Option<i32>>,
  pub custom_system_prompt: Option<Option<String>>,
  pub include_human_comments: Option<bool>,
}

// ============================================================================
// Debate State
// ============================================================================

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = debate_state))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The current state of a debate
pub struct DebateState {
  pub id: i32,
  pub post_id: PostId,
  pub status: String,
  pub current_round: i32,
  pub total_rounds: i32,
  pub last_activity_at: DateTime<Utc>,
  pub error_message: Option<String>,
  pub metadata: Option<serde_json::Value>,
}

impl DebateState {
  pub fn get_status(&self) -> Option<DebateStatus> {
    DebateStatus::from_str(&self.status)
  }
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = debate_state))]
pub struct DebateStateInsertForm {
  pub post_id: PostId,
  pub status: String,
  pub total_rounds: i32,
  #[new(default)]
  pub current_round: i32,
  #[new(default)]
  pub error_message: Option<String>,
  #[new(default)]
  pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = debate_state))]
pub struct DebateStateUpdateForm {
  pub status: Option<String>,
  pub current_round: Option<i32>,
  pub total_rounds: Option<i32>,
  pub last_activity_at: Option<DateTime<Utc>>,
  pub error_message: Option<Option<String>>,
  pub metadata: Option<Option<serde_json::Value>>,
}

// ============================================================================
// Debate Round
// ============================================================================

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = debate_round))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A single round of debate by one AI agent
pub struct DebateRound {
  pub id: i32,
  pub post_id: PostId,
  pub round_number: i32,
  pub ai_agent_id: i32,
  pub comment_id: CommentId,
  pub tokens_used: Option<i32>,
  pub api_latency_ms: Option<i32>,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = debate_round))]
pub struct DebateRoundInsertForm {
  pub post_id: PostId,
  pub round_number: i32,
  pub ai_agent_id: i32,
  pub comment_id: CommentId,
  #[new(default)]
  pub tokens_used: Option<i32>,
  #[new(default)]
  pub api_latency_ms: Option<i32>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = debate_round))]
pub struct DebateRoundUpdateForm {
  pub tokens_used: Option<Option<i32>>,
  pub api_latency_ms: Option<Option<i32>>,
}
