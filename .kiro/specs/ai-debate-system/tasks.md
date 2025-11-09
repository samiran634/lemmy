# Implementation Plan

- [x] 1. Set up database schema and migrations





  - Create migration files for new tables (debate_config, debate_state, ai_agent, debate_round)
  - Add indexes for performance (post_id, status, model_identifier)
  - Test migrations on development database
  - _Requirements: 1.4, 2.2, 2.3_

- [x] 2. Create debate crate structure and core models




  - [x] 2.1 Initialize new crate at crates/debate


    - Create Cargo.toml with dependencies (reqwest, serde, tokio, diesel)
    - Set up lib.rs with module declarations
    - Add workspace dependency in root Cargo.toml
    - _Requirements: 3.1, 3.2_


  - [x] 2.2 Implement data models and database schema types

    - Create models.rs with DebateConfig, DebateState, AiAgent, DebateRound structs
    - Implement Queryable and Insertable derives for Diesel
    - Add DebateStyle and DebateStatus enums
    - Create form structs for inserts and updates
    - _Requirements: 1.4, 2.2, 4.5_


  - [x] 2.3 Create database trait implementations

    - Implement CRUD operations for debate_config table
    - Implement CRUD operations for debate_state table
    - Implement CRUD operations for ai_agent table
    - Implement CRUD operations for debate_round table
    - Add query methods for finding active debates
    - _Requirements: 4.2, 4.5_

- [x] 3. Implement OpenRouter API client




  - [x] 3.1 Create OpenRouter client structure


    - Define OpenRouterClient struct with HTTP client and configuration
    - Implement request/response models (OpenRouterRequest, OpenRouterResponse, Message, Choice)
    - Add authentication header handling
    - _Requirements: 3.1, 3.2_

  - [x] 3.2 Implement API communication methods


    - Create generate_response method with proper error handling
    - Implement retry logic with exponential backoff
    - Add timeout handling (60 seconds)
    - Implement rate limiting tracking
    - _Requirements: 3.2, 3.3, 3.4_

  - [x] 3.3 Add API response validation and parsing


    - Validate response structure before processing
    - Extract generated text from response
    - Parse usage statistics (tokens, cost)
    - Handle API error responses gracefully
    - _Requirements: 3.6, 3.4_

- [x] 4. Implement AI agent management system





  - [x] 4.1 Create AgentManager component


    - Implement get_or_create_agent method
    - Create person accounts for AI agents with bot_account=true
    - Assign unique display names and avatars
    - Store model metadata (provider, version)
    - _Requirements: 2.1, 2.2, 2.3, 2.4_

  - [x] 4.2 Implement agent lifecycle methods


    - Add list_active_agents query method
    - Implement update_agent for metadata changes
    - Add method to deactivate agents
    - Create default agent initialization on system startup
    - _Requirements: 2.5, 8.3_

- [x] 5. Build prompt construction system




  - [x] 5.1 Create PromptBuilder component


    - Implement build_system_prompt for different debate styles
    - Create style-specific prompt templates (formal, casual, adversarial, collaborative, socratic)
    - Add role assignment logic for adversarial debates
    - Support custom system prompts
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

  - [x] 5.2 Implement context building methods


    - Create build_context to format debate history
    - Convert Lemmy comments to OpenRouter message format
    - Include original topic and all previous rounds
    - Add round number and progress indicators
    - Filter or include human comments based on configuration
    - _Requirements: 4.2, 4.3, 9.2_

- [x] 6. Implement debate orchestration engine





  - [x] 6.1 Create DebateOrchestrator structure


    - Initialize with database pool, OpenRouter client, and configuration
    - Implement start_debate method to initialize debate state
    - Create execute_round method for processing one debate round
    - Add pause_debate, resume_debate, and complete_debate control methods
    - _Requirements: 4.1, 4.2, 7.2, 7.3, 7.4_

  - [x] 6.2 Implement round execution logic

    - Load debate configuration and current state
    - Build context from previous comments
    - Iterate through AI agents for the round
    - Call OpenRouter API for each agent
    - Create comments from AI responses
    - Update debate state after round completion
    - _Requirements: 4.2, 4.3, 4.4, 4.5, 5.1, 5.2_

  - [x] 6.3 Add error handling and recovery

    - Handle API failures gracefully (continue with other agents)
    - Log errors with full context
    - Update debate state to error status when critical failures occur
    - Implement transaction rollback on database errors
    - _Requirements: 3.4, 7.5_

  - [x] 6.4 Implement comment creation from AI responses

    - Create Comment structs with AI agent as creator
    - Set proper parent-child relationships for threading
    - Add metadata indicating AI-generated content
    - Update post statistics (comment count, activity)
    - Respect comment depth limits
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 7. Create background worker for debate processing



  - [x] 7.1 Implement debate worker task


    - Create async worker function that polls for active debates
    - Use tokio interval for periodic checking (configurable interval)
    - Query database for pending and active debates
    - Process debates with concurrency limits (semaphore)
    - _Requirements: 4.6, 10.1, 10.2_

  - [x] 7.2 Integrate worker with server startup


    - Spawn debate worker task in server initialization
    - Pass LemmyContext to worker
    - Add graceful shutdown handling
    - Ensure worker doesn't block HTTP server
    - _Requirements: 10.1, 10.4_

  - [x] 7.3 Add worker monitoring and metrics


    - Track active debate count
    - Log worker activity and errors
    - Measure processing time per debate
    - Implement health check endpoint
    - _Requirements: 10.5_
