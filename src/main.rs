use libcleaner::cleaner::database_cleaner::DatabaseCleaner;
use libcleaner::cleaner::mysql::MySQLCleaner;
use libcleaner::cleaner::postgres::PostgresCleaner;
use libcleaner::colors::{BLUE, GREEN, RED, RESET, YELLOW};
use libcleaner::config::{CleanerConfig, ConnectionEngine, DatabaseConfig};
use libcleaner::logger::{log_and_print, LogType};
use std::io::{self, Write};
use std::time::Instant;

const CONFIG_FILE: &str = "cleaner.json";

#[tokio::main]
async fn main() {
    let start = Instant::now();

    let cleaner_config = match load_configuration() {
        Ok(config) => config,
        Err(e) => {
            log_and_print(&e, &LogType::Critical);
            std::process::exit(1);
        }
    };

    display_header();
    display_database_list(&cleaner_config);

    if cleaner_config.dry_run {
        println!("\n{YELLOW}🔍 DRY RUN MODE - No changes will be made{RESET}");
    }

    if !confirm_operation(&cleaner_config) {
        println!("{RED}❌ Operation cancelled by user{RESET}");
        return;
    }

    println!("\n{GREEN}✓ Starting cleanup operations...{RESET}\n");

    let (success_count, failed_count) = process_databases(&cleaner_config).await;

    display_summary(&cleaner_config, success_count, failed_count, start.elapsed());
}

/// Load the configuration from the file
fn load_configuration() -> Result<CleanerConfig, String> {
    CleanerConfig::from_file(CONFIG_FILE).map_err(|e| format!("{e}"))
}

/// Display the application header
fn display_header() {
    println!("{BLUE}╔══════════════════════════════════════════════════════╗{RESET}");
    println!("{BLUE}║{RESET}          DBMSCleaner - Database Optimizer          {BLUE}║{RESET}");
    println!("{BLUE}╚══════════════════════════════════════════════════════╝{RESET}\n");
}

/// Display the list of databases to be cleaned
fn display_database_list(config: &CleanerConfig) {
    println!("Found {} database(s) to clean:", config.databases.len());

    for (i, db) in config.databases.iter().enumerate() {
        let db_name = get_database_name(db, i);
        println!(
            "  {}. {BLUE}{}{RESET} ({}) - {}:{}/{}",
            i + 1,
            db_name,
            db.driver,
            db.host,
            db.port,
            db.schema
        );
    }
}

/// Get the database name or generate a default one
fn get_database_name(db: &DatabaseConfig, index: usize) -> String {
    db.name
        .clone()
        .unwrap_or_else(|| format!("Database #{}", index + 1))
}

/// Ask for user confirmation if required
fn confirm_operation(config: &CleanerConfig) -> bool {
    if !config.require_confirmation || config.dry_run {
        return true;
    }

    println!("\n{YELLOW}⚠️  WARNING: This will perform optimization operations on the databases.{RESET}");
    println!("{YELLOW}   Operations include: VACUUM, REINDEX, OPTIMIZE, LOG PURGING{RESET}");
    print!("\nDo you want to continue? (yes/no): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let input = input.trim().to_lowercase();
    matches!(input.as_str(), "yes" | "y")
}

/// Process all databases and return success/failure counts
async fn process_databases(config: &CleanerConfig) -> (usize, usize) {
    let mut success_count = 0;
    let mut failed_count = 0;

    for (i, db_config) in config.databases.iter().enumerate() {
        let db_name = get_database_name(db_config, i);

        print_database_header(&db_name);

        if config.dry_run {
            handle_dry_run(db_config, &db_name);
            success_count += 1;
            continue;
        }

        match process_single_database(db_config, &db_name).await {
            Ok(()) => {
                println!("{GREEN}✓ Successfully cleaned {}{RESET}\n", db_name);
                success_count += 1;
            }
            Err(e) => {
                log_and_print(&e, &LogType::Error);
                failed_count += 1;
            }
        }
    }

    (success_count, failed_count)
}

/// Print the database processing header
fn print_database_header(db_name: &str) {
    println!("{BLUE}═══════════════════════════════════════════════════════{RESET}");
    println!("{BLUE}Processing: {}{RESET}", db_name);
    println!("{BLUE}═══════════════════════════════════════════════════════{RESET}");
}

/// Handle dry run mode for a database
fn handle_dry_run(db_config: &DatabaseConfig, db_name: &str) {
    println!(
        "{YELLOW}[DRY RUN] Would clean {} ({}) at {}:{}/{}{RESET}",
        db_name, db_config.driver, db_config.host, db_config.port, db_config.schema
    );
}

/// Process a single database
async fn process_single_database(db_config: &DatabaseConfig, db_name: &str) -> Result<(), String> {
    let cleaner = create_cleaner(db_config, db_name)?;
    cleaner
        .clean()
        .await
        .map_err(|e| format!("Failed to clean {}: {}", db_name, e))
}

/// Create the appropriate database cleaner based on the driver
fn create_cleaner(
    db_config: &DatabaseConfig,
    db_name: &str,
) -> Result<Box<dyn DatabaseCleaner>, String> {
    match db_config.driver {
        ConnectionEngine::MariaDB | ConnectionEngine::Mysql => {
            Ok(Box::new(MySQLCleaner::from_config(db_config.clone())))
        }
        ConnectionEngine::Postgres => {
            Ok(Box::new(PostgresCleaner::from_config(db_config.clone())))
        }
        ConnectionEngine::Invalid => Err(format!(
            "Unsupported database driver for {}: {:?}",
            db_name, db_config.driver
        )),
    }
}

/// Display the final summary
fn display_summary(
    config: &CleanerConfig,
    success_count: usize,
    failed_count: usize,
    elapsed: std::time::Duration,
) {
    println!("{BLUE}═══════════════════════════════════════════════════════{RESET}");
    println!("{BLUE}║{RESET}                    SUMMARY                        {BLUE}║{RESET}");
    println!("{BLUE}═══════════════════════════════════════════════════════{RESET}");
    println!("Total databases: {}", config.databases.len());
    println!("{GREEN}✓ Successful: {}{RESET}", success_count);

    if failed_count > 0 {
        println!("{RED}✗ Failed: {}{RESET}", failed_count);
    }

    println!("Total time: {GREEN}{:?}{RESET}", elapsed);
    println!("{BLUE}═══════════════════════════════════════════════════════{RESET}");
}
