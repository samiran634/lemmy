use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error};

/// OpenRouter API client for communicating with AI models
#[derive(Clone)]
pub struct OpenRouterClient {
  api_key: String,
  base_url: String,
  http_client: Client,
}

// Custom Debug implementation that doesn't expose the API key
impl std::fmt::Debug for OpenRouterClient {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OpenRouterClient")
      .field("api_key", &"[REDACTED]")
      .field("base_url", &self.base_url)
      .field("http_client", &"<Client>")
      .finish()
  }
}

/// Request structure for OpenRouter API
#[derive(Debug, Clone, Serialize)]
pub struct OpenRouterRequest {
  pub model: String,
  pub messages: Vec<Message>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_tokens: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub temperature: Option<f32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub top_p: Option<f32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub frequency_penalty: Option<f32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub presence_penalty: Option<f32>,
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
  pub role: String,
  pub content: String,
}

impl Message {
  pub fn system(content: impl Into<String>) -> Self {
    Self {
      role: "system".to_string(),
      content: content.into(),
    }
  }

  pub fn user(content: impl Into<String>) -> Self {
    Self {
      role: "user".to_string(),
      content: content.into(),
    }
  }

  pub fn assistant(content: impl Into<String>) -> Self {
    Self {
      role: "assistant".to_string(),
      content: content.into(),
    }
  }
}

/// Response structure from OpenRouter API
#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterResponse {
  pub id: String,
  pub model: String,
  pub choices: Vec<Choice>,
  #[serde(default)]
  pub usage: Option<Usage>,
}

/// A single choice in the response
#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
  pub index: u32,
  pub message: Message,
  pub finish_reason: Option<String>,
}

/// Token usage statistics
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
  pub prompt_tokens: u32,
  pub completion_tokens: u32,
  pub total_tokens: u32,
}

/// Error response from OpenRouter API
#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterError {
  pub error: ErrorDetail,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorDetail {
  pub message: String,
  #[serde(rename = "type")]
  pub error_type: Option<String>,
  pub code: Option<String>,
}

