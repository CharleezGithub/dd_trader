import sqlite3


def has_other_trader_gold_left(discord_id: str, channel_id: str) -> bool:
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

    # Retrieve the trade with the given channel ID
    cursor.execute(
        "SELECT trader1_id, trader2_id, trader1_gold_traded, trader2_gold_traded FROM trades WHERE channel_id=?",
        (channel_id,),
    )
    trade_data = cursor.fetchone()

    # If trade does not exist for given channel, return False
    if not trade_data:
        conn.close()
        return False

    trader1_id, trader2_id, trader1_gold_traded, trader2_gold_traded = trade_data

    # Check the gold traded by the other trader
    if trader_id == trader1_id:
        other_trader_gold_traded = trader2_gold_traded
    elif trader_id == trader2_id:
        other_trader_gold_traded = trader1_gold_traded
    else:
        # If the given trader is not part of the trade in the given channel, return False
        conn.close()
        return False

    # Close the connection to the database
    conn.close()

    # Return True if the other trader still has gold left, otherwise return False
    return other_trader_gold_traded > 0


# Testing
print(has_other_trader_gold_left("545698998221144084", "1161644569419579452"))
