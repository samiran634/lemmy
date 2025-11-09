-- Create debate_config table
CREATE TABLE debate_config (
    id serial PRIMARY KEY,
    post_id int NOT NULL REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    creator_id int NOT NULL REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ai_models text[] NOT NULL,
    debate_style varchar(50) NOT NULL DEFAULT 'casual',
    max_rounds int NOT NULL DEFAULT 3,
    max_tokens_per_response int,
    custom_system_prompt text,
    include_human_comments boolean NOT NULL DEFAULT TRUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (post_id)
);

-- Create debate_state table
CREATE TABLE debate_state (
    id serial PRIMARY KEY,
    post_id int NOT NULL REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    status varchar(20) NOT NULL DEFAULT 'pending',
    current_round int NOT NULL DEFAULT 0,
    total_rounds int NOT NULL,
    last_activity_at timestamptz NOT NULL DEFAULT now(),
    error_message text,
    metadata jsonb,
    UNIQUE (post_id)
);

-- Create ai_agent table
CREATE TABLE ai_agent (
    id serial PRIMARY KEY,
    person_id int NOT NULL REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE,
    model_identifier varchar(100) NOT NULL UNIQUE,
    display_name varchar(100) NOT NULL,
    provider varchar(50) NOT NULL,
    model_version varchar(50),
    avatar_url text,
    is_active boolean NOT NULL DEFAULT TRUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (person_id)
);

-- Create debate_round table
CREATE TABLE debate_round (
    id serial PRIMARY KEY,
    post_id int NOT NULL REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    round_number int NOT NULL,
    ai_agent_id int NOT NULL REFERENCES ai_agent (id) ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id int NOT NULL REFERENCES comment (id) ON UPDATE CASCADE ON DELETE CASCADE,
    tokens_used int,
    api_latency_ms int,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (post_id, round_number, ai_agent_id)
);

-- Create indexes for performance
CREATE INDEX idx_debate_config_post_id ON debate_config (post_id);

CREATE INDEX idx_debate_config_creator_id ON debate_config (creator_id);

CREATE INDEX idx_debate_state_post_id ON debate_state (post_id);

CREATE INDEX idx_debate_state_status ON debate_state (status);

CREATE INDEX idx_debate_state_last_activity ON debate_state (last_activity_at DESC);

CREATE INDEX idx_ai_agent_model_identifier ON ai_agent (model_identifier);

CREATE INDEX idx_ai_agent_person_id ON ai_agent (person_id);

CREATE INDEX idx_ai_agent_is_active ON ai_agent (is_active);

CREATE INDEX idx_debate_round_post_id ON debate_round (post_id);

CREATE INDEX idx_debate_round_ai_agent_id ON debate_round (ai_agent_id);

CREATE INDEX idx_debate_round_comment_id ON debate_round (comment_id);

CREATE INDEX idx_debate_round_round_number ON debate_round (post_id, round_number);

