use libcleaner::cleaner::database_cleaner::DatabaseCleaner;
use libcleaner::cleaner::mysql::MySQLCleaner;
use libcleaner::cleaner::postgres::PostgresCleaner;
use libcleaner::colors::{BLUE, GREEN, RED, RESET, YELLOW};
use libcleaner::config::{CleanerConfig, ConnectionEngine};
use libcleaner::logger::{log_and_print, LogType};
use std::io::{self, Write};
use std::time::Instant;

#[tokio::main]
async fn main() {
    let start: Instant = Instant::now();

    // Load configuration (supports both legacy and new multi-database format)
    let cleaner_config: CleanerConfig =
        CleanerConfig::from_file("cleaner.json").unwrap_or_else(|e| {
            log_and_print(&format!("{e}"), &LogType::Critical);
            panic!("Failed to read configuration file");
        });

    // Display configuration summary
    println!("{BLUE}╔══════════════════════════════════════════════════════╗{RESET}");
    println!("{BLUE}║{RESET}          DBMSCleaner - Database Optimizer          {BLUE}║{RESET}");
    println!("{BLUE}╚══════════════════════════════════════════════════════╝{RESET}\n");

    println!(
        "Found {} database(s) to clean:",
        cleaner_config.databases.len()
    );
    for (i, db) in cleaner_config.databases.iter().enumerate() {
        let default_name = format!("Database #{}", i + 1);
        let db_name = db.name.as_ref().unwrap_or(&default_name);
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

    if cleaner_config.dry_run {
        println!("\n{YELLOW}🔍 DRY RUN MODE - No changes will be made{RESET}");
    }

    // Ask for confirmation if required
    if cleaner_config.require_confirmation && !cleaner_config.dry_run {
        println!("\n{YELLOW}⚠️  WARNING: This will perform optimization operations on the databases.{RESET}");
        println!("{YELLOW}   Operations include: VACUUM, REINDEX, OPTIMIZE, LOG PURGING{RESET}");
        print!("\nDo you want to continue? (yes/no): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let input = input.trim().to_lowercase();
        if input != "yes" && input != "y" {
            println!("{RED}❌ Operation cancelled by user{RESET}");
            return;
        }
    }

    println!("\n{GREEN}✓ Starting cleanup operations...{RESET}\n");

    // Process each database
    let mut success_count = 0;
    let mut failed_count = 0;

    for (i, db_config) in cleaner_config.databases.iter().enumerate() {
        let default_name = format!("Database #{}", i + 1);
        let db_name = db_config.name.as_ref().unwrap_or(&default_name);

        println!("{BLUE}═══════════════════════════════════════════════════════{RESET}");
        println!("{BLUE}Processing: {}{RESET}", db_name);
        println!("{BLUE}═══════════════════════════════════════════════════════{RESET}");

        if cleaner_config.dry_run {
            println!("{YELLOW}[DRY RUN] Would clean {} ({}) at {}:{}/{}{RESET}",
                db_name, db_config.driver, db_config.host, db_config.port, db_config.schema);
            success_count += 1;
            continue;
        }

        // Create appropriate cleaner
        let cleaner: Box<dyn DatabaseCleaner> = match db_config.driver {
            ConnectionEngine::MariaDB | ConnectionEngine::Mysql => {
                Box::new(MySQLCleaner::from_config(db_config.clone()))
            }
            ConnectionEngine::Postgres => Box::new(PostgresCleaner::from_config(db_config.clone())),
            ConnectionEngine::Invalid => {
                log_and_print(
                    &format!("Unsupported database driver for {}: {:?}", db_name, db_config.driver),
                    &LogType::Error,
                );
                failed_count += 1;
                continue;
            }
        };

        // Execute cleaning
        match cleaner.clean().await {
            Ok(()) => {
                println!("{GREEN}✓ Successfully cleaned {}{RESET}\n", db_name);
                success_count += 1;
            }
            Err(e) => {
                log_and_print(
                    &format!("Failed to clean {}: {}", db_name, e),
                    &LogType::Error,
                );
                failed_count += 1;
            }
        }
    }

    // Summary
    println!("{BLUE}═══════════════════════════════════════════════════════{RESET}");
    println!("{BLUE}║{RESET}                    SUMMARY                        {BLUE}║{RESET}");
    println!("{BLUE}═══════════════════════════════════════════════════════{RESET}");
    println!(
        "Total databases: {}",
        cleaner_config.databases.len()
    );
    println!("{GREEN}✓ Successful: {}{RESET}", success_count);
    if failed_count > 0 {
        println!("{RED}✗ Failed: {}{RESET}", failed_count);
    }
    println!("Total time: {GREEN}{:?}{RESET}", start.elapsed());
    println!("{BLUE}═══════════════════════════════════════════════════════{RESET}");
}
