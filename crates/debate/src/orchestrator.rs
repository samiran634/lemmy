//! Debate orchestration engine
//!
//! This module provides the core orchestration logic for AI debates. The `DebateOrchestrator`
//! manages the entire lifecycle of a debate, from initialization to completion, including:
//!
//! - Starting new debates
//! - Executing debate rounds
//! - Managing debate state transitions
//! - Creating comments from AI responses
//! - Error handling and recovery
//!
//! # Architecture
//!
//! The orchestrator coordinates between multiple components:
//! - Database (for state persistence)
//! - OpenRouter API client (for AI responses)
//! - Prompt builder (for context construction)
//! - Agent manager (for AI agent management)
//!
//! # Example
//!
//! ```rust,ignore
//! use lemmy_debate::DebateOrchestrator;
//!
//! let orchestrator = DebateOrchestrator::new(pool, openrouter_client);
//! orchestrator.start_debate(post_id).await?;
//! orchestrator.execute_round(post_id).await?;
//! ```

use crate::{
  agent_manager::AgentManager,
  metrics::{DebateMetrics, Timer},
  models::{
    AiAgent,
    DebateConfig,
    DebateRoundInsertForm,
    DebateState,
    DebateStateInsertForm,
    DebateStateUpdateForm,
    DebateStatus,
  },
  openrouter::{Message, OpenRouterClient, OpenRouterRequest},
  prompt_builder::{context_helpers, PromptBuilder},
};
use chrono::Utc;
use lemmy_db_schema::{
  newtypes::{CommentId, LanguageId, PersonId, PostId},
  source::{
    comment::{Comment, CommentInsertForm},
    post::Post,
  },
  traits::ApubActor,
};
use lemmy_diesel_utils::{connection::DbPool, traits::Crud};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use std::collections::{HashMap, HashSet};
use tracing::{debug, error, info, warn};

/// Main orchestrator for managing AI debates
///
/// The `DebateOrchestrator` is responsible for coordinating all aspects of a debate,
/// including state management, AI agent coordination, and comment creation.
#[derive(Clone)]
pub struct DebateOrchestrator {
  openrouter_client: OpenRouterClient,
  settings: Settings,
  metrics: DebateMetrics,
}

impl DebateOrchestrator {
  /// Create a new DebateOrchestrator
  ///
  /// # Arguments
  /// * `openrouter_client` - Client for communicating with OpenRouter API
  /// * `settings` - Lemmy settings for URL generation and configuration
  pub fn new(openrouter_client: OpenRouterClient, settings: Settings) -> Self {
    Self {
      openrouter_client,
      settings,
      metrics: DebateMetrics::new(),
    }
  }

  /// Get metrics for monitoring
  pub fn metrics(&self) -> &DebateMetrics {
    &self.metrics
  }

  /// Initialize a new debate from a post
  ///
  /// This method:
  /// 1. Validates that a debate config exists for the post
  /// 2. Creates initial debate state with "pending" status
  /// 3. Validates that all configured AI models have agents
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `post_id` - ID of the post to start debate on
  ///
  /// # Errors
  /// Returns error if:
  /// - No debate config exists for the post
  /// - Debate state already exists
  /// - AI agents cannot be created
  pub async fn start_debate(&self, pool: &mut DbPool<'_>, post_id: PostId) -> LemmyResult<()> {
    let _timer = Timer::new(format!("start_debate(post_id={})", post_id));
    info!("Starting debate for post {}", post_id);

    // Load debate configuration
    let config = DebateConfig::find_by_post_id(pool, post_id)
      .await?
      .ok_or_else(|| {
        LemmyErrorType::Unknown(format!("No debate config found for post {}", post_id))
      })?;

    // Check if debate state already exists
    if let Some(existing_state) = DebateState::find_by_post_id(pool, post_id).await? {
      warn!(
        "Debate state already exists for post {} with status: {}",
        post_id, existing_state.status
      );
      return Err(
        LemmyErrorType::Unknown(format!(
          "Debate already exists for post {} with status: {}",
          post_id, existing_state.status
        ))
        .into(),
      );
    }

    // Ensure all AI agents exist
    for model_identifier in &config.ai_models {
      AgentManager::get_or_create_agent(pool, model_identifier, &self.settings).await?;
    }

    // Create initial debate state
    let state_form = DebateStateInsertForm::new(
      post_id,
      DebateStatus::Pending.as_str().to_string(),
      config.max_rounds,
    );

    DebateState::create(pool, &state_form).await?;

    self.metrics.increment_active_debates();

    info!(
      "Debate initialized for post {} with {} rounds and {} agents",
      post_id,
      config.max_rounds,
      config.ai_models.len()
    );

    Ok(())
  }

