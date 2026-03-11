pub mod auth;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "posthog",
    about = "Agent-first CLI for the PostHog analytics API",
    version,
    propagate_version = true,
    after_help = "Environment variables:\n  POSTHOG_TOKEN        Personal API key (phx_...)\n  POSTHOG_HOST         PostHog instance URL\n  POSTHOG_PROJECT_ID   Project/environment ID\n  POSTHOG_FORMAT       Default output format"
)]
pub struct Cli {
    /// PostHog instance URL
    #[arg(long, short = 'H', global = true, env = "POSTHOG_HOST")]
    pub host: Option<String>,

    /// Personal API key (phx_...)
    #[arg(long, short = 't', global = true, env = "POSTHOG_TOKEN")]
    pub token: Option<String>,

    /// Project/environment ID
    #[arg(long, short = 'p', global = true, env = "POSTHOG_PROJECT_ID")]
    pub project: Option<String>,

    /// Output format: json, table, csv, jsonl
    #[arg(long, short = 'f', global = true, env = "POSTHOG_FORMAT", value_enum)]
    pub format: Option<Format>,

    /// Suppress non-data output (progress, hints)
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Show HTTP request/response debug info
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    /// Bypass local response cache
    #[arg(long, global = true)]
    pub no_cache: bool,

    /// HTTP timeout in seconds
    #[arg(long, global = true, env = "POSTHOG_TIMEOUT", default_value = "30")]
    pub timeout: u64,

    /// Number of retries on transient failures
    #[arg(long, global = true, default_value = "3")]
    pub retry: u32,

    /// Pagination page size
    #[arg(long, global = true, default_value = "100")]
    pub page_size: u32,

    /// Automatically paginate and return all results
    #[arg(long, global = true)]
    pub all_pages: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Format {
    Json,
    Table,
    Csv,
    Jsonl,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage authentication and credentials
    Auth {
        #[command(subcommand)]
        command: auth::AuthCommand,
    },

    /// Run HogQL and structured queries
    Query {
        #[command(subcommand)]
        command: QueryCommand,
    },

    /// Manage feature flags
    Flags {
        #[command(subcommand)]
        command: FlagCommand,
    },

    /// Manage saved insights
    Insights {
        #[command(subcommand)]
        command: InsightCommand,
    },

    /// Manage dashboards
    Dashboards {
        #[command(subcommand)]
        command: DashboardCommand,
    },

    /// Manage annotations
    Annotations {
        #[command(subcommand)]
        command: AnnotationCommand,
    },

    /// Send events, identify users, manage groups
    Capture {
        #[command(subcommand)]
        command: CaptureCommand,
    },

    /// Manage experiments and A/B tests
    Experiments {
        #[command(subcommand)]
        command: ExperimentCommand,
    },

    /// Manage surveys
    Surveys {
        #[command(subcommand)]
        command: SurveyCommand,
    },

    /// Manage persons
    Persons {
        #[command(subcommand)]
        command: PersonCommand,
    },

    /// Manage cohorts
    Cohorts {
        #[command(subcommand)]
        command: CohortCommand,
    },

    /// Track and manage errors
    Errors {
        #[command(subcommand)]
        command: ErrorCommand,
    },

    /// Manage actions (reusable event groups)
    Actions {
        #[command(subcommand)]
        command: ActionCommand,
    },

    /// Manage event and property definitions
    Definitions {
        #[command(subcommand)]
        command: DefinitionCommand,
    },

    /// Manage local response cache
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
}

// ── Cache subcommands ───────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum CacheCommand {
    /// Clear all cached responses
    Clear,

    /// Show cache statistics
    Stats,
}

