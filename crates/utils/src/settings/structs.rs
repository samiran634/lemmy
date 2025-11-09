use super::pictrs_placeholder_url;
use doku::Document;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::{
  env,
  net::{IpAddr, Ipv4Addr},
};
use url::Url;

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
  /// settings related to the postgresql database
  pub database: DatabaseConfig,
  /// Pictrs image server configuration.
  #[default(Some(Default::default()))]
  pub(crate) pictrs: Option<PictrsConfig>,
  /// Email sending configuration. All options except login/password are mandatory
  #[doku(example = "Some(Default::default())")]
  pub email: Option<EmailConfig>,
  /// Parameters for automatic configuration of new instance (only used at first start)
  #[doku(example = "Some(Default::default())")]
  pub setup: Option<SetupConfig>,
  /// the domain name of your instance (mandatory)
  #[default("unset")]
  #[doku(example = "example.com")]
  pub hostname: String,
  /// Address where lemmy should listen for incoming requests
  #[default(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
  #[doku(as = "String")]
  pub bind: IpAddr,
  /// Port where lemmy should listen for incoming requests
  #[default(8536)]
  pub port: u16,
  /// Whether the site is available over TLS. Needs to be true for federation to work.
  #[default(true)]
  pub tls_enabled: bool,
  /// Set the URL for opentelemetry exports. If you do not have an opentelemetry collector, do not
  /// set this option
  #[doku(skip)]
  pub opentelemetry_url: Option<Url>,
  pub federation: FederationWorkerConfig,
  // Prometheus configuration.
  #[doku(example = "Some(Default::default())")]
  pub prometheus: Option<PrometheusConfig>,
  /// AI Debate system configuration
  #[doku(example = "Some(Default::default())")]
  pub debate: Option<DebateConfig>,
  /// Sets a response Access-Control-Allow-Origin CORS header. Can also be set via environment:
  /// `LEMMY_CORS_ORIGIN=example.org,site.com`
  /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin
  #[doku(example = "lemmy.tld")]
  cors_origin: Vec<String>,
  /// Print logs in JSON format. You can also disable ANSI colors in logs with env var `NO_COLOR`.
  pub json_logging: bool,
}

impl Settings {
  pub fn cors_origin(&self) -> Vec<String> {
    env::var("LEMMY_CORS_ORIGIN")
      .ok()
      .map(|e| e.split(',').map(ToString::to_string).collect())
      .unwrap_or(self.cors_origin.clone())
  }
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
pub struct PictrsConfig {
  /// Address where pictrs is available (for image hosting)
  #[default(pictrs_placeholder_url())]
  #[doku(example = "http://localhost:8080")]
  pub url: Url,

  /// Set a custom pictrs API key. ( Required for deleting images )
  pub api_key: Option<String>,

  /// Specifies how to handle remote images, so that users don't have to connect directly to remote
  /// servers.
  #[default(PictrsImageMode::ProxyAllImages)]
  pub image_mode: PictrsImageMode,

  /// Allows bypassing proxy for specific image hosts when using ProxyAllImages.
  ///
  /// imgur.com is bypassed by default to avoid rate limit errors. When specifying any bypass
  /// in the config, this default is ignored and you need to list imgur explicitly. To proxy imgur
  /// requests, specify a noop bypass list, eg `proxy_bypass_domains ["example.org"]`.
  #[default(vec!["i.imgur.com".to_string()])]
  #[doku(example = "i.imgur.com")]
  pub proxy_bypass_domains: Vec<String>,

  /// Timeout for uploading images to pictrs (in seconds)
  #[default(30)]
  pub upload_timeout: u64,

  /// Resize post thumbnails to this maximum width/height.
  #[default(512)]
  pub max_thumbnail_size: u32,

  /// Maximum size for user avatar, community icon and site icon. Larger images are downscaled.
  #[default(512)]
  pub max_avatar_size: u32,

  /// Maximum size for user, community and site banner. Larger images are downscaled.
  #[default(1024)]
  pub max_banner_size: u32,

  /// Maximum size for other uploads (e.g. post images or markdown embed images). Larger
  /// images are downscaled.
  #[doku(example = "1024")]
  pub max_upload_size: Option<u32>,

  /// Whether users can upload videos as post image or markdown embed.
  #[default(true)]
  pub allow_video_uploads: bool,

  /// Prevent users from uploading images for posts or embedding in markdown. Avatars, icons and
  /// banners can still be uploaded.
  #[default(false)]
  pub image_upload_disabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, Document, PartialEq)]
