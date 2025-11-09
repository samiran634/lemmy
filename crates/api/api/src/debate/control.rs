use actix_web::web::{Data, Json, Path};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::newtypes::PostId;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_debate::{
  DebateConfig,
  DebateControlAction,
  DebateControlRequest,
  DebateControlResponse,
  DebateState,
  DebateStateUpdateForm,
  DebateStatus,
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn control_debate(
  post_id: Path<PostId>,
  data: Json<DebateControlRequest>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<DebateControlResponse>> {
  check_local_user_valid(&local_user_view)?;

  let post_id = post_id.into_inner();

  // Get debate config to verify ownership
  let config = DebateConfig::read_by_post_id(&mut context.pool(), post_id)
    .await
    .map_err(|_| LemmyErrorType::DebateNotFound)?;

  // Verify user is the debate creator
  if config.creator_id != local_user_view.person.id {
    Err(LemmyErrorType::NotPostCreator)?;
  }

  // Get current state
  let state = DebateState::read_by_post_id(&mut context.pool(), post_id)
    .await
    .map_err(|_| LemmyErrorType::DebateNotFound)?;

  let current_status = state.get_status().unwrap_or(DebateStatus::Pending);

  // Process the control action
  let (new_status, message) = match data.action {
    DebateControlAction::Pause => {
      if current_status != DebateStatus::Active {
        Err(LemmyErrorType::DebateNotActive)?;
      }
      (
        DebateStatus::Paused,
        "Debate paused successfully".to_string(),
      )
    }
    DebateControlAction::Resume => {
      if current_status != DebateStatus::Paused {
        Err(LemmyErrorType::DebateNotPaused)?;
      }
      (
        DebateStatus::Active,
        "Debate resumed successfully".to_string(),
      )
    }
    DebateControlAction::Stop => {
      if current_status == DebateStatus::Completed {
        Err(LemmyErrorType::DebateAlreadyCompleted)?;
      }
      (
        DebateStatus::Completed,
        "Debate stopped successfully".to_string(),
      )
    }
    DebateControlAction::AddRounds => {
      let additional_rounds = data
        .additional_rounds
        .ok_or(LemmyErrorType::DebateMissingAdditionalRounds)?;

      if additional_rounds < 1 || additional_rounds > 10 {
        Err(LemmyErrorType::DebateInvalidRounds)?;
      }

      let new_total = state.total_rounds + additional_rounds;
      if new_total > 20 {
        Err(LemmyErrorType::DebateTooManyRounds)?;
      }

      // Update total rounds
      let update_form = DebateStateUpdateForm {
        total_rounds: Some(new_total),
        ..Default::default()
      };

      DebateState::update(&mut context.pool(), state.id, &update_form).await?;

      return Ok(Json(DebateControlResponse {
        status: current_status,
        message: format!("Added {} rounds. Total rounds: {}", additional_rounds, new_total),
      }));
    }
  };

  // Update state
  let update_form = DebateStateUpdateForm {
    status: Some(new_status.as_str().to_string()),
    ..Default::default()
  };

  DebateState::update(&mut context.pool(), state.id, &update_form).await?;

  Ok(Json(DebateControlResponse {
    status: new_status,
    message,
  }))
}
