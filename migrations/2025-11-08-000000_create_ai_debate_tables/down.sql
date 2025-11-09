-- Drop indexes
DROP INDEX IF EXISTS idx_debate_round_round_number;

DROP INDEX IF EXISTS idx_debate_round_comment_id;

DROP INDEX IF EXISTS idx_debate_round_ai_agent_id;

DROP INDEX IF EXISTS idx_debate_round_post_id;

DROP INDEX IF EXISTS idx_ai_agent_is_active;

DROP INDEX IF EXISTS idx_ai_agent_person_id;

DROP INDEX IF EXISTS idx_ai_agent_model_identifier;

DROP INDEX IF EXISTS idx_debate_state_last_activity;

DROP INDEX IF EXISTS idx_debate_state_status;

DROP INDEX IF EXISTS idx_debate_state_post_id;

DROP INDEX IF EXISTS idx_debate_config_creator_id;

DROP INDEX IF EXISTS idx_debate_config_post_id;

-- Drop tables in reverse order (respecting foreign key dependencies)
DROP TABLE IF EXISTS debate_round;

DROP TABLE IF EXISTS ai_agent;

DROP TABLE IF EXISTS debate_state;

DROP TABLE IF EXISTS debate_config;

