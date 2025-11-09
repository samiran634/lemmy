# Run Lemmy Debate System in GitHub Codespaces (FREE)

## Why Codespaces?
- ✅ Free 60 hours/month
- ✅ All build tools pre-installed (CMake, GCC, etc.)
- ✅ No local setup needed
- ✅ Works in browser
- ✅ Full VS Code environment

## Setup Steps (5 minutes)

### 1. Create Codespace

1. Go to your GitHub repo
2. Click "Code" button (green)
3. Click "Codespaces" tab
4. Click "Create codespace on main"
5. Wait for environment to load (~2 min)

### 2. Install PostgreSQL

```bash
# Update packages
sudo apt-get update

# Install PostgreSQL
sudo apt-get install -y postgresql postgresql-contrib

# Start PostgreSQL
sudo service postgresql start

# Create database user
sudo -u postgres createuser -s $USER

# Create database
createdb lemmy
```

### 3. Install Diesel CLI

```bash
cargo install diesel_cli --no-default-features --features postgres
```

### 4. Setup Database

```bash
# Set database URL
export DATABASE_URL=postgres://$(whoami)@localhost/lemmy

# Run migrations
diesel migration run
```

### 5. Configure Debate System

Create `config/config.hjson`:
```hjson
{
  database: {
    connection: "postgres://vscode@localhost/lemmy"
  }
  hostname: "localhost"
  bind: "0.0.0.0"
  port: 8536
  tls_enabled: false
  
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
    ]
  }
}
```

### 6. Build and Run

```bash
# Build (first time takes ~10 min)
cargo build --release

# Run server
cargo run --release
```

### 7. Access Your Instance

Codespaces will show a popup: "Your application running on port 8536 is available"
- Click "Open in Browser"
- Or go to "Ports" tab and click the URL

### 8. Test the Debate API

```bash
# In a new terminal
curl http://localhost:8536/api/v4/debate/metrics
```

## Create Your First Debate

### 1. Create an account
Visit your Codespace URL and sign up

### 2. Create a post
Create a post to debate about

### 3. Get your JWT token
- Login via API or browser
- Copy JWT from browser cookies or API response

### 4. Create debate
```bash
curl -X POST http://localhost:8536/api/v4/debate/create \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "post_id": 1,
    "ai_models": ["openai/gpt-4-turbo", "anthropic/claude-3-opus"],
    "debate_style": "casual",
    "max_rounds": 3
  }'
```

### 5. Check status
```bash
curl http://localhost:8536/api/v4/debate/1/status \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## Tips

### Keep Codespace Running
- Codespaces auto-sleep after 30 min of inactivity
- Keep a terminal open with: `watch -n 60 date`

### Port Forwarding
- Codespaces automatically forwards port 8536
- Make it public: Ports tab → Right-click → Port Visibility → Public

### Save Your Work
- Changes are auto-saved to your GitHub repo
- Commit and push regularly: `git add . && git commit -m "update" && git push`

### Database Persistence
- Database is stored in Codespace
- Survives restarts
- Lost if Codespace is deleted

## Troubleshooting

**PostgreSQL not starting?**
```bash
sudo service postgresql restart
```

**Database connection error?**
```bash
# Check PostgreSQL is running
sudo service postgresql status

# Check database exists
psql -l
```

**Build errors?**
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

**Port already in use?**
```bash
# Kill existing process
pkill lemmy_server
```

## Cost

- **Free tier:** 60 hours/month
- **After free tier:** $0.18/hour
- **Storage:** Free up to 15GB

## Alternative: Use Codespace for Development Only

If you want to save hours:
1. Develop and test in Codespaces
2. Deploy to Railway/Render for production
3. Only use Codespaces when making changes

## Next Steps

Once running in Codespaces:
1. Test all API endpoints
2. Create sample debates
3. Monitor worker logs
4. Check database for results
5. Deploy to production (Railway/Render)

## Production Deployment

After testing in Codespaces, deploy to production:

### Railway (Recommended)
1. Push code to GitHub
2. Connect Railway to your repo
3. Add PostgreSQL addon
4. Set environment variables
5. Deploy automatically

### Render
1. Connect GitHub repo
2. Add PostgreSQL database
3. Configure build/start commands
4. Deploy

Both have free tiers and are much easier than local setup!
