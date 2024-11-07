package util

import (
	"DBMSCleaner/database"
	"database/sql"
	"encoding/json"
	"io/ioutil"
	"log"
	"os"
)

// GetDbSize returns the size of the database in bytes for the given SQL query
func GetDbSize(db *sql.DB, sql string) int {
	var size int
	err := db.QueryRow(sql).Scan(&size)
	if err != nil {
		log.Fatal("Erreur lors de la récupération de la taille de la base de données:", err)
	}
	return size
}

// GetDbConfig reads the config.json file and returns the database configuration as DBConfig
func GetDbConfig() database.DBConfig {
	jsonFile, err := os.Open("config.json")
	if err != nil {
		log.Fatal(err)
	}
	defer func(jsonFile *os.File) {
		err := jsonFile.Close()
		if err != nil {
			log.Fatal("Error while closing the database connection:", err)
		}
	}(jsonFile)

	byteValue, err := ioutil.ReadAll(jsonFile)
	if err != nil {
		log.Fatal(err)
	}

	config := database.DBConfig{}

	err = json.Unmarshal(byteValue, &config)
	if err != nil {
		log.Fatal(err)
	}
	return config
}
