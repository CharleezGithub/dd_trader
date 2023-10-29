import sqlite3


def has_other_trader_escrow_items(discord_id: str, channel_id: str) -> bool:
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

    trader_id = trader_id[0] # get the actual ID value

    # Retrieve the trade data with the given channel ID
    cursor.execute("SELECT trader1_id, trader2_id FROM trades WHERE channel_id=?", (channel_id,))
    trade_data = cursor.fetchone()

    # If no trade exists for the given channel, return False
    if not trade_data:
        conn.close()
        return False

    trader1_id, trader2_id = trade_data

    # Identify the other trader
    other_trader_id = trader1_id if trader_id == trader2_id else trader2_id

    # Check for items with the status "in escrow" for the other trader
    cursor.execute("SELECT id FROM items WHERE trader_id=? AND status='in escrow'", (other_trader_id,))
    items = cursor.fetchall()

    # Close the connection to the database
    conn.close()

    # Return True if there are items with the status "in escrow" for the other trader, otherwise return False
    return len(items) > 0


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
    if len(items) > 0:
        return len(items) > 0
    
    # IF there where no untraded items, then check if there is untraded gold
    # Check if the trader is trader1 or trader2 in the channel
    cursor.execute("""
        SELECT trader1_id, trader2_id, trader1_gold, trader2_gold
        FROM trades
        INNER JOIN traders ON traders.id = trades.trader1_id OR traders.id = trades.trader2_id
        WHERE traders.discord_id = ? AND trades.channel_id = ?
    """, (discord_id, channel_id))
    
    result = cursor.fetchone()
    
    # Close the connection
    conn.close()
    
    if result:
        # Unpack the result
        trader1_id, trader2_id, trader1_gold, trader2_gold = result
        
        # Check which trader the discord_id corresponds to and if their gold is greater than 0
        if trader1_id and trader1_gold - 30 > 0:
            return True
        elif trader2_id and trader2_gold - 30 > 0:
            return True
        else:
            return False
    else:
        return False


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

    trade_id = trade_id[0]  # get the actual trade ID value

    # Check for items with the status "not traded" linked with the trade
    cursor.execute("SELECT id FROM items WHERE trade_id=? AND status='not traded'", (trade_id,))
    items = cursor.fetchall()

    # Close the connection to the database
    conn.close()

    # If any item has the status "not traded", return False, otherwise return True
    return len(items) == 0