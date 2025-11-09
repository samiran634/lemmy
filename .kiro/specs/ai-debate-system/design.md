# AI Debate System Design

## Overview

The AI Debate System extends Lemmy's existing architecture to support automated debates between multiple AI models. The design maintains Lemmy's core structure while adding new components for debate orchestration, AI agent management, and OpenRouter API integration. The system operates asynchronously to avoid blocking the main HTTP server and integrates seamlessly with existing post and comment functionality.

## Architecture

### High-Level Architecture

```
┌─────────────────┐
│  Human User     │
│  (Web/API)      │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────────────────────┐
│           Lemmy HTTP Server (Actix-Web)             │
│  ┌──────────────────────────────────────────────┐  │
│  │  Existing API Routes (posts, comments, etc)  │  │
│  └──────────────────┬───────────────────────────┘  │
│                     │                                │
│  ┌──────────────────▼───────────────────────────┐  │
│  │     New Debate API Routes                    │  │
│  │  - POST /api/v4/debate/create                │  │
│  │  - GET  /api/v4/debate/status/:id            │  │
│  │  - POST /api/v4/debate/control/:id           │  │
│  └──────────────────┬───────────────────────────┘  │
└────────────────────┬┴───────────────────────────────┘
                     │
         ┌───────────┴───────────┐
         ▼                       ▼
┌─────────────────┐    ┌──────────────────────┐
│  Debate Service │    │   Database (Postgres) │
│  (Background)   │◄───┤  - debate_config      │
│                 │    │  - debate_state       │
│  ┌───────────┐  │    │  - ai_agent           │
│  │ Orchestr. │  │    │  - post (existing)    │
│  │  Engine   │  │    │  - comment (existing) │
│  └─────┬─────┘  │    │  - person (existing)  │
│        │        │    └──────────────────────┘
│  ┌─────▼─────┐  │
│  │ OpenRouter│  │
│  │  Client   │  │
│  └─────┬─────┘  │
└────────┼────────┘
         │
         ▼
┌─────────────────────┐
│  OpenRouter API     │
│  (External Service) │
│  - GPT-4            │
│  - Claude           │
│  - Gemini           │
│  - etc.             │
└─────────────────────┘
```

### Component Interaction Flow

1. **User submits debate topic** → HTTP API → Database (creates post + debate_config)
2. **Debate Service** (background task) → Polls for new debates → Loads config
3. **Orchestration Engine** → Manages rounds → Calls OpenRouter Client
4. **OpenRouter Client** → Makes API calls → Returns AI responses
5. **Orchestration Engine** → Creates comments → Updates debate state
6. **User views debate** → HTTP API → Reads posts/comments (standard Lemmy flow)

## Components and Interfaces

### 1. Database Schema Extensions

#### New Table: `debate_config`

```sql
CREATE TABLE debate_config (
    id SERIAL PRIMARY KEY,
    post_id INTEGER NOT NULL REFERENCES post(id) ON DELETE CASCADE,
    creator_id INTEGER NOT NULL REFERENCES person(id),
    ai_models TEXT[] NOT NULL,  -- Array of model identifiers
    debate_style VARCHAR(50) NOT NULL DEFAULT 'casual',
    max_rounds INTEGER NOT NULL DEFAULT 3,
    max_tokens_per_response INTEGER DEFAULT 500,
    custom_system_prompt TEXT,
    include_human_comments BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(post_id)
);
```

#### New Table: `debate_state`

```sql
CREATE TABLE debate_state (
    id SERIAL PRIMARY KEY,
    post_id INTEGER NOT NULL REFERENCES post(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, active, paused, completed, error
    current_round INTEGER NOT NULL DEFAULT 0,
    total_rounds INTEGER NOT NULL,
    last_activity_at TIMESTAMP NOT NULL DEFAULT NOW(),
    error_message TEXT,
    metadata JSONB,  -- For extensibility
    UNIQUE(post_id)
);
```

#### New Table: `ai_agent`

```sql
CREATE TABLE ai_agent (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    model_identifier VARCHAR(100) NOT NULL UNIQUE,  -- e.g., "openai/gpt-4-turbo"
    display_name VARCHAR(100) NOT NULL,
    provider VARCHAR(50) NOT NULL,  -- openai, anthropic, google, etc.
    model_version VARCHAR(50),
    avatar_url TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(person_id)
);
```

#### New Table: `debate_round`

```sql
CREATE TABLE debate_round (
    id SERIAL PRIMARY KEY,
    post_id INTEGER NOT NULL REFERENCES post(id) ON DELETE CASCADE,
    round_number INTEGER NOT NULL,
    ai_agent_id INTEGER NOT NULL REFERENCES ai_agent(id),
    comment_id INTEGER NOT NULL REFERENCES comment(id),
    tokens_used INTEGER,
    api_latency_ms INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(post_id, round_number, ai_agent_id)
);
```

### 2. Rust Crate Structure

#### New Crate: `crates/debate`

```
crates/debate/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── config.rs          # Configuration management
    ├── orchestrator.rs    # Main debate orchestration logic
    ├── openrouter.rs      # OpenRouter API client
    ├── agent_manager.rs   # AI agent lifecycle management
    ├── prompt_builder.rs  # System prompt construction
    └── models.rs          # Data structures
```

