# AI Debate System Database Migration

This migration creates the database schema for the AI Debate System feature.

## Tables Created

### debate_config
Stores configuration for AI debates associated with posts.
- Links to post and creator (person)
- Stores AI model selections, debate style, and parameters
- One config per post (enforced by UNIQUE constraint)

### debate_state
Tracks the current state of ongoing debates.
- Links to post
- Tracks status (pending, active, paused, completed, error)
- Maintains current round number and activity timestamp
- One state per post (enforced by UNIQUE constraint)

### ai_agent
Represents AI models as person accounts in the system.
- Links to person account
- Stores model identifier (e.g., "openai/gpt-4-turbo")
- Includes provider and version information
- Each agent has unique model_identifier and person_id

### debate_round
Records individual rounds of debate with metadata.
- Links to post, ai_agent, and the comment created
- Tracks tokens used and API latency
- Unique constraint ensures each agent responds once per round

## Indexes

Performance indexes are created on:
- Foreign key columns (post_id, creator_id, ai_agent_id, comment_id)
- Query columns (status, model_identifier, is_active)
- Sorting columns (last_activity_at)
- Composite indexes for common queries (post_id + round_number)

## Foreign Key Relationships

All foreign keys use `ON UPDATE CASCADE ON DELETE CASCADE` to ensure:
- When a post is deleted, all related debate data is removed
- When a person is deleted, their debate configs and AI agents are removed
- When a comment is deleted, the debate_round entry is removed

## Running the Migration

To apply this migration:
```bash
cargo run --bin lemmy_diesel_utils
```

To rollback:
```bash
# Use diesel CLI or custom migration runner with revert option
```
