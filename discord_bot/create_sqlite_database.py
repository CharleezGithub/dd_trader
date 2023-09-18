import sqlite3

conn = sqlite3.connect('trades.db')
cursor = conn.cursor()

cursor.execute('''
CREATE TABLE IF NOT EXISTS trade_items (
    channel_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    item_link TEXT NOT NULL,
    PRIMARY KEY(channel_id, user_id, item_link)
)
''')

conn.commit()
conn.close()