// ── Query subcommands ───────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum QueryCommand {
    /// Execute a HogQL SQL query
    Sql {
        /// SQL query string
        query: Option<String>,

        /// Read query from file
        #[arg(long)]
        file: Option<String>,

        /// Query parameter (key=value), can be repeated
        #[arg(long = "param", value_name = "KEY=VALUE")]
        params: Vec<String>,

        /// Name to tag the query with
        #[arg(long)]
        name: Option<String>,

        /// Submit async and return query_id immediately
        #[arg(long = "async", conflicts_with = "wait")]
        async_mode: bool,

        /// Submit async, poll until complete (default behavior)
        #[arg(long)]
        wait: bool,

        /// Cache/execution mode: blocking, force_cache, force_blocking, async
        #[arg(long, default_value = "blocking")]
        refresh: String,
    },

    /// Query trends data
    Trends {
        /// Event name to query
        #[arg(long)]
        event: String,

        /// Time interval: hour, day, week, month
        #[arg(long, default_value = "day")]
        interval: String,

        /// Start date (e.g. -7d, 2025-01-01)
        #[arg(long)]
        date_from: Option<String>,

        /// End date
        #[arg(long)]
        date_to: Option<String>,
    },

    /// Query funnel conversion
    Funnels {
        /// Funnel step events in order
        #[arg(long = "steps", num_args = 2..)]
        steps: Vec<String>,

        /// Start date
        #[arg(long)]
        date_from: Option<String>,

        /// End date
        #[arg(long)]
        date_to: Option<String>,
    },

    /// Query retention data
    Retention {
        /// Target event
        #[arg(long)]
        target_event: String,

        /// Return event
        #[arg(long)]
        return_event: String,

        /// Period: day, week, month
        #[arg(long, default_value = "week")]
        period: String,

        /// Start date
        #[arg(long)]
        date_from: Option<String>,
    },

    /// Check async query status
    Status {
        /// Query ID to check
        query_id: String,
    },

    /// Fetch completed async query result
    Result {
        /// Query ID to fetch
        query_id: String,
    },

    /// Cancel a running query
    Cancel {
        /// Query ID to cancel
        query_id: String,
    },
}

// ── Feature flag subcommands ────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum FlagCommand {
    /// List feature flags
    List {
        /// Only show active flags
        #[arg(long)]
        active: bool,

        /// Search by key or name
        #[arg(long)]
        search: Option<String>,
    },

    /// Get a feature flag by key or ID
    Get {
        /// Flag key or numeric ID
        key: String,
    },

    /// Create a new feature flag
    Create {
        /// Flag key (unique identifier)
        #[arg(long)]
        key: String,

        /// Human-readable name
        #[arg(long)]
        name: Option<String>,

        /// Rollout percentage (0-100)
        #[arg(long)]
        rollout: Option<u8>,

        /// Whether the flag should be active immediately
        #[arg(long)]
        active: bool,
    },

    /// Update an existing feature flag
    Update {
        /// Flag key or ID
        key: String,

        /// New rollout percentage
        #[arg(long)]
        rollout: Option<u8>,

        /// Set active state
        #[arg(long)]
        active: Option<bool>,

        /// New name
        #[arg(long)]
        name: Option<String>,
    },

    /// Delete a feature flag
    Delete {
        /// Flag key or ID
        key: String,
    },

    /// Evaluate a flag for a specific user
    Evaluate {
        /// Flag key
        key: String,

        /// Distinct ID of the user to evaluate for
        #[arg(long)]
        distinct_id: String,
    },

    /// Evaluate all flags for a specific user
    EvaluateAll {
        /// Distinct ID of the user
        #[arg(long)]
        distinct_id: String,
    },
}

// ── Insight subcommands ─────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum InsightCommand {
    /// List insights
    List {
        /// Search by name
        #[arg(long)]
        search: Option<String>,

        /// Only show saved insights
        #[arg(long)]
        saved: bool,

        /// Only show favourited insights
        #[arg(long)]
        favorited: bool,
    },

    /// Get an insight by ID
    Get {
        /// Insight ID
        id: u64,

        /// Force fresh computation: blocking, force_cache, force_blocking
        #[arg(long)]
        refresh: Option<String>,
    },

    /// Create a new insight
    Create {
        /// Insight name
        #[arg(long)]
        name: String,

        /// Query definition as JSON
        #[arg(long)]
        query: String,
    },

    /// Update an existing insight
    Update {
        /// Insight ID
        id: u64,

        /// New name
        #[arg(long)]
        name: Option<String>,

        /// Tags as JSON array
        #[arg(long)]
        tags: Option<String>,
    },

    /// Delete an insight
    Delete {
        /// Insight ID
        id: u64,
    },
}

// ── Dashboard subcommands ───────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum DashboardCommand {
    /// List dashboards
    List {
        /// Search by name
        #[arg(long)]
        search: Option<String>,

        /// Only show pinned dashboards
        #[arg(long)]
        pinned: bool,
    },

    /// Get a dashboard by ID
    Get {
        /// Dashboard ID
        id: u64,
    },

    /// Create a new dashboard
    Create {
        /// Dashboard name
        #[arg(long)]
        name: String,

        /// Description
        #[arg(long)]
        description: Option<String>,

        /// Pin the dashboard
        #[arg(long)]
        pinned: bool,
    },

    /// Update an existing dashboard
    Update {
        /// Dashboard ID
        id: u64,

        /// New name
        #[arg(long)]
        name: Option<String>,

        /// Tags as JSON array
        #[arg(long)]
        tags: Option<String>,
    },

    /// Delete a dashboard
    Delete {
        /// Dashboard ID
        id: u64,
    },
}