  /// Execute one round of the debate
  ///
  /// This method:
  /// 1. Loads debate configuration and current state
  /// 2. Validates that the debate can proceed
  /// 3. Builds context from previous comments
  /// 4. Iterates through AI agents for the round
  /// 5. Calls OpenRouter API for each agent
  /// 6. Creates comments from AI responses
  /// 7. Updates debate state after round completion
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `post_id` - ID of the post to execute round for
  ///
  /// # Errors
  /// Returns error if:
  /// - Debate config or state not found
  /// - Debate is not in active or pending status
  /// - All rounds are already complete
  pub async fn execute_round(&self, pool: &mut DbPool<'_>, post_id: PostId) -> LemmyResult<()> {
    let _timer = Timer::new(format!("execute_round(post_id={})", post_id));
    info!("Executing debate round for post {}", post_id);

    // Load debate configuration and state
    let config = DebateConfig::find_by_post_id(pool, post_id)
      .await?
      .ok_or_else(|| {
        LemmyErrorType::Unknown(format!("No debate config found for post {}", post_id))
      })?;

    let mut state = DebateState::find_by_post_id(pool, post_id)
      .await?
      .ok_or_else(|| {
        LemmyErrorType::Unknown(format!("No debate state found for post {}", post_id))
      })?;

    // Validate debate can proceed
    let status = state.get_status().ok_or_else(|| {
      LemmyErrorType::Unknown(format!("Invalid debate status: {}", state.status))
    })?;

    match status {
      DebateStatus::Pending => {
        // Transition to active
        let update_form = DebateStateUpdateForm {
          status: Some(DebateStatus::Active.as_str().to_string()),
          current_round: Some(1),
          last_activity_at: Some(Utc::now()),
          ..Default::default()
        };
        state = DebateState::update_by_post_id(pool, post_id, &update_form).await?;
        info!("Debate transitioned to active status");
      }
      DebateStatus::Active => {
        // Check if all rounds are complete
        if state.current_round >= state.total_rounds {
          info!("All rounds complete, marking debate as completed");
          let update_form = DebateStateUpdateForm {
            status: Some(DebateStatus::Completed.as_str().to_string()),
            last_activity_at: Some(Utc::now()),
            ..Default::default()
          };
          DebateState::update_by_post_id(pool, post_id, &update_form).await?;
          return Ok(());
        }
        // Increment round
        let update_form = DebateStateUpdateForm {
          current_round: Some(state.current_round + 1),
          last_activity_at: Some(Utc::now()),
          ..Default::default()
        };
        state = DebateState::update_by_post_id(pool, post_id, &update_form).await?;
      }
      DebateStatus::Paused => {
        return Err(
          LemmyErrorType::Unknown(format!("Debate is paused for post {}", post_id)).into(),
        );
      }
      DebateStatus::Completed => {
        return Err(
          LemmyErrorType::Unknown(format!("Debate is already completed for post {}", post_id))
            .into(),
        );
      }
      DebateStatus::Error => {
        return Err(
          LemmyErrorType::Unknown(format!("Debate is in error state for post {}", post_id)).into(),
        );
      }
    }

    // Load post
    let post = Post::read(pool, post_id).await?;

    // Load AI agents
    let mut agents = Vec::new();
    for model_identifier in &config.ai_models {
      let agent = AgentManager::get_or_create_agent(pool, model_identifier, &self.settings).await?;
      agents.push(agent);
    }

    // Load previous comments for context
    let previous_comments = self.load_debate_comments(pool, post_id).await?;

    // Build context
    let prompt_builder = PromptBuilder::from_config(&config);
    let ai_person_ids = context_helpers::build_ai_person_id_set(&agents);

    // Execute round for each agent
    let mut successful_responses = 0;
    let mut failed_agents = Vec::new();

    for (agent_index, agent) in agents.iter().enumerate() {
      match self
        .execute_agent_turn(
          pool,
          &config,
          &state,
          &post,
          agent,
          agent_index,
          &previous_comments,
          &prompt_builder,
          &ai_person_ids,
        )
        .await
      {
        Ok(_) => {
          successful_responses += 1;
          debug!("Agent {} completed turn successfully", agent.display_name);
        }
        Err(e) => {
          error!(
            "Agent {} failed to complete turn: {}",
            agent.display_name, e
          );
          failed_agents.push(agent.display_name.clone());
          // Continue with other agents
        }
      }
    }

    // Update debate state
    if successful_responses == 0 {
      // All agents failed - mark as error
      error!("All agents failed for round {}", state.current_round);
      let update_form = DebateStateUpdateForm {
        status: Some(DebateStatus::Error.as_str().to_string()),
        error_message: Some(Some(format!(
          "All agents failed in round {}: {}",
          state.current_round,
          failed_agents.join(", ")
        ))),
        last_activity_at: Some(Utc::now()),
        ..Default::default()
      };
      DebateState::update_by_post_id(pool, post_id, &update_form).await?;
      self.metrics.decrement_active_debates();
      self.metrics.increment_failed_debates();
      return Err(
        LemmyErrorType::Unknown(format!(
          "All agents failed in round {}",
          state.current_round
        ))
        .into(),
      );
    }

    // Check if debate is complete
    if state.current_round >= state.total_rounds {
      info!("Debate completed after round {}", state.current_round);
      let update_form = DebateStateUpdateForm {
        status: Some(DebateStatus::Completed.as_str().to_string()),
        last_activity_at: Some(Utc::now()),
        ..Default::default()
      };
      DebateState::update_by_post_id(pool, post_id, &update_form).await?;
      self.metrics.decrement_active_debates();
      self.metrics.increment_completed_debates();
    } else {
      // Update last activity time
      let update_form = DebateStateUpdateForm {
        last_activity_at: Some(Utc::now()),
        ..Default::default()
      };
      DebateState::update_by_post_id(pool, post_id, &update_form).await?;
    }

    self.metrics.increment_rounds_executed();

    info!(
      "Round {} completed for post {} ({}/{} agents successful)",
      state.current_round, post_id, successful_responses, agents.len()
    );

    Ok(())
  }

