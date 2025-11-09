use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::{
  newtypes::PostId,
  source::post::Post,
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_debate::{
  CreateDebateRequest,
  CreateDebateResponse,
  DebateConfig,
  DebateConfigInsertForm,
  DebateState,
  DebateStateInsertForm,
  DebateStatus,
  DebateStyle,
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn create_debate(
  data: Json<CreateDebateRequest>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CreateDebateResponse>> {
  check_local_user_valid(&local_user_view)?;

  // Verify the post exists and user has permission
  let post = Post::read(&mut context.pool(), data.post_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPost)?;

  // Check if debate already exists for this post
  let existing_config = DebateConfig::read_by_post_id(&mut context.pool(), data.post_id).await;
  if existing_config.is_ok() {
    Err(LemmyErrorType::DebateAlreadyExists)?;
  }

  // Validate and set defaults
  let ai_models = data.ai_models.clone().unwrap_or_else(|| {
    vec![
      "openai/gpt-4-turbo".to_string(),
      "anthropic/claude-3-opus".to_string(),
      "google/gemini-pro".to_string(),
    ]
  });

  // Validate at least 2 models
  if ai_models.len() < 2 {
    Err(LemmyErrorType::DebateNeedsTwoModels)?;
  }

  let debate_style = data
    .debate_style
    .clone()
    .unwrap_or(DebateStyle::Casual)
    .as_str()
    .to_string();

  let max_rounds = data.max_rounds.unwrap_or(3);
  if max_rounds < 1 || max_rounds > 10 {
    Err(LemmyErrorType::DebateInvalidRounds)?;
  }

  let include_human_comments = data.include_human_comments.unwrap_or(true);

  // Create debate config
  let config_form = DebateConfigInsertForm::new(
    data.post_id,
    local_user_view.person.id,
    ai_models,
    debate_style,
    max_rounds,
  )
  .with_max_tokens_per_response(data.max_tokens_per_response)
  .with_custom_system_prompt(data.custom_system_prompt.clone())
  .with_include_human_comments(include_human_comments);

  let config = DebateConfig::create(&mut context.pool(), &config_form).await?;

  // Create debate state
  let state_form = DebateStateInsertForm::new(
    data.post_id,
    DebateStatus::Pending.as_str().to_string(),
    max_rounds,
  );

  let state = DebateState::create(&mut context.pool(), &state_form).await?;

  Ok(Json(CreateDebateResponse {
    debate_id: config.id,
    status: state.get_status().unwrap_or(DebateStatus::Pending),
    message: "Debate created successfully. It will start shortly.".to_string(),
  }))
}
