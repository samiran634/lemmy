use crate::models::{AiAgent, AiAgentInsertForm, AiAgentUpdateForm};
use activitypub_federation::http_signatures::generate_actor_keypair;
use lemmy_db_schema::{
  newtypes::InstanceId,
  source::{
    instance::Instance,
    person::{Person, PersonInsertForm},
  },
  traits::ApubActor,
};
use lemmy_diesel_utils::{connection::DbPool, dburl::DbUrl, traits::Crud};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use tracing::{debug, info};
use url::Url;

/// Manages AI agent lifecycle including creation, retrieval, and updates
pub struct AgentManager;

impl AgentManager {
  /// Get an existing AI agent or create a new one if it doesn't exist
  ///
  /// This method:
  /// 1. Checks if an agent with the given model_identifier exists
  /// 2. If not, creates a new Person account with bot_account=true
  /// 3. Creates an AiAgent record linked to the Person
  /// 4. Assigns display name, avatar, and metadata
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `model_identifier` - Unique identifier for the AI model (e.g., "openai/gpt-4-turbo")
  /// * `settings` - Lemmy settings for generating URLs
  ///
  /// # Returns
  /// The existing or newly created AiAgent
  pub async fn get_or_create_agent(
    pool: &mut DbPool<'_>,
    model_identifier: &str,
    settings: &Settings,
  ) -> LemmyResult<AiAgent> {
    // Check if agent already exists
    if let Some(existing_agent) = AiAgent::find_by_model_identifier(pool, model_identifier).await?
    {
      debug!("Found existing AI agent: {}", model_identifier);
      return Ok(existing_agent);
    }

    info!("Creating new AI agent: {}", model_identifier);

    // Parse model identifier to extract provider and model name
    let (provider, model_name) = Self::parse_model_identifier(model_identifier)?;

    // Generate a unique username for the AI agent
    let username = Self::generate_username(model_identifier);

    // Get the local instance
    let instance = Instance::read_or_create(pool, &settings.hostname).await?;

    // Generate keypair for the person account
    let keypair = generate_actor_keypair()?;

    // Generate AP ID for the person
    let ap_id = Person::generate_local_actor_url(&username, settings)?;

    // Generate inbox URL
    let inbox_url: DbUrl = {
      let url = format!("{}/inbox", settings.get_protocol_and_hostname());
      Url::parse(&url)?.into()
    };

    // Create person account for the AI agent
    let person_form = PersonInsertForm {
      name: username.clone(),
      public_key: keypair.public_key,
      instance_id: instance.id,
      display_name: Some(Self::generate_display_name(&provider, &model_name)),
      ap_id: Some(ap_id),
      inbox_url: Some(inbox_url),
      private_key: Some(keypair.private_key),
      bot_account: Some(true),
      local: Some(true),
      bio: Some(format!(
        "AI agent powered by {} model: {}",
        provider, model_name
      )),
      ..PersonInsertForm::new(username, "placeholder".to_string(), instance.id)
    };

    let person = Person::create(pool, &person_form).await?;

    // Create AI agent record
    let agent_form = AiAgentInsertForm::new(
      person.id,
      model_identifier.to_string(),
      Self::generate_display_name(&provider, &model_name),
      provider.to_string(),
    )
    .with_model_version(Some(model_name.to_string()))
    .with_avatar_url(Self::generate_avatar_url(&provider))
    .with_is_active(true);

    let agent = AiAgent::create(pool, &agent_form).await?;

    info!(
      "Created AI agent: {} (person_id: {}, agent_id: {})",
      model_identifier, person.id, agent.id
    );

    Ok(agent)
  }

  /// Update an existing AI agent's metadata
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `agent_id` - ID of the agent to update
  /// * `form` - Update form with new values
  pub async fn update_agent(
    pool: &mut DbPool<'_>,
    agent_id: i32,
    form: &AiAgentUpdateForm,
  ) -> LemmyResult<AiAgent> {
    debug!("Updating AI agent: {}", agent_id);
    AiAgent::update(pool, agent_id, form).await
  }

  /// List all active AI agents
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  ///
  /// # Returns
  /// Vector of active AI agents
  pub async fn list_active_agents(pool: &mut DbPool<'_>) -> LemmyResult<Vec<AiAgent>> {
    AiAgent::list_active(pool).await
  }

  /// Deactivate an AI agent
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `agent_id` - ID of the agent to deactivate
  pub async fn deactivate_agent(pool: &mut DbPool<'_>, agent_id: i32) -> LemmyResult<AiAgent> {
    debug!("Deactivating AI agent: {}", agent_id);
    let form = AiAgentUpdateForm {
      is_active: Some(false),
      ..Default::default()
    };
    AiAgent::update(pool, agent_id, &form).await
  }

