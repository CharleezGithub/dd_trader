import sqlite3

# Connect to a database (or create it if it doesn't exist)
conn = sqlite3.connect('sample_database.db')
cursor = conn.cursor()

def create_table():
    # Create a new table
    cursor.execute('''
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        username TEXT NOT NULL,
        age INTEGER NOT NULL
    )
    ''')
    conn.commit()

def insert_data(username, age):
    # Insert a new row of data
    cursor.execute("INSERT INTO users (username, age) VALUES (?, ?)", (username, age))
    conn.commit()

def retrieve_data():
    # Fetch all rows from the table
    cursor.execute("SELECT * FROM users")
    return cursor.fetchall()

def update_data(user_id, new_age):
    # Update data for a specific user
    cursor.execute("UPDATE users SET age = ? WHERE id = ?", (new_age, user_id))
    conn.commit()

def delete_data(user_id):
    # Delete a specific user by ID
    cursor.execute("DELETE FROM users WHERE id = ?", (user_id,))
    conn.commit()

if __name__ == "__main__":
    create_table()
    insert_data("Alice", 30)
    insert_data("Bob", 25)

    users = retrieve_data()
    for user in users:
        print(user)

    update_data(1, 31)  # Update Alice's age to 31

    delete_data(2)  # Delete Bob

    users = retrieve_data()
    for user in users:
        print(user)

    conn.close()
