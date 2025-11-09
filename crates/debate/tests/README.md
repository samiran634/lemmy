# Debate System Tests

This directory contains integration tests for the AI debate system. These tests verify end-to-end functionality and require a test database setup.

## Test Categories

### Unit Tests (in src/)

Unit tests are located alongside the source code in `src/` files:

1. **PromptBuilder Tests** (`src/prompt_builder.rs`)
   - ✅ Prompt builder creation
   - ✅ Custom prompt handling
   - ✅ All debate style prompts (formal, casual, adversarial, collaborative, socratic)
   - ✅ Context building with topics and agents
   - ✅ Round prompts
   - ✅ Adversarial role assignment

2. **OpenRouterClient Tests** (`src/openrouter.rs`)
   - ✅ Message constructors (system, user, assistant)
   - ✅ Response validation
   - ✅ Token usage cost estimation
   - ✅ Truncation detection

3. **AgentManager Tests** (`src/agent_manager.rs`)
   - ✅ Model identifier parsing
   - ✅ Username generation
   - ✅ Display name generation

4. **Worker Tests** (`src/worker.rs`)
   - ✅ Worker statistics tracking
   - ✅ Success/failure recording
   - ✅ Average processing time calculation
   - ✅ Success rate calculation

5. **Orchestrator Tests** (`src/orchestrator.rs`)
   - ✅ Orchestrator creation
   - ✅ Metrics tracking
   - ✅ Comment insert form builders
   - ✅ Debate round insert form builders

### Integration Tests (in tests/)

Integration tests are located in this directory and test end-to-end workflows:

1. **Debate Lifecycle** (`integration_tests.rs`)
   - ⏸️ Create debate config → Start debate → Execute rounds → Complete
   - ⏸️ Verify state transitions
   - ⏸️ Check comment creation

2. **Concurrent Debates** (`integration_tests.rs`)
   - ⏸️ Multiple debates running simultaneously
   - ⏸️ Concurrency limits
   - ⏸️ Resource management

3. **Pause/Resume/Stop** (`integration_tests.rs`)
   - ⏸️ Pause active debate
   - ⏸️ Resume paused debate
   - ⏸️ Stop debate early

4. **Error Handling** (`integration_tests.rs`)
   - ⏸️ API failures
   - ⏸️ Rate limiting
   - ⏸️ State recovery

5. **Human Interaction** (`integration_tests.rs`)
   - ⏸️ Human comments during debate
   - ⏸️ Context inclusion
   - ⏸️ Attribution

## Running Tests

### Run All Unit Tests

```bash
cargo test --package lemmy_debate --lib
```

### Run Specific Test Module

```bash
cargo test --package lemmy_debate --lib prompt_builder
cargo test --package lemmy_debate --lib openrouter
cargo test --package lemmy_debate --lib agent_manager
```

### Run Integration Tests (Requires Database)

Integration tests are marked with `#[ignore]` by default because they require a test database setup.

```bash
# Run all integration tests
cargo test --package lemmy_debate --test integration_tests -- --ignored

# Run specific integration test
cargo test --package lemmy_debate --test integration_tests test_debate_lifecycle -- --ignored
```

### Run All Tests (Unit + Integration)

```bash
cargo test --package lemmy_debate -- --include-ignored
```

## Test Database Setup

Integration tests require a PostgreSQL test database. To set up:

1. Create a test database:
   ```bash
   createdb lemmy_test
   ```

2. Run migrations:
   ```bash
   diesel migration run --database-url postgres://localhost/lemmy_test
   ```

3. Set environment variable:
   ```bash
   export LEMMY_DATABASE_URL=postgres://localhost/lemmy_test
   ```

## Test Coverage

Current test coverage focuses on:

- ✅ Core logic and business rules
- ✅ Data transformations
- ✅ Error handling paths
- ✅ Edge cases in prompt building
- ⏸️ End-to-end workflows (requires database)
- ⏸️ Concurrent processing (requires database)
- ⏸️ State transitions (requires database)

## Manual Testing Checklist

For features that are difficult to automate, manual testing is recommended:

### Debate Creation
- [ ] Create debate with default models
- [ ] Create debate with custom models
- [ ] Create debate with each debate style
- [ ] Verify debate config is saved correctly

### Debate Execution
- [ ] Execute full debate (all rounds)
- [ ] Verify AI responses are properly formatted
- [ ] Check comment threading
- [ ] Verify agent attribution

### Debate Control
- [ ] Pause active debate
- [ ] Resume paused debate
- [ ] Stop debate early
- [ ] Add additional rounds

### Error Scenarios
- [ ] Invalid API key
- [ ] Rate limit exceeded
- [ ] Network timeout
- [ ] Invalid model identifier
- [ ] Database connection failure

### Human Interaction
- [ ] Post human comment during debate
- [ ] Verify human comment appears in thread
- [ ] Check if human comment is included in next round
- [ ] Vote on AI comments
- [ ] Save debate thread

### UI Display
- [ ] Debate status indicator
- [ ] Round progress display
- [ ] Agent identification
- [ ] Debate controls visibility
- [ ] Error messages

## Adding New Tests

When adding new functionality, follow these guidelines:

1. **Unit Tests**: Add tests in the same file as the code
   - Test individual functions and methods
   - Mock external dependencies
   - Focus on edge cases and error paths

2. **Integration Tests**: Add tests in `tests/integration_tests.rs`
   - Test complete workflows
   - Use real database (mark with `#[ignore]`)
   - Verify state changes persist

3. **Test Naming**: Use descriptive names
   - `test_<feature>_<scenario>_<expected_result>`
   - Example: `test_debate_pause_when_active_succeeds`

4. **Test Documentation**: Add comments explaining
   - What is being tested
   - Why it's important
   - Any special setup required

## Known Limitations

- Integration tests require manual database setup
- API mocking is not yet implemented (tests use real API or are ignored)
- Performance tests are not included
- Load testing requires separate infrastructure

## Future Improvements

- [ ] Add API mocking for integration tests
- [ ] Implement test database fixtures
- [ ] Add performance benchmarks
- [ ] Create load testing suite
- [ ] Add property-based testing for prompt generation
- [ ] Implement snapshot testing for prompts
