//! Prompt construction system for AI debates
//!
//! This module provides the `PromptBuilder` component for constructing system prompts
//! and conversation contexts for AI agents participating in debates. It supports multiple
//! debate styles and handles the conversion of Lemmy posts and comments into OpenRouter
//! message format.
//!
//! # Features
//!
//! - Style-specific prompt templates (formal, casual, adversarial, collaborative, socratic)
//! - Role assignment for adversarial debates
//! - Custom system prompt support
//! - Conversation context building from debate history
//! - Human comment filtering
//! - Agent attribution in context
//!
//! # Example
//!
//! ```rust,ignore
//! use lemmy_debate::{PromptBuilder, DebateStyle, AiAgent};
//!
//! let builder = PromptBuilder::new(DebateStyle::Formal);
//! let agent = get_ai_agent();
//! let system_prompt = builder.build_system_prompt(&agent, None);
//! ```

use crate::{
  models::{AiAgent, DebateConfig, DebateStyle},
  openrouter::Message,
};
use lemmy_db_schema::source::{comment::Comment, post::Post};

/// Builder for constructing prompts for AI debate participants
///
/// The `PromptBuilder` is responsible for creating system prompts and conversation
/// contexts that guide AI agents through debates. It adapts its behavior based on
/// the debate style and can incorporate custom instructions.
#[derive(Debug, Clone)]
pub struct PromptBuilder {
  debate_style: DebateStyle,
  custom_prompt: Option<String>,
}

impl PromptBuilder {
  /// Create a new PromptBuilder with the specified debate style
  pub fn new(debate_style: DebateStyle) -> Self {
    Self {
      debate_style,
      custom_prompt: None,
    }
  }

  /// Create a PromptBuilder from a DebateConfig
  pub fn from_config(config: &DebateConfig) -> Self {
    let debate_style = config
      .get_debate_style()
      .unwrap_or(DebateStyle::Casual);
    
    Self {
      debate_style,
      custom_prompt: config.custom_system_prompt.clone(),
    }
  }

  /// Set a custom system prompt that overrides the default style-based prompt
  pub fn with_custom_prompt(mut self, prompt: Option<String>) -> Self {
    self.custom_prompt = prompt;
    self
  }

  /// Build a system prompt for an AI agent based on debate style and role
  ///
  /// # Arguments
  /// * `agent` - The AI agent that will receive this prompt
  /// * `role` - Optional role assignment (e.g., "proponent", "opponent" for adversarial debates)
  pub fn build_system_prompt(&self, agent: &AiAgent, role: Option<&str>) -> String {
    // If custom prompt is provided, use it
    if let Some(custom) = &self.custom_prompt {
      return custom.clone();
    }

    // Build style-specific prompt
    let base_prompt = self.get_style_base_prompt();
    let role_instruction = self.get_role_instruction(role);
    
    format!(
      "{}\n\nYou are {}, an AI model participating in this debate.\n\n{}",
      base_prompt,
      agent.display_name,
      role_instruction
    )
  }

