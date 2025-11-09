use actix_web::web::{Data, Json, Path};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::newtypes::PostId;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_debate::{
  DebateConfig,
  DebateRound,
  DebateState,
  DebateStatus,
  DebateStatusResponse,
  ParticipatingAgent,
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use std::collections::HashMap;

pub async fn get_debate_status(
  post_id: Path<PostId>,
  context: Data<LemmyContext>,
  _local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<DebateStatusResponse>> {
  let post_id = post_id.into_inner();

  // Get debate config and state
  let config = DebateConfig::read_by_post_id(&mut context.pool(), post_id)
    .await
    .map_err(|_| LemmyErrorType::DebateNotFound)?;

  let state = DebateState::read_by_post_id(&mut context.pool(), post_id)
    .await
    .map_err(|_| LemmyErrorType::DebateNotFound)?;

  // Get all rounds for this debate
  let rounds = DebateRound::list_by_post_id(&mut context.pool(), post_id).await?;

  // Count comments per agent
  let mut agent_comment_counts: HashMap<String, i64> = HashMap::new();
  for round in &rounds {
    let agent = lemmy_debate::AiAgent::read(&mut context.pool(), round.ai_agent_id).await?;
    *agent_comment_counts
      .entry(agent.model_identifier.clone())
      .or_insert(0) += 1;
  }

  // Build participating agents list
  let mut participating_agents = Vec::new();
  for model in &config.ai_models {
    let comment_count = agent_comment_counts.get(model).copied().unwrap_or(0);
    
    // Try to get display name from agent if it exists
    let display_name = if let Ok(agent) =
      lemmy_debate::AiAgent::read_by_model_identifier(&mut context.pool(), model).await
    {
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

  Ok(Json(DebateStatusResponse {
    post_id,
    status: state.get_status().unwrap_or(DebateStatus::Pending),
    current_round: state.current_round,
    total_rounds: state.total_rounds,
    participating_agents,
    last_activity: state.last_activity_at,
    error_message: state.error_message,
  }))
}
