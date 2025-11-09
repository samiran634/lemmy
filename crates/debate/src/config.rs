use crate::context::DebateContext;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::source::secret::Secret;
use lemmy_db_schema_file::schema::secret;
use lemmy_diesel_utils::{connection::DbPool, sensitive::SensitiveString};
use lemmy_utils::{error::LemmyResult, settings::structs::DebateConfig};

/// Get the OpenRouter API key from the database secret table
/// Falls back to the config file if not set in database
pub async fn get_openrouter_api_key(context: &DebateContext) -> LemmyResult<Option<String>> {
  let mut conn = context.pool().get().await?;

  // Try to get from database first
  let secret = secret::table
    .select(Secret::as_select())
    .first::<Secret>(&mut conn)
    .await?;

  if let Some(ref api_key) = secret.openrouter_api_key {
    // Return database value if present
    return Ok(Some(api_key.to_string()));
  }

  // Fall back to config file
  if let Some(ref debate_config) = context.settings().debate {
    return Ok(debate_config.openrouter_api_key.clone());
  }

  Ok(None)
}

/// Get the OpenRouter API key from DbPool
/// Falls back to the config file if not set in database
pub async fn get_openrouter_api_key_from_pool(
  pool: &mut DbPool<'_>,
  debate_config: Option<&DebateConfig>,
) -> LemmyResult<Option<String>> {
  // Try to get from database first
  let secret = secret::table
    .select(Secret::as_select())
    .first::<Secret>(&mut pool.conn().await?)
    .await?;

  if let Some(ref api_key) = secret.openrouter_api_key {
    // Return database value if present
    return Ok(Some(api_key.to_string()));
  }

  // Fall back to config file
  if let Some(config) = debate_config {
    return Ok(config.openrouter_api_key.clone());
  }

  Ok(None)
}

/// Update the OpenRouter API key in the database
pub async fn set_openrouter_api_key(
  context: &DebateContext,
  api_key: String,
) -> LemmyResult<()> {
  use diesel::update;

  let mut conn = context.pool().get().await?;

  // Update the first (and only) row in the secret table
  update(secret::table)
    .set(secret::openrouter_api_key.eq(Some(SensitiveString::from(api_key))))
    .execute(&mut conn)
    .await?;

  Ok(())
}

/// Remove the OpenRouter API key from the database
pub async fn clear_openrouter_api_key(context: &DebateContext) -> LemmyResult<()> {
  use diesel::update;

  let mut conn = context.pool().get().await?;

  // Clear the API key
  update(secret::table)
    .set(secret::openrouter_api_key.eq::<Option<SensitiveString>>(None))
    .execute(&mut conn)
    .await?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_config_module_compiles() {
    // Verify the module compiles and basic types are accessible
    // Full integration tests would require database setup
    assert!(true);
  }
}
