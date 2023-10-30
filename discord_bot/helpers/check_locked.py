import sqlite3


def all_items_traded(channel_id: str) -> bool:
    # Connect to the database
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    # Retrieve the trade ID with the given channel ID
    cursor.execute("SELECT id FROM trades WHERE channel_id=?", (channel_id,))
    trade_id = cursor.fetchone()

    # If no trade exists for the given channel, return False
    if not trade_id:
        conn.close()
        return False

    # Check for items with the status "not traded" linked with the trade
    cursor.execute(
        "SELECT locked FROM trades WHERE channel_id=?", (channel_id,)
    )
    locked_status = cursor.fetchone()

    # Close the connection to the database
    conn.close()

    # returns false if the trade is not locked and returns true if the trade is locked
    return locked_status
