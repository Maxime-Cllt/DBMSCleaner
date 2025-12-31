#!/bin/bash
set -e

echo "🔧 Initializing test databases with sample data..."

# PostgreSQL
echo "📊 Creating test tables in PostgreSQL..."
docker exec -i dbms-cleaner-postgres psql -U postgres -d test_db <<-EOSQL
    CREATE TABLE IF NOT EXISTS users (
        id SERIAL PRIMARY KEY,
        username VARCHAR(50) NOT NULL,
        email VARCHAR(100),
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS products (
        id SERIAL PRIMARY KEY,
        name VARCHAR(100) NOT NULL,
        price DECIMAL(10, 2),
        description TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS orders (
        id SERIAL PRIMARY KEY,
        user_id INTEGER REFERENCES users(id),
        product_id INTEGER REFERENCES products(id),
        quantity INTEGER,
        order_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );

    -- Insert test data
    INSERT INTO users (username, email) VALUES
        ('user1', 'user1@example.com'),
        ('user2', 'user2@example.com'),
        ('user3', 'user3@example.com');

    INSERT INTO products (name, price, description) VALUES
        ('Product A', 19.99, 'Description for Product A'),
        ('Product B', 29.99, 'Description for Product B'),
        ('Product C', 39.99, 'Description for Product C');

    INSERT INTO orders (user_id, product_id, quantity) VALUES
        (1, 1, 2),
        (2, 2, 1),
        (3, 3, 3);

    -- Create some indexes
    CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
    CREATE INDEX IF NOT EXISTS idx_products_name ON products(name);
    CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id);
EOSQL

# MySQL
echo "📊 Creating test tables in MySQL..."
docker exec -i dbms-cleaner-mysql mysql -u root -pmysql_password test_db <<-EOSQL
    CREATE TABLE IF NOT EXISTS users (
        id INT AUTO_INCREMENT PRIMARY KEY,
        username VARCHAR(50) NOT NULL,
        email VARCHAR(100),
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    ) ENGINE=InnoDB;

    CREATE TABLE IF NOT EXISTS products (
        id INT AUTO_INCREMENT PRIMARY KEY,
        name VARCHAR(100) NOT NULL,
        price DECIMAL(10, 2),
        description TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    ) ENGINE=InnoDB;

    CREATE TABLE IF NOT EXISTS orders (
        id INT AUTO_INCREMENT PRIMARY KEY,
        user_id INT,
        product_id INT,
        quantity INT,
        order_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (user_id) REFERENCES users(id),
        FOREIGN KEY (product_id) REFERENCES products(id)
    ) ENGINE=InnoDB;

    -- Insert test data
    INSERT INTO users (username, email) VALUES
        ('user1', 'user1@example.com'),
        ('user2', 'user2@example.com'),
        ('user3', 'user3@example.com');

    INSERT INTO products (name, price, description) VALUES
        ('Product A', 19.99, 'Description for Product A'),
        ('Product B', 29.99, 'Description for Product B'),
        ('Product C', 39.99, 'Description for Product C');

    INSERT INTO orders (user_id, product_id, quantity) VALUES
        (1, 1, 2),
        (2, 2, 1),
        (3, 3, 3);

    -- Create some indexes
    CREATE INDEX idx_users_email ON users(email);
    CREATE INDEX idx_products_name ON products(name);
    CREATE INDEX idx_orders_user_id ON orders(user_id);
EOSQL

# MariaDB
echo "📊 Creating test tables in MariaDB..."
docker exec -i dbms-cleaner-mariadb mysql -u root -pmariadb_password test_db <<-EOSQL
    CREATE TABLE IF NOT EXISTS users (
        id INT AUTO_INCREMENT PRIMARY KEY,
        username VARCHAR(50) NOT NULL,
        email VARCHAR(100),
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    ) ENGINE=InnoDB;

    CREATE TABLE IF NOT EXISTS products (
        id INT AUTO_INCREMENT PRIMARY KEY,
        name VARCHAR(100) NOT NULL,
        price DECIMAL(10, 2),
        description TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    ) ENGINE=InnoDB;

    CREATE TABLE IF NOT EXISTS orders (
        id INT AUTO_INCREMENT PRIMARY KEY,
        user_id INT,
        product_id INT,
        quantity INT,
        order_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (user_id) REFERENCES users(id),
        FOREIGN KEY (product_id) REFERENCES products(id)
    ) ENGINE=InnoDB;

    -- Insert test data
    INSERT INTO users (username, email) VALUES
        ('user1', 'user1@example.com'),
        ('user2', 'user2@example.com'),
        ('user3', 'user3@example.com');

    INSERT INTO products (name, price, description) VALUES
        ('Product A', 19.99, 'Description for Product A'),
        ('Product B', 29.99, 'Description for Product B'),
        ('Product C', 39.99, 'Description for Product C');

    INSERT INTO orders (user_id, product_id, quantity) VALUES
        (1, 1, 2),
        (2, 2, 1),
        (3, 3, 3);

    -- Create some indexes
    CREATE INDEX idx_users_email ON users(email);
    CREATE INDEX idx_products_name ON products(name);
    CREATE INDEX idx_orders_user_id ON orders(user_id);
EOSQL

echo "✅ Test data initialized successfully!"
echo ""
echo "You can now run the DBMSCleaner to test the optimization."