  /// Pause an ongoing debate
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `post_id` - ID of the post to pause debate for
  pub async fn pause_debate(&self, pool: &mut DbPool<'_>, post_id: PostId) -> LemmyResult<()> {
    info!("Pausing debate for post {}", post_id);

    let state = DebateState::find_by_post_id(pool, post_id)
      .await?
      .ok_or_else(|| {
        LemmyErrorType::Unknown(format!("No debate state found for post {}", post_id))
      })?;

    let status = state.get_status().ok_or_else(|| {
      LemmyErrorType::Unknown(format!("Invalid debate status: {}", state.status))
    })?;

    if status != DebateStatus::Active {
      return Err(
        LemmyErrorType::Unknown(format!(
          "Cannot pause debate in status: {}",
          state.status
        ))
        .into(),
      );
    }

    let update_form = DebateStateUpdateForm {
      status: Some(DebateStatus::Paused.as_str().to_string()),
      last_activity_at: Some(Utc::now()),
      ..Default::default()
    };

    DebateState::update_by_post_id(pool, post_id, &update_form).await?;

    info!("Debate paused for post {}", post_id);
    Ok(())
  }

  /// Resume a paused debate
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `post_id` - ID of the post to resume debate for
  pub async fn resume_debate(&self, pool: &mut DbPool<'_>, post_id: PostId) -> LemmyResult<()> {
    info!("Resuming debate for post {}", post_id);

    let state = DebateState::find_by_post_id(pool, post_id)
      .await?
      .ok_or_else(|| {
        LemmyErrorType::Unknown(format!("No debate state found for post {}", post_id))
      })?;

    let status = state.get_status().ok_or_else(|| {
      LemmyErrorType::Unknown(format!("Invalid debate status: {}", state.status))
    })?;

    if status != DebateStatus::Paused {
      return Err(
        LemmyErrorType::Unknown(format!(
          "Cannot resume debate in status: {}",
          state.status
        ))
        .into(),
      );
    }

    let update_form = DebateStateUpdateForm {
      status: Some(DebateStatus::Active.as_str().to_string()),
      last_activity_at: Some(Utc::now()),
      ..Default::default()
    };

    DebateState::update_by_post_id(pool, post_id, &update_form).await?;

    info!("Debate resumed for post {}", post_id);
    Ok(())
  }

