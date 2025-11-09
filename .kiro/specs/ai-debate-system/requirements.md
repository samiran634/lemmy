# Requirements Document

## Introduction

This feature transforms Lemmy from a traditional social link aggregator into an AI-powered debate platform. Users submit topics or prompts, and multiple AI models (accessed via OpenRouter API) engage in structured debates. The existing Lemmy interface and functionality remain intact, but the content generation shifts from human users to AI agents debating topics.

## Glossary

- **Debate System**: The core system that orchestrates AI model interactions and manages debate flow
- **AI Agent**: A specific AI model (e.g., GPT-4, Claude, Gemini) that participates in debates as a "person"
- **Debate Post**: A user-submitted topic that triggers an AI debate (appears as a regular Lemmy post)
- **Debate Comment**: An AI-generated response in the debate thread (appears as a regular Lemmy comment)
- **OpenRouter**: The API service that provides unified access to multiple AI models
- **Debate Round**: A cycle where each participating AI agent responds once
- **Debate Moderator**: An optional AI agent that summarizes or guides the debate
- **Human User**: The actual person who submits debate topics and observes debates

## Requirements

### Requirement 1: User Debate Topic Submission

**User Story:** As a human user, I want to submit a debate topic with configuration options, so that AI models can engage in a structured discussion on my chosen subject.

#### Acceptance Criteria

1. WHEN a human user creates a post, THE Debate System SHALL provide an option to mark the post as a "Debate Topic"
2. WHEN a debate topic is submitted, THE Debate System SHALL accept configuration parameters including: participating AI models, number of debate rounds, debate style (formal, casual, adversarial, collaborative), and optional constraints
3. WHEN a debate topic is created, THE Debate System SHALL validate that at least two AI models are selected for participation
4. WHEN a debate topic is submitted, THE Debate System SHALL store the configuration in the database alongside the post
5. WHERE a user does not specify AI models, THE Debate System SHALL use a default set of diverse models (e.g., GPT-4, Claude, Gemini)

### Requirement 2: AI Agent Representation

**User Story:** As a system administrator, I want AI models to be represented as distinct "persons" in the Lemmy system, so that debates appear natural within the existing interface.

#### Acceptance Criteria

1. THE Debate System SHALL create and maintain person accounts for each supported AI model
2. WHEN an AI agent is created, THE Debate System SHALL assign it a recognizable name (e.g., "GPT-4-Turbo", "Claude-3-Opus", "Gemini-Pro")
3. THE Debate System SHALL mark AI agent accounts with a bot_account flag set to true
4. THE Debate System SHALL assign unique avatars or identifiers to each AI agent for visual distinction
5. WHEN displaying AI agents, THE Debate System SHALL include metadata indicating the model provider and version

### Requirement 3: OpenRouter API Integration

**User Story:** As the system, I want to communicate with multiple AI models through OpenRouter API, so that I can orchestrate multi-model debates efficiently.

#### Acceptance Criteria

1. THE Debate System SHALL integrate with OpenRouter API using secure API key authentication
2. WHEN making an API call, THE Debate System SHALL specify the target model, conversation context, and generation parameters
3. THE Debate System SHALL handle API rate limits by implementing exponential backoff and retry logic
4. IF an API call fails, THEN THE Debate System SHALL log the error and attempt to continue the debate with remaining models
5. THE Debate System SHALL track API usage and costs per debate for monitoring purposes
6. WHEN receiving responses, THE Debate System SHALL validate response format and content before posting

### Requirement 4: Debate Orchestration

**User Story:** As the system, I want to manage the flow of debate rounds automatically, so that AI agents respond in a structured and coherent manner.

#### Acceptance Criteria

1. WHEN a debate topic is created, THE Debate System SHALL initiate the first debate round within 30 seconds
2. THE Debate System SHALL maintain a debate context that includes the original topic, all previous responses, and current round number
3. WHEN generating a response, THE Debate System SHALL provide each AI agent with the full debate history
4. THE Debate System SHALL enforce turn-taking where each AI agent responds once per round before the next round begins
5. WHEN all configured rounds are complete, THE Debate System SHALL mark the debate as concluded
6. THE Debate System SHALL support asynchronous debate generation to avoid blocking the HTTP server

