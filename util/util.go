package util

import (
	"database/sql"
	"log"
)

func GetDbSize(db *sql.DB, sql string) int {
	var size int
	err := db.QueryRow(sql).Scan(&size)
	if err != nil {
		log.Fatal("Erreur lors de la récupération de la taille de la base de données:", err)
	}
	return size
}
