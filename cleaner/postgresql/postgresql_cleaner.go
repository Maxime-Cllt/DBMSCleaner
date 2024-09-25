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
		log.Fatal("Impossible de se connecter à la base de données:", err)
	}
	defer db.Close()

	err = db.Ping()
	if err != nil {
		log.Fatal("Impossible de se connecter à la base de données:", err)
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
		log.Fatal("Erreur lors de la récupération des tables:", err)
	}
	defer rows.Close()

	var dbName string
	reindex := "REINDEX DATABASE "
	for rows.Next() {
		err := rows.Scan(&dbName)
		if err != nil {
			log.Fatal("Erreur lors de la lecture de la ligne:", err)
		}
		_, err = db.Exec(reindex + dbName)
		if err != nil {
			log.Fatal("Erreur lors de l'exécution de la requête:", err)
		}
	}
}

// cleanAllTables clean all tables in the database
func cleanAllTables(db *sql.DB) {
	// clean all tables
	rows, err := db.Query("SELECT table_schema, table_name FROM information_schema.tables WHERE table_schema NOT IN ('information_schema', 'pg_catalog')")
	if err != nil {
		log.Fatal("Erreur lors de la récupération des tables:", err)
	}
	defer rows.Close()
	vacuum := "VACUUM "
	analyse := "ANALYSE "

	var stmt string
	for rows.Next() {
		err := rows.Scan(&stmt)
		_, err = db.Exec(vacuum + stmt)
		if err != nil {
			log.Fatal("Erreur lors de l'exécution de la requête:", err)
		}
		_, err = db.Exec(analyse + stmt)
		if err != nil {
			log.Fatal("Erreur lors de l'exécution de la requête:", err)
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
		"SELECT pg_switch_wal();",
		"VACUUM FULL;",
		"SELECT pg_rotate_logfile();",
	}

	for _, cmd := range list {
		_, err := db.Exec(cmd)
		if err != nil {
			log.Fatal("Erreur lors de l'exécution de la requête:", err)
		}
	}
}