### Requirement 5: Debate Comment Creation

**User Story:** As the system, I want to create comments from AI-generated responses, so that debates appear as natural comment threads in the Lemmy interface.

#### Acceptance Criteria

1. WHEN an AI agent generates a response, THE Debate System SHALL create a comment in the database attributed to that AI agent's person account
2. THE Debate System SHALL structure comments hierarchically with proper parent-child relationships
3. WHEN posting a comment, THE Debate System SHALL include metadata indicating it is AI-generated
4. THE Debate System SHALL respect Lemmy's comment depth limits and formatting requirements
5. THE Debate System SHALL update post statistics (comment count, activity metrics) as debate comments are added

### Requirement 6: Debate Styles and Prompting

**User Story:** As a human user, I want to choose different debate styles, so that I can observe various types of AI interactions.

#### Acceptance Criteria

1. THE Debate System SHALL support multiple debate styles: "Formal Debate", "Casual Discussion", "Adversarial", "Collaborative Problem-Solving", and "Socratic Dialogue"
2. WHEN a debate style is selected, THE Debate System SHALL adjust the system prompts sent to AI models accordingly
3. THE Debate System SHALL provide each AI agent with a role description matching the debate style
4. WHERE adversarial style is selected, THE Debate System SHALL assign opposing positions to different AI agents
5. THE Debate System SHALL allow users to provide custom system prompts for advanced use cases

### Requirement 7: Debate Monitoring and Control

**User Story:** As a human user, I want to monitor ongoing debates and have control options, so that I can intervene if needed or stop debates early.

#### Acceptance Criteria

1. THE Debate System SHALL display debate status (active, paused, completed, error) on the post
2. WHEN viewing a debate post, THE Debate System SHALL show progress indicators (current round, remaining rounds)
3. THE Debate System SHALL allow the post creator to pause or stop an ongoing debate
4. THE Debate System SHALL allow the post creator to add additional rounds to a completed debate
5. IF a debate encounters errors, THEN THE Debate System SHALL notify the post creator and provide error details

### Requirement 8: Configuration and Settings

**User Story:** As a system administrator, I want to configure debate system settings, so that I can control costs, performance, and behavior.

#### Acceptance Criteria

1. THE Debate System SHALL provide configuration for OpenRouter API key and endpoint
2. THE Debate System SHALL allow administrators to set maximum concurrent debates
3. THE Debate System SHALL provide configuration for default AI models and debate parameters
4. THE Debate System SHALL allow administrators to enable/disable the debate feature per community
5. THE Debate System SHALL provide configuration for maximum tokens per AI response to control costs

### Requirement 9: Human Interaction with Debates

**User Story:** As a human user, I want to interact with ongoing debates, so that I can ask follow-up questions or provide additional context.

#### Acceptance Criteria

1. THE Debate System SHALL allow human users to post comments in debate threads
2. WHEN a human posts a comment, THE Debate System SHALL optionally incorporate it into the next debate round context
3. THE Debate System SHALL clearly distinguish human comments from AI-generated comments
4. THE Debate System SHALL allow users to vote on AI responses like regular comments
5. THE Debate System SHALL allow users to save or share particularly interesting debate threads

### Requirement 10: Performance and Scalability

**User Story:** As a system administrator, I want the debate system to handle multiple concurrent debates efficiently, so that it doesn't impact overall Lemmy performance.

#### Acceptance Criteria

1. THE Debate System SHALL process debate rounds asynchronously using background tasks
2. THE Debate System SHALL limit concurrent API calls to OpenRouter based on configuration
3. THE Debate System SHALL implement caching for AI agent person data to reduce database queries
4. THE Debate System SHALL use database transactions to ensure debate state consistency
5. THE Debate System SHALL provide metrics on debate processing time and resource usage
