use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub rate_limit: RateLimitConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default = "ServerConfig::default_log_level")]
    pub log_level: String,
}

impl ServerConfig {
    fn default_log_level() -> String {
        "info".to_string()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "DatabaseConfig::default_max_connections")]
    pub max_connections: u32,
    #[serde(default)]
    pub run_migrations_on_boot: bool,
}

impl DatabaseConfig {
    fn default_max_connections() -> u32 {
        10
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub access_secret: String,
    pub refresh_secret: String,
    #[serde(default = "JwtConfig::default_access_ttl")]
    pub access_ttl_seconds: i64,
    #[serde(default = "JwtConfig::default_refresh_ttl")]
    pub refresh_ttl_seconds: i64,
}

impl JwtConfig {
    fn default_access_ttl() -> i64 {
        15 * 60 // 15 minutes
    }

    fn default_refresh_ttl() -> i64 {
        7 * 24 * 60 * 60 // 7 days
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "RateLimitConfig::default_max_requests")]
    pub max_requests: u32,
    #[serde(default = "RateLimitConfig::default_window_seconds")]
    pub window_seconds: u32,
    /// Only trust the `X-Forwarded-For` header when the app is actually
    /// deployed behind a reverse proxy that overwrites/strips client-sent
    /// values. Defaults to `false` so a client can never spoof its own IP
    /// and bypass rate limiting by rotating the header.
    #[serde(default)]
    pub trust_proxy: bool,
}

impl RateLimitConfig {
    fn default_max_requests() -> u32 {
        100
    }

    fn default_window_seconds() -> u32 {
        60
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    /// Local-disk root that uploaded files are written under. Relative
    /// paths resolve against the process's working directory.
    #[serde(default = "StorageConfig::default_base_path")]
    pub base_path: String,
    /// Hard cap on a single upload, in bytes. Also used to set
    /// `DefaultBodyLimit` on the upload route, so a request over this size
    /// is rejected before the body is even read into memory.
    #[serde(default = "StorageConfig::default_max_upload_bytes")]
    pub max_upload_bytes: usize,
}

impl StorageConfig {
    fn default_base_path() -> String {
        "storage/files".to_string()
    }

    fn default_max_upload_bytes() -> usize {
        20 * 1024 * 1024 // 20 MB
    }
}

impl AppConfig {
    /// Loads config in layers, later layers win:
    /// 1. `config/config.yaml`            (defaults, safe to commit)
    /// 2. `config/config.local.yaml`      (local overrides, gitignored)
    /// 3. environment variables prefixed `APP__` (e.g. `APP__DATABASE__URL`)
    pub fn load() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();

        let builder = config::Config::builder()
            .add_source(config::File::with_name("config/config").required(false))
            .add_source(config::File::with_name("config/config.local").required(false))
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            );

        let cfg = builder.build()?;
        let app_config: AppConfig = cfg.try_deserialize()?;
        Ok(app_config)
    }
}
