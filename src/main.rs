use posthog::api;
use posthog::api::PostHogClient;
use posthog::cache::DiskCache;
use posthog::cli::{
    ActionCommand, AnnotationCommand, CacheCommand, CaptureCommand, Cli, CohortCommand, Commands,
    DashboardCommand, DefinitionCommand, ErrorCommand, EventDefinitionCommand, ExperimentCommand,
    FlagCommand, InsightCommand, PersonCommand, PropertyDefinitionCommand, QueryCommand,
    SurveyCommand,
};
use posthog::config::{AppConfig, ResolvedAuth};
use posthog::error::AppError;
use posthog::output::{self, OutputFormat};

use clap::Parser;
use colored::Colorize;
use dialoguer::Password;
use serde_json::json;
use std::io::IsTerminal;
use std::time::Instant;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Set up logging — writes to stderr so stdout stays clean for data
    let filter = if cli.verbose {
        "posthog=debug"
    } else {
        &std::env::var("POSTHOG_LOG").unwrap_or_else(|_| "warn".to_string())
    };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_new(filter).unwrap_or_else(|_| EnvFilter::new("warn")))
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    let config = AppConfig::load().unwrap_or_default();
    let output_format = OutputFormat::from_cli(cli.format);

    // Handle non-API commands first
    match &cli.command {
        Commands::Completions { shell } => {
            let mut cmd = <Cli as clap::CommandFactory>::command();
            clap_complete::generate(*shell, &mut cmd, "posthog", &mut std::io::stdout());
            return;
        }
        Commands::Auth { command } => {
            if let Err(e) = handle_auth(command, &cli, output_format).await {
                e.print_and_exit();
            }
            return;
        }
        Commands::Cache { command } => {
            handle_cache(command, &config);
            return;
        }
        _ => {}
    }

    // All remaining commands need authentication
    let auth = match ResolvedAuth::resolve(
        cli.token.as_deref(),
        cli.host.as_deref(),
        cli.project.as_deref(),
    ) {
        Ok(a) => a,
        Err(e) => e.print_and_exit(),
    };

    let client = match PostHogClient::new(
        auth.host.clone(),
        auth.token.clone(),
        auth.project_id.clone(),
        cli.timeout,
        cli.retry,
    ) {
        Ok(c) => c,
        Err(e) => e.print_and_exit(),
    };

    // Set up disk cache
    let cache_ttl = config.cache.ttl_secs;
    let cache = DiskCache::new(AppConfig::cache_dir(), cache_ttl);

    let start = Instant::now();
    let result = execute_command(&cli, &client, &cache).await;
    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok((data, cached)) => {
            let meta = json!({
                "cached": cached,
                "duration_ms": duration_ms,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });

            let rendered = output::render(output_format, &data, &meta);
            println!("{rendered}");
        }
        Err(e) => e.print_and_exit(),
    }
}

fn handle_cache(command: &CacheCommand, config: &AppConfig) {
    let cache = DiskCache::new(AppConfig::cache_dir(), config.cache.ttl_secs);
    match command {
        CacheCommand::Clear => match cache.clear() {
            Ok(n) => println!("Cleared {n} cached entries."),
            Err(e) => eprintln!("Failed to clear cache: {e}"),
        },
        CacheCommand::Stats => {
            let (count, size) = cache.stats();
            println!("Cache entries: {count}");
            println!("Cache size: {} KB", size / 1024);
            println!("Cache dir: {}", AppConfig::cache_dir().display());
            println!("Cache TTL: {}s", config.cache.ttl_secs);
        }
    }
}