  /// Get the base prompt template for the current debate style
  fn get_style_base_prompt(&self) -> &str {
    match self.debate_style {
      DebateStyle::Formal => {
        "You are participating in a formal academic debate. Your responses should be:\n\
         - Well-structured with clear arguments\n\
         - Supported by logical reasoning\n\
         - Professional and respectful in tone\n\
         - Focused on the substance of the topic\n\
         - Free from personal attacks or emotional appeals\n\
         \n\
         Present your arguments clearly, acknowledge valid points from others, and build upon \
         the discussion constructively."
      }
      DebateStyle::Casual => {
        "You are participating in a casual, friendly discussion. Your responses should be:\n\
         - Conversational and approachable\n\
         - Clear and easy to understand\n\
         - Engaging and thoughtful\n\
         - Respectful of different viewpoints\n\
         - Natural and authentic\n\
         \n\
         Feel free to use examples, analogies, and relatable language. The goal is to have \
         an interesting and informative conversation."
      }
      DebateStyle::Adversarial => {
        "You are participating in an adversarial debate where you will take a specific position. \
         Your responses should be:\n\
         - Strongly advocating for your assigned position\n\
         - Challenging opposing arguments directly\n\
         - Identifying weaknesses in other perspectives\n\
         - Defending your position against criticism\n\
         - Strategic and persuasive\n\
         \n\
         While you should argue forcefully for your position, remain respectful and focus on \
         the strength of arguments rather than personal attacks."
      }
      DebateStyle::Collaborative => {
        "You are participating in a collaborative problem-solving discussion. Your responses should be:\n\
         - Focused on finding common ground\n\
         - Building on others' ideas\n\
         - Exploring solutions together\n\
         - Acknowledging the value in different perspectives\n\
         - Constructive and solution-oriented\n\
         \n\
         The goal is not to win an argument, but to work together to develop the best possible \
         understanding or solution to the topic at hand."
      }
      DebateStyle::Socratic => {
        "You are participating in a Socratic dialogue. Your responses should be:\n\
         - Primarily asking thoughtful, probing questions\n\
         - Examining assumptions and definitions\n\
         - Exploring implications and consequences\n\
         - Guiding the discussion through inquiry\n\
         - Helping uncover deeper truths through questioning\n\
         \n\
         Use questions to challenge ideas, clarify concepts, and lead the discussion toward \
         greater understanding. You may make brief statements, but questions should be your \
         primary tool."
      }
    }
  }

  /// Get role-specific instructions for adversarial debates
  fn get_role_instruction(&self, role: Option<&str>) -> String {
    match (self.debate_style, role) {
      (DebateStyle::Adversarial, Some(role_name)) => {
        format!(
          "Your assigned role: {}\n\
           You must consistently argue from this perspective throughout the debate. \
           Challenge arguments that oppose your position and strengthen arguments that support it.",
          role_name
        )
      }
      _ => {
        "Engage with the topic thoughtfully and respond to the points raised by other participants."
          .to_string()
      }
    }
  }

  /// Build conversation context from debate history
  ///
  /// # Arguments
  /// * `post` - The post containing the debate topic
  /// * `previous_comments` - All previous comments in the debate, ordered chronologically
  /// * `current_round` - The current round number
  /// * `total_rounds` - Total number of rounds in the debate
  /// * `include_human_comments` - Whether to include comments from human users
  /// * `ai_person_ids` - Set of person IDs that are AI agents (for filtering)
  pub fn build_context(
    &self,
    post: &Post,
    previous_comments: &[Comment],
    current_round: i32,
    total_rounds: i32,
    include_human_comments: bool,
    ai_person_ids: &std::collections::HashSet<lemmy_db_schema::newtypes::PersonId>,
  ) -> Vec<Message> {
    let mut messages = Vec::new();

    // Add the debate topic as the initial user message
    let topic = self.format_topic(post);
    messages.push(Message::user(topic));

    // Add previous comments as conversation history
    for comment in previous_comments {
      let is_ai = ai_person_ids.contains(&comment.creator_id);
      
      // Skip human comments if configured to do so
      if !include_human_comments && !is_ai {
        continue;
      }

      // Format the comment as an assistant message
      // All previous responses are treated as assistant messages to maintain conversation flow
      messages.push(Message::assistant(&comment.content));
    }

    // Add round progress indicator
    if current_round > 0 {
      let progress = format!(
        "\n[Round {}/{} - Continue the discussion]",
        current_round, total_rounds
      );
      messages.push(Message::user(progress));
    }

    messages
  }