### 3. Core Interfaces

#### DebateOrchestrator Interface

```rust
pub struct DebateOrchestrator {
    db_pool: DbPool,
    openrouter_client: OpenRouterClient,
    config: DebateConfig,
}

impl DebateOrchestrator {
    /// Initialize a new debate from a post
    pub async fn start_debate(&self, post_id: PostId) -> LemmyResult<()>;
    
    /// Execute one round of the debate
    pub async fn execute_round(&self, post_id: PostId) -> LemmyResult<()>;
    
    /// Pause an ongoing debate
    pub async fn pause_debate(&self, post_id: PostId) -> LemmyResult<()>;
    
    /// Resume a paused debate
    pub async fn resume_debate(&self, post_id: PostId) -> LemmyResult<()>;
    
    /// Complete a debate early
    pub async fn complete_debate(&self, post_id: PostId) -> LemmyResult<()>;
}
```

#### OpenRouterClient Interface

```rust
pub struct OpenRouterClient {
    api_key: String,
    base_url: String,
    http_client: reqwest::Client,
}

#[derive(Debug, Serialize)]
pub struct OpenRouterRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

impl OpenRouterClient {
    /// Send a request to OpenRouter API
    pub async fn generate_response(
        &self,
        request: OpenRouterRequest,
    ) -> LemmyResult<OpenRouterResponse>;
    
    /// Get available models
    pub async fn list_models(&self) -> LemmyResult<Vec<ModelInfo>>;
}
```

#### AgentManager Interface

```rust
pub struct AgentManager {
    db_pool: DbPool,
}

impl AgentManager {
    /// Create or retrieve an AI agent person account
    pub async fn get_or_create_agent(
        &self,
        model_identifier: &str,
    ) -> LemmyResult<AiAgent>;
    
    /// List all active AI agents
    pub async fn list_active_agents(&self) -> LemmyResult<Vec<AiAgent>>;
    
    /// Update agent metadata
    pub async fn update_agent(&self, agent: &AiAgent) -> LemmyResult<()>;
}
```

#### PromptBuilder Interface

```rust
pub struct PromptBuilder {
    debate_style: DebateStyle,
    custom_prompt: Option<String>,
}

impl PromptBuilder {
    /// Build system prompt for an AI agent
    pub fn build_system_prompt(
        &self,
        agent: &AiAgent,
        role: Option<&str>,
    ) -> String;
    
    /// Build conversation context from debate history
    pub fn build_context(
        &self,
        topic: &str,
        previous_comments: &[Comment],
    ) -> Vec<Message>;
    
    /// Format a debate round prompt
    pub fn build_round_prompt(
        &self,
        round_number: u32,
        total_rounds: u32,
    ) -> String;
}
```

## Data Models

### Debate Configuration Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateConfig {
    pub id: i32,
    pub post_id: PostId,
    pub creator_id: PersonId,
    pub ai_models: Vec<String>,
    pub debate_style: DebateStyle,
    pub max_rounds: i32,
    pub max_tokens_per_response: Option<i32>,
    pub custom_system_prompt: Option<String>,
    pub include_human_comments: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebateStyle {
    Formal,
    Casual,
    Adversarial,
    Collaborative,
    Socratic,
}
```

### Debate State Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateState {
    pub id: i32,
    pub post_id: PostId,
    pub status: DebateStatus,
    pub current_round: i32,
    pub total_rounds: i32,
    pub last_activity_at: DateTime<Utc>,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebateStatus {
    Pending,
    Active,
    Paused,
    Completed,
    Error,
}
```

### AI Agent Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAgent {
    pub id: i32,
    pub person_id: PersonId,
    pub model_identifier: String,
    pub display_name: String,
    pub provider: String,
    pub model_version: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum DebateError {
    #[error("OpenRouter API error: {0}")]
    ApiError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid debate configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Debate not found: {0}")]
    DebateNotFound(PostId),
    
    #[error("AI agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),
}
```

### Error Recovery Strategy

1. **API Failures**: Retry with exponential backoff (3 attempts)
2. **Rate Limits**: Queue request and retry after delay
3. **Invalid Responses**: Log error, skip agent for this round
4. **Database Errors**: Rollback transaction, mark debate as error state
5. **Timeout**: Cancel request after 60 seconds, continue with other agents

## Testing Strategy

### Unit Tests

1. **PromptBuilder Tests**
   - Test system prompt generation for each debate style
   - Test context building with various comment histories
   - Test prompt formatting edge cases

2. **OpenRouterClient Tests**
   - Mock API responses
   - Test error handling and retries
   - Test request formatting

3. **AgentManager Tests**
   - Test agent creation and retrieval
   - Test duplicate handling
   - Test agent activation/deactivation

### Integration Tests

1. **End-to-End Debate Flow**
   - Create debate post
   - Execute multiple rounds
   - Verify comment creation
   - Check state transitions

2. **Concurrent Debates**
   - Start multiple debates simultaneously
   - Verify no race conditions
   - Check resource limits

3. **Human Interaction**
   - Post human comment during debate
   - Verify incorporation into next round
   - Test pause/resume functionality

### Performance Tests

1. **Load Testing**
   - 10 concurrent debates
   - Measure API latency
   - Monitor database connections

2. **Stress Testing**
   - Maximum configured debates
   - API failure scenarios
   - Database connection exhaustion

## Configuration

### Settings Structure

```hjson
{
  debate: {
    enabled: true
    openrouter_api_key: "sk-or-v1-..."
    openrouter_base_url: "https://openrouter.ai/api/v1"
    
    # Default models if user doesn't specify
    default_models: [
      "openai/gpt-4-turbo"
      "anthropic/claude-3-opus"
      "google/gemini-pro"
    ]
    
    # Limits
    max_concurrent_debates: 5
    max_rounds_per_debate: 10
    max_tokens_per_response: 1000
    request_timeout_seconds: 60
    
    # Rate limiting
    max_api_calls_per_minute: 60
    
    # Background task interval
    poll_interval_seconds: 10
  }
}
```

## API Endpoints

### Create Debate

```
POST /api/v4/debate/create
Content-Type: application/json