// ── Annotation subcommands ──────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum AnnotationCommand {
    /// List annotations
    List {
        /// Search by content
        #[arg(long)]
        search: Option<String>,
    },

    /// Get an annotation by ID
    Get {
        /// Annotation ID
        id: u64,
    },

    /// Create an annotation
    Create {
        /// Annotation content/message
        #[arg(long)]
        content: String,

        /// Date/time for the annotation (ISO 8601)
        #[arg(long)]
        date: String,

        /// Scope: project or organization
        #[arg(long, default_value = "project")]
        scope: String,
    },

    /// Update an annotation
    Update {
        /// Annotation ID
        id: u64,

        /// New content
        #[arg(long)]
        content: Option<String>,
    },

    /// Delete an annotation
    Delete {
        /// Annotation ID
        id: u64,
    },
}

// ── Capture subcommands ─────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum CaptureCommand {
    /// Capture a single event
    Event {
        /// Event name
        #[arg(long)]
        event: String,

        /// Distinct ID of the user
        #[arg(long)]
        distinct_id: String,

        /// Properties as JSON object
        #[arg(long)]
        properties: Option<String>,
    },

    /// Batch capture events from a JSONL file
    Batch {
        /// Path to JSONL file
        #[arg(long)]
        file: String,
    },

    /// Identify a user (set person properties)
    Identify {
        /// Distinct ID of the user
        #[arg(long)]
        distinct_id: String,

        /// Properties to set as JSON
        #[arg(long)]
        set: String,
    },

    /// Set group properties
    Group {
        /// Group type (e.g. "company")
        #[arg(long)]
        group_type: String,

        /// Group key (e.g. "acme-inc")
        #[arg(long)]
        group_key: String,

        /// Properties to set as JSON
        #[arg(long)]
        set: String,
    },

    /// Create a user alias
    Alias {
        /// Distinct ID
        #[arg(long)]
        distinct_id: String,

        /// Alias to associate
        #[arg(long)]
        alias: String,
    },
}

// ── Experiment subcommands ─────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum ExperimentCommand {
    /// List experiments
    List {
        /// Filter by status: draft, running, complete
        #[arg(long)]
        status: Option<String>,
    },

    /// Get experiment details
    Get {
        /// Experiment ID
        id: u64,
    },

    /// Create an experiment
    Create {
        /// Experiment name
        #[arg(long)]
        name: String,

        /// Feature flag key to use
        #[arg(long)]
        feature_flag_key: String,

        /// Description
        #[arg(long)]
        description: Option<String>,

        /// Metrics configuration as JSON
        #[arg(long)]
        metrics: Option<String>,
    },

    /// Update an experiment
    Update {
        /// Experiment ID
        id: u64,

        /// New name
        #[arg(long)]
        name: Option<String>,

        /// New description
        #[arg(long)]
        description: Option<String>,
    },

    /// Start an experiment (set start_date to now)
    Start {
        /// Experiment ID
        id: u64,
    },

    /// Stop an experiment (set end_date to now)
    Stop {
        /// Experiment ID
        id: u64,
    },

    /// Fetch experiment results
    Results {
        /// Experiment ID
        id: u64,
    },

    /// Delete an experiment
    Delete {
        /// Experiment ID
        id: u64,
    },
}

// ── Survey subcommands ─────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum SurveyCommand {
    /// List surveys
    List {
        /// Filter by status: active, draft, complete
        #[arg(long)]
        status: Option<String>,
    },

    /// Get a survey by ID
    Get {
        /// Survey ID
        id: u64,
    },

    /// Create a new survey
    Create {
        /// Survey name
        #[arg(long)]
        name: String,

        /// Questions as JSON array
        #[arg(long)]
        questions: String,

        /// Targeting rules as JSON
        #[arg(long)]
        targeting: Option<String>,
    },

    /// Update a survey
    Update {
        /// Survey ID
        id: u64,

        /// New name
        #[arg(long)]
        name: Option<String>,

        /// End date (ISO 8601)
        #[arg(long)]
        end_date: Option<String>,
    },

    /// Launch a survey (set start_date to now)
    Launch {
        /// Survey ID
        id: u64,
    },

    /// Stop a survey (set end_date to now)
    Stop {
        /// Survey ID
        id: u64,
    },

    /// Archive a survey
    Archive {
        /// Survey ID
        id: u64,
    },

    /// Delete a survey
    Delete {
        /// Survey ID
        id: u64,
    },
}

