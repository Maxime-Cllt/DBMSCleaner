<div align=center>
<img src="/assets/dbcleaner.png" width="100px" height="100px"  alt="DBCleaner" align="center" />
<h1>DBCleaner</h1>
</div>

## Description

DBCleaner is a program made to be run on the backend of a server or an application to clean the database. It will reduce
the storage of the database and optimise all tables except system tables. It is a simple and efficient way to keep your
database clean and optimised without having to do it manually. It is a great way to keep your database running in the
best
conditions possible. Using Go language, it is compatible with all platforms and can be run on any server or application.

## Features

<ul>
<li>Reduce storage of the database</li>
<li>Optimise all tables except system tables</li>
<li>Simple and efficient way to keep your database clean</li>
<li>Compatible with all platforms</li>
<li>Maintain your database in the best conditions possible</li>
<li>Don't require any dump or backup</li>
<li>Easily run on any server or application</li>
<li>Easy to use</li>
</ul>

## Supported Databases

- [x] MySQL
- [x] MariaDB
- [x] PostgreSQL

## Platforms & Requirements

<div align="center">
<img src="https://img.shields.io/badge/OS-MacOS-informational?style=flat&logo=apple&logoColor=white&color=53a863" alt="MacOS" />
<img src="https://img.shields.io/badge/OS-Linux-informational?style=flat&logo=linux&logoColor=white&color=53a863" alt="Linux" />
<img src="https://img.shields.io/badge/OS-Windows-informational?style=flat&logo=windows&logoColor=white&color=53a863" alt="Windows" />
</div>

<div align="center">
<img src="https://img.shields.io/badge/Golang-1.16-informational?style=flat&logo=go&logoColor=white&color=53a863" alt="Golang" />
</div>

## Installation

To run the program :

1. Clone the repository:

```bash
git clone https://github.com/Maxime-Cllt/GoSqlCleaner.git
```

2. Import the libraries:

```bash
go mod tidy
```

3. Compile the program:

```bash
go build -o GoSqlCleaner
```

4. Run the program with the following your database information:

You need to create a file named `config.json` in the same directory as the program with the following content:

```json
{
  "driver": "mysql", 
  "host": "localhost",
  "port": "3306",
  "username": "root",
  "password": "password",
  "database": "testdb"
}
```

Then run the program with the following command:

### MacOS & Linux

```bash
./GoSqlCleaner
```

### Windows

```bash
GoSqlCleaner.exe
```

## Notes

- Time complexity: O(n) where n is the number of tables in the database
- Don't clean triggers, stored procedures, functions, and views
- May not reduce much storage but don't cost much time to run and can be run frequently
- Require some privileges to connect to the database and to perform the cleaning

