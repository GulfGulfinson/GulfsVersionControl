mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "gvc")]
#[command(about = "GulfsVersionControl - A modern version control system", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new repository
    Init {
        /// Path to initialize repository (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Show working tree status
    Status,

    /// Add file(s) to staging area
    Add {
        /// Files or directories to add
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },

    /// Record changes to the repository
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: String,

        /// Author name (default: from config or system)
        #[arg(short, long)]
        author: Option<String>,
    },

    /// Show commit history
    Log {
        /// Maximum number of commits to show
        #[arg(short = 'n', long)]
        max_count: Option<usize>,

        /// Show in oneline format
        #[arg(long)]
        oneline: bool,
    },

    /// Show differences
    Diff {
        /// Compare staged changes
        #[arg(long)]
        staged: bool,
    },

    /// Show object content
    Show {
        /// Object hash to show
        hash: String,
    },

    /// Branch operations
    #[command(subcommand)]
    Branch(BranchCommands),

    /// Switch branches
    Checkout {
        /// Branch name to checkout
        branch: String,
    },

    /// Switch branches (alias for checkout)
    Switch {
        /// Branch name to switch to
        branch: String,
    },

    /// Tag operations
    #[command(subcommand)]
    Tag(TagCommands),

    /// Unstage files (remove from staging area)
    Reset {
        /// Files to unstage (leave empty to unstage all)
        paths: Vec<PathBuf>,
    },

    /// Module operations
    #[command(subcommand)]
    Module(ModuleCommands),

    /// Remote operations
    #[command(subcommand)]
    Remote(RemoteCommands),

    /// Push to a remote repository
    Push {
        /// Remote name (default: origin)
        #[arg(default_value = "origin")]
        remote: String,

        /// Branch name (default: current branch)
        #[arg(short, long)]
        branch: Option<String>,

        /// Force push
        #[arg(short, long)]
        force: bool,
    },

    /// Fetch from a remote repository
    Fetch {
        /// Remote name (default: origin)
        #[arg(default_value = "origin")]
        remote: String,
    },

    /// Pull from a remote repository
    Pull {
        /// Remote name (default: origin)
        #[arg(default_value = "origin")]
        remote: String,

        /// Branch name (default: current branch)
        #[arg(short, long)]
        branch: Option<String>,
    },

    /// Clone a remote repository
    Clone {
        /// Repository URL
        url: String,

        /// Target directory (default: repo name from URL)
        directory: Option<PathBuf>,
    },

    /// Run garbage collection
    Gc {
        /// Dry run (don't actually remove anything)
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Merge a branch into current branch
    Merge {
        /// Branch to merge
        branch: String,

        /// Merge strategy (ours, theirs, recursive)
        #[arg(short, long)]
        strategy: Option<String>,

        /// Merge commit message
        #[arg(short, long)]
        message: Option<String>,

        /// Only fast-forward merge
        #[arg(long)]
        ff_only: bool,
    },
}

#[derive(Subcommand)]
enum ModuleCommands {
    /// Create a new module scaffold
    Create {
        /// Module name
        name: String,

        /// Target directory (default: ./<name>)
        #[arg(long)]
        path: Option<PathBuf>,

        /// Author name
        #[arg(long)]
        author: Option<String>,
    },

    /// Install a module from path
    Install {
        /// Path to module directory
        path: PathBuf,
    },

    /// Activate a module in current repository
    Add {
        /// Module identifier (name@version)
        identifier: String,
    },

    /// Deactivate a module in current repository
    Remove {
        /// Module identifier (name@version)
        identifier: String,
    },

    /// List modules
    List {
        /// Show only active modules
        #[arg(long)]
        active: bool,

        /// Show only installed modules
        #[arg(long)]
        installed: bool,
    },

    /// Show module information
    Info {
        /// Module identifier (name@version)
        identifier: String,
    },
}

#[derive(Subcommand)]
enum BranchCommands {
    /// Create a new branch
    Create {
        /// Branch name
        name: String,
    },

    /// Delete a branch
    Delete {
        /// Branch name
        name: String,
    },

    /// List all branches
    List,
}

#[derive(Subcommand)]
enum TagCommands {
    /// Create a new tag
    Create {
        /// Tag name
        name: String,
    },

    /// List all tags
    List,
}

#[derive(Subcommand)]
enum RemoteCommands {
    /// Add a remote repository
    Add {
        /// Remote name
        name: String,
        /// Remote URL
        url: String,
    },

    /// Remove a remote repository
    Remove {
        /// Remote name
        name: String,
    },

    /// Rename a remote repository
    Rename {
        /// Old name
        old_name: String,
        /// New name
        new_name: String,
    },

    /// List remote repositories
    List {
        /// Show URLs
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { path } => commands::init(&path),
        Commands::Status => commands::status(),
        Commands::Add { paths } => commands::add(&paths),
        Commands::Commit { message, author } => commands::commit(&message, author.as_deref()),
        Commands::Log { max_count, oneline } => commands::log(max_count, oneline),
        Commands::Diff { staged } => commands::diff(staged),
        Commands::Show { hash } => commands::show(&hash),
        Commands::Branch(cmd) => match cmd {
            BranchCommands::Create { name } => commands::branch_create(&name),
            BranchCommands::Delete { name } => commands::branch_delete(&name),
            BranchCommands::List => commands::branch_list(),
        },
        Commands::Checkout { branch } => commands::checkout(&branch),
        Commands::Switch { branch } => commands::checkout(&branch),
        Commands::Tag(cmd) => match cmd {
            TagCommands::Create { name } => commands::tag_create(&name),
            TagCommands::List => commands::tag_list(),
        },
        Commands::Reset { paths } => commands::reset(&paths),
        Commands::Module(cmd) => match cmd {
            ModuleCommands::Create { name, path, author } => {
                commands::module_create(&name, path.as_deref(), author.as_deref())
            }
            ModuleCommands::Install { path } => commands::module_install(&path),
            ModuleCommands::Add { identifier } => commands::module_add(&identifier),
            ModuleCommands::Remove { identifier } => commands::module_remove(&identifier),
            ModuleCommands::List { active, installed } => {
                commands::module_list(active, installed)
            }
            ModuleCommands::Info { identifier } => commands::module_info(&identifier),
        },
        Commands::Remote(cmd) => match cmd {
            RemoteCommands::Add { name, url } => commands::remote_add(&name, &url),
            RemoteCommands::Remove { name } => commands::remote_remove(&name),
            RemoteCommands::Rename { old_name, new_name } => {
                commands::remote_rename(&old_name, &new_name)
            }
            RemoteCommands::List { verbose } => commands::remote_list(verbose),
        },
        Commands::Push { remote, branch, force } => {
            commands::push(&remote, branch.as_deref(), force)
        }
        Commands::Fetch { remote } => commands::fetch(&remote),
        Commands::Pull { remote, branch } => commands::pull(&remote, branch.as_deref()),
        Commands::Clone { url, directory } => commands::clone(&url, directory.as_deref()),
        Commands::Gc { dry_run, verbose } => commands::gc(dry_run, verbose),
        Commands::Merge { branch, strategy, message, ff_only } => {
            commands::merge(&branch, strategy.as_deref(), message.as_deref(), ff_only)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

