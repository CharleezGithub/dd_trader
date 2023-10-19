import sqlite3


def check_all_items_in_escrow(channel_id):
    # Replace with your database path
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    try:
        cursor.execute(
            """
            SELECT items.status
            FROM items
            JOIN trades ON items.trade_id = trades.id
            WHERE trades.channel_id = ?
        """,
            (channel_id,),
        )

        statuses = cursor.fetchall()

        # Check if all item statuses in the trade are "in escrow"
        return all(status[0] == "in escrow" for status in statuses)
    except sqlite3.Error as e:
        print(f"An error occurred: {e}")
        return False
    finally:
        conn.close()
