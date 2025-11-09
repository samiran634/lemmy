#[cfg(feature = "full")]
pub mod agent_manager;
pub mod api;
#[cfg(feature = "full")]
pub mod config;
#[cfg(feature = "full")]
pub mod context;
#[cfg(feature = "full")]
pub mod impls;
#[cfg(feature = "full")]
pub mod metrics;
pub mod models;
#[cfg(feature = "full")]
pub mod openrouter;
#[cfg(feature = "full")]
pub mod orchestrator;
#[cfg(feature = "full")]
pub mod prompt_builder;
#[cfg(feature = "full")]
pub mod worker;

pub use api::*;
pub use models::*;

#[cfg(feature = "full")]
pub use agent_manager::AgentManager;
#[cfg(feature = "full")]
pub use context::DebateContext;
#[cfg(feature = "full")]
pub use impls::{
  enrich_post_view_with_debate,
  enrich_post_views_with_debate,
  load_debate_metadata_for_post,
  load_debate_metadata_for_posts,
};
#[cfg(feature = "full")]
pub use metrics::{DebateMetrics, Timer};
#[cfg(feature = "full")]
pub use openrouter::{Message, OpenRouterClient, OpenRouterRequest, OpenRouterResponse};
#[cfg(feature = "full")]
pub use orchestrator::DebateOrchestrator;
#[cfg(feature = "full")]
pub use prompt_builder::PromptBuilder;
#[cfg(feature = "full")]
pub use worker::{debate_worker_task, get_worker_stats, WorkerStats};
