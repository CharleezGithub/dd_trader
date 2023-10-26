import sqlite3


def archive_trades_by_channel(channel_id, archive_db_name="trading_bot_archive.db"):
    # Connect to the current database
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    # Connect to the archive database
    archive_conn = sqlite3.connect(archive_db_name)
    archive_cursor = archive_conn.cursor()

    # Create traders table in archive
    archive_cursor.execute(
        """
        CREATE TABLE IF NOT EXISTS traders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            discord_id TEXT NOT NULL UNIQUE
        );
        """
    )

    # Create trades table in archive
    archive_cursor.execute(
        """
        CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            trader1_id INTEGER,
            trader2_id INTEGER,
            channel_id TEXT NOT NULL UNIQUE,
            trader1_gold INTEGER DEFAULT 0,
            trader2_gold INTEGER DEFAULT 0,
            trader1_gold_traded INTEGER DEFAULT 0,
            trader2_gold_traded INTEGER DEFAULT 0,
            trader1_paid BOOLEAN DEFAULT 0,
            trader2_paid BOOLEAN DEFAULT 0,
            status TEXT DEFAULT 'ongoing'
        );
        """
    )

    # Create items table in archive
    archive_cursor.execute(
        """
        CREATE TABLE IF NOT EXISTS items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            trade_id INTEGER,
            trader_id INTEGER,
            item_image_url TEXT NOT NULL,
            info_image_url TEXT NOT NULL,
            status TEXT DEFAULT 'not traded'
        );
        """
    )

    # Fetch unique traders associated with the trade
    cursor.execute(
        """
    SELECT DISTINCT traders.id, traders.discord_id
    FROM trades
    JOIN traders ON trades.trader1_id = traders.id OR trades.trader2_id = traders.id
    WHERE trades.channel_id=?
    """,
        (channel_id,),
    )

    traders_data = cursor.fetchall()

    # Insert unique traders into the archive database's traders table
    for trader in traders_data:
        try:
            archive_cursor.execute(
                "INSERT INTO traders (id, discord_id) VALUES (?, ?)", trader
            )
        except (
            sqlite3.IntegrityError
        ):  # Handle cases where trader is already in archive DB
            pass

    # Fetch records from the current database that match the channel_id
    cursor.execute(
        """
    SELECT trades.*, t1.discord_id as trader1_discord, t2.discord_id as trader2_discord 
    FROM trades 
    LEFT JOIN traders as t1 ON trades.trader1_id = t1.id 
    LEFT JOIN traders as t2 ON trades.trader2_id = t2.id 
    WHERE trades.channel_id=?
    """,
        (channel_id,),
    )

    trades_data = cursor.fetchall()

    cursor.execute(
        """
    SELECT items.* FROM items 
    JOIN trades ON trades.id = items.trade_id 
    WHERE trades.channel_id=?""",
        (channel_id,),
    )

    items_data = cursor.fetchall()

    # Insert records into the archive database
    for trade in trades_data:
        archive_cursor.execute(
            """
            INSERT INTO trades (trader1_id, trader2_id, channel_id, trader1_gold, trader2_gold, trader1_gold_traded, trader2_gold_traded, trader1_gold_received, trader2_gold_received, trader1_paid, trader2_paid, status) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            trade[1:13],
        )

    for item in items_data:
        archive_cursor.execute(
            "INSERT INTO items (trade_id, trader_id, item_image_url, info_image_url, status) VALUES (?, ?, ?, ?, ?)",
            item[1:],
        )

    # Commit changes to the archive database
    archive_conn.commit()
    archive_conn.close()

    print(f"Records associated with channel_id {channel_id} archived successfully!")


archive_trades_by_channel("1161644569419579452")
