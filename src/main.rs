use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use datadog_api_client::datadog;
use datadog_api_client::datadog::APIKey;
use datadog_api_client::datadogV1::api_logs::LogsAPI;
use datadog_api_client::datadogV1::model::LogsListRequest;
use datadog_api_client::datadogV1::model::LogsListRequestTime;
use datadog_api_client::datadogV1::model::LogsSort;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Deserialize, Serialize, Debug, Default)]
struct Config {
    api_key: Option<String>,
    app_key: Option<String>,
    site: Option<String>,
}

impl Config {
    fn load() -> Self {
        // Load from config file first
        let config_path = Self::config_path();
        let mut config = if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => toml::from_str(&content).unwrap_or_default(),
                Err(_) => Config::default(),
            }
        } else {
            Config::default()
        };

        // Env vars override config file
        if let Ok(key) = std::env::var("DD_API_KEY") {
            config.api_key = Some(key);
        }
        if let Ok(key) = std::env::var("DD_APP_KEY") {
            config.app_key = Some(key);
        }
        if let Ok(site) = std::env::var("DD_SITE") {
            config.site = Some(site);
        }

        config
    }

    fn config_path() -> PathBuf {
        // Use ~/.config on all platforms for consistency
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".config");
        path.push("ddlogs");
        path.push("config.toml");
        path
    }
}

#[derive(Parser, Debug)]
#[command(name = "ddlogs")]
#[command(about = "Tail logs from Datadog", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Follow mode - continuously poll for new logs
    #[arg(short, long)]
    follow: bool,

    /// Filter by service
    #[arg(long)]
    service: Option<String>,

    /// Filter by source
    #[arg(long)]
    source: Option<String>,

    /// Filter by host
    #[arg(long)]
    host: Option<String>,

    /// Raw Datadog query string
    #[arg(short, long)]
    query: Option<String>,

    /// Number of logs to retrieve
    #[arg(short, long, default_value = "100")]
    limit: i32,

    /// Poll interval in seconds for follow mode (default: 12s to respect Datadog's 300 req/hour limit)
    #[arg(long, default_value = "12")]
    interval: u64,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Configure ddlogs with API credentials and site
    Configure,
}

#[derive(Error, Debug)]
enum DdLogsError {
    #[error("Datadog API error: {0}")]
    DatadogError(String),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Missing API credentials. Set DD_API_KEY and DD_APP_KEY environment variables")]
    MissingCredentials,

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("TOML serialization error: {0}")]
    TomlError(#[from] toml::ser::Error),
}

fn build_query(args: &Args) -> String {
    let mut parts = Vec::new();

    if let Some(service) = &args.service {
        parts.push(format!("service:{}", service));
    }

    if let Some(source) = &args.source {
        parts.push(format!("source:{}", source));
    }

    if let Some(host) = &args.host {
        parts.push(format!("host:{}", host));
    }

    if let Some(query) = &args.query {
        parts.push(query.clone());
    }

    if parts.is_empty() {
        "*".to_string()
    } else {
        parts.join(" ")
    }
}

fn create_api(config: &Config) -> LogsAPI {
    let mut configuration = datadog::Configuration::new();

    // Set API keys
    if let Some(api_key) = &config.api_key {
        configuration.set_auth_key(
            "apiKeyAuth",
            APIKey {
                key: api_key.clone(),
                prefix: String::new(),
            },
        );
    }

    if let Some(app_key) = &config.app_key {
        configuration.set_auth_key(
            "appKeyAuth",
            APIKey {
                key: app_key.clone(),
                prefix: String::new(),
            },
        );
    }

    // Set the Datadog site from config
    if let Some(site) = &config.site {
        configuration
            .server_variables
            .insert("site".into(), site.clone());
    }

    LogsAPI::with_config(configuration)
}

async fn fetch_logs(args: &Args, config: &Config) -> Result<(), DdLogsError> {
    let api = create_api(config);
    let query_str = build_query(args);

    // Default to last 1 hour
    let now = Utc::now();
    let one_hour_ago = now - Duration::hours(1);

    let time = LogsListRequestTime::new(one_hour_ago, now);

    let mut body = LogsListRequest::new(time)
        .query(query_str)
        .sort(LogsSort::TIME_ASCENDING)
        .limit(args.limit);

    // If no filters provided, use a wildcard query
    if args.service.is_none()
        && args.source.is_none()
        && args.host.is_none()
        && args.query.is_none()
    {
        body = body.query("*".to_string());
    }

    let response = api
        .list_logs(body)
        .await
        .map_err(|e| DdLogsError::DatadogError(format!("{:#?}", e)))?;

    // Output each log as a single line of JSON
    if let Some(logs) = response.logs {
        for log in logs {
            let json = serde_json::to_string(&log)?;
            println!("{}", json);
        }
    }

    Ok(())
}

