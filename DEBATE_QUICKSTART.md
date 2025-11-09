# AI Debate System - Quick Start

## Current Status

✅ All debate system code compiles successfully
✅ Database migrations ready
✅ API endpoints implemented
⚠️ Full server build blocked by aws-lc-sys dependency (requires CMake/C compiler on Windows)

## What Works

The debate system is fully implemented with:
- Database schema for debates, agents, rounds, and state
- API endpoints for creating and controlling debates
- Background worker for processing debates
- OpenRouter integration for AI models
- Metrics and monitoring

## Quick Test (Without Full Build)

Run the test script to verify debate components compile:

```powershell
.\test_debate_simple.ps1
```

## Database Setup

1. Make sure PostgreSQL is running
2. Run migrations:
```bash
diesel migration run
```

## Configuration

Add to your `config/config.hjson`:

```hjson
{
  debate: {
    openrouter_api_key: "sk-or-v1-YOUR_KEY_HERE"
    max_concurrent_debates: 5
    poll_interval_seconds: 10
    max_rounds_per_debate: 10
    max_tokens_per_response: 1000
    request_timeout_seconds: 60
    max_api_calls_per_minute: 60
    default_models: [
      "openai/gpt-4-turbo"
      "anthropic/claude-3-opus"
      "google/gemini-pro"
    ]
  }
}
```

## API Endpoints

Once the server is running:

### Create a Debate
```bash
POST /api/v4/debate/create
{
  "post_id": 1,
  "ai_models": ["openai/gpt-4-turbo", "anthropic/claude-3-opus"],
  "debate_style": "casual",
  "max_rounds": 3
}
```

### Check Status
```bash
GET /api/v4/debate/{post_id}/status
```

### Control Debate
```bash
POST /api/v4/debate/{post_id}/control
{
  "action": "pause" // or "resume", "stop", "add_rounds"
}
```

## Fixing the Build Issue

The aws-lc-sys error is a system dependency issue. To fix:

### Windows
Install build tools:
```powershell
# Install Visual Studio Build Tools
# Or install CMake and NASM separately
choco install cmake nasm
```

### Linux
```bash
sudo apt-get install cmake build-essential
```

### Mac
```bash
brew install cmake
```

## Next Steps

1. Fix the aws-lc-sys build dependency
2. Run `cargo build` to build the full server
3. Start the server with `cargo run`
4. Test the API endpoints
5. Monitor debates in the database

## Files Modified

- `crates/debate/` - New debate system crate
- `crates/db_schema/` - Added debate tables
- `crates/api/api/src/debate/` - API endpoints
- `crates/server/src/lib.rs` - Worker integration
- `migrations/2025-11-08-000000_create_ai_debate_tables/` - Database schema

## Architecture

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ HTTP
       ▼
┌─────────────┐
│ API Routes  │ (create, control, status)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Database   │ (debate_config, debate_state, debate_round)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Worker    │ (polls for active debates)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Orchestrator│ (manages debate rounds)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ OpenRouter  │ (AI model API)
└─────────────┘
```

## Troubleshooting

**Q: Compilation errors?**
A: Run `.\test_debate_simple.ps1` to check individual components

**Q: Database errors?**
A: Make sure migrations are run: `diesel migration run`

**Q: API key not working?**
A: Check it's set in config or database (see SECRETS_MANAGEMENT.md)

**Q: aws-lc-sys build error?**
A: Install CMake and build tools (see "Fixing the Build Issue" above)
