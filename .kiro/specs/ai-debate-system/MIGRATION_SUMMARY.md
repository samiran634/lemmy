# Database Migration Summary

## Migration Created: 2025-11-08-000000_create_ai_debate_tables

### Overview
Successfully created database schema migration for the AI Debate System feature. The migration includes four new tables with proper foreign key relationships, indexes, and constraints.

### Files Created
1. `migrations/2025-11-08-000000_create_ai_debate_tables/up.sql` - Forward migration
2. `migrations/2025-11-08-000000_create_ai_debate_tables/down.sql` - Rollback migration
3. `migrations/2025-11-08-000000_create_ai_debate_tables/README.md` - Documentation

### Tables Created

#### 1. debate_config
- **Purpose**: Stores debate configuration for posts
- **Key Fields**: post_id, creator_id, ai_models[], debate_style, max_rounds
- **Constraints**: UNIQUE(post_id) - one config per post
- **Foreign Keys**: post(id), person(id)

#### 2. debate_state
- **Purpose**: Tracks current state of debates
- **Key Fields**: post_id, status, current_round, total_rounds, last_activity_at
- **Constraints**: UNIQUE(post_id) - one state per post
- **Foreign Keys**: post(id)

#### 3. ai_agent
- **Purpose**: Represents AI models as person accounts
- **Key Fields**: person_id, model_identifier, display_name, provider
- **Constraints**: UNIQUE(model_identifier), UNIQUE(person_id)
- **Foreign Keys**: person(id)

#### 4. debate_round
- **Purpose**: Records individual debate rounds
- **Key Fields**: post_id, round_number, ai_agent_id, comment_id, tokens_used
- **Constraints**: UNIQUE(post_id, round_number, ai_agent_id)
- **Foreign Keys**: post(id), ai_agent(id), comment(id)

### Indexes Created (13 total)
- **debate_config**: post_id, creator_id
- **debate_state**: post_id, status, last_activity_at (DESC)
- **ai_agent**: model_identifier, person_id, is_active
- **debate_round**: post_id, ai_agent_id, comment_id, (post_id, round_number)

### Design Decisions

1. **Cascade Deletes**: All foreign keys use `ON DELETE CASCADE` to ensure referential integrity when posts, persons, or comments are deleted.

2. **Timestamp Fields**: Used `timestamptz` (timestamp with timezone) for all timestamp fields, consistent with Lemmy's existing schema.

3. **Array Type**: Used PostgreSQL's native `text[]` array type for storing multiple AI model identifiers in debate_config.

4. **JSONB Metadata**: Added flexible `metadata` field in debate_state for future extensibility without schema changes.

5. **Index Strategy**: 
   - Foreign key columns for join performance
   - Status and activity fields for filtering active debates
   - Composite index on (post_id, round_number) for round queries

### Requirements Satisfied
- ✅ Requirement 1.4: Store debate configuration with post
- ✅ Requirement 2.2: AI agent representation with person accounts
- ✅ Requirement 2.3: Bot account flag support (uses existing person.bot_account)

### Next Steps
To apply this migration to a database:
```bash
# Set database URL
export LEMMY_DATABASE_URL="postgresql://lemmy:password@localhost:5432/lemmy"

# Run migrations
cargo run --bin lemmy_diesel_utils
```

### Testing Notes
- Migration syntax validated against existing Lemmy migration patterns
- Foreign key references verified against schema.rs
- Index naming follows Lemmy conventions (idx_tablename_columnname)
- Rollback migration properly reverses all changes in correct order

### Compatibility
- PostgreSQL 12+ (uses JSONB, array types, timestamptz)
- Compatible with Lemmy's existing schema structure
- No breaking changes to existing tables