async fn follow_logs(args: &Args, config: &Config) -> Result<(), DdLogsError> {
    let api = create_api(config);
    let query_str = build_query(args);

    // First, fetch recent logs (last hour) to show initial state
    let now = Utc::now();
    let one_hour_ago = now - Duration::hours(1);
    let time = LogsListRequestTime::new(one_hour_ago, now);

    let mut body = LogsListRequest::new(time)
        .query(query_str.clone())
        .sort(LogsSort::TIME_ASCENDING)
        .limit(args.limit);

    if args.service.is_none()
        && args.source.is_none()
        && args.host.is_none()
        && args.query.is_none()
    {
        body = body.query("*".to_string());
    }

    let initial_response = api
        .list_logs(body)
        .await
        .map_err(|e| DdLogsError::DatadogError(format!("{:#?}", e)))?;

    // Output initial logs and track last timestamp
    let mut last_timestamp = now;
    if let Some(ref logs) = initial_response.logs {
        for log in logs {
            if let Some(content) = &log.content
                && let Some(timestamp) = content.timestamp
            {
                last_timestamp = timestamp;
            }
            let json = serde_json::to_string(&log)?;
            println!("{}", json);
        }
    }

    // Now poll forward from the last timestamp

    loop {
        let now = Utc::now();
        let time = LogsListRequestTime::new(last_timestamp, now);

        let mut body = LogsListRequest::new(time)
            .query(query_str.clone())
            .sort(LogsSort::TIME_ASCENDING)
            .limit(args.limit);

        // If no filters provided, use a wildcard query
        if args.service.is_none()
            && args.source.is_none()
            && args.host.is_none()
            && args.query.is_none()
        {
            body = body.query("*".to_string());
        }

        let response = api
            .list_logs(body)
            .await
            .map_err(|e| DdLogsError::DatadogError(format!("{:#?}", e)))?;

        // Output each log as a single line of JSON
        if let Some(ref logs) = response.logs {
            if logs.is_empty() {
                last_timestamp = now;
            } else {
                for log in logs {
                    // Update last_timestamp to the latest log timestamp
                    if let Some(content) = &log.content
                        && let Some(timestamp) = content.timestamp
                    {
                        last_timestamp = timestamp;
                    }

                    let json = serde_json::to_string(&log)?;
                    println!("{}", json);
                }
            }
        } else {
            // Update timestamp if no logs were found
            last_timestamp = now;
        }

        // Sleep for the specified interval
        tokio::time::sleep(tokio::time::Duration::from_secs(args.interval)).await;
    }
}

fn configure() -> Result<(), DdLogsError> {
    println!("Configure ddlogs");
    println!();

    // Prompt for API key
    print!("Datadog API Key: ");
    io::stdout().flush()?;
    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    // Prompt for App key
    print!("Datadog Application Key: ");
    io::stdout().flush()?;
    let mut app_key = String::new();
    io::stdin().read_line(&mut app_key)?;
    let app_key = app_key.trim().to_string();

    // Prompt for site with default
    print!("Datadog Site [datadoghq.com]: ");
    io::stdout().flush()?;
    let mut site = String::new();
    io::stdin().read_line(&mut site)?;
    let site = site.trim();
    let site = if site.is_empty() {
        "datadoghq.com".to_string()
    } else {
        site.to_string()
    };

    // Create config
    let config = Config {
        api_key: Some(api_key),
        app_key: Some(app_key),
        site: Some(site),
    };

    // Create config directory if it doesn't exist
    let config_path = Config::config_path();
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write config file
    let toml_string = toml::to_string_pretty(&config)?;
    std::fs::write(&config_path, toml_string)?;

    println!();
    println!("Configuration saved to {}", config_path.display());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), DdLogsError> {
    let args = Args::parse();

    // Handle configure subcommand
    if let Some(Commands::Configure) = args.command {
        return configure();
    }

    let config = Config::load();

    // Verify credentials exist
    if config.api_key.is_none() || config.app_key.is_none() {
        return Err(DdLogsError::MissingCredentials);
    }

    if args.follow {
        follow_logs(&args, &config).await?;
    } else {
        fetch_logs(&args, &config).await?;
    }

    Ok(())
}
