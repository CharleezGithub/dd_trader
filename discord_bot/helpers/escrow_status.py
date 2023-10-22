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
        return all(status[0] == "in escrow" or "traded" for status in statuses)
    except sqlite3.Error as e:
        print(f"An error occurred: {e}")
        return False
    finally:
        conn.close()


def has_untraded_items(discord_id: str, channel_id: str) -> bool:
    # Connect to the database
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    # Retrieve the ID of the trader with the given discord_id
    cursor.execute("SELECT id FROM traders WHERE discord_id=?", (discord_id,))
    trader_id = cursor.fetchone()

    # If trader does not exist, return False
    if not trader_id:
        conn.close()
        return False

    trader_id = trader_id[0]  # get the actual ID value

    # Retrieve the trade ID with the given channel ID
    cursor.execute("SELECT id FROM trades WHERE channel_id=?", (channel_id,))
    trade_id = cursor.fetchone()

    # If no trade exists for the given channel, return False
    if not trade_id:
        conn.close()
        return False

    trade_id = trade_id[0]  # get the actual trade ID value

    # Check for items with the tag "not traded" for the trader
    cursor.execute(
        "SELECT id FROM items WHERE trade_id=? AND trader_id=? AND status='not traded'",
        (trade_id, trader_id),
    )
    items = cursor.fetchall()

    # Close the connection to the database
    conn.close()

    # Return True if there are untraded items, otherwise return False
    return len(items) > 0