{
  "post_id": 123,
  "ai_models": ["openai/gpt-4-turbo", "anthropic/claude-3-opus"],
  "debate_style": "formal",
  "max_rounds": 5,
  "max_tokens_per_response": 500,
  "custom_system_prompt": "Optional custom prompt"
}

Response: 200 OK
{
  "debate_id": 456,
  "status": "pending",
  "estimated_start_time": "2024-01-15T10:30:00Z"
}
```

### Get Debate Status

```
GET /api/v4/debate/status/:post_id

Response: 200 OK
{
  "post_id": 123,
  "status": "active",
  "current_round": 2,
  "total_rounds": 5,
  "participating_agents": [
    {
      "model": "openai/gpt-4-turbo",
      "display_name": "GPT-4 Turbo",
      "comment_count": 2
    },
    {
      "model": "anthropic/claude-3-opus",
      "display_name": "Claude 3 Opus",
      "comment_count": 2
    }
  ],
  "last_activity": "2024-01-15T10:35:00Z"
}
```

### Control Debate

```
POST /api/v4/debate/control/:post_id
Content-Type: application/json

{
  "action": "pause" | "resume" | "stop" | "add_rounds",
  "additional_rounds": 3  // Only for add_rounds action
}

Response: 200 OK
{
  "status": "paused",
  "message": "Debate paused successfully"
}
```

## Background Task Architecture

### Debate Worker

```rust
pub async fn debate_worker_task(context: LemmyContext) -> LemmyResult<()> {
    let mut interval = tokio::time::interval(
        Duration::from_secs(SETTINGS.debate.poll_interval_seconds)
    );
    
    loop {
        interval.tick().await;
        
        // Find pending or active debates
        let debates = find_active_debates(&context.pool()).await?;
        
        // Process each debate (with concurrency limit)
        let semaphore = Arc::new(Semaphore::new(
            SETTINGS.debate.max_concurrent_debates
        ));
        
        for debate in debates {
            let permit = semaphore.clone().acquire_owned().await?;
            let context = context.clone();
            
            tokio::spawn(async move {
                let _permit = permit;
                if let Err(e) = process_debate(context, debate).await {
                    tracing::error!("Debate processing error: {}", e);
                }
            });
        }
    }
}
```

## Security Considerations

1. **API Key Protection**: Store OpenRouter API key in secrets table, never expose in responses
2. **Rate Limiting**: Enforce per-user limits on debate creation
3. **Content Moderation**: Apply existing Lemmy moderation to AI-generated content
4. **Cost Control**: Implement spending limits and alerts
5. **Input Validation**: Sanitize all user inputs before sending to AI models
6. **Bot Account Restrictions**: Prevent AI agents from performing actions outside debates

## Migration Strategy

1. **Phase 1**: Add new database tables (debate_config, debate_state, ai_agent, debate_round)
2. **Phase 2**: Deploy debate crate and background worker
3. **Phase 3**: Add API endpoints
4. **Phase 4**: Update UI to show debate controls (optional, can use existing UI)
5. **Phase 5**: Create initial AI agent accounts

## Monitoring and Observability

### Metrics to Track

- Active debates count
- Average debate duration
- API calls per minute
- API error rate
- Token usage per debate
- Cost per debate
- Background task lag

### Logging

- Debate lifecycle events (start, round complete, finish)
- API request/response (sanitized)
- Error conditions with full context
- Performance metrics per round

## Future Enhancements

1. **Debate Templates**: Pre-configured debate formats (Oxford style, Lincoln-Douglas, etc.)
2. **Voting System**: Allow users to vote on which AI made better arguments
3. **Debate Summaries**: Use an AI to generate summaries of completed debates
4. **Multi-Language Support**: Debates in different languages
5. **Custom AI Personalities**: Allow users to define AI agent personalities
6. **Debate Tournaments**: Multiple debates with elimination rounds
7. **Real-time Updates**: WebSocket support for live debate viewing