impl OpenRouterClient {
  /// Create a new OpenRouter client
  pub fn new(api_key: String, base_url: Option<String>) -> LemmyResult<Self> {
    if api_key.is_empty() {
      return Err(LemmyErrorType::Unknown("OpenRouter API key is required".to_string()).into());
    }

    let http_client = Client::builder()
      .timeout(Duration::from_secs(60))
      .build()
      .map_err(|e| {
        LemmyErrorType::Unknown(format!("Failed to create HTTP client: {}", e)).into()
      })?;

    Ok(Self {
      api_key,
      base_url: base_url.unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string()),
      http_client,
    })
  }

  /// Build authorization headers for API requests
  fn build_headers(&self) -> header::HeaderMap {
    let mut headers = header::HeaderMap::new();

    // Add authorization header
    let auth_value = format!("Bearer {}", self.api_key);
    if let Ok(header_value) = header::HeaderValue::from_str(&auth_value) {
      headers.insert(header::AUTHORIZATION, header_value);
    }

    // Add content type
    headers.insert(
      header::CONTENT_TYPE,
      header::HeaderValue::from_static("application/json"),
    );

    // Add HTTP-Referer for OpenRouter (optional but recommended)
    if let Ok(referer) = header::HeaderValue::from_static("https://lemmy.ml") {
      headers.insert(header::REFERER, referer);
    }

    headers
  }

  /// Generate a response from an AI model via OpenRouter API
  ///
  /// This method includes retry logic with exponential backoff and proper error handling.
  pub async fn generate_response(
    &self,
    request: OpenRouterRequest,
  ) -> LemmyResult<OpenRouterResponse> {
    let max_retries = 3;
    let mut retry_count = 0;
    let mut last_error: Option<LemmyError> = None;

    while retry_count < max_retries {
      match self.try_generate_response(&request).await {
        Ok(response) => {
          debug!(
            "Successfully generated response from model: {}",
            request.model
          );
          return Ok(response);
        }
        Err(e) => {
          retry_count += 1;
          last_error = Some(e);

          if retry_count < max_retries {
            // Exponential backoff: 1s, 2s, 4s
            let delay_secs = 2_u64.pow(retry_count - 1);
            debug!(
              "API call failed, retrying in {} seconds (attempt {}/{})",
              delay_secs, retry_count, max_retries
            );
            tokio::time::sleep(Duration::from_secs(delay_secs)).await;
          }
        }
      }
    }

    // All retries exhausted
    error!(
      "Failed to generate response after {} retries",
      max_retries
    );
    Err(
      last_error.unwrap_or_else(|| {
        LemmyErrorType::Unknown("Failed to generate response".to_string()).into()
      }),
    )
  }

  /// Internal method to attempt a single API call
  async fn try_generate_response(
    &self,
    request: &OpenRouterRequest,
  ) -> LemmyResult<OpenRouterResponse> {
    let url = format!("{}/chat/completions", self.base_url);
    let headers = self.build_headers();

    debug!("Sending request to OpenRouter API: {}", url);

    let response = self
      .http_client
      .post(&url)
      .headers(headers)
      .json(request)
      .send()
      .await
      .map_err(|e| {
        error!("HTTP request failed: {}", e);
        LemmyErrorType::Unknown(format!("OpenRouter API request failed: {}", e)).into()
      })?;

    let status = response.status();
    debug!("Received response with status: {}", status);

    // Handle rate limiting
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
      error!("Rate limit exceeded");
      return Err(LemmyErrorType::TooManyRequests.into());
    }

    // Handle other error status codes
    if !status.is_success() {
      let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
      error!("API error response: {}", error_text);

      // Try to parse as OpenRouter error
      if let Ok(api_error) = serde_json::from_str::<OpenRouterError>(&error_text) {
        return Err(
          LemmyErrorType::Unknown(format!("OpenRouter API error: {}", api_error.error.message))
            .into(),
        );
      }

      return Err(
        LemmyErrorType::Unknown(format!("OpenRouter API error ({}): {}", status, error_text))
          .into(),
      );
    }

    // Parse successful response
    let response_text = response.text().await.map_err(|e| {
      error!("Failed to read response body: {}", e);
      LemmyErrorType::Unknown(format!("Failed to read API response: {}", e)).into()
    })?;

    debug!("Response body: {}", response_text);

    serde_json::from_str::<OpenRouterResponse>(&response_text).map_err(|e| {
      error!("Failed to parse response JSON: {}", e);
      LemmyErrorType::Unknown(format!("Failed to parse API response: {}", e)).into()
    })
  }
}

/// Helper methods for response validation and parsing
impl OpenRouterResponse {
  /// Validate the response structure
  pub fn validate(&self) -> LemmyResult<()> {
    if self.choices.is_empty() {
      return Err(
        LemmyErrorType::Unknown("API response contains no choices".to_string()).into(),
      );
    }

    // Check that at least one choice has content
    let has_content = self
      .choices
      .iter()
      .any(|choice| !choice.message.content.is_empty());

    if !has_content {
      return Err(
        LemmyErrorType::Unknown("API response contains no content".to_string()).into(),
      );
    }

    Ok(())
  }

  /// Extract the generated text from the first choice
  pub fn get_text(&self) -> LemmyResult<String> {
    self.validate()?;

    self
      .choices
      .first()
      .map(|choice| choice.message.content.clone())
      .ok_or_else(|| LemmyErrorType::Unknown("No text in API response".to_string()).into())
  }

  /// Get token usage statistics
  pub fn get_usage(&self) -> Option<&Usage> {
    self.usage.as_ref()
  }

  /// Get total tokens used
  pub fn get_total_tokens(&self) -> Option<u32> {
    self.usage.as_ref().map(|u| u.total_tokens)
  }

  /// Get the finish reason for the first choice
  pub fn get_finish_reason(&self) -> Option<&str> {
    self
      .choices
      .first()
      .and_then(|choice| choice.finish_reason.as_deref())
  }

  /// Check if the response was truncated due to token limits
  pub fn is_truncated(&self) -> bool {
    self
      .get_finish_reason()
      .map(|reason| reason == "length")
      .unwrap_or(false)
  }
}

