import sqlite3

def has_user_paid_fee(discord_id, channel_id):
    # Replace with your database path
    conn = sqlite3.connect("trading_bot_test.db")
    cursor = conn.cursor()
    
    try:
        cursor.execute("""
            SELECT trader1_paid, trader2_paid, trader1_id, trader2_id
            FROM trades
            JOIN traders ON traders.id = trades.trader1_id OR traders.id = trades.trader2_id
            WHERE trades.channel_id = ? AND traders.discord_id = ?
        """, (channel_id, discord_id))
        
        result = cursor.fetchone()
        
        # Check if result is not None and verify payment status
        if result:
            trader1_paid, trader2_paid, trader1_id, trader2_id = result
            if (discord_id == trader1_id and trader1_paid) or (discord_id == trader2_id and trader2_paid):
                return True
        
        return False
    except sqlite3.Error as e:
        print(f"An error occurred: {e}")
        return False
    finally:
        conn.close()

# Example usage:
# discord_id = "user_discord_id_here"
# channel_id = "channel_id_here"
# if has_user_paid_fee(discord_id, channel_id):
#     print(f"User {discord_id} has paid the fee for trade in channel {channel_id}!")
# else:
#     print(f"User {discord_id} has NOT paid the fee for trade in channel {channel_id}.")