  /// Complete a debate early
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `post_id` - ID of the post to complete debate for
  pub async fn complete_debate(&self, pool: &mut DbPool<'_>, post_id: PostId) -> LemmyResult<()> {
    info!("Completing debate early for post {}", post_id);

    let state = DebateState::find_by_post_id(pool, post_id)
      .await?
      .ok_or_else(|| {
        LemmyErrorType::Unknown(format!("No debate state found for post {}", post_id))
      })?;

    let status = state.get_status().ok_or_else(|| {
      LemmyErrorType::Unknown(format!("Invalid debate status: {}", state.status))
    })?;

    if status == DebateStatus::Completed {
      return Err(
        LemmyErrorType::Unknown(format!("Debate is already completed for post {}", post_id))
          .into(),
      );
    }

    let update_form = DebateStateUpdateForm {
      status: Some(DebateStatus::Completed.as_str().to_string()),
      last_activity_at: Some(Utc::now()),
      ..Default::default()
    };

    DebateState::update_by_post_id(pool, post_id, &update_form).await?;

    info!("Debate completed for post {}", post_id);
    Ok(())
  }

  /// Execute a single agent's turn in the debate
  ///
  /// This is an internal method that handles:
  /// - Building the prompt for the agent
  /// - Calling the OpenRouter API
  /// - Creating a comment from the response
  /// - Recording the debate round
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `config` - Debate configuration
  /// * `state` - Current debate state
  /// * `post` - The post being debated
  /// * `agent` - The AI agent taking its turn
  /// * `agent_index` - Index of the agent in the debate
  /// * `previous_comments` - All previous comments in the debate
  /// * `prompt_builder` - Builder for constructing prompts
  /// * `ai_person_ids` - Set of person IDs that are AI agents
  async fn execute_agent_turn(
    &self,
    pool: &mut DbPool<'_>,
    config: &DebateConfig,
    state: &DebateState,
    post: &Post,
    agent: &AiAgent,
    agent_index: usize,
    previous_comments: &[Comment],
    prompt_builder: &PromptBuilder,
    ai_person_ids: &HashSet<PersonId>,
  ) -> LemmyResult<CommentId> {
    debug!(
      "Executing turn for agent {} in round {}",
      agent.display_name, state.current_round
    );

    // Build system prompt
    let role = if config.get_debate_style() == Some(crate::models::DebateStyle::Adversarial) {
      Some(prompt_builder.assign_adversarial_role(agent_index, config.ai_models.len()))
    } else {
      None
    };

    let system_prompt = prompt_builder.build_system_prompt(agent, role.as_deref());

    // Build conversation context
    let mut messages = prompt_builder.build_context(
      post,
      previous_comments,
      state.current_round,
      state.total_rounds,
      config.include_human_comments,
      ai_person_ids,
    );

    // Add round-specific prompt
    let round_prompt = prompt_builder.build_round_prompt(state.current_round, state.total_rounds);
    messages.push(Message::user(round_prompt));

    // Prepend system message
    messages.insert(0, Message::system(system_prompt));

    // Build OpenRouter request
    let request = OpenRouterRequest {
      model: agent.model_identifier.clone(),
      messages,
      max_tokens: config.max_tokens_per_response.map(|t| t as u32),
      temperature: Some(0.7),
      top_p: None,
      frequency_penalty: None,
      presence_penalty: None,
    };

    // Call OpenRouter API
    let start_time = std::time::Instant::now();
    self.metrics.increment_api_calls();
    let response = match self.openrouter_client.generate_response(request).await {
      Ok(r) => r,
      Err(e) => {
        self.metrics.increment_api_errors();
        return Err(e);
      }
    };
    let api_latency_ms = start_time.elapsed().as_millis() as i32;

    // Validate and extract response text
    response.validate()?;
    let content = response.get_text()?;

    // Track token usage
    if let Some(tokens) = response.get_total_tokens() {
      self.metrics.add_tokens_used(tokens as u64);
    }

    // Create comment from AI response
    let comment_id = self
      .create_comment_from_response(pool, post.id, agent.person_id, &content)
      .await?;

    // Record debate round
    let round_form = DebateRoundInsertForm::new(post.id, state.current_round, agent.id, comment_id)
      .with_tokens_used(response.get_total_tokens().map(|t| t as i32))
      .with_api_latency_ms(Some(api_latency_ms));

    crate::models::DebateRound::create(pool, &round_form).await?;

    debug!(
      "Agent {} completed turn, created comment {}",
      agent.display_name, comment_id
    );

    Ok(comment_id)
  }