  /// Build conversation context with agent identification
  ///
  /// This version includes agent names in the context for better tracking
  ///
  /// # Arguments
  /// * `post` - The post containing the debate topic
  /// * `previous_comments` - All previous comments with their creators
  /// * `agent_map` - Map of person_id to agent display name
  /// * `current_round` - The current round number
  /// * `total_rounds` - Total number of rounds in the debate
  /// * `include_human_comments` - Whether to include comments from human users
  pub fn build_context_with_agents(
    &self,
    post: &Post,
    previous_comments: &[(Comment, Option<String>)], // (comment, agent_name)
    current_round: i32,
    total_rounds: i32,
    include_human_comments: bool,
  ) -> Vec<Message> {
    let mut messages = Vec::new();

    // Add the debate topic as the initial user message
    let topic = self.format_topic(post);
    messages.push(Message::user(topic));

    // Add previous comments as conversation history with agent attribution
    for (comment, agent_name) in previous_comments {
      // Skip human comments if configured to do so
      if !include_human_comments && agent_name.is_none() {
        continue;
      }

      // Format the comment with attribution
      let content = if let Some(name) = agent_name {
        format!("[{}]: {}", name, comment.content)
      } else {
        format!("[Human]: {}", comment.content)
      };

      messages.push(Message::assistant(content));
    }

    // Add round progress indicator
    if current_round > 0 {
      let progress = format!(
        "\n[Round {}/{} - Continue the discussion]",
        current_round, total_rounds
      );
      messages.push(Message::user(progress));
    }

    messages
  }

  /// Format the debate topic from a post
  fn format_topic(&self, post: &Post) -> String {
    let mut topic = format!("Debate Topic: {}", post.name);
    
    if let Some(body) = &post.body {
      topic.push_str(&format!("\n\nAdditional Context:\n{}", body));
    }

    topic
  }

  /// Build a round-specific prompt to guide the AI's response
  pub fn build_round_prompt(&self, round_number: i32, total_rounds: i32) -> String {
    if round_number == 1 {
      "This is the opening round. Present your initial thoughts on the topic.".to_string()
    } else if round_number == total_rounds {
      format!(
        "This is the final round ({}/{}). Provide your concluding thoughts and summarize your key points.",
        round_number, total_rounds
      )
    } else {
      format!(
        "This is round {}/{}. Respond to the previous points and develop your arguments further.",
        round_number, total_rounds
      )
    }
  }

  /// Assign roles for adversarial debates based on agent index
  ///
  /// # Arguments
  /// * `agent_index` - The index of the agent in the debate (0-based)
  /// * `total_agents` - Total number of agents in the debate
  pub fn assign_adversarial_role(&self, agent_index: usize, total_agents: usize) -> String {
    if total_agents == 2 {
      // Simple two-sided debate
      if agent_index == 0 {
        "Proponent (arguing FOR the topic)".to_string()
      } else {
        "Opponent (arguing AGAINST the topic)".to_string()
      }
    } else {
      // Multi-agent debate - alternate between supporting and opposing
      if agent_index % 2 == 0 {
        format!("Proponent {} (arguing FOR the topic)", agent_index / 2 + 1)
      } else {
        format!("Opponent {} (arguing AGAINST the topic)", (agent_index + 1) / 2)
      }
    }
  }
}

/// Helper functions for working with AI agents in context building
pub mod context_helpers {
  use crate::models::AiAgent;
  use lemmy_db_schema::newtypes::PersonId;
  use std::collections::{HashMap, HashSet};

  /// Build a set of person IDs that are AI agents
  pub fn build_ai_person_id_set(agents: &[AiAgent]) -> HashSet<PersonId> {
    agents.iter().map(|agent| agent.person_id).collect()
  }

  /// Build a map of person IDs to agent display names
  pub fn build_agent_name_map(agents: &[AiAgent]) -> HashMap<PersonId, String> {
    agents
      .iter()
      .map(|agent| (agent.person_id, agent.display_name.clone()))
      .collect()
  }