- [x] 8. Add API endpoints for debate control




- [ ] 8. Add API endpoints for debate control

  - [x] 8.1 Create debate API route handlers


    - Implement POST /api/v4/debate/create endpoint
    - Implement GET /api/v4/debate/status/:post_id endpoint
    - Implement POST /api/v4/debate/control/:post_id endpoint
    - Add request validation and authentication
    - _Requirements: 1.1, 1.2, 7.1, 7.2, 7.3, 7.4_


  - [x] 8.2 Integrate debate routes with Actix-Web

    - Add debate routes to lemmy_api_routes configuration
    - Apply rate limiting middleware
    - Add authentication checks (user must be post creator for control)
    - _Requirements: 8.4, 9.4_

  - [x] 8.3 Implement request/response models


    - Create CreateDebateRequest struct
    - Create DebateStatusResponse struct
    - Create DebateControlRequest struct
    - Add validation logic for all request types
    - _Requirements: 1.2, 1.3, 1.5_

- [x] 9. Add configuration and settings




  - [x] 9.1 Extend Lemmy settings structure


    - Add debate section to config/defaults.hjson
    - Define DebateSettings struct in lemmy_utils
    - Include OpenRouter API key, base URL, and limits
    - Add default models configuration
    - _Requirements: 8.1, 8.2, 8.3, 8.5_

  - [x] 9.2 Implement configuration validation


    - Validate API key format on startup
    - Check that default models are valid
    - Ensure limits are within reasonable ranges
    - Log configuration warnings
    - _Requirements: 8.1, 8.5_

  - [x] 9.3 Add secrets management for API key


    - Store OpenRouter API key in secrets table (encrypted)
    - Retrieve key securely at runtime
    - Never expose key in API responses or logs
    - _Requirements: 8.1_

- [x] 10. Implement human interaction features



  - [x] 10.1 Add debate metadata to post responses


    - Include debate status in post API responses
    - Show current round and total rounds
    - Display participating AI agents
    - Add debate controls for post creator
    - _Requirements: 7.1, 7.2_

  - [x] 10.2 Enable human comments in debates


    - Allow users to post comments in debate threads
    - Mark human comments distinctly from AI comments
    - Optionally include human comments in next round context
    - _Requirements: 9.1, 9.2, 9.3_

  - [x] 10.3 Implement voting and interaction


    - Enable voting on AI-generated comments
    - Allow saving debate threads
    - Support standard Lemmy comment actions (reply, report)
    - _Requirements: 9.4, 9.5_

- [x] 11. Add monitoring and observability

  - [x] 11.1 Implement metrics collection
    - Track active debates count
    - Measure API latency and error rates
    - Monitor token usage and costs
    - Record debate duration statistics
    - _Requirements: 3.5, 10.5_

  - [x] 11.2 Add comprehensive logging
    - Log debate lifecycle events (start, round, complete)
    - Log API requests/responses (sanitized)
    - Log errors with full context
    - Add structured logging with tracing
    - _Requirements: 3.4, 7.5_

  - [x] 11.3 Create admin monitoring endpoints
    - Add endpoint to view active debates
    - Add endpoint to view API usage statistics
    - Add endpoint to view error logs
    - Restrict access to admin users only
    - _Requirements: 8.2, 10.5_

- [-] 12. Testing and validation



  - [x] 12.1 Write unit tests for core components

    - Test PromptBuilder with various debate styles
    - Test OpenRouterClient with mocked responses
    - Test AgentManager CRUD operations
    - Test DebateOrchestrator state transitions
    - _Requirements: All_


  - [x] 12.2 Write integration tests

    - Test end-to-end debate flow (create → rounds → complete)
    - Test concurrent debates
    - Test human interaction during debates
    - Test pause/resume/stop functionality
    - _Requirements: All_


  - [ ] 12.3 Perform manual testing
    - Create test debates with different configurations
    - Verify AI responses are properly formatted
    - Test error scenarios (API failures, rate limits)
    - Validate UI display of debates
    - _Requirements: All_

- [ ] 13. Documentation and deployment
  - [ ] 13.1 Write developer documentation
    - Document debate system architecture
    - Create API endpoint documentation
    - Write configuration guide
    - Add troubleshooting section
    - _Requirements: All_

  - [ ] 13.2 Create user guide
    - Explain how to create debates
    - Document debate styles and options
    - Provide examples of good debate topics
    - Explain cost implications
    - _Requirements: 1.1, 1.2, 6.1_

  - [ ] 13.3 Prepare deployment checklist
    - Database migration steps
    - Configuration requirements
    - OpenRouter API key setup
    - Initial AI agent creation
    - Monitoring setup
    - _Requirements: 8.1, 8.3_
