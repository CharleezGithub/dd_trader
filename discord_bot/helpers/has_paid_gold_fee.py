import sqlite3

def has_user_paid_fee(discord_id, channel_id):
    # Replace with your database path
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()
    
    try:
        # Get trader ID for the given discord_id
        cursor.execute("""
            SELECT id
            FROM traders
            WHERE discord_id = ?
        """, (discord_id,))
        result = cursor.fetchone()
        if not result:
            print("User not found")
            return False
        trader_id = result[0]
        
        # Fetch payment status based on trader ID
        cursor.execute("""
            SELECT trader1_paid, trader2_paid 
            FROM trades 
            WHERE channel_id = ? AND (trader1_id = ? OR trader2_id = ?)
        """, (channel_id, trader_id, trader_id))
        
        result = cursor.fetchone()
        
        # Check if result is not None and verify payment status
        if result:
            trader1_paid, trader2_paid = result
            if (trader_id == trader_id and trader1_paid) or (trader_id == trader_id and trader2_paid):
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
