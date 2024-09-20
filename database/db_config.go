package database

type DBConfig struct {
	Host     string `json:"host"`
	Port     string `json:"port"`
	User     string `json:"user"`
	Password string `json:"password"`
	Database string `json:"database"`
	Driver   string `json:"driver"`
}

func (c *DBConfig) DisplayInfo() {
	println("Host:", c.Host)
	println("Port:", c.Port)
	println("User:", c.User)
	println("Password:", c.Password)
	println("Database:", c.Database)
	println("Driver:", c.Driver)
}