// ── Person subcommands ─────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum PersonCommand {
    /// List persons
    List {
        /// Search by email or distinct ID
        #[arg(long)]
        search: Option<String>,

        /// Filter by properties as JSON
        #[arg(long)]
        properties: Option<String>,
    },

    /// Get a person by ID
    Get {
        /// Person ID (UUID)
        id: Option<String>,

        /// Get by distinct ID instead
        #[arg(long)]
        distinct_id: Option<String>,
    },

    /// Update person properties
    Update {
        /// Person ID (UUID)
        id: String,

        /// Properties to set as JSON
        #[arg(long)]
        set: Option<String>,

        /// Properties to unset as JSON array
        #[arg(long)]
        unset: Option<String>,
    },

    /// Delete a person
    Delete {
        /// Person ID (UUID)
        id: String,
    },

    /// Delete a person and all their data (GDPR)
    DeleteWithData {
        /// Person ID (UUID)
        id: String,
    },

    /// Split a merged person
    Split {
        /// Person ID (UUID)
        id: String,
    },

    /// View person activity log
    Activity {
        /// Person ID (UUID)
        id: String,
    },
}

// ── Cohort subcommands ─────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum CohortCommand {
    /// List cohorts
    List,

    /// Get a cohort by ID
    Get {
        /// Cohort ID
        id: u64,
    },

    /// Create a cohort
    Create {
        /// Cohort name
        #[arg(long)]
        name: String,

        /// Filter definition as JSON
        #[arg(long)]
        filters: String,

        /// Create as static cohort
        #[arg(long)]
        is_static: bool,
    },

    /// Update a cohort
    Update {
        /// Cohort ID
        id: u64,

        /// New name
        #[arg(long)]
        name: Option<String>,
    },

    /// Delete a cohort
    Delete {
        /// Cohort ID
        id: u64,
    },
}

// ── Error tracking subcommands ─────────────────────────────────────────────

#[derive(Subcommand)]
pub enum ErrorCommand {
    /// List error issues
    List {
        /// Filter by status: active, resolved, ignored
        #[arg(long)]
        status: Option<String>,

        /// Start date filter
        #[arg(long)]
        date_from: Option<String>,

        /// Order by field (e.g. last_seen)
        #[arg(long)]
        order_by: Option<String>,
    },

    /// Get error issue details
    Get {
        /// Issue ID
        id: String,

        /// Start date filter
        #[arg(long)]
        date_from: Option<String>,
    },

    /// Mark an error issue as resolved
    Resolve {
        /// Issue ID
        id: String,
    },

    /// Mark an error issue as ignored
    Ignore {
        /// Issue ID
        id: String,
    },

    /// Reopen an error issue
    Reopen {
        /// Issue ID
        id: String,
    },
}

// ── Action subcommands ─────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum ActionCommand {
    /// List actions
    List,

    /// Get an action by ID
    Get {
        /// Action ID
        id: u64,
    },

    /// Create an action
    Create {
        /// Action name
        #[arg(long)]
        name: String,

        /// Steps as JSON array
        #[arg(long)]
        steps: String,
    },

    /// Update an action
    Update {
        /// Action ID
        id: u64,

        /// New name
        #[arg(long)]
        name: Option<String>,
    },

    /// Delete an action
    Delete {
        /// Action ID
        id: u64,
    },
}

// ── Definition subcommands ─────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum DefinitionCommand {
    /// Manage event definitions
    Events {
        #[command(subcommand)]
        command: EventDefinitionCommand,
    },

    /// Manage property definitions
    Properties {
        #[command(subcommand)]
        command: PropertyDefinitionCommand,
    },
}

#[derive(Subcommand)]
pub enum EventDefinitionCommand {
    /// List event definitions
    List {
        /// Search by name
        #[arg(long)]
        search: Option<String>,
    },

    /// Get an event definition
    Get {
        /// Event definition name or ID
        id: String,
    },

    /// Update an event definition
    Update {
        /// Event definition ID
        id: String,

        /// Description
        #[arg(long)]
        description: Option<String>,

        /// Tags as JSON array
        #[arg(long)]
        tags: Option<String>,

        /// Mark as verified
        #[arg(long)]
        verified: Option<bool>,
    },

    /// Delete an event definition
    Delete {
        /// Event definition ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum PropertyDefinitionCommand {
    /// List property definitions
    List {
        /// Search by name
        #[arg(long)]
        search: Option<String>,

        /// Filter by event names as JSON array
        #[arg(long)]
        event_names: Option<String>,
    },

    /// Get a property definition
    Get {
        /// Property definition ID
        id: String,
    },

    /// Update a property definition
    Update {
        /// Property definition ID
        id: String,

        /// Description
        #[arg(long)]
        description: Option<String>,
    },
}