  /// Parse model identifier into provider and model name
  ///
  /// Expected format: "provider/model-name" (e.g., "openai/gpt-4-turbo")
  fn parse_model_identifier(model_identifier: &str) -> LemmyResult<(String, String)> {
    let parts: Vec<&str> = model_identifier.split('/').collect();
    if parts.len() != 2 {
      return Err(LemmyErrorType::InvalidInput.into());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
  }

  /// Generate a unique username for an AI agent
  ///
  /// Converts "openai/gpt-4-turbo" to "ai_openai_gpt_4_turbo"
  fn generate_username(model_identifier: &str) -> String {
    let sanitized = model_identifier
      .replace('/', "_")
      .replace('-', "_")
      .replace('.', "_")
      .to_lowercase();
    format!("ai_{}", sanitized)
  }

  /// Generate a display name for an AI agent
  ///
  /// Converts "openai" and "gpt-4-turbo" to "GPT-4 Turbo (OpenAI)"
  fn generate_display_name(provider: &str, model_name: &str) -> String {
    let formatted_model = model_name
      .split('-')
      .map(|part| {
        let mut chars = part.chars();
        match chars.next() {
          None => String::new(),
          Some(first) => {
            first.to_uppercase().collect::<String>() + chars.as_str().to_uppercase().as_str()
          }
        }
      })
      .collect::<Vec<_>>()
      .join("-");

    let formatted_provider = provider
      .chars()
      .next()
      .map(|c| c.to_uppercase().collect::<String>() + &provider[1..])
      .unwrap_or_default();

    format!("{} ({})", formatted_model, formatted_provider)
  }

  /// Generate an avatar URL for an AI agent based on provider
  ///
  /// Returns None for now, but could be extended to use provider logos
  fn generate_avatar_url(_provider: &str) -> Option<String> {
    // Could be extended to use actual provider logos
    None
  }

  /// Initialize default AI agents on system startup
  ///
  /// This creates a set of default AI agents if they don't already exist.
  /// The default agents are commonly used models that provide good debate diversity.
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `settings` - Lemmy settings for generating URLs
  ///
  /// # Returns
  /// Vector of created or existing default agents
  pub async fn initialize_default_agents(
    pool: &mut DbPool<'_>,
    settings: &Settings,
  ) -> LemmyResult<Vec<AiAgent>> {
    info!("Initializing default AI agents");

    // Default models that provide good debate diversity
    let default_models = vec![
      "openai/gpt-4-turbo",
      "anthropic/claude-3-opus",
      "google/gemini-pro",
    ];

    let mut agents = Vec::new();

    for model_identifier in default_models {
      match Self::get_or_create_agent(pool, model_identifier, settings).await {
        Ok(agent) => {
          agents.push(agent);
        }
        Err(e) => {
          tracing::error!(
            "Failed to initialize default agent {}: {}",
            model_identifier,
            e
          );
          // Continue with other agents even if one fails
        }
      }
    }

    info!("Initialized {} default AI agents", agents.len());

    Ok(agents)
  }

  /// Get agent by ID
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `agent_id` - ID of the agent to retrieve
  pub async fn get_agent_by_id(pool: &mut DbPool<'_>, agent_id: i32) -> LemmyResult<AiAgent> {
    AiAgent::read(pool, agent_id).await
  }

  /// Get agent by model identifier
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `model_identifier` - Model identifier to search for
  pub async fn get_agent_by_model_identifier(
    pool: &mut DbPool<'_>,
    model_identifier: &str,
  ) -> LemmyResult<Option<AiAgent>> {
    AiAgent::find_by_model_identifier(pool, model_identifier).await
  }

  /// List all AI agents (active and inactive)
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  pub async fn list_all_agents(pool: &mut DbPool<'_>) -> LemmyResult<Vec<AiAgent>> {
    AiAgent::list_all(pool).await
  }

  /// Activate an AI agent
  ///
  /// # Arguments
  /// * `pool` - Database connection pool
  /// * `agent_id` - ID of the agent to activate
  pub async fn activate_agent(pool: &mut DbPool<'_>, agent_id: i32) -> LemmyResult<AiAgent> {
    debug!("Activating AI agent: {}", agent_id);
    let form = AiAgentUpdateForm {
      is_active: Some(true),
      ..Default::default()
    };
    AiAgent::update(pool, agent_id, &form).await
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_model_identifier() {
    let (provider, model) = AgentManager::parse_model_identifier("openai/gpt-4-turbo").unwrap();
    assert_eq!(provider, "openai");
    assert_eq!(model, "gpt-4-turbo");

    let (provider, model) =
      AgentManager::parse_model_identifier("anthropic/claude-3-opus").unwrap();
    assert_eq!(provider, "anthropic");
    assert_eq!(model, "claude-3-opus");

    // Invalid format
    assert!(AgentManager::parse_model_identifier("invalid").is_err());
  }

  #[test]
  fn test_generate_username() {
    assert_eq!(
      AgentManager::generate_username("openai/gpt-4-turbo"),
      "ai_openai_gpt_4_turbo"
    );
    assert_eq!(
      AgentManager::generate_username("anthropic/claude-3-opus"),
      "ai_anthropic_claude_3_opus"
    );
  }

  #[test]
  fn test_generate_display_name() {
    assert_eq!(
      AgentManager::generate_display_name("openai", "gpt-4-turbo"),
      "GPT-4-TURBO (Openai)"
    );
    assert_eq!(
      AgentManager::generate_display_name("anthropic", "claude-3-opus"),
      "CLAUDE-3-OPUS (Anthropic)"
    );
  }
}
