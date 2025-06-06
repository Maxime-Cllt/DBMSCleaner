<div align="center">
    <h1>DBMSCleaner</h1>
</div>

<div align="center">
    <img src="https://img.shields.io/badge/Rust-dea584?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
    <img src="https://img.shields.io/badge/Database-Cleaner-53a863?style=for-the-badge" alt="Database Cleaner" />
    <img src="https://img.shields.io/badge/Version-1.0.2-informational?style=for-the-badge" alt="Version" />
</div>

## 📖 Overview

**DBMS Cleaner** is a lightweight, efficient tool designed to optimize and clean your database. Built with Rust, it
ensures optimal performance by reducing storage usage and optimizing all tables (except system tables) without altering
configurations or requiring manual intervention.

Whether you're running a server or an application, DBMS Cleaner keeps your database in peak condition, compatible across
all major platforms.

---

## ✨ Key Features

- 🚀 **Efficient Storage Optimization:** Reduce database size by rebuilding indexes.
- ⚙️ **Table Optimization:** Ensures all tables (excluding system tables) are optimized.
- 🖥️ **Cross-Platform Support:** Seamlessly run on Windows, MacOS, and Linux.
- 🛠️ **Simple Integration:** No changes to FILE configurations required.
- 🛡️ **Safe and Reliable:** Maintains database integrity without the need for backups.
- 🔧 **Customizable:** Easily configure settings via `config.json`.
- ⏱️ **Fast Execution:** Run it as a cron job or scheduled task for continuous optimization.

---

## 🗄️ Supported Databases

<div align="center">
    <img src="https://img.shields.io/badge/MySQL-00758F?style=for-the-badge&logo=mysql&logoColor=white" alt="MySQL" />
    <img src="https://img.shields.io/badge/MariaDB-003545?style=for-the-badge&logo=mariadb&logoColor=white" alt="MariaDB" />
    <img src="https://img.shields.io/badge/PostgreSQL-336791?style=for-the-badge&logo=postgresql&logoColor=white" alt="PostgreSQL" />
</div>

---

## 💻 Platforms & Requirements

<div align="center">
    <img src="https://img.shields.io/badge/OS-MacOS-000000?style=for-the-badge&logo=apple&logoColor=white" alt="MacOS" />
    <img src="https://img.shields.io/badge/OS-Linux-228B22?style=for-the-badge&logo=linux&logoColor=white" alt="Linux" />
    <img src="https://img.shields.io/badge/OS-Windows-0078d4?style=for-the-badge&logo=windows&logoColor=white" alt="Windows" />
</div>

### 📋 Prerequisites

- **Rust Compiler** (Install via [Rustup](https://rustup.rs/))
- **Cargo Package Manager** (Installed with Rust)
- Supported database drivers: `mysql`, `mariadb`, or `postgres`

---

## 🔧 Installation

Follow these steps to get started:

### 1. Clone the Repository

```bash
git clone https://github.com/Maxime-Cllt/DBMSCleaner.git
cd DBMSCleaner
```

### 2. Compile the Program

```bash
cargo build --release
```

### 3. Configure the Connection

Create a file named `cleaner.json` in the same directory as the compiled program with the following content:

```json
{
  "driver": "mysql|mariadb|postgres",
  "host": "localhost",
  "port": "3306",
  "username": "root",
  "password": "",
  "schema": "test"
}
```

You can also include mutliple schemas in the `schema` field, separated by a comma or use the `*` keyword to clean all
schemas (except system schemas).

```json
"schema": "test1,test2,test3"
```

### 4. Run the Program

#### For MacOS & Linux:

```bash
./target/release/DBMSCleaner
```

#### For Windows:

```bash
.\target\release\DBMSCleaner.exe
```

---

## 📝 Notes

- **Exclusions:** Does not clean triggers, stored procedures, functions, or views.
- **Privileges Required:** Ensure the program has sufficient privileges to connect and clean the database.
- **Frequency:** Safe to run frequently for continuous optimization.

---

## 📊 Performance and Results

I use this program to clean my database every night. Here is a graph that show the size of my database before and after
the cleaning process.

### Size benefit

<div align="center">
    <img src="assets/Graph.png" width="50%" height="50%" alt="Graph" />
</div>


## 🔗 See Also

- [SQLiteCleaner](https://github.com/Maxime-Cllt/SqliteCleaner)
- [Rust Language](https://www.rust-lang.org/)

---

### 📜 License

This project is licensed under the [MIT License](https://opensource.org/licenses/MIT).

## 🤝 Contributing

Contributions are welcome! To contribute:

- **Fork the Repository**
- **Create a Feature Branch**:
  ```bash
  git checkout -b feature/your-feature-name
    ```