import sqlite3

# Create a connection to the database (This will create the file if it doesn't exist)
conn = sqlite3.connect("trading_bot.db")
cursor = conn.cursor()

# Create traders table
cursor.execute(
    """
CREATE TABLE IF NOT EXISTS traders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    discord_id TEXT NOT NULL UNIQUE
);
"""
)

# Create trades table
cursor.execute(
    """
CREATE TABLE IF NOT EXISTS trades (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trader1_id INTEGER,
    trader2_id INTEGER,
    channel_id TEXT NOT NULL UNIQUE,
    trader1_gold INTEGER DEFAULT 0,
    trader2_gold INTEGER DEFAULT 0,
    trader1_gold_traded INTEGER DEFAULT 0,
    trader2_gold_traded INTEGER DEFAULT 0,
    trader1_gold_received INTEGER DEFAULT 0,
    trader2_gold_received INTEGER DEFAULT 0,
    trader1_paid BOOLEAN DEFAULT 0,
    trader2_paid BOOLEAN DEFAULT 0,
    status TEXT DEFAULT 'ongoing',
    locked BOOLEAN DEFAULT 0,
    FOREIGN KEY (trader1_id) REFERENCES traders(id),
    FOREIGN KEY (trader2_id) REFERENCES traders(id)
);
"""
)

# Create items table
cursor.execute(
    """
CREATE TABLE IF NOT EXISTS items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trade_id INTEGER,
    trader_id INTEGER,
    item_image_url TEXT NOT NULL,
    info_image_url TEXT NOT NULL,
    status TEXT DEFAULT 'not traded',
    FOREIGN KEY (trade_id) REFERENCES trades(id),
    FOREIGN KEY (trader_id) REFERENCES traders(id)
);
"""
)

# Commit changes and close the connection
conn.commit()
conn.close()

print("Database modified successfully!")
