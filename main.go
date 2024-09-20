package main

import (
	"GoSqlCleaner/cleaner"
	"GoSqlCleaner/cleaner/mariadb"
	"GoSqlCleaner/cleaner/mysql"
	"GoSqlCleaner/cleaner/postgresql"
	"GoSqlCleaner/database"
	"GoSqlCleaner/util"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"time"
)

func main() {

	// Get the database configuration
	config := GetDbConfig()

	// Create the database info
	databaseInfo := database.Database{
		Host:     config.Host,
		Port:     config.Port,
		Username: config.User,
		Password: config.Password,
		Database: config.Database,
		Driver:   config.Driver,
	}

	// Create the cleaner map for the supported databases
	baseCleaners := map[string]func() cleaner.Cleaner{
		"mysql":      func() cleaner.Cleaner { return &mysql.MySqlCleaner{Database: databaseInfo} },
		"mariadb":    func() cleaner.Cleaner { return &mariadb.MariaDbCleaner{Database: databaseInfo} },
		"postgresql": func() cleaner.Cleaner { return &postgresql.Postgresql{Database: databaseInfo} },
	}

	// Get the current time in nanoseconds
	startTime := time.Now().UnixNano()
	cleanerKey := config.Driver
	if cleanerFunc, exists := baseCleaners[cleanerKey]; exists {
		cleaner := cleanerFunc()
		result := cleaner.Clean()
		if result {
			fmt.Println("Database cleaned successfully for:", cleanerKey)
		} else {
			fmt.Println("Error while executing cleaner for:", util.Red, cleanerKey, util.Reset)
		}
	}

	fmt.Println("Time taken to execute the cleaner:", util.Green, (time.Now().UnixNano()-startTime)/1000000, "ms", util.Reset)
}

func GetDbConfig() database.DBConfig {
	jsonFile, err := os.Open("config.json")
	if err != nil {
		log.Fatal(err)
	}
	defer jsonFile.Close()

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