async fn handle_auth(
    command: &posthog::cli::auth::AuthCommand,
    cli: &Cli,
    format: OutputFormat,
) -> Result<(), AppError> {
    use posthog::cli::auth::AuthCommand;

    let is_json = format == OutputFormat::Json || !std::io::stdout().is_terminal();

    match command {
        AuthCommand::Login { token } => {
            let api_token = if let Some(t) = token {
                t.clone()
            } else {
                Password::new()
                    .with_prompt("Enter your PostHog personal API key (phx_...)")
                    .interact()
                    .map_err(|e| AppError::Config {
                        message: format!("Input error: {e}"),
                    })?
            };

            if api_token.is_empty() {
                return Err(AppError::Validation {
                    message: "API token cannot be empty".into(),
                });
            }

            let host = cli
                .host
                .clone()
                .unwrap_or_else(|| "https://us.posthog.com".to_string());

            let temp_client =
                PostHogClient::new(host.clone(), api_token.clone(), "0".into(), 30, 1)?;
            let me = temp_client.get_me().await.map_err(|_| AppError::Auth {
                message: "Invalid API token".into(),
            })?;

            if let Err(e) = ResolvedAuth::store_token(&api_token) {
                if !is_json {
                    eprintln!(
                        "{} Could not store token in keyring: {e}",
                        "warning:".yellow().bold()
                    );
                    eprintln!("Use POSTHOG_TOKEN env var instead.");
                }
            }

            let mut config = AppConfig::load()?;
            config.host = Some(host);
            config.save()?;

            let name = me
                .get("first_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let email = me
                .get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            if is_json {
                let out = json!({
                    "status": "authenticated",
                    "user": name,
                    "email": email,
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap());
            } else {
                println!(
                    "{} Logged in as {} ({})",
                    "Success!".green().bold(),
                    name,
                    email
                );
            }
        }

        AuthCommand::Logout => {
            ResolvedAuth::delete_token()?;

            if is_json {
                println!(r#"{{"status": "logged_out"}}"#);
            } else {
                println!("Credentials removed from keyring.");
            }
        }

        AuthCommand::Status => {
            match ResolvedAuth::resolve(
                cli.token.as_deref(),
                cli.host.as_deref(),
                cli.project.as_deref(),
            ) {
                Ok(auth) => {
                    let temp_client = PostHogClient::new(
                        auth.host.clone(),
                        auth.token.clone(),
                        auth.project_id.clone(),
                        30,
                        1,
                    )?;
                    let valid = temp_client.get_me().await.is_ok();

                    if is_json {
                        let out = json!({
                            "authenticated": true,
                            "token_source": auth.token_source,
                            "host": auth.host,
                            "project_id": auth.project_id,
                            "token_valid": valid,
                        });
                        println!("{}", serde_json::to_string_pretty(&out).unwrap());
                    } else {
                        println!("Authenticated: yes");
                        println!("Token source: {}", auth.token_source);
                        println!("Host: {}", auth.host);
                        println!("Project ID: {}", auth.project_id);
                        println!(
                            "Token valid: {}",
                            if valid {
                                "yes".green().to_string()
                            } else {
                                "no".red().to_string()
                            }
                        );
                    }
                }
                Err(_) => {
                    if is_json {
                        println!(r#"{{"authenticated": false}}"#);
                    } else {
                        println!("Not authenticated.");
                        println!("Run 'posthog auth login' or set POSTHOG_TOKEN.");
                    }
                }
            }
        }

        AuthCommand::Whoami => {
            let auth = ResolvedAuth::resolve(
                cli.token.as_deref(),
                cli.host.as_deref(),
                cli.project.as_deref(),
            )?;
            let client = PostHogClient::new(
                auth.host,
                auth.token,
                auth.project_id,
                cli.timeout,
                cli.retry,
            )?;
            let me = client.get_me().await?;

            let rendered = output::render_value(format, &me);
            println!("{rendered}");
        }

        AuthCommand::Switch { project_id } => {
            let mut config = AppConfig::load()?;
            config.project_id = Some(project_id.clone());
            config.save()?;

            if is_json {
                let out = json!({"project_id": project_id});
                println!("{}", serde_json::to_string_pretty(&out).unwrap());
            } else {
                println!("Switched to project {project_id}.");
            }
        }
    }

    Ok(())
}

/// Returns (data, cached).
async fn execute_command(
    cli: &Cli,
    client: &PostHogClient,
    cache: &DiskCache,
) -> Result<(serde_json::Value, bool), AppError> {
    // Build a cache key from the command
    let cache_key = build_cache_key(cli, &client.base_url, &client.project_id);

    // Check cache first for read-only commands (not mutations)
    if !cli.no_cache && is_cacheable_command(&cli.command) {
        if let Some(cached_json) = cache.get("posthog", &cache_key) {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&cached_json) {
                tracing::debug!("Serving from cache");
                return Ok((data, true));
            }
        }
    }

    let data = match &cli.command {
        Commands::Query { command } => execute_query(client, command, cli.quiet).await?,
        Commands::Flags { command } => execute_flags(client, command, cli.all_pages, cli.page_size).await?,
        Commands::Insights { command } => execute_insights(client, command, cli.all_pages, cli.page_size).await?,
        Commands::Dashboards { command } => execute_dashboards(client, command, cli.all_pages, cli.page_size).await?,
        Commands::Annotations { command } => execute_annotations(client, command, cli.all_pages, cli.page_size).await?,
        Commands::Capture { command } => execute_capture(client, command).await?,
        Commands::Experiments { command } => execute_experiments(client, command, cli.all_pages, cli.page_size).await?,
        Commands::Surveys { command } => execute_surveys(client, command, cli.all_pages, cli.page_size).await?,
        Commands::Persons { command } => execute_persons(client, command, cli.all_pages, cli.page_size).await?,
        Commands::Cohorts { command } => execute_cohorts(client, command, cli.all_pages, cli.page_size).await?,
        Commands::Errors { command } => execute_errors(client, command, cli.all_pages, cli.page_size).await?,
        Commands::Actions { command } => execute_actions(client, command, cli.all_pages, cli.page_size).await?,
        Commands::Definitions { command } => execute_definitions(client, command).await?,
        Commands::Auth { .. } | Commands::Completions { .. } | Commands::Cache { .. } => {
            unreachable!()
        }
    };

    // Cache the result for cacheable commands
    if !cli.no_cache && is_cacheable_command(&cli.command) {
        if let Ok(json_str) = serde_json::to_string(&data) {
            cache.set("posthog", &cache_key, &json_str);
        }
    }

    Ok((data, false))
}

fn is_cacheable_command(cmd: &Commands) -> bool {
    matches!(
        cmd,
        Commands::Query { command: QueryCommand::Sql { .. } | QueryCommand::Trends { .. } | QueryCommand::Funnels { .. } | QueryCommand::Retention { .. } }
        | Commands::Flags { command: FlagCommand::List { .. } | FlagCommand::Get { .. } }
        | Commands::Insights { command: InsightCommand::List { .. } | InsightCommand::Get { .. } }
        | Commands::Dashboards { command: DashboardCommand::List { .. } | DashboardCommand::Get { .. } }
        | Commands::Annotations { command: AnnotationCommand::List { .. } | AnnotationCommand::Get { .. } }
        | Commands::Experiments { command: ExperimentCommand::List { .. } | ExperimentCommand::Get { .. } | ExperimentCommand::Results { .. } }
        | Commands::Surveys { command: SurveyCommand::List { .. } | SurveyCommand::Get { .. } }
        | Commands::Persons { command: PersonCommand::List { .. } | PersonCommand::Get { .. } }
        | Commands::Cohorts { command: CohortCommand::List | CohortCommand::Get { .. } }
        | Commands::Errors { command: ErrorCommand::List { .. } | ErrorCommand::Get { .. } }
        | Commands::Actions { command: ActionCommand::List | ActionCommand::Get { .. } }
    )
}

fn build_cache_key(cli: &Cli, host: &str, project_id: &str) -> String {
    // Build a deterministic cache key from command args, excluding sensitive tokens
    // Include host + project_id to prevent cross-project cache collisions
    let args: Vec<String> = std::env::args().collect();
    let mut filtered = Vec::new();
    let mut skip_next = false;
    for arg in &args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg == "--token" || arg == "-t" {
            skip_next = true;
            continue;
        }
        if arg.starts_with("phx_") || arg.starts_with("phk_") {
            continue;
        }
        // Also skip host/project flags since we add them explicitly
        if arg == "--host" || arg == "-H" || arg == "--project" || arg == "-p" {
            skip_next = true;
            continue;
        }
        filtered.push(arg.as_str());
    }
    let _ = cli; // used for future extensions
    format!("{}|{}|{}", host, project_id, filtered.join("|"))
}

async fn execute_query(
    client: &PostHogClient,
    command: &QueryCommand,
    quiet: bool,
) -> Result<serde_json::Value, AppError> {
    match command {
        QueryCommand::Sql {
            query,
            file,
            params,
            name: _,
            async_mode,
            wait,
            refresh,
        } => {
            let sql = if let Some(f) = file {
                std::fs::read_to_string(f).map_err(|e| AppError::Validation {
                    message: format!("Failed to read SQL file: {e}"),
                })?
            } else if let Some(q) = query {
                let mut sql = q.clone();
                for p in params {
                    if let Some((key, val)) = p.split_once('=') {
                        sql = sql.replace(&format!("{{{key}}}"), val);
                    }
                }
                sql
            } else {
                return Err(AppError::Validation {
                    message: "Provide a SQL query string or --file".into(),
                });
            };

            if *async_mode {
                // Return immediately with query_id
                api::endpoints::query::hogql(client, &sql, refresh, true).await
            } else if *wait {
                // Submit async, poll until complete
                api::endpoints::query::hogql_wait(client, &sql, refresh, quiet).await
            } else {
                // Default: synchronous blocking query
                api::endpoints::query::hogql(client, &sql, refresh, false).await
            }
        }

        QueryCommand::Trends {
            event,
            interval,
            date_from,
            date_to,
        } => {
            api::endpoints::query::trends(
                client,
                event,
                interval,
                date_from.as_deref(),
                date_to.as_deref(),
            )
            .await
        }

        QueryCommand::Funnels {
            steps,
            date_from,
            date_to,
        } => {
            api::endpoints::query::funnels(
                client,
                steps,
                date_from.as_deref(),
                date_to.as_deref(),
            )
            .await
        }

        QueryCommand::Retention {
            target_event,
            return_event,
            period,
            date_from,
        } => {
            api::endpoints::query::retention(
                client,
                target_event,
                return_event,
                period,
                date_from.as_deref(),
            )
            .await
        }

        QueryCommand::Status { query_id } => {
            api::endpoints::query::query_status(client, query_id).await
        }

        QueryCommand::Result { query_id } => {
            api::endpoints::query::query_result(client, query_id).await
        }

        QueryCommand::Cancel { query_id } => {
            api::endpoints::query::query_cancel(client, query_id).await?;
            Ok(json!({"status": "cancelled", "query_id": query_id}))
        }
    }
}

async fn execute_flags(
    client: &PostHogClient,
    command: &FlagCommand,
    all_pages: bool,
    page_size: u32,
) -> Result<serde_json::Value, AppError> {
    match command {
        FlagCommand::List { active, search } => {
            let mut params: Vec<(&str, &str)> = Vec::new();
            let active_str;
            if *active {
                active_str = "true".to_string();
                params.push(("active", &active_str));
            }
            if let Some(s) = search {
                params.push(("search", s));
            }
            if all_pages {
                client.get_all_pages("feature_flags/", &params, page_size).await
            } else {
                api::endpoints::flags::list(client, *active, search.as_deref()).await
            }
        }
        FlagCommand::Get { key } => api::endpoints::flags::get(client, key).await,
        FlagCommand::Create {
            key,
            name,
            rollout,
            active,
        } => {
            api::endpoints::flags::create(client, key, name.as_deref(), *rollout, *active).await
        }
        FlagCommand::Update {
            key,
            rollout,
            active,
            name,
        } => {
            api::endpoints::flags::update(client, key, *rollout, *active, name.as_deref()).await
        }
        FlagCommand::Delete { key } => {
            api::endpoints::flags::delete(client, key).await?;
            Ok(json!({"status": "deleted", "key": key}))
        }
        FlagCommand::Evaluate { key, distinct_id } => {
            api::endpoints::flags::evaluate(client, key, distinct_id).await
        }
        FlagCommand::EvaluateAll { distinct_id } => {
            api::endpoints::flags::evaluate_all(client, distinct_id).await
        }
    }
}

async fn execute_insights(
    client: &PostHogClient,
    command: &InsightCommand,
    all_pages: bool,
    page_size: u32,
) -> Result<serde_json::Value, AppError> {
    match command {
        InsightCommand::List {
            search,
            saved,
            favorited,
        } => {
            if all_pages {
                let mut params: Vec<(&str, &str)> = Vec::new();
                if let Some(s) = search {
                    params.push(("search", s));
                }
                let saved_str;
                if *saved {
                    saved_str = "true".to_string();
                    params.push(("saved", &saved_str));
                }
                let fav_str;
                if *favorited {
                    fav_str = "true".to_string();
                    params.push(("favorited", &fav_str));
                }
                client.get_all_pages("insights/", &params, page_size).await
            } else {
                api::endpoints::insights::list(client, search.as_deref(), *saved, *favorited).await
            }
        }
        InsightCommand::Get { id, refresh } => {
            api::endpoints::insights::get(client, *id, refresh.as_deref()).await
        }
        InsightCommand::Create { name, query } => {
            api::endpoints::insights::create(client, name, query).await
        }
        InsightCommand::Update { id, name, tags } => {
            api::endpoints::insights::update(client, *id, name.as_deref(), tags.as_deref()).await
        }
        InsightCommand::Delete { id } => {
            api::endpoints::insights::delete(client, *id).await?;
            Ok(json!({"status": "deleted", "id": id}))
        }
    }
}

async fn execute_dashboards(
    client: &PostHogClient,
    command: &DashboardCommand,
    all_pages: bool,
    page_size: u32,
) -> Result<serde_json::Value, AppError> {
    match command {
        DashboardCommand::List { search, pinned } => {
            if all_pages {
                let mut params: Vec<(&str, &str)> = Vec::new();
                if let Some(s) = search {
                    params.push(("search", s));
                }
                let pinned_str;
                if *pinned {
                    pinned_str = "true".to_string();
                    params.push(("pinned", &pinned_str));
                }
                client.get_all_pages("dashboards/", &params, page_size).await
            } else {
                api::endpoints::dashboards::list(client, search.as_deref(), *pinned).await
            }
        }
        DashboardCommand::Get { id } => api::endpoints::dashboards::get(client, *id).await,
        DashboardCommand::Create {
            name,
            description,
            pinned,
        } => {
            api::endpoints::dashboards::create(client, name, description.as_deref(), *pinned).await
        }
        DashboardCommand::Update { id, name, tags } => {
            api::endpoints::dashboards::update(client, *id, name.as_deref(), tags.as_deref()).await
        }
        DashboardCommand::Delete { id } => {
            api::endpoints::dashboards::delete(client, *id).await?;
            Ok(json!({"status": "deleted", "id": id}))
        }
    }
}

async fn execute_annotations(
    client: &PostHogClient,
    command: &AnnotationCommand,
    all_pages: bool,
    page_size: u32,
) -> Result<serde_json::Value, AppError> {
    match command {
        AnnotationCommand::List { search } => {
            if all_pages {
                let mut params: Vec<(&str, &str)> = Vec::new();
                if let Some(s) = search {
                    params.push(("search", s));
                }
                client.get_all_pages("annotations/", &params, page_size).await
            } else {
                api::endpoints::annotations::list(client, search.as_deref()).await
            }
        }
        AnnotationCommand::Get { id } => api::endpoints::annotations::get(client, *id).await,
        AnnotationCommand::Create {
            content,
            date,
            scope,
        } => api::endpoints::annotations::create(client, content, date, scope).await,
        AnnotationCommand::Update { id, content } => {
            api::endpoints::annotations::update(client, *id, content.as_deref()).await
        }
        AnnotationCommand::Delete { id } => {
            api::endpoints::annotations::delete(client, *id).await?;
            Ok(json!({"status": "deleted", "id": id}))
        }
    }
}

async fn execute_capture(
    client: &PostHogClient,
    command: &CaptureCommand,
) -> Result<serde_json::Value, AppError> {
    match command {
        CaptureCommand::Event {
            event,
            distinct_id,
            properties,
        } => {
            api::endpoints::capture::event(client, event, distinct_id, properties.as_deref()).await
        }
        CaptureCommand::Batch { file } => api::endpoints::capture::batch(client, file).await,
        CaptureCommand::Identify { distinct_id, set } => {
            api::endpoints::capture::identify(client, distinct_id, set).await
        }
        CaptureCommand::Group {
            group_type,
            group_key,
            set,
        } => api::endpoints::capture::group(client, group_type, group_key, set).await,
        CaptureCommand::Alias {
            distinct_id,
            alias,
        } => api::endpoints::capture::alias(client, distinct_id, alias).await,
    }
}

async fn execute_experiments(
    client: &PostHogClient,
    command: &ExperimentCommand,
    all_pages: bool,
    page_size: u32,
) -> Result<serde_json::Value, AppError> {
    match command {
        ExperimentCommand::List { status } => {
            if all_pages {
                let mut params: Vec<(&str, &str)> = Vec::new();
                if let Some(s) = status {
                    params.push(("status", s));
                }
                client.get_all_pages("experiments/", &params, page_size).await
            } else {
                api::endpoints::experiments::list(client, status.as_deref()).await
            }
        }
        ExperimentCommand::Get { id } => api::endpoints::experiments::get(client, *id).await,
        ExperimentCommand::Create {
            name,
            feature_flag_key,
            description,
            metrics,
        } => {
            api::endpoints::experiments::create(
                client,
                name,
                feature_flag_key,
                description.as_deref(),
                metrics.as_deref(),
            )
            .await
        }
        ExperimentCommand::Update {
            id,
            name,
            description,
        } => {
            api::endpoints::experiments::update(
                client,
                *id,
                description.as_deref(),
                name.as_deref(),
            )
            .await
        }
        ExperimentCommand::Start { id } => api::endpoints::experiments::start(client, *id).await,
        ExperimentCommand::Stop { id } => api::endpoints::experiments::stop(client, *id).await,
        ExperimentCommand::Results { id } => {
            api::endpoints::experiments::results(client, *id).await
        }
        ExperimentCommand::Delete { id } => {
            api::endpoints::experiments::delete(client, *id).await?;
            Ok(json!({"status": "deleted", "id": id}))
        }
    }
}

async fn execute_surveys(
    client: &PostHogClient,
    command: &SurveyCommand,
    all_pages: bool,
    page_size: u32,
) -> Result<serde_json::Value, AppError> {
    match command {
        SurveyCommand::List { status } => {
            if all_pages {
                let mut params: Vec<(&str, &str)> = Vec::new();
                if let Some(s) = status {
                    params.push(("status", s));
                }
                client.get_all_pages("surveys/", &params, page_size).await
            } else {
                api::endpoints::surveys::list(client, status.as_deref()).await
            }
        }
        SurveyCommand::Get { id } => api::endpoints::surveys::get(client, *id).await,
        SurveyCommand::Create {
            name,
            questions,
            targeting,
        } => {
            api::endpoints::surveys::create(client, name, questions, targeting.as_deref()).await
        }
        SurveyCommand::Update {
            id,
            name,
            end_date,
        } => {
            api::endpoints::surveys::update(
                client,
                *id,
                end_date.as_deref(),
                name.as_deref(),
            )
            .await
        }
        SurveyCommand::Launch { id } => api::endpoints::surveys::launch(client, *id).await,
        SurveyCommand::Stop { id } => api::endpoints::surveys::stop(client, *id).await,
        SurveyCommand::Archive { id } => api::endpoints::surveys::archive(client, *id).await,
        SurveyCommand::Delete { id } => {
            api::endpoints::surveys::delete(client, *id).await?;
            Ok(json!({"status": "deleted", "id": id}))
        }
    }
}

async fn execute_persons(
    client: &PostHogClient,
    command: &PersonCommand,
    all_pages: bool,
    page_size: u32,
) -> Result<serde_json::Value, AppError> {
    match command {
        PersonCommand::List {
            search,
            properties,
        } => {
            if all_pages {
                let mut params: Vec<(&str, &str)> = Vec::new();
                if let Some(s) = search {
                    params.push(("search", s));
                }
                if let Some(p) = properties {
                    params.push(("properties", p));
                }
                client.get_all_pages("persons/", &params, page_size).await
            } else {
                api::endpoints::persons::list(client, search.as_deref(), properties.as_deref())
                    .await
            }
        }
        PersonCommand::Get { id, distinct_id } => {
            if let Some(did) = distinct_id {
                api::endpoints::persons::get_by_distinct_id(client, did).await
            } else if let Some(pid) = id {
                api::endpoints::persons::get(client, pid).await
            } else {
                Err(AppError::Validation {
                    message: "Provide either a person ID or --distinct-id".into(),
                })
            }
        }
        PersonCommand::Update { id, set, unset } => {
            api::endpoints::persons::update(client, id, set.as_deref(), unset.as_deref()).await
        }
        PersonCommand::Delete { id } => {
            api::endpoints::persons::delete(client, id).await?;
            Ok(json!({"status": "deleted", "id": id}))
        }
        PersonCommand::DeleteWithData { id } => {
            api::endpoints::persons::delete_with_data(client, id).await
        }
        PersonCommand::Split { id } => api::endpoints::persons::split(client, id).await,
        PersonCommand::Activity { id } => api::endpoints::persons::activity(client, id).await,
    }
}

async fn execute_cohorts(
    client: &PostHogClient,
    command: &CohortCommand,
    all_pages: bool,
    page_size: u32,
) -> Result<serde_json::Value, AppError> {
    match command {
        CohortCommand::List => {
            if all_pages {
                client
                    .get_all_pages("cohorts/", &[], page_size)
                    .await
            } else {
                api::endpoints::cohorts::list(client).await
            }
        }
        CohortCommand::Get { id } => api::endpoints::cohorts::get(client, *id).await,
        CohortCommand::Create {
            name,
            filters,
            is_static,
        } => api::endpoints::cohorts::create(client, name, filters, *is_static).await,
        CohortCommand::Update { id, name } => {
            api::endpoints::cohorts::update(client, *id, name.as_deref()).await
        }
        CohortCommand::Delete { id } => {
            api::endpoints::cohorts::delete(client, *id).await?;
            Ok(json!({"status": "deleted", "id": id}))
        }
    }
}

async fn execute_errors(
    client: &PostHogClient,
    command: &ErrorCommand,
    all_pages: bool,
    page_size: u32,
) -> Result<serde_json::Value, AppError> {
    match command {
        ErrorCommand::List {
            status,
            date_from,
            order_by,
        } => {
            if all_pages {
                let mut params: Vec<(&str, &str)> = Vec::new();
                if let Some(s) = status {
                    params.push(("status", s));
                }
                if let Some(d) = date_from {
                    params.push(("date_from", d));
                }
                if let Some(o) = order_by {
                    params.push(("order_by", o));
                }
                client
                    .get_all_pages("error_tracking/issue/", &params, page_size)
                    .await
            } else {
                api::endpoints::errors::list(
                    client,
                    status.as_deref(),
                    date_from.as_deref(),
                    order_by.as_deref(),
                )
                .await
            }
        }
        ErrorCommand::Get { id, date_from } => {
            api::endpoints::errors::get(client, id, date_from.as_deref()).await
        }
        ErrorCommand::Resolve { id } => {
            api::endpoints::errors::update_status(client, id, "resolved").await
        }
        ErrorCommand::Ignore { id } => {
            api::endpoints::errors::update_status(client, id, "ignored").await
        }
        ErrorCommand::Reopen { id } => {
            api::endpoints::errors::update_status(client, id, "active").await
        }
    }
}

async fn execute_actions(
    client: &PostHogClient,
    command: &ActionCommand,
    all_pages: bool,
    page_size: u32,
) -> Result<serde_json::Value, AppError> {
    match command {
        ActionCommand::List => {
            if all_pages {
                client.get_all_pages("actions/", &[], page_size).await
            } else {
                api::endpoints::actions::list(client).await
            }
        }
        ActionCommand::Get { id } => api::endpoints::actions::get(client, *id).await,
        ActionCommand::Create { name, steps } => {
            api::endpoints::actions::create(client, name, steps).await
        }
        ActionCommand::Update { id, name } => {
            api::endpoints::actions::update(client, *id, name.as_deref()).await
        }
        ActionCommand::Delete { id } => {
            api::endpoints::actions::delete(client, *id).await?;
            Ok(json!({"status": "deleted", "id": id}))
        }
    }
}

async fn execute_definitions(
    client: &PostHogClient,
    command: &DefinitionCommand,
) -> Result<serde_json::Value, AppError> {
    match command {
        DefinitionCommand::Events { command } => match command {
            EventDefinitionCommand::List { search } => {
                api::endpoints::definitions::list_events(client, search.as_deref()).await
            }
            EventDefinitionCommand::Get { id } => {
                api::endpoints::definitions::get_event(client, id).await
            }
            EventDefinitionCommand::Update {
                id,
                description,
                tags,
                verified,
            } => {
                api::endpoints::definitions::update_event(
                    client,
                    id,
                    description.as_deref(),
                    tags.as_deref(),
                    *verified,
                )
                .await
            }
            EventDefinitionCommand::Delete { id } => {
                api::endpoints::definitions::delete_event(client, id).await?;
                Ok(json!({"status": "deleted", "id": id}))
            }
        },
        DefinitionCommand::Properties { command } => match command {
            PropertyDefinitionCommand::List {
                search,
                event_names,
            } => {
                api::endpoints::definitions::list_properties(
                    client,
                    search.as_deref(),
                    event_names.as_deref(),
                )
                .await
            }
            PropertyDefinitionCommand::Get { id } => {
                api::endpoints::definitions::get_property(client, id).await
            }
            PropertyDefinitionCommand::Update { id, description } => {
                api::endpoints::definitions::update_property(client, id, description.as_deref())
                    .await
            }
        },
    }
}
