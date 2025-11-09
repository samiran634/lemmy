# AI Debate System - API Implementation Summary

## Overview
Successfully implemented task 8 "Add API endpoints for debate control" including all three subtasks.

## Completed Subtasks

### 8.3 Implement request/response models ✅
Created `crates/debate/src/api.rs` with the following models:

**Request Models:**
- `CreateDebateRequest` - Request to create a new debate with optional configuration
- `DebateControlRequest` - Request to control debate (pause/resume/stop/add_rounds)

**Response Models:**
- `CreateDebateResponse` - Response after creating a debate
- `DebateStatusResponse` - Detailed status information about a debate
- `DebateControlResponse` - Response after controlling a debate
- `ParticipatingAgent` - Information about AI agents in the debate

**Enums:**
- `DebateControlAction` - Actions that can be performed (Pause, Resume, Stop, AddRounds)

### 8.1 Create debate API route handlers ✅
Created three handler files in `crates/api/api/src/debate/`:

**1. `create.rs` - Create Debate Handler**
- Validates user authentication
- Checks if post exists
- Prevents duplicate debates on same post
- Validates AI models (minimum 2 required)
- Sets defaults for optional parameters
- Creates debate config and state in database
- Returns debate ID and status

**2. `status.rs` - Get Debate Status Handler**
- Retrieves debate configuration and state
- Lists all rounds for the debate
- Counts comments per AI agent
- Returns comprehensive status including:
  - Current round and total rounds
  - Participating agents with comment counts
  - Last activity timestamp
  - Error messages if any

**3. `control.rs` - Control Debate Handler**
- Validates user is the debate creator
- Supports four control actions:
  - **Pause**: Pauses an active debate
  - **Resume**: Resumes a paused debate
  - **Stop**: Completes a debate early
  - **AddRounds**: Adds additional rounds to debate
- Updates debate state accordingly
- Returns new status and confirmation message

### 8.2 Integrate debate routes with Actix-Web ✅

**Route Configuration:**
Added debate routes to `crates/api/routes/src/lib.rs`:
- `POST /api/v4/debate/create` - Create a new debate
- `GET /api/v4/debate/status/{post_id}` - Get debate status
- `POST /api/v4/debate/control/{post_id}` - Control debate

**Middleware:**
- Applied rate limiting (post rate limit)
- Authentication handled via `LocalUserView` parameter

## Supporting Changes

### Error Types
Added debate-specific error types to `crates/utils/src/error.rs`:
- `DebateAlreadyExists` - Debate already exists for this post
- `DebateNotFound` - Debate not found
- `DebateNeedsTwoModels` - At least 2 AI models required
- `DebateInvalidRounds` - Invalid number of rounds
- `DebateNotActive` - Debate is not active (for pause)
- `DebateNotPaused` - Debate is not paused (for resume)
- `DebateAlreadyCompleted` - Debate already completed
- `DebateMissingAdditionalRounds` - Missing additional_rounds parameter
- `DebateTooManyRounds` - Too many rounds requested
- `NotPostCreator` - User is not the post creator
- `CouldntFindPost` - Post not found

### Database CRUD Methods
Enhanced `crates/debate/src/impls.rs` with additional methods:
- `DebateConfig::read_by_post_id()` - Read config by post ID (error if not found)
- `DebateState::read_by_post_id()` - Read state by post ID (error if not found)
- `AiAgent::read_by_model_identifier()` - Read agent by model ID (error if not found)
- `DebateRound::list_by_post_id()` - List all rounds for a post
- Added `read()` implementations for all Crud traits

### Model Enhancements
Added builder methods to `DebateConfigInsertForm` in `crates/debate/src/models.rs`:
- `with_max_tokens_per_response()`
- `with_custom_system_prompt()`
- `with_include_human_comments()`

### Dependencies
Updated `crates/api/api/Cargo.toml` to include:
```toml
lemmy_debate = { workspace = true, features = ["full"] }
```

## API Endpoints Documentation

### POST /api/v4/debate/create
Creates a new debate for a post.

**Request Body:**
```json
{
  "post_id": 123,
  "ai_models": ["openai/gpt-4-turbo", "anthropic/claude-3-opus"],
  "debate_style": "formal",
  "max_rounds": 5,
  "max_tokens_per_response": 500,
  "custom_system_prompt": "Optional custom prompt",
  "include_human_comments": true
}
```

**Response:**
```json
{
  "debate_id": 456,
  "status": "pending",
  "message": "Debate created successfully. It will start shortly."
}
```

### GET /api/v4/debate/status/{post_id}
Gets the current status of a debate.

**Response:**
```json
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
    }
  ],
  "last_activity": "2024-01-15T10:35:00Z",
  "error_message": null
}
```

### POST /api/v4/debate/control/{post_id}
Controls a debate (pause, resume, stop, or add rounds).

**Request Body:**
```json
{
  "action": "pause",
  "additional_rounds": 3  // Only for "add_rounds" action
}
```

**Response:**
```json
{
  "status": "paused",
  "message": "Debate paused successfully"
}
```

## Validation Rules

### Create Debate
- Post must exist
- No existing debate for the post
- Minimum 2 AI models required
- Rounds must be between 1 and 10
- User must be authenticated

### Control Debate
- User must be the debate creator
- Pause: Debate must be active
- Resume: Debate must be paused
- Stop: Debate must not be completed
- AddRounds: Must specify 1-10 additional rounds, total cannot exceed 20

## Next Steps
The API endpoints are now ready for integration with the debate worker system. The worker will:
1. Poll for pending debates
2. Execute debate rounds
3. Update debate state
4. Create AI-generated comments

## Testing Recommendations
1. Test debate creation with various configurations
2. Test status endpoint with active and completed debates
3. Test all control actions (pause, resume, stop, add_rounds)
4. Test error cases (invalid post, unauthorized user, etc.)
5. Test concurrent debate creation attempts
6. Test rate limiting on debate endpoints