pub enum PictrsImageMode {
  /// Leave images unchanged, don't generate any local thumbnails for post urls. Instead the
  /// Opengraph image is directly returned as thumbnail
  None,
  /// Generate thumbnails for external post urls and store them persistently in pict-rs. This
  /// ensures that they can be reliably retrieved and can be resized using pict-rs APIs. However
  /// it also increases storage usage.
  ///
  /// This behaviour matches Lemmy 0.18.
  StoreLinkPreviews,
  /// If enabled, all images from remote domains are rewritten to pass through
  /// `/api/v4/image/proxy`, including embedded images in markdown. Images are stored temporarily
  /// in pict-rs for caching. This improves privacy as users don't expose their IP to untrusted
  /// servers, and decreases load on other servers. However it increases bandwidth use for the
  /// local server.
  ///
  /// Requires pict-rs 0.5
  #[default]
  ProxyAllImages,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
pub struct DatabaseConfig {
  /// Configure the database by specifying URI pointing to a postgres instance. This parameter can
  /// also be set by environment variable `LEMMY_DATABASE_URL`.
  ///
  /// For an explanation of how to use connection URIs, see PostgreSQL's documentation:
  /// https://www.postgresql.org/docs/current/libpq-connect.html#id-1.7.3.8.3.6
  #[default("postgres://lemmy:password@localhost:5432/lemmy")]
  #[doku(example = "postgresql:///lemmy?user=lemmy&host=/var/run/postgresql")]
  pub(crate) connection: String,

  /// Maximum number of active sql connections
  ///
  /// A high value here can result in errors "could not resize shared memory segment". In this case
  /// it is necessary to increase shared memory size in Docker: https://stackoverflow.com/a/56754077
  #[default(30)]
  pub pool_size: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone, Document, SmartDefault)]
#[serde(default, deny_unknown_fields)]
pub struct EmailConfig {
  /// https://docs.rs/lettre/0.11.14/lettre/transport/smtp/struct.AsyncSmtpTransport.html#method.from_url
  #[default("smtp://localhost:25")]
  #[doku(example = "smtps://user:pass@hostname:port")]
  pub connection: String,
  /// Address to send emails from, eg "noreply@your-instance.com"
  #[doku(example = "noreply@example.com")]
  pub smtp_from_address: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, Document)]
#[serde(default, deny_unknown_fields)]
pub struct SetupConfig {
  /// Username for the admin user
  #[doku(example = "admin")]
  pub admin_username: String,
  /// Password for the admin user. It must be between 10 and 60 characters.
  #[doku(example = "tf6HHDS4RolWfFhk4Rq9")]
  pub admin_password: String,
  /// Name of the site, can be changed later. Maximum 20 characters.
  #[doku(example = "My Lemmy Instance")]
  pub site_name: String,
  /// Email for the admin user (optional, can be omitted and set later through the website)
  #[doku(example = "user@example.com")]
  pub admin_email: Option<String>,
  /// By default a new Lemmy instance gets populated with data from the most popular communities.
  /// Set this to true to start with an empty instance instead.
  pub no_default_data: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
pub struct PrometheusConfig {
  // Address that the Prometheus metrics will be served on.
  #[default(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
  #[doku(example = "127.0.0.1")]
  pub bind: IpAddr,
  // Port that the Prometheus metrics will be served on.
  #[default(10002)]
  #[doku(example = "10002")]
  pub port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
// named federation"worker"config to disambiguate from the activitypub library configuration
pub struct FederationWorkerConfig {
  /// Limit to the number of concurrent outgoing federation requests per target instance.
  /// Set this to a higher value than 1 (e.g. 6) only if you have a huge instance (>10 activities
  /// per second) and if a receiving instance is not keeping up.
  #[default(1)]
  pub concurrent_sends_per_instance: i8,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
pub struct DebateConfig {
  /// OpenRouter API key for accessing AI models
  #[doku(example = "sk-or-v1-...")]
  pub openrouter_api_key: Option<String>,

  /// OpenRouter API base URL (optional, defaults to https://openrouter.ai/api/v1)
  #[doku(example = "https://openrouter.ai/api/v1")]
  pub openrouter_base_url: Option<String>,

  /// Maximum number of concurrent debates to process
  #[default(Some(5))]
  pub max_concurrent_debates: Option<i32>,

  /// Interval in seconds for polling active debates
  #[default(Some(10))]
  pub poll_interval_seconds: Option<u64>,

  /// Maximum number of rounds per debate
  #[default(Some(10))]
  pub max_rounds_per_debate: Option<i32>,

  /// Maximum tokens per AI response
  #[default(Some(1000))]
  pub max_tokens_per_response: Option<i32>,

  /// Request timeout in seconds
  #[default(Some(60))]
  pub request_timeout_seconds: Option<u64>,

  /// Maximum API calls per minute (rate limiting)
  #[default(Some(60))]
  pub max_api_calls_per_minute: Option<i32>,

  /// Default AI models to use if user doesn't specify
  #[default(vec![
    "openai/gpt-4-turbo".to_string(),
    "anthropic/claude-3-opus".to_string(),
    "google/gemini-pro".to_string()
  ])]
  pub default_models: Vec<String>,
}