impl Usage {
  /// Calculate the total cost estimate (in USD) based on token usage
  /// Note: This is a rough estimate and actual costs may vary by model
  pub fn estimate_cost(&self, model: &str) -> f64 {
    // Rough cost estimates per 1M tokens (as of 2024)
    // These are approximate and should be updated based on actual pricing
    let (prompt_cost_per_m, completion_cost_per_m) = if model.contains("gpt-4") {
      (30.0, 60.0) // GPT-4 Turbo approximate pricing
    } else if model.contains("gpt-3.5") {
      (0.5, 1.5) // GPT-3.5 Turbo approximate pricing
    } else if model.contains("claude-3-opus") {
      (15.0, 75.0) // Claude 3 Opus approximate pricing
    } else if model.contains("claude-3-sonnet") {
      (3.0, 15.0) // Claude 3 Sonnet approximate pricing
    } else if model.contains("claude") {
      (8.0, 24.0) // Claude 2 approximate pricing
    } else if model.contains("gemini-pro") {
      (0.5, 1.5) // Gemini Pro approximate pricing
    } else {
      (1.0, 2.0) // Default fallback
    };

    let prompt_cost = (self.prompt_tokens as f64 / 1_000_000.0) * prompt_cost_per_m;
    let completion_cost = (self.completion_tokens as f64 / 1_000_000.0) * completion_cost_per_m;

    prompt_cost + completion_cost
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_message_constructors() {
    let system_msg = Message::system("You are a helpful assistant");
    assert_eq!(system_msg.role, "system");
    assert_eq!(system_msg.content, "You are a helpful assistant");

    let user_msg = Message::user("Hello");
    assert_eq!(user_msg.role, "user");
    assert_eq!(user_msg.content, "Hello");

    let assistant_msg = Message::assistant("Hi there!");
    assert_eq!(assistant_msg.role, "assistant");
    assert_eq!(assistant_msg.content, "Hi there!");
  }

  #[test]
  fn test_response_validation() {
    // Valid response
    let valid_response = OpenRouterResponse {
      id: "test-id".to_string(),
      model: "gpt-4".to_string(),
      choices: vec![Choice {
        index: 0,
        message: Message::assistant("Test response"),
        finish_reason: Some("stop".to_string()),
      }],
      usage: Some(Usage {
        prompt_tokens: 10,
        completion_tokens: 20,
        total_tokens: 30,
      }),
    };

    assert!(valid_response.validate().is_ok());
    assert_eq!(valid_response.get_text().unwrap(), "Test response");
    assert_eq!(valid_response.get_total_tokens(), Some(30));
    assert_eq!(valid_response.get_finish_reason(), Some("stop"));
    assert!(!valid_response.is_truncated());

    // Empty choices
    let empty_response = OpenRouterResponse {
      id: "test-id".to_string(),
      model: "gpt-4".to_string(),
      choices: vec![],
      usage: None,
    };

    assert!(empty_response.validate().is_err());

    // Truncated response
    let truncated_response = OpenRouterResponse {
      id: "test-id".to_string(),
      model: "gpt-4".to_string(),
      choices: vec![Choice {
        index: 0,
        message: Message::assistant("Truncated..."),
        finish_reason: Some("length".to_string()),
      }],
      usage: None,
    };

    assert!(truncated_response.is_truncated());
  }

  #[test]
  fn test_usage_cost_estimation() {
    let usage = Usage {
      prompt_tokens: 1000,
      completion_tokens: 500,
      total_tokens: 1500,
    };

    // Test GPT-4 cost estimation
    let gpt4_cost = usage.estimate_cost("openai/gpt-4-turbo");
    assert!(gpt4_cost > 0.0);

    // Test GPT-3.5 cost estimation
    let gpt35_cost = usage.estimate_cost("openai/gpt-3.5-turbo");
    assert!(gpt35_cost > 0.0);
    assert!(gpt35_cost < gpt4_cost); // GPT-3.5 should be cheaper

    // Test Claude cost estimation
    let claude_cost = usage.estimate_cost("anthropic/claude-3-opus");
    assert!(claude_cost > 0.0);
  }
}
