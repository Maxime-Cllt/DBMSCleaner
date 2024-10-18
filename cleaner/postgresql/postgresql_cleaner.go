package postgresql

import (
	"GoSqlCleaner/database"
	"GoSqlCleaner/util"
	"database/sql"
	"fmt"
	_ "github.com/lib/pq"
	"log"
)

// Postgresql struct
type Postgresql struct {
	database.Database
}

// Clean cleans the database
func (c *Postgresql) Clean() bool {

	dsn := fmt.Sprintf("host=%s port=%s user=%s password=%s dbname=%s sslmode=disable", c.Database.Host, c.Database.Port, c.Database.Username, c.Database.Password, c.Database.Database)

	db, err := sql.Open("postgres", dsn)
	if err != nil {
		log.Fatal("Error connecting to the database:", err)
	}
	defer func(db *sql.DB) {
		err := db.Close()
		if err != nil {
			log.Fatal("Error while closing the database connection:", err)
		}
	}(db)

	err = db.Ping()
	if err != nil {
		log.Fatal("Error connecting to the database:", err)
	}

	totalSize := getTotalSizeSql()
	startSize := util.GetDbSize(db, totalSize)

	println("Start size of the database:", util.Green, startSize, " bytes", util.Reset)

	// Reindex database
	println("Reindexing database...")
	reindexDatabase(db)

	// Clean all tables
	println("Cleaning all tables...")
	cleanAllTables(db)

	// Clean temporary tables and bloat
	println("Cleaning up temporary tables and bloat...")
	clearTempTablesAndBloat(db)

	// Clear logs
	println("Clearing logs...")
	clearLogs(db)

	endSize := util.GetDbSize(db, totalSize)
	println("End size of the database:", util.Green, endSize, " bytes", util.Reset)
	println("Optimization of ", util.Green, startSize-endSize, util.Reset, " bytes")

	util.LogMessage(startSize, endSize)
	return true
}

// reindexDatabase reindex all databases except template0 and template1
func reindexDatabase(db *sql.DB) {
	rows, err := db.Query("SELECT datname FROM pg_database WHERE datname NOT IN ('template0', 'template1');")
	if err != nil {
		log.Fatal("Error while fetching databases:", err)
	}
	defer func(rows *sql.Rows) {
		err := rows.Close()
		if err != nil {
			log.Fatal("Error while closing the rows:", err)
		}
	}(rows)

	var dbName string
	const reindex string = "REINDEX DATABASE "
	for rows.Next() {
		err := rows.Scan(&dbName)
		if err != nil {
			log.Fatal("Error while scanning the database name:", err)
		}
		_, err = db.Exec(reindex + dbName + ";")
		if err != nil {
			log.Fatal("Error while executing the command:", err)
		}
	}
}

// cleanAllTables clean all tables in the database
func cleanAllTables(db *sql.DB) {
	rows, err := db.Query("SELECT table_schema, table_name FROM information_schema.tables WHERE table_schema NOT IN ('information_schema', 'pg_catalog');")
	if err != nil {
		log.Fatal("Error while fetching tables:", err)
	}
	defer func(rows *sql.Rows) {
		err := rows.Close()
		if err != nil {
			log.Fatal("Error while closing the rows:", err)
		}
	}(rows)

	const vacuumStr = "VACUUM FULL "
	const analyseStr = "ANALYZE "

	var schema, table string
	for rows.Next() {
		err := rows.Scan(&schema, &table)
		stmt := fmt.Sprintf("%s.%s", schema, table)

		_, err = db.Exec(vacuumStr + stmt + ";")
		if err != nil {
			log.Fatal("Error while executing the command:", err)
		}

		_, err = db.Exec(analyseStr + stmt + ";")
		if err != nil {
			log.Fatal("Error while executing the command:", err)
		}
	}
}

// getTotalSizeSql get the total size of the database in bytes
func getTotalSizeSql() string {
	return "SELECT SUM(pg_database_size(datname)) AS total_size_bytes FROM pg_database;"
}

// clearLogs clear all logs in the database
func clearLogs(db *sql.DB) {
	list := []string{
		"CHECKPOINT;",
		"SELECT pg_switch_wal();",     // Switch WAL segment files
		"VACUUM FULL;",                // Reclaim space more aggressively
		"SELECT pg_rotate_logfile();", // Rotate the transaction log files
	}

	for _, cmd := range list {
		_, err := db.Exec(cmd)
		if err != nil {
			log.Fatal("Error while executing the command:", err)
		}
	}
}

// clearTempTablesAndBloat clean up temporary tables and remove bloat
func clearTempTablesAndBloat(db *sql.DB) {
	_, err := db.Exec("DROP TABLE IF EXISTS pg_temp CASCADE;") // Remove temporary tables
	if err != nil {
		log.Fatal("Error while dropping temporary tables:", err)
	}

	_, err = db.Exec("VACUUM FULL;") // Reclaim space more aggressively
	if err != nil {
		log.Fatal("Error while executing the command:", err)
	}
}