  /// Get agent name for a person ID, or None if not an AI agent
  pub fn get_agent_name(
    person_id: PersonId,
    agent_map: &HashMap<PersonId, String>,
  ) -> Option<String> {
    agent_map.get(&person_id).cloned()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use lemmy_db_schema::newtypes::{CommunityId, LanguageId, PersonId, PostId};

  fn create_test_agent() -> AiAgent {
    AiAgent {
      id: 1,
      person_id: PersonId(1),
      model_identifier: "openai/gpt-4-turbo".to_string(),
      display_name: "GPT-4 Turbo".to_string(),
      provider: "openai".to_string(),
      model_version: Some("turbo".to_string()),
      avatar_url: None,
      is_active: true,
      created_at: Utc::now(),
    }
  }

  fn create_test_post() -> Post {
    Post {
      id: PostId(1),
      name: "Should AI replace human moderators?".to_string(),
      body: Some("Discuss the pros and cons of using AI for content moderation.".to_string()),
      creator_id: PersonId(1),
      community_id: CommunityId(1),
      removed: false,
      locked: false,
      published_at: Utc::now(),
      updated_at: None,
      deleted: false,
      nsfw: false,
      embed_title: None,
      embed_description: None,
      thumbnail_url: None,
      ap_id: "http://example.com/post/1".parse().unwrap(),
      local: true,
      embed_video_url: None,
      language_id: LanguageId(0),
      featured_community: false,
      featured_local: false,
      url_content_type: None,
      alt_text: None,
      scheduled_publish_time_at: None,
      newest_comment_time_necro_at: None,
      newest_comment_time_at: None,
      comments: 0,
      score: 0,
      upvotes: 0,
      downvotes: 0,
      hot_rank: 0.0,
      hot_rank_active: 0.0,
      controversy_rank: 0.0,
      scaled_rank: 0.0,
      report_count: 0,
      unresolved_report_count: 0,
      federation_pending: false,
      embed_video_width: None,
      embed_video_height: None,
      url: None,
    }
  }

  #[test]
  fn test_prompt_builder_creation() {
    let builder = PromptBuilder::new(DebateStyle::Formal);
    assert!(matches!(builder.debate_style, DebateStyle::Formal));
    assert!(builder.custom_prompt.is_none());
  }

  #[test]
  fn test_custom_prompt() {
    let custom = "Custom instructions".to_string();
    let builder = PromptBuilder::new(DebateStyle::Casual)
      .with_custom_prompt(Some(custom.clone()));
    
    let agent = create_test_agent();
    let prompt = builder.build_system_prompt(&agent, None);
    
    assert_eq!(prompt, custom);
  }

  #[test]
  fn test_formal_style_prompt() {
    let builder = PromptBuilder::new(DebateStyle::Formal);
    let agent = create_test_agent();
    let prompt = builder.build_system_prompt(&agent, None);
    
    assert!(prompt.contains("formal academic debate"));
    assert!(prompt.contains("GPT-4 Turbo"));
    assert!(prompt.contains("Well-structured"));
  }

  #[test]
  fn test_adversarial_style_with_role() {
    let builder = PromptBuilder::new(DebateStyle::Adversarial);
    let agent = create_test_agent();
    let prompt = builder.build_system_prompt(&agent, Some("Proponent"));
    
    assert!(prompt.contains("adversarial debate"));
    assert!(prompt.contains("Your assigned role: Proponent"));
  }

  #[test]
  fn test_socratic_style_prompt() {
    let builder = PromptBuilder::new(DebateStyle::Socratic);
    let agent = create_test_agent();
    let prompt = builder.build_system_prompt(&agent, None);
    
    assert!(prompt.contains("Socratic dialogue"));
    assert!(prompt.contains("asking thoughtful, probing questions"));
  }

  #[test]
  fn test_build_context_with_topic() {
    let builder = PromptBuilder::new(DebateStyle::Casual);
    let post = create_test_post();
    let ai_person_ids = std::collections::HashSet::new();
    
    let messages = builder.build_context(&post, &[], 1, 3, true, &ai_person_ids);
    
    assert_eq!(messages.len(), 2); // Topic + round indicator
    assert_eq!(messages[0].role, "user");
    assert!(messages[0].content.contains("Should AI replace human moderators?"));
    assert!(messages[0].content.contains("pros and cons"));
  }

  #[test]
  fn test_build_context_with_agents() {
    let builder = PromptBuilder::new(DebateStyle::Formal);
    let post = create_test_post();
    
    // Create test comments with agent attribution
    let comments = vec![
      (create_test_comment("First argument"), Some("GPT-4".to_string())),
      (create_test_comment("Counter argument"), Some("Claude".to_string())),
      (create_test_comment("Human input"), None),
    ];
    
    // Include human comments
    let messages = builder.build_context_with_agents(&post, &comments, 2, 3, true);
    assert_eq!(messages.len(), 5); // Topic + 3 comments + round indicator
    assert!(messages[1].content.contains("[GPT-4]"));
    assert!(messages[2].content.contains("[Claude]"));
    assert!(messages[3].content.contains("[Human]"));
    
    // Exclude human comments
    let messages = builder.build_context_with_agents(&post, &comments, 2, 3, false);
    assert_eq!(messages.len(), 4); // Topic + 2 AI comments + round indicator
  }

  fn create_test_comment(content: &str) -> Comment {
    use lemmy_db_schema::newtypes::CommentId;
    
    Comment {
      id: CommentId(1),
      creator_id: PersonId(1),
      post_id: PostId(1),
      content: content.to_string(),
      removed: false,
      published_at: Utc::now(),
      updated_at: None,
      deleted: false,
      ap_id: "http://example.com/comment/1".parse().unwrap(),
      local: true,
      path: "0.1".to_string(),
      distinguished: false,
      language_id: LanguageId(0),
      score: 0,
      upvotes: 0,
      downvotes: 0,
      child_count: 0,
      hot_rank: 0.0,
      controversy_rank: 0.0,
      report_count: 0,
      unresolved_report_count: 0,
      federation_pending: false,
      locked: false,
    }
  }

  #[test]
  fn test_round_prompts() {
    let builder = PromptBuilder::new(DebateStyle::Formal);
    
    let opening = builder.build_round_prompt(1, 5);
    assert!(opening.contains("opening round"));
    
    let middle = builder.build_round_prompt(3, 5);
    assert!(middle.contains("round 3/5"));
    
    let closing = builder.build_round_prompt(5, 5);
    assert!(closing.contains("final round"));
    assert!(closing.contains("concluding thoughts"));
  }

  #[test]
  fn test_adversarial_role_assignment() {
    let builder = PromptBuilder::new(DebateStyle::Adversarial);
    
    // Two-agent debate
    let role0 = builder.assign_adversarial_role(0, 2);
    let role1 = builder.assign_adversarial_role(1, 2);
    
    assert!(role0.contains("Proponent"));
    assert!(role0.contains("FOR"));
    assert!(role1.contains("Opponent"));
    assert!(role1.contains("AGAINST"));
    
    // Multi-agent debate
    let role2 = builder.assign_adversarial_role(2, 4);
    assert!(role2.contains("Proponent 2"));
  }

  #[test]
  fn test_all_debate_styles() {
    let agent = create_test_agent();
    
    for style in [
      DebateStyle::Formal,
      DebateStyle::Casual,
      DebateStyle::Adversarial,
      DebateStyle::Collaborative,
      DebateStyle::Socratic,
    ] {
      let builder = PromptBuilder::new(style.clone());
      let prompt = builder.build_system_prompt(&agent, None);
      
      // All prompts should include the agent name
      assert!(prompt.contains("GPT-4 Turbo"));
      // All prompts should have substantial content
      assert!(prompt.len() > 100);
    }
  }
}
