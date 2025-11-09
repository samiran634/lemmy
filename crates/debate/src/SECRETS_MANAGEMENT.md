# Secrets Management for AI Debate System

## Overview

The AI Debate System requires an OpenRouter API key to function. This key is sensitive and must be protected from exposure in logs, API responses, or debug output.

## Storage Options

The OpenRouter API key can be stored in two locations, with the following priority:

1. **Database (Recommended)**: Stored in the `secret` table's `openrouter_api_key` column
2. **Configuration File (Fallback)**: Stored in the `debate.openrouter_api_key` setting

### Database Storage (Recommended)

Storing the API key in the database provides better security because:
- The key is not stored in plain text configuration files
- The key can be updated without restarting the server
- The key is stored using Lemmy's `SensitiveString` type which provides additional protection

To set the API key in the database, use the provided functions:

```rust
use lemmy_debate::config::{set_openrouter_api_key, clear_openrouter_api_key};

// Set the API key
set_openrouter_api_key(&context, "sk-or-v1-...".to_string()).await?;

// Clear the API key
clear_openrouter_api_key(&context).await?;
```

### Configuration File Storage (Fallback)

If no API key is found in the database, the system will fall back to the configuration file:

```hjson
{
  debate: {
    openrouter_api_key: "sk-or-v1-..."
  }
}
```

**Warning**: Configuration files may be committed to version control or exposed in backups. Use database storage for production deployments.

## Retrieval

The debate system automatically retrieves the API key using the secure retrieval functions:

```rust
use lemmy_debate::config::get_openrouter_api_key;

// Get API key (checks database first, then config)
let api_key = get_openrouter_api_key(&context).await?;
```

## Security Measures

The following security measures are in place to protect the API key:

1. **No Debug Output**: The `OpenRouterClient` has a custom `Debug` implementation that redacts the API key
2. **SensitiveString Type**: The database stores the key using `SensitiveString` which prevents accidental logging
3. **No API Exposure**: The API key is never included in API responses
4. **Validation**: The configuration validation warns if the API key format appears invalid

## Best Practices

1. **Use Database Storage**: Always store the API key in the database for production deployments
2. **Rotate Keys**: Regularly rotate your OpenRouter API key
3. **Monitor Usage**: Track API usage to detect unauthorized access
4. **Restrict Access**: Limit database access to the `secret` table
5. **Environment Variables**: For development, consider using environment variables instead of config files

## Migration

When upgrading to this version, the system will automatically:
1. Check for an API key in the database
2. Fall back to the configuration file if not found
3. Continue to function with either storage method

To migrate from config file to database storage:
1. Retrieve your current API key from the config file
2. Use `set_openrouter_api_key()` to store it in the database
3. Remove the key from your config file
4. Restart the server to verify it works