impl DebateConfig {
  /// Validate the debate configuration and log warnings for issues
  pub fn validate(&self) -> Result<(), String> {
    // Validate API key format if present
    if let Some(ref api_key) = self.openrouter_api_key {
      if api_key.is_empty() {
        return Err("OpenRouter API key cannot be empty".to_string());
      }
      if !api_key.starts_with("sk-or-") {
        tracing::warn!(
          "OpenRouter API key does not start with 'sk-or-'. This may indicate an invalid key format."
        );
      }
    } else {
      tracing::warn!("No OpenRouter API key configured. Debate system will not function without an API key.");
    }

    // Validate base URL if present
    if let Some(ref base_url) = self.openrouter_base_url {
      if base_url.is_empty() {
        return Err("OpenRouter base URL cannot be empty".to_string());
      }
      if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
        return Err(format!("OpenRouter base URL must start with http:// or https://: {}", base_url));
      }
    }

    // Validate limits are within reasonable ranges
    if let Some(max_concurrent) = self.max_concurrent_debates {
      if max_concurrent < 1 {
        return Err("max_concurrent_debates must be at least 1".to_string());
      }
      if max_concurrent > 100 {
        tracing::warn!(
          "max_concurrent_debates is set to {}. This is very high and may cause performance issues.",
          max_concurrent
        );
      }
    }

    if let Some(poll_interval) = self.poll_interval_seconds {
      if poll_interval < 1 {
        return Err("poll_interval_seconds must be at least 1".to_string());
      }
      if poll_interval > 300 {
        tracing::warn!(
          "poll_interval_seconds is set to {}. This is very high and debates may be slow to start.",
          poll_interval
        );
      }
    }

    if let Some(max_rounds) = self.max_rounds_per_debate {
      if max_rounds < 1 {
        return Err("max_rounds_per_debate must be at least 1".to_string());
      }
      if max_rounds > 50 {
        tracing::warn!(
          "max_rounds_per_debate is set to {}. This is very high and may result in very long debates.",
          max_rounds
        );
      }
    }

    if let Some(max_tokens) = self.max_tokens_per_response {
      if max_tokens < 10 {
        return Err("max_tokens_per_response must be at least 10".to_string());
      }
      if max_tokens > 4000 {
        tracing::warn!(
          "max_tokens_per_response is set to {}. This is very high and may result in high API costs.",
          max_tokens
        );
      }
    }

    if let Some(timeout) = self.request_timeout_seconds {
      if timeout < 5 {
        return Err("request_timeout_seconds must be at least 5".to_string());
      }
      if timeout > 300 {
        tracing::warn!(
          "request_timeout_seconds is set to {}. This is very high and may cause long waits.",
          timeout
        );
      }
    }

    if let Some(max_calls) = self.max_api_calls_per_minute {
      if max_calls < 1 {
        return Err("max_api_calls_per_minute must be at least 1".to_string());
      }
      if max_calls > 1000 {
        tracing::warn!(
          "max_api_calls_per_minute is set to {}. This is very high and may exceed API rate limits.",
          max_calls
        );
      }
    }

    // Validate default models
    if self.default_models.is_empty() {
      return Err("default_models cannot be empty. At least one model must be specified.".to_string());
    }

    for model in &self.default_models {
      if model.is_empty() {
        return Err("default_models contains an empty model identifier".to_string());
      }
      if !model.contains('/') {
        tracing::warn!(
          "Model identifier '{}' does not contain a '/'. Valid format is 'provider/model-name'.",
          model
        );
      }
    }

    Ok(())
  }

  /// Get the OpenRouter base URL, using default if not configured
  pub fn get_base_url(&self) -> String {
    self.openrouter_base_url
      .clone()
      .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string())
  }

  /// Get the maximum concurrent debates, using default if not configured
  pub fn get_max_concurrent_debates(&self) -> i32 {
    self.max_concurrent_debates.unwrap_or(5)
  }

  /// Get the poll interval, using default if not configured
  pub fn get_poll_interval_seconds(&self) -> u64 {
    self.poll_interval_seconds.unwrap_or(10)
  }

  /// Get the maximum rounds per debate, using default if not configured
  pub fn get_max_rounds_per_debate(&self) -> i32 {
    self.max_rounds_per_debate.unwrap_or(10)
  }

  /// Get the maximum tokens per response, using default if not configured
  pub fn get_max_tokens_per_response(&self) -> i32 {
    self.max_tokens_per_response.unwrap_or(1000)
  }

  /// Get the request timeout, using default if not configured
  pub fn get_request_timeout_seconds(&self) -> u64 {
    self.request_timeout_seconds.unwrap_or(60)
  }

  /// Get the maximum API calls per minute, using default if not configured
  pub fn get_max_api_calls_per_minute(&self) -> i32 {
    self.max_api_calls_per_minute.unwrap_or(60)
  }
}
