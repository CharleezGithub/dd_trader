import sqlite3

conn = sqlite3.connect('trades.db')
cursor = conn.cursor()

# Create 'trade_items' table to list all channels
cursor.execute('''
CREATE TABLE IF NOT EXISTS trade_items (
    channel_id INTEGER PRIMARY KEY
)
''')

# Create 'trade_data' table to store the trade data
cursor.execute('''
CREATE TABLE IF NOT EXISTS trade_data (
    id INTEGER PRIMARY KEY,
    channel_id INTEGER NOT NULL REFERENCES trade_items(channel_id),
    user_id INTEGER NOT NULL,
    item_link TEXT NOT NULL,
    UNIQUE(channel_id, user_id, item_link)
)
''')

conn.commit()
conn.close()