  /// Create a comment from an AI response
  ///
  /// This method:
  /// 1. Creates a Comment struct with the AI agent as creator
  /// 2. Sets proper parent-child relationships for threading
  /// 3. Generates AP ID for the comment
  /// 4. Marks the comment as local
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `post_id` - ID of the post to comment on
  /// * `creator_id` - Person ID of the AI agent
  /// * `content` - The comment content
  ///
  /// # Returns
  /// The ID of the created comment
  async fn create_comment_from_response(
    &self,
    pool: &mut DbPool<'_>,
    post_id: PostId,
    creator_id: PersonId,
    content: &str,
  ) -> LemmyResult<CommentId> {
    debug!("Creating comment for post {} by agent {}", post_id, creator_id);

    // Create comment form
    let comment_form = CommentInsertForm::new(creator_id, post_id, content.to_string())
      .with_local(Some(true))
      .with_language_id(Some(LanguageId::default()));

    // Create comment (parent_path is None for top-level comments)
    let comment = Comment::create(pool, &comment_form, None).await?;

    debug!("Created comment {} for post {}", comment.id, post_id);

    Ok(comment.id)
  }

  /// Load all comments for a debate, ordered chronologically
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `post_id` - ID of the post to load comments for
  async fn load_debate_comments(
    &self,
    pool: &mut DbPool<'_>,
    post_id: PostId,
  ) -> LemmyResult<Vec<Comment>> {
    use diesel::prelude::*;
    use lemmy_db_schema_file::schema::comment;
    use lemmy_diesel_utils::connection::get_conn;

    let conn = &mut get_conn(pool).await?;

    comment::table
      .filter(comment::post_id.eq(post_id))
      .filter(comment::deleted.eq(false))
      .filter(comment::removed.eq(false))
      .order(comment::published_at.asc())
      .load::<Comment>(conn)
      .await
      .map_err(|e| LemmyErrorType::Unknown(format!("Failed to load comments: {}", e)).into())
  }
}

impl CommentInsertForm {
  /// Builder method to set local flag
  pub fn with_local(mut self, local: Option<bool>) -> Self {
    self.local = local;
    self
  }

  /// Builder method to set language_id
  pub fn with_language_id(mut self, language_id: Option<LanguageId>) -> Self {
    self.language_id = language_id;
    self
  }
}

impl DebateRoundInsertForm {
  /// Builder method to set tokens_used
  pub fn with_tokens_used(mut self, tokens_used: Option<i32>) -> Self {
    self.tokens_used = tokens_used;
    self
  }

  /// Builder method to set api_latency_ms
  pub fn with_api_latency_ms(mut self, api_latency_ms: Option<i32>) -> Self {
    self.api_latency_ms = api_latency_ms;
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::DebateStyle;

  #[test]
  fn test_orchestrator_creation() {
    let client = OpenRouterClient::new("test-key".to_string(), None).unwrap();
    let settings = Settings::default();
    let orchestrator = DebateOrchestrator::new(client, settings);
    // Verify metrics are initialized
    assert_eq!(orchestrator.metrics().get_active_debates(), 0);
    assert_eq!(orchestrator.metrics().get_completed_debates(), 0);
  }

  #[test]
  fn test_orchestrator_metrics() {
    let client = OpenRouterClient::new("test-key".to_string(), None).unwrap();
    let settings = Settings::default();
    let orchestrator = DebateOrchestrator::new(client, settings);
    
    // Test metrics tracking
    orchestrator.metrics().increment_active_debates();
    assert_eq!(orchestrator.metrics().get_active_debates(), 1);
    
    orchestrator.metrics().increment_completed_debates();
    assert_eq!(orchestrator.metrics().get_completed_debates(), 1);
    
    orchestrator.metrics().decrement_active_debates();
    assert_eq!(orchestrator.metrics().get_active_debates(), 0);
    
    orchestrator.metrics().increment_api_calls();
    assert_eq!(orchestrator.metrics().get_api_calls(), 1);
    
    orchestrator.metrics().add_tokens_used(1000);
    assert_eq!(orchestrator.metrics().get_tokens_used(), 1000);
  }

  #[test]
  fn test_comment_insert_form_builders() {
    use lemmy_db_schema::newtypes::{PersonId, PostId, LanguageId};
    
    let form = CommentInsertForm::new(
      PersonId(1),
      PostId(1),
      "Test content".to_string()
    )
    .with_local(Some(true))
    .with_language_id(Some(LanguageId(0)));
    
    assert_eq!(form.local, Some(true));
    assert_eq!(form.language_id, Some(LanguageId(0)));
  }

  #[test]
  fn test_debate_round_insert_form_builders() {
    use lemmy_db_schema::newtypes::{PostId, CommentId};
    
    let form = DebateRoundInsertForm::new(
      PostId(1),
      1,
      1,
      CommentId(1)
    )
    .with_tokens_used(Some(500))
    .with_api_latency_ms(Some(1500));
    
    assert_eq!(form.tokens_used, Some(500));
    assert_eq!(form.api_latency_ms, Some(1500));
  }
}
