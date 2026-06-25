use clap::{Parser, Subcommand};

mod commands;

/// Rjango management tool — like Django's manage.py / django-admin.
#[derive(Parser)]
#[command(name = "rjango", version, about = "Rjango web framework CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to settings module or config file
    #[arg(short, long, default_value = "settings.toml")]
    settings: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the development server
    Runserver {
        /// Address to bind to [default: 127.0.0.1:8000]
        addr: Option<String>,
    },
    /// Create a new project
    Startproject {
        name: String,
        /// Directory to create the project in
        #[arg(default_value = ".")]
        dir: String,
    },
    /// Create a new app
    Startapp {
        name: String,
    },
    /// Run database migrations
    Migrate,
    /// Show migration status
    Showmigrations,
    /// Create new migration files
    Makemigrations {
        /// App label to make migrations for
        app_label: Option<String>,
    },
    /// Open a Rust shell
    Shell,
    /// Open a database shell
    Dbshell,
    /// Check the project for issues
    Check,
    /// Collect static files
    Collectstatic,
    /// Validate models
    Validate,
    /// Run tests
    Test {
        /// Test labels, e.g. "app.tests.TestClass.test_method"
        args: Vec<String>,
    },
    /// Create a superuser
    Createsuperuser,
    /// Enter an interactive console
    Console,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let cli = Cli::parse();

    // Load settings (may fail, but commands like startproject/startapp don't need it)
    let settings_result = load_settings(&cli.settings);

    match &cli.command {
        Commands::Runserver { addr } => {
            let addr_str = addr.as_deref().unwrap_or("127.0.0.1:8000");
            let addr: std::net::SocketAddr = addr_str.parse()
                .expect("Invalid address format. Use IP:PORT, e.g. 127.0.0.1:8000");
            let settings = settings_result.unwrap_or_else(|e| {
                eprintln!("Warning: {}", e);
                rjango_core::Settings::default()
            });
            let app = build_app(settings);
            tracing::info!("Starting server at http://{}", addr);
            commands::runserver::run(&addr, app);
        }

        Commands::Startproject { name, dir } => {
            commands::startproject::run(name, dir);
        }

        Commands::Startapp { name } => {
            commands::startapp::run(name);
        }

        Commands::Migrate => {
            let db_url = settings_result.ok()
                .and_then(|s| s.databases.get("default").cloned())
                .map(|db| db.url())
                .unwrap_or_else(|| "db.sqlite3".to_string());
            commands::migrate::run(&db_url);
        }

        Commands::Showmigrations => {
            commands::showmigrations::run();
        }

        Commands::Makemigrations { app_label } => {
            commands::makemigrations::run(app_label.as_deref());
        }

        Commands::Shell => {
            commands::shell::run();
        }

        Commands::Dbshell => {
            let db_url = settings_result.ok()
                .and_then(|s| s.databases.get("default").cloned())
                .map(|db| db.url());
            commands::dbshell::run(db_url.as_deref());
        }

        Commands::Check => {
            match &settings_result {
                Ok(settings) => commands::check::run(settings),
                Err(e) => {
                    eprintln!("Could not load settings: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Collectstatic => {
            let root = settings_result.as_ref()
                .map(|s| s.static_root.to_string_lossy().to_string())
                .unwrap_or_else(|_| "static".into());
            commands::collectstatic::run(&root, &[]);
        }

        Commands::Validate => {
            commands::validate::run();
        }

        Commands::Test { args } => {
            commands::test::run(args);
        }

        Commands::Createsuperuser => {
            commands::createsuperuser::run();
        }

        Commands::Console => {
            commands::console::run();
        }
    }
}

fn load_settings(path: &str) -> Result<rjango_core::Settings, String> {
    if std::path::Path::new(path).exists() {
        rjango_core::Settings::from_toml(path)
            .map_err(|e| format!("Settings error: {}", e))
    } else {
        Ok(rjango_core::Settings::default())
    }
}

fn build_app(settings: rjango_core::Settings) -> rjango_server::Application {
    let mut app = rjango_server::Application::new()
        .configure(settings);

    // Set up default middleware
    let mut mw = rjango_middleware::MiddlewareStack::new();
    mw.add(rjango_middleware::security::SecurityMiddleware);
    mw.add(rjango_middleware::session::SessionMiddleware::new());
    mw.add(rjango_middleware::csrf::CsrfMiddleware);
    mw.add(rjango_middleware::messages::MessageMiddleware);
    app = app.with_middleware(mw);

    app
}
