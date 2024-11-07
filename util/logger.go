package util

import (
	"fmt"
	"os"
	"time"
)

// writeLog writes the log message to the log file
func writeLog(logMessage string) error {
	file, err := os.OpenFile("DBMSCleaner.log", os.O_RDWR|os.O_CREATE|os.O_APPEND, 0644)
	if err != nil {
		return fmt.Errorf("error opening the file: %v", err)
	}
	defer func(file *os.File) {
		err := file.Close()
		if err != nil {
			fmt.Println("Error while closing the file:", err)
		}
	}(file)

	timestamp := time.Now().Format("2006-01-02 15:04:05")

	fullMessage := fmt.Sprintf("%s: %s\n", timestamp, logMessage)

	_, err = file.WriteString(fullMessage)
	if err != nil {
		return fmt.Errorf("error writing to the file: %v", err)
	}

	return nil
}

// LogMessage logs the message to the log file
func LogMessage(startSize int, endSize int) {
	logMsg := fmt.Sprintf("FROM [%d] bytes TO [%d] bytes | OPTIMIZATION [%d] bytes", startSize, endSize, startSize-endSize)
	err := writeLog(logMsg)
	if err != nil {
		return
	}
}
