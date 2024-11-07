package main

import (
	"DBMSCleaner/cleaner"
	"DBMSCleaner/cleaner/mariadb"
	"DBMSCleaner/cleaner/mysql"
	"DBMSCleaner/cleaner/postgresql"
	"DBMSCleaner/database"
	"DBMSCleaner/util"
	"fmt"
	"time"
)

func main() {

	// Get the database configuration
	configFile := util.GetDbConfig()

	// Create the database info
	databaseInfo := database.Database{
		Host:     configFile.Host,
		Port:     configFile.Port,
		Username: configFile.User,
		Password: configFile.Password,
		Database: configFile.Database,
		Driver:   configFile.Driver,
	}

	// Create the cleaner map for the supported databases
	baseCleaners := map[string]func() cleaner.Cleaner{
		"mysql":      func() cleaner.Cleaner { return &mysql.MySqlCleaner{Database: databaseInfo} },
		"mariadb":    func() cleaner.Cleaner { return &mariadb.MariaDbCleaner{Database: databaseInfo} },
		"postgresql": func() cleaner.Cleaner { return &postgresql.Postgresql{Database: databaseInfo} },
	}

	// Get the current time in nanoseconds
	startTime := time.Now().UnixNano()
	cleanerKey := configFile.Driver
	if cleanerFunc, exists := baseCleaners[cleanerKey]; exists {
		cleanerInterface := cleanerFunc()
		result := cleanerInterface.Clean()
		if result {
			fmt.Println(util.Green+"Database cleaned successfully for:", cleanerKey, util.Reset)
		} else {
			fmt.Println("Error while executing cleanerInterface for:", util.Red, cleanerKey, util.Reset)
		}
	}

	fmt.Println("Time taken to execute the cleaner:", util.Green, (time.Now().UnixNano()-startTime)/1000000, "ms", util.Reset)
}
