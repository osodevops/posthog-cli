use clap::Subcommand;

#[derive(Subcommand)]
pub enum AuthCommand {
    /// Interactive login (stores credentials in OS keyring)
    Login {
        /// Provide token directly instead of interactive prompt
        #[arg(long)]
        token: Option<String>,
    },

    /// Show current authentication status
    Status,

    /// Remove stored credentials
    Logout,

    /// Show current user details
    Whoami,

    /// Switch active project/environment
    Switch {
        /// Project ID to switch to
        project_id: String,
    },
}
