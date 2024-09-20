package database

type Database struct {
	Host     string
	Port     string
	Username string
	Password string
	Database string
	Driver   string
}

func (d *Database) DisplayInfo() string {
	return "Host: " + d.Host + " Port: " + d.Port + " Username: " + d.Username + " Password: " + d.Password + " Database: " + d.Database + " Driver: " + d.Driver
}
