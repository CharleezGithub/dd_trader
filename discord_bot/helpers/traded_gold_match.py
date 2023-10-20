import sqlite3


def check_gold(channel_id):
    # Connect to the database
    conn = sqlite3.connect("trading_bot.db")
    # Set the row factory to access results using column names
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    # Retrieve the relevant trade record using the channel ID
    cursor.execute("SELECT * FROM trades WHERE channel_id=?", (channel_id,))
    trade = cursor.fetchone()

    if not trade:
        conn.close()
        return False, "Trade not found for the given channel ID"

    insufficient_traders = []

    # Check trader1's gold
    if trade["trader1_gold"] > trade["trader1_gold_traded"]:
        cursor.execute(
            "SELECT discord_id FROM traders WHERE id=?", (trade["trader1_id"],)
        )
        trader1_discord = cursor.fetchone()
        insufficient_traders.append(trader1_discord["discord_id"])

    # Check trader2's gold
    if trade["trader2_gold"] > trade["trader2_gold_traded"]:
        cursor.execute(
            "SELECT discord_id FROM traders WHERE id=?", (trade["trader2_id"],)
        )
        trader2_discord = cursor.fetchone()
        insufficient_traders.append(trader2_discord["discord_id"])

    conn.close()

    if len(insufficient_traders) == 0:
        return True, None
    else:
        return False, insufficient_traders


def handle_check_result(result):
    has_enough_gold, traders_missing = result

    if traders_missing is None:
        print("Both traders have paid.")
    elif isinstance(traders_missing, str):  # It's an error message
        print(traders_missing)
    else:  # It's a list of discord IDs
        if len(traders_missing) == 1:
            print(f"Trader with discord ID {traders_missing[0]} doesn't have enough gold.")
        else:
            print(f"Traders with discord IDs {', '.join(traders_missing)} don't have enough gold.")


# Test
channel_id = '1161644569419579452'
result = check_gold(channel_id)
handle_check_result(result)