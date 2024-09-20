package mariadb

import (
	"GoSqlCleaner/database"
	"GoSqlCleaner/util"
	"database/sql"
	"fmt"
	"log"
)

type MariaDbCleaner struct {
	database.Database
}

func (c *MariaDbCleaner) Clean() bool {

	// Connexion à la base de données MySQL
	dsn := fmt.Sprintf("%s:%s@tcp(%s:%s)/", c.Username, c.Password, c.Host, c.Port)

	db, err := sql.Open("mysql", dsn)
	if err != nil {
		log.Fatal("Erreur lors de la connexion à la base de données:", err)
	}
	defer db.Close()

	err = db.Ping()
	if err != nil {
		log.Fatal("Impossible de se connecter à la base de données:", err)
	}

	totalSize := getTotalSizeSql()
	startSize := util.GetDbSize(db, totalSize)

	println("Start size of the database:", util.Green, startSize, " bytes", util.Reset)

	// Set global variables OFF
	println("Setting global variables OFF...")
	setGlobalVariablesOFF(db)

	// Rebuild index
	println("Rebuilding index...")

	// Repair tables
	println("Repairing tables...")
	repairTables(db)

	// Clean all tables
	println("Cleaning all tables...")
	cleanAllTables(db)

	// Clear logs
	println("Clearing logs...")
	clearLogs(db)

	// Set global variables ON
	println("Setting global variables ON...")
	setGlobalVariablesON(db)

	endSize := util.GetDbSize(db, totalSize)
	println("End size of the database:", util.Green, endSize, " bytes", util.Reset)
	println("Optimization of ", util.Green, startSize-endSize, util.Reset, " bytes")

	return true
}

func rebuildIndex(db *sql.DB) {
	// Rebuild index
	rows, err := db.Query("SELECT CONCAT('ALTER TABLE `', TABLE_SCHEMA, '`.`', TABLE_NAME, '` ENGINE=InnoDB') AS stmt FROM information_schema.TABLES WHERE ENGINE = 'InnoDB' AND TABLE_SCHEMA NOT IN ('sys', 'performance_schema', 'information_schema', 'mysql')")
	if err != nil {
		log.Fatal("Erreur lors de la récupération des tables:", err)
	}
	defer rows.Close()

	// iterate over the result
	var stmt string
	for rows.Next() {
		err := rows.Scan(&stmt)
		if err != nil {
			log.Fatal("Erreur lors de la lecture de la ligne:", err)
		}
		_, err = db.Exec(stmt)
		if err != nil {
			log.Fatal("Erreur lors de l'exécution de la requête:", err)
		}
	}
}

func repairTables(db *sql.DB) {
	// repair tables
	rows, err := db.Query("SELECT CONCAT('REPAIR TABLE ',TABLE_SCHEMA, TABLE_NAME, ' EXTENDED') FROM information_schema.TABLES WHERE ENGINE IN ('MyISAM', 'ARCHIVE', 'CSV') AND TABLE_SCHEMA NOT IN ('sys', 'performance_schema', 'information_schema', 'mysql');")
	if err != nil {
		log.Fatal("Erreur lors de la récupération des tables:", err)
	}
	defer rows.Close()

	var stmt string
	for rows.Next() {
		err := rows.Scan(&stmt)
		if err != nil {
			log.Fatal("Erreur lors de la lecture de la ligne:", err)
		}
		_, err = db.Exec(stmt)
		if err != nil {
			log.Fatal("Erreur lors de l'exécution de la requête:", err)
		}
	}
}

func cleanAllTables(db *sql.DB) {
	// clean all tables
	rows, err := db.Query("SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS stmt FROM information_schema.TABLES WHERE TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys')")
	if err != nil {
		log.Fatal("Erreur lors de la récupération des tables:", err)
	}
	defer rows.Close()

	var stmt string
	for rows.Next() {
		err := rows.Scan(&stmt)
		if err != nil {
			log.Fatal("Erreur lors de la lecture de la ligne:", err)
		}
		_, err = db.Exec("ANALYZE TABLE " + stmt)
		if err != nil {
			log.Fatal("Erreur lors de l'exécution de la requête:", err)
		}

		_, err = db.Exec("OPTIMIZE TABLE " + stmt)
		if err != nil {
			log.Fatal("Erreur lors de l'exécution de la requête:", err)
		}
	}
}

func getTotalSizeSql() string {
	return "SELECT SUM(data_length + index_length) AS 'size' FROM information_schema.TABLES WHERE TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys')"
}

func clearLogs(db *sql.DB) {
	list := []string{
		"FLUSH LOGS",
		"PURGE BINARY LOGS BEFORE DATE_SUB(NOW(), INTERVAL 30 DAY)",
		"FLUSH PRIVILEGES",
		"FLUSH TABLES",
		"FLUSH TABLES WITH READ LOCK",
		"UNLOCK TABLES",
		"FLUSH STATUS",
		"RESET MASTER",
		"RESET SLAVE",
	}

	for _, cmd := range list {
		_, err := db.Exec(cmd)
		if err != nil {
			log.Fatal("Erreur lors de l'exécution de la requête:", err)
		}
	}
}

func setGlobalVariablesOFF(db *sql.DB) {
	globalVariables := []string{
		"SET GLOBAL general_log = 'OFF'",                  // Disable general log to avoid performance issues during cleaning
		"SET GLOBAL slow_query_log = 'OFF'",               // Disable slow query log to avoid performance issues during cleaning
		"SET GLOBAL log_output = 'TABLE'",                 // Set log output to table to avoid performance issues during cleaning
		"SET GLOBAL log_queries_not_using_indexes = 'ON'", // Enable logging of queries not using indexes
		"SET GLOBAL log_slow_admin_statements = 'ON'",     // Enable logging of slow admin statements
		"SET GLOBAL log_slow_slave_statements = 'ON'",     // Enable logging of slow slave statements
	}

	for _, cmd := range globalVariables {
		_, err := db.Exec(cmd)
		if err != nil {
			log.Fatal("Erreur lors de l'exécution de la requête:", err)
		}
	}
}

func setGlobalVariablesON(db *sql.DB) {
	globalVariables := []string{
		"SET GLOBAL general_log = 'ON'",                    // Enable general log to monitor database activities
		"SET GLOBAL slow_query_log = 'ON'",                 // Enable slow query log to monitor slow queries
		"SET GLOBAL log_output = 'FILE'",                   // Set log output to file to monitor database activities
		"SET GLOBAL log_queries_not_using_indexes = 'OFF'", // Disable logging of queries not using indexes
		"SET GLOBAL log_slow_admin_statements = 'OFF'",     // Disable logging of slow admin statements
		"SET GLOBAL log_slow_slave_statements = 'OFF'",     // Disable logging of slow slave statements
	}
	for _, cmd := range globalVariables {
		_, err := db.Exec(cmd)
		if err != nil {
			log.Fatal("Erreur lors de l'exécution de la requête:", err)
		}
	}
}
