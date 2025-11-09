// Context type for debate operations
// This avoids circular dependencies with lemmy_api_utils

use lemmy_diesel_utils::connection::ActualDbPool;
use lemmy_utils::settings::structs::Settings;

/// Minimal context for debate operations
/// Wraps the database pool and settings
#[derive(Clone)]
pub struct DebateContext {
  pool: ActualDbPool,
  settings: &'static Settings,
}

impl DebateContext {
  pub fn new(pool: ActualDbPool, settings: &'static Settings) -> Self {
    Self { pool, settings }
  }

  pub fn pool(&self) -> &ActualDbPool {
    &self.pool
  }

  pub fn settings(&self) -> &Settings {
    self.settings
  }
}
