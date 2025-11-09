use crate::models::*;
use diesel::{
  dsl::insert_into,
  result::Error,
  ExpressionMethods,
  QueryDsl,
  update,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::{PersonId, PostId};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  traits::Crud,
};
use lemmy_db_schema_file::schema::{ai_agent, debate_config, debate_round, debate_state};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

// ============================================================================
// AI Agent CRUD
// ============================================================================

impl Crud for AiAgent {
  type InsertForm = AiAgentInsertForm;
  type UpdateForm = AiAgentUpdateForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(ai_agent::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    update(ai_agent::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  async fn read(pool: &mut DbPool<'_>, id: Self::IdType) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    ai_agent::table
      .find(id)
      .first::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl AiAgent {
  /// Read an AI agent by model identifier (returns error if not found)
  pub async fn read_by_model_identifier(
    pool: &mut DbPool<'_>,
    model_id: &str,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    ai_agent::table
      .filter(ai_agent::model_identifier.eq(model_id))
      .first::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Find an AI agent by model identifier
  pub async fn find_by_model_identifier(
    pool: &mut DbPool<'_>,
    model_id: &str,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    ai_agent::table
      .filter(ai_agent::model_identifier.eq(model_id))
      .first::<Self>(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Find an AI agent by person_id
  pub async fn find_by_person_id(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    ai_agent::table
      .filter(ai_agent::person_id.eq(person_id))
      .first::<Self>(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// List all active AI agents
  pub async fn list_active(pool: &mut DbPool<'_>) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    ai_agent::table
      .filter(ai_agent::is_active.eq(true))
      .order(ai_agent::created_at.asc())
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// List all AI agents
  pub async fn list_all(pool: &mut DbPool<'_>) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    ai_agent::table
      .order(ai_agent::created_at.asc())
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

// ============================================================================
// Debate Config CRUD
// ============================================================================

impl Crud for DebateConfig {
  type InsertForm = DebateConfigInsertForm;
  type UpdateForm = DebateConfigUpdateForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(debate_config::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    update(debate_config::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  async fn read(pool: &mut DbPool<'_>, id: Self::IdType) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    debate_config::table
      .find(id)
      .first::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl DebateConfig {
  /// Read debate config by post_id (returns error if not found)
  pub async fn read_by_post_id(
    pool: &mut DbPool<'_>,
    post_id: PostId,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    debate_config::table
      .filter(debate_config::post_id.eq(post_id))
      .first::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Find debate config by post_id
  pub async fn find_by_post_id(
    pool: &mut DbPool<'_>,
    post_id: PostId,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    debate_config::table
      .filter(debate_config::post_id.eq(post_id))
      .first::<Self>(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Delete debate config by post_id
  pub async fn delete_by_post_id(pool: &mut DbPool<'_>, post_id: PostId) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(debate_config::table.filter(debate_config::post_id.eq(post_id)))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntDelete)
  }
}

// ============================================================================
// Debate State CRUD
// ============================================================================

impl Crud for DebateState {
  type InsertForm = DebateStateInsertForm;
  type UpdateForm = DebateStateUpdateForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(debate_state::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    update(debate_state::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  async fn read(pool: &mut DbPool<'_>, id: Self::IdType) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    debate_state::table
      .find(id)
      .first::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl DebateState {
  /// Read debate state by post_id (returns error if not found)
  pub async fn read_by_post_id(
    pool: &mut DbPool<'_>,
    post_id: PostId,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    debate_state::table
      .filter(debate_state::post_id.eq(post_id))
      .first::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Find debate state by post_id
  pub async fn find_by_post_id(
    pool: &mut DbPool<'_>,
    post_id: PostId,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    debate_state::table
      .filter(debate_state::post_id.eq(post_id))
      .first::<Self>(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Find all active debates (pending or active status)
  pub async fn find_active_debates(pool: &mut DbPool<'_>) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    debate_state::table
      .filter(
        debate_state::status
          .eq("pending")
          .or(debate_state::status.eq("active")),
      )
      .order(debate_state::last_activity_at.asc())
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Find debates by status
  pub async fn find_by_status(
    pool: &mut DbPool<'_>,
    status: &str,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    debate_state::table
      .filter(debate_state::status.eq(status))
      .order(debate_state::last_activity_at.desc())
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Update debate state by post_id
  pub async fn update_by_post_id(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    form: &DebateStateUpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    update(debate_state::table.filter(debate_state::post_id.eq(post_id)))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  /// Delete debate state by post_id
  pub async fn delete_by_post_id(pool: &mut DbPool<'_>, post_id: PostId) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(debate_state::table.filter(debate_state::post_id.eq(post_id)))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntDelete)
  }
}

// ============================================================================
// Debate Round CRUD
// ============================================================================

impl Crud for DebateRound {
  type InsertForm = DebateRoundInsertForm;
  type UpdateForm = DebateRoundUpdateForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(debate_round::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    update(debate_round::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  async fn read(pool: &mut DbPool<'_>, id: Self::IdType) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    debate_round::table
      .find(id)
      .first::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl DebateRound {
  /// List all rounds for a post
  pub async fn list_by_post_id(
    pool: &mut DbPool<'_>,
    post_id: PostId,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    debate_round::table
      .filter(debate_round::post_id.eq(post_id))
      .order(debate_round::round_number.asc())
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Find all rounds for a post
  pub async fn find_by_post_id(
    pool: &mut DbPool<'_>,
    post_id: PostId,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    debate_round::table
      .filter(debate_round::post_id.eq(post_id))
      .order(debate_round::round_number.asc())
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Find rounds for a specific round number
  pub async fn find_by_post_and_round(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    round_number: i32,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    debate_round::table
      .filter(debate_round::post_id.eq(post_id))
      .filter(debate_round::round_number.eq(round_number))
      .order(debate_round::created_at.asc())
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Find all rounds by an AI agent
  pub async fn find_by_agent(
    pool: &mut DbPool<'_>,
    ai_agent_id: i32,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    debate_round::table
      .filter(debate_round::ai_agent_id.eq(ai_agent_id))
      .order(debate_round::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Delete all rounds for a post
  pub async fn delete_by_post_id(pool: &mut DbPool<'_>, post_id: PostId) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(debate_round::table.filter(debate_round::post_id.eq(post_id)))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntDelete)
  }
}

// ============================================================================
// Helper functions for loading debate metadata
// ============================================================================

use crate::api::{DebateMetadata, ParticipatingAgent};
use std::collections::HashMap;

/// Load debate metadata for a single post
pub async fn load_debate_metadata_for_post(
  pool: &mut DbPool<'_>,
  post_id: PostId,
  viewer_person_id: Option<PersonId>,
) -> LemmyResult<Option<DebateMetadata>> {
  // Try to get debate config - if it doesn't exist, this post is not a debate
  let config = match DebateConfig::read_by_post_id(pool, post_id).await {
    Ok(c) => c,
    Err(_) => return Ok(None),
  };

  // Get debate state
  let state = match DebateState::read_by_post_id(pool, post_id).await {
    Ok(s) => s,
    Err(_) => return Ok(None),
  };

  // Get all rounds for this debate
  let rounds = DebateRound::list_by_post_id(pool, post_id).await?;

  // Count comments per agent
  let mut agent_comment_counts: HashMap<String, i64> = HashMap::new();
  for round in &rounds {
    let agent = AiAgent::read(pool, round.ai_agent_id).await?;
    *agent_comment_counts
      .entry(agent.model_identifier.clone())
      .or_insert(0) += 1;
  }

  // Build participating agents list
  let mut participating_agents = Vec::new();
  for model in &config.ai_models {
    let comment_count = agent_comment_counts.get(model).copied().unwrap_or(0);

    // Try to get display name from agent if it exists
    let display_name = if let Ok(agent) = AiAgent::read_by_model_identifier(pool, model).await {
      agent.display_name
    } else {
      model.clone()
    };

    participating_agents.push(ParticipatingAgent {
      model: model.clone(),
      display_name,
      comment_count,
    });
  }

  // Determine if viewer can control this debate
  let can_control = viewer_person_id.map_or(false, |pid| pid == config.creator_id);

  Ok(Some(DebateMetadata {
    status: state.get_status().unwrap_or(DebateStatus::Pending),
    current_round: state.current_round,
    total_rounds: state.total_rounds,
    participating_agents,
    can_control,
  }))
}

/// Load debate metadata for multiple posts efficiently
pub async fn load_debate_metadata_for_posts(
  pool: &mut DbPool<'_>,
  post_ids: &[PostId],
  viewer_person_id: Option<PersonId>,
) -> LemmyResult<HashMap<PostId, DebateMetadata>> {
  let mut result = HashMap::new();

  // For now, load each post's metadata individually
  // TODO: Optimize with batch queries if needed
  for &post_id in post_ids {
    if let Some(metadata) = load_debate_metadata_for_post(pool, post_id, viewer_person_id).await? {
      result.insert(post_id, metadata);
    }
  }

  Ok(result)
}


/// Enrich a PostView with debate metadata
pub async fn enrich_post_view_with_debate(
  pool: &mut DbPool<'_>,
  mut post_view: lemmy_db_views_post::PostView,
  viewer_person_id: Option<PersonId>,
) -> LemmyResult<lemmy_db_views_post::PostView> {
  post_view.debate_metadata =
    load_debate_metadata_for_post(pool, post_view.post.id, viewer_person_id).await?;
  Ok(post_view)
}

/// Enrich multiple PostViews with debate metadata
pub async fn enrich_post_views_with_debate(
  pool: &mut DbPool<'_>,
  mut post_views: Vec<lemmy_db_views_post::PostView>,
  viewer_person_id: Option<PersonId>,
) -> LemmyResult<Vec<lemmy_db_views_post::PostView>> {
  let post_ids: Vec<PostId> = post_views.iter().map(|pv| pv.post.id).collect();
  let metadata_map = load_debate_metadata_for_posts(pool, &post_ids, viewer_person_id).await?;

  for post_view in &mut post_views {
    post_view.debate_metadata = metadata_map.get(&post_view.post.id).cloned();
  }

  Ok(post_views)
}
