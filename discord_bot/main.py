import requests
import threading
import os
import time

from io import BytesIO

import sqlite3

import queue
import asyncio

import discord
from discord.ext import commands

from PIL import Image
from PIL import ImageDraw, ImageFont

from helpers.stitching import stitch_images

TOKEN = "MTE1MTQ0MTUwMjk1NDg0ODMwNw.GtRAIh.YBUChh8QJi3Cs8jeFbuE18kRJYrAwiCpcxcnz8"

intents = discord.Intents.default()
intents.members = True
intents.messages = True

bot = commands.Bot(command_prefix="!", intents=intents)

# Dictionary to store trade requests. Format: {requester_id: requestee_id}
trade_requests = {}

# Dict to store unlock requests from traders. It requires both traders to unlock a trade.
# Format:
"""
{
    "channel_id_1": ["discord_id_of_trader1", "discord_id_of_trader2"],
    "channel_id_2": ["discord_id_of_trader3"],
    # ... additional entries for other trade channels
}
"""
unlock_requests = {}

# Stores channels and their time to be deleted. They are set to be deleted 1 hour after a user requests to end a trade
# Format: {channel_id: 1 hour ahead in unix time}
channels_to_be_deleted = {}

# Instantiate the queue
trade_queue = queue.Queue()

response_file_path = "shared/ipc_communication.txt"

def delete_ended_trade_channels():
    while True:
        try:
            for channel_id, deletion_time_unix in channels_to_be_deleted:
                if time.time() > deletion_time_unix:
                    channel = bot.get_channel(channel_id)
                    channel.delete()
        except Exception as e:
            print("Error looking through dict.\nError:", e)
        time.sleep(120)

t = threading.Thread(delete_ended_trade_channels, deamon=True)
t.start()

def read_file_contents(path):
    with open(path, "r") as file:
        return file.read()


def file_has_changed(path, last_mod_time):
    try:
        current_mod_time = os.stat(path).st_mtime
        if current_mod_time != last_mod_time:
            contents = read_file_contents(path)
            return True, current_mod_time, contents
        else:
            return False, current_mod_time, None
    except FileNotFoundError:
        return None, None, None  # Indicate the file is not accessible


def monitor_file_changes(path_to_watch, interval=1):
    last_mod_time = os.stat(path_to_watch).st_mtime

    while True:
        changed, new_mod_time, contents = file_has_changed(path_to_watch, last_mod_time)
        if changed is None:  # File not found or inaccessible
            return  # Stop the generator
        if changed:
            yield contents  # Yield the new contents of the file
            last_mod_time = new_mod_time
        time.sleep(interval)


@bot.event
async def on_ready():
    print(f"Logged in as {bot.user.name} ({bot.user.id})")
    await bot.change_presence(
        activity=discord.Game(
            name="!help - Keeping the trading community happy and healthy"
        )
    )
    # This endless loop runs the functions in the que with a first in first out principle. In the future there will be priority que for paying members hopefully.
    while True:
        if not trade_queue.empty():
            task = trade_queue.get()
            await task  # Await the coroutine object directly
        await asyncio.sleep(1)  # Use asyncio.sleep to not block the event loop


# Disable the default help command
bot.remove_command("help")


@bot.command(name="help", aliases=["h"])
async def help_command(ctx, *, command_name=None):
    """Displays help information for available commands."""

    if command_name is None:
        embed = discord.Embed(title="DarkerBot Help", color=discord.Color.blue())

        # Discord only commands
        embed.add_field(
            name="Discord Only Commands",
            value="These commands are used only in Discord.",
            inline=False,
        )
        embed.add_field(name="!help", value="Displays this help message.", inline=True)
        embed.add_field(
            name="!trade @user",
            value="Sends a trade request to the specified player.",
            inline=True,
        )
        embed.add_field(
            name="!trade-accept @user",
            value="Accepts the trade request from the specified player.",
            inline=True,
        )
        embed.add_field(
            name="!show-trade",
            value="Provides a visual representation of the current trade.",
            inline=True,
        )
        embed.add_field(
            name="!add-gold", value="Adds gold to the current trade.", inline=True
        )
        embed.add_field(
            name="!add-items", value="Adds items to the current trade.", inline=True
        )
        embed.add_field(
            name="!end-trade", value="End the current trade. ", inline=True
        )
        embed.add_field(
            name="!cancel-trade", value="Cancel the current trade.", inline=True
        )

        # Commands that interact with the game
        embed.add_field(
            name="In-game Interaction Commands",
            value="These commands allow DarkerBot to interact with the game.",
            inline=False,
        )
        embed.add_field(
            name="!pay-fee in_game_name",
            value="Bot sends a trade request in-game to the specified player.",
            inline=True,
        )
        embed.add_field(
            name="!deposit in_game_name",
            value="Deposit items/gold to the trading bot in-game.",
            inline=True,
        )
        embed.add_field(
            name="!claim-items in_game_name",
            value="Claim items that you've traded for from the in-game bot.",
            inline=True,
        )
        embed.add_field(
            name="!claim-gold in_game_name",
            value="Claim gold that you've traded for from the in-game bot.",
            inline=True,
        )
        embed.add_field(
            name="!claim-gold in_game_name",
            value="Claim gold that you've traded for from the in-game bot.",
            inline=True,
        )

        await ctx.send(embed=embed)

    else:
        if command_name.lower() in ["trade", "!trade"]:
            embed = discord.Embed(
                title="!trade @user",
                description="Initiates a trade with the specified Discord user. This sends them a trade request to commence the trading process.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!trade @user")
            embed.add_field(
                name="Notes", value="You cannot trade with bots or yourself."
            )
            await ctx.send(embed=embed)

        elif command_name.lower() in ["trade-accept", "!trade-accept"]:
            embed = discord.Embed(
                title="!trade-accept @user",
                description="Accepts a trade request from the specified Discord user, allowing the trade to proceed.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!trade-accept @user")
            embed.add_field(
                name="Notes",
                value="Ensure to verify the trade details before accepting.",
            )
            await ctx.send(embed=embed)

        elif command_name.lower() in ["show-trade", "!show-trade"]:
            embed = discord.Embed(
                title="!show-trade",
                description="Displays a visual representation of the current trade, detailing items and gold from both trading parties.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!show-trade")
            await ctx.send(embed=embed)

        elif command_name.lower() in ["add-gold", "!add-gold"]:
            embed = discord.Embed(
                title="!add-gold",
                description="Add a specific amount of gold to the ongoing trade.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!add-gold [amount]")
            embed.add_field(
                name="Notes", value="Ensure you have enough gold before adding."
            )
            await ctx.send(embed=embed)
        elif command_name.lower() in ["add-items", "!add-items"]:
            embed = discord.Embed(
                title="!add-items",
                description="Contribute specific items from your inventory to the current trade. Make sure to specify which items you wish to add.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!add-items [item1, item2, ...]")
            embed.add_field(
                name="Notes",
                value="Ensure to double-check the items you're adding to prevent mistakes.",
            )
            await ctx.send(embed=embed)

        elif command_name.lower() in ["pay-fee", "!pay-fee"]:
            embed = discord.Embed(
                title="!pay-fee [in_game_name]",
                description="Instructs DarkerBot to head over to the in-game trading channel and send a fee request to the specified player's in-game name.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!pay-fee [in_game_name]")
            embed.add_field(
                name="Notes",
                value="Ensure you have enough in-game currency to cover the fee.",
            )
            await ctx.send(embed=embed)

        elif command_name.lower() in ["deposit", "!deposit"]:
            embed = discord.Embed(
                title="!deposit [in_game_name]",
                description="Deposit specific in-game items or gold to DarkerBot's in-game counterpart. This is part of the escrow system during a trade.",
                color=0x55A7F7,
            )
            embed.add_field(
                name="Usage",
                value="!deposit [in_game_name] [item1, item2, ... OR gold amount]",
            )
            embed.add_field(
                name="Notes",
                value="Ensure the in-game bot is available to receive the deposit.",
            )
            await ctx.send(embed=embed)

        elif command_name.lower() in ["claim-items", "!claim-items"]:
            embed = discord.Embed(
                title="!claim-items [in_game_name]",
                description="Retrieve items that you have acquired from a completed trade. The bot will transfer the items in-game to the specified player's account.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!claim-items [in_game_name]")
            await ctx.send(embed=embed)

        elif command_name.lower() in ["claim-gold", "!claim-gold"]:
            embed = discord.Embed(
                title="!claim-gold [in_game_name]",
                description="Claim the gold you've accumulated from a trade. DarkerBot will transfer the gold to your in-game account.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!claim-gold [in_game_name]")
            await ctx.send(embed=embed)
        elif command_name.lower() in ["cancel-trade", "!cancel-trade"]:
            embed = discord.Embed(
                title="!cancel-trade",
                description="Cancel the trade. You can only cancel the trade if no items or gold have been claimed by either trader.\nAfter canceling the trade do !return-items [in_game_name] and or !return-gold [in_game_name]",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!claim-gold [in_game_name]")
            await ctx.send(embed=embed)
        elif command_name.lower() in ["end-trade", "!end-trade"]:
            embed = discord.Embed(
                title="!end-trade",
                description="End the trade. This command will close the trade. You can only end a trade if there are no items pending.\nIf you wish to cancel the trade, to !cancel-trade.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!claim-gold [in_game_name]")
            await ctx.send(embed=embed)

        else:
            await ctx.send(
                f"No help information found for '{command_name}'. Try using `!help` for a list of commands."
            )


import traceback


@bot.event
async def on_command_error(ctx, error):
    """Handle errors triggered by bot commands."""
    if isinstance(error, commands.MissingRequiredArgument):
        await ctx.send(
            "You missed a required argument. Please check the command and try again."
        )
    elif isinstance(error, commands.MemberNotFound):
        await ctx.send(
            "I couldn't find that member. Please tag a valid user and try again."
        )
    elif isinstance(error, commands.CommandNotFound):
        await ctx.send(
            f"I don't recognize the command `{ctx.invoked_with}`. Please check and try again."
        )
    else:
        await ctx.send("An error occurred. Please try again later.")
        error_traceback = traceback.format_exception(
            type(error), error, error.__traceback__
        )
        print("".join(error_traceback))


@bot.command()
async def trade(ctx, user: discord.Member):
    """Send a trade request to a user."""
    if user.bot:
        await ctx.send("You can't trade with bots!")
        return

    if ctx.author.id == user.id:
        await ctx.send("You can't trade with yourself!")
        return

    trade_requests[ctx.author.id] = user.id
    await ctx.send(
        f"{ctx.author.mention} has sent a trade request to {user.mention}! {user.mention}, use `!trade-accept @{ctx.author.name}` to accept the trade!"
    )


@bot.command(name="trade-accept")
async def trade_accept(ctx, user: discord.Member):
    """Accept a trade request."""
    # Check if there's a pending trade from the mentioned user to the command user
    if user.id in trade_requests and trade_requests[user.id] == ctx.author.id:
        conn = sqlite3.connect("trading_bot.db")
        cursor = conn.cursor()

        # Register the traders if they don't exist in the traders table
        cursor.execute(
            "INSERT OR IGNORE INTO traders (discord_id) VALUES (?)",
            (str(ctx.author.id),),
        )  # The person who accepts the trade will be id 1
        cursor.execute(
            "INSERT OR IGNORE INTO traders (discord_id) VALUES (?)", (str(user.id),)
        )  # The person who sent the trade will be id 2

        # Fetching the IDs of traders from the traders table
        cursor.execute(
            "SELECT id FROM traders WHERE discord_id=?", (str(ctx.author.id),)
        )
        trader1_id = cursor.fetchone()[0]

        cursor.execute("SELECT id FROM traders WHERE discord_id=?", (str(user.id),))
        trader2_id = cursor.fetchone()[0]

        # Fetch or create the "Middleman Trades" category
        category_name = "Middleman Trades"
        category = discord.utils.get(ctx.guild.categories, name=category_name)
        if category is None:
            category = await ctx.guild.create_category(category_name)

        # Create a private channel with permissions for only the two trading users and the bot
        overwrites = {
            ctx.guild.default_role: discord.PermissionOverwrite(read_messages=False),
            ctx.author: discord.PermissionOverwrite(
                read_messages=True, send_messages=True
            ),
            user: discord.PermissionOverwrite(read_messages=True, send_messages=True),
            ctx.guild.me: discord.PermissionOverwrite(
                read_messages=True, send_messages=True
            ),
        }

        channel_name = f"trade-{ctx.author.name}-and-{user.name}"
        trade_channel = await ctx.guild.create_text_channel(
            channel_name, overwrites=overwrites, category=category
        )

        # Register the trade in the trades table with the obtained IDs of the traders and the ID of the newly created channel
        cursor.execute(
            "INSERT INTO trades (trader1_id, trader2_id, channel_id) VALUES (?, ?, ?)",
            (trader1_id, trader2_id, str(trade_channel.id)),
        )

        # Commit the transaction and close the connection to the database
        conn.commit()
        conn.close()

        await ctx.send(
            f"{ctx.author.mention} has accepted the trade request from {user.mention}!"
        )

        await trade_channel.send(
            f"This channel has been created for {ctx.author.mention} and {user.mention} to discuss their trade. Please keep all trade discussions in this channel.\nThe processing fee is 50 gold."
        )

        del trade_requests[user.id]
    else:
        await ctx.send(
            f"{ctx.author.mention}, you don't have a pending trade request from {user.mention}!"
        )


@bot.command(name="show-trade")
async def show_trade(ctx):
    """Display the items and gold for both users in a specific trade."""

    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return

    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    try:
        channel_id = str(ctx.channel.id)

        # Fetch the trade details and traders' Discord IDs
        cursor.execute(
            """
            SELECT trades.id, t1.discord_id, t2.discord_id, trader1_gold, trader2_gold, trader1_gold_traded, trader2_gold_traded, trader1_gold_received, trader2_gold_received
            FROM trades 
            JOIN traders t1 ON trades.trader1_id = t1.id
            JOIN traders t2 ON trades.trader2_id = t2.id
            WHERE channel_id = ? AND status = 'ongoing'
        """,
            (channel_id,),
        )
        trade = cursor.fetchone()

        if not trade:
            await ctx.send("No ongoing trade found in this channel.")
            return

        user_discord_id = trade[1] if int(trade[1]) != ctx.author.id else trade[2]
        other_user_discord_id = trade[2] if int(trade[1]) != ctx.author.id else trade[1]

        user_gold = trade[3] if trade[1] != str(ctx.author.id) else trade[4]
        other_user_gold = trade[4] if trade[1] != str(ctx.author.id) else trade[3]

        traded_gold = trade[5] if trade[1] != str(ctx.author.id) else trade[6]
        other_traded_gold = trade[6] if trade[1] != str(ctx.author.id) else trade[5]

        received_gold = trade[7] if trade[1] != str(ctx.author.id) else trade[8]
        other_received_gold = trade[8] if trade[1] != str(ctx.author.id) else trade[7]

        # Use JOIN to get the discord_id along with trader_id and info_image_url.
        cursor.execute(
            """
            SELECT t.discord_id, i.info_image_url, i.status
            FROM items i
            JOIN traders t ON i.trader_id = t.id
            WHERE i.trade_id = ?
        """,
            (trade[0],),
        )

        rows = cursor.fetchall()

        # The emoji IDs can be found by typing \:emojiName: in Discord chat.
        # Green: traded
        # Yellow: in escrow
        # Red: not traded
        greenCircle = "🟢"
        yellowCircle = "🟡"
        redCircle = "🔴"

        trade_data = {}
        for discord_id, info_image_url, status in rows:
            if discord_id not in trade_data:
                trade_data[discord_id] = []

            emoji_status = (
                greenCircle
                if status == "traded"
                else (yellowCircle if status == "in escrow" else redCircle)
            )
            trade_data[discord_id].append((info_image_url, emoji_status))

        # Now, when you access trade_data, use discord_id
        user_items = trade_data.get(str(user_discord_id), [])
        other_user_items = trade_data.get(str(other_user_discord_id), [])

        # Fetch the user
        try:
            user = await bot.fetch_user(int(user_discord_id))
            user_name = user.name
        except discord.NotFound:
            user_name = "Unknown User"
        # Fetch the other user
        try:
            other_user = await bot.fetch_user(int(other_user_discord_id))
            other_user_name = other_user.name
        except discord.NotFound:
            other_user_name = "Unknown User"

        embed = discord.Embed(
            title="Items and Gold for Trade",
            description=f"Trade between {user_name} and {other_user_name}\n{redCircle}: Not traded yet\n{yellowCircle}: In escrow\n{greenCircle}: Traded",
            color=0x55A7F7,
        )

        user_items_value = (
            "\n".join(
                [
                    f"{status} [Item {i + 1}]({link})"
                    for i, (link, status) in enumerate(user_items)
                ]
            )
            if user_items
            else "No items added."
        )
        other_user_items_value = (
            "\n".join(
                [
                    f"{status} [Item {i + 1}]({link})"
                    for i, (link, status) in enumerate(other_user_items)
                ]
            )
            if other_user_items
            else "No items added."
        )

        embed.add_field(
            name=f"{user_name}'s Items and Gold",
            value=f"{user_items_value}\nGold: {user_gold}\nGold in escrow: {traded_gold}\nClaimed Gold: {received_gold}",
            inline=True,
        )
        embed.add_field(
            name=f"{other_user_name}'s Items and Gold",
            value=f"{other_user_items_value}\nGold: {other_user_gold}\nGold in escrow: {other_traded_gold}\nClaimed Gold: {other_received_gold}",
            inline=True,
        )

        print(user_items)
        print(other_user_items)
        buffer = await stitch_images(user_items, other_user_items)
        embed.set_image(url="attachment://items.png")

        await ctx.send(embed=embed, file=discord.File(buffer, filename="items.png"))

    except sqlite3.Error as e:
        await ctx.send(f"An error occurred: {e}")
    finally:
        conn.close()


@bot.command(name="cancel-trade")
async def cancel_trade(ctx):
    """Cancel a trade"""

    # Ensure the command is used in the "Middleman Trades" category
    if ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return

    check = cancel_trade_check(ctx.author.id, ctx.channel.id)

    if not check:
        await ctx.send("This trade is not eligable for cancellation at this time.")
        await ctx.send("Message @asdgew if there has been a mistake.")
        return

    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    cursor.execute(
        """
        UPDATE trades
        SET status = 'canceled'
        WHERE channel_id = ?
        """,
        (ctx.channel.id,),
    )


@bot.command(name="add-gold")
async def add_gold(ctx, gold: int):
    """Add gold to a specific trade."""

    # Ensure the command is used in the "Middleman Trades" category
    if ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return

    if gold % 50 != 0:
        await ctx.send("Gold has to be in increments of 50!")
        return

    discord_id = str(ctx.author.id)  # Get user ID from context
    channel_id = str(ctx.channel.id)  # Get channel ID from context

    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    try:
        # Fetch the trader's ID from the database using the Discord ID
        cursor.execute("SELECT id FROM traders WHERE discord_id = ?", (discord_id,))
        trader = cursor.fetchone()

        if not trader:
            await ctx.send("You are not registered as a trader.")
            return

        trader_id = trader[0]

        # Fetch the ongoing trade from the database using the unique channel ID
        cursor.execute(
            "SELECT id, trader1_id, trader2_id FROM trades WHERE channel_id = ? AND status = 'ongoing'",
            (channel_id,),
        )
        trade = cursor.fetchone()

        if not trade:
            await ctx.send("No ongoing trade found in this channel.")
            return

        # Update the appropriate gold amount
        if trade[1] == trader_id:  # If the trader is trader1
            cursor.execute(
                "UPDATE trades SET trader1_gold = trader1_gold + ? WHERE id = ?",
                (gold, trade[0]),
            )
        elif trade[2] == trader_id:  # If the trader is trader2
            cursor.execute(
                "UPDATE trades SET trader2_gold = trader2_gold + ? WHERE id = ?",
                (gold, trade[0]),
            )
        else:
            await ctx.send("You are not part of the trade in this channel.")
            return

        conn.commit()
        await ctx.send(f"Successfully added {gold} gold to the trade in this channel.")

    except sqlite3.Error as e:
        await ctx.send(f"An error occurred: {e}")
    finally:
        conn.close()


@bot.command(name="add-items")
async def add_items(ctx, *args: str):
    """Add item image links to a specific trade."""

    # Ensure the command is used in the "Middleman Trades" category
    if ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return

    # Ensure the user provided pairs of links
    if len(args) % 2 != 0:
        await ctx.send("Please provide pairs of item_image_url and info_image_url!")
        return

    # Validate links
    for link in args:
        if not link.startswith("http"):
            await ctx.send(
                f"The link `{link}` seems invalid. Make sure to provide valid URLs!"
            )
            return

    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    # Fetch the trade_id for the current channel from the trades table
    cursor.execute("SELECT id FROM trades WHERE channel_id=?", (str(ctx.channel.id),))
    trade_id = cursor.fetchone()
    if not trade_id:
        await ctx.send("No trade associated with this channel!")
        conn.close()
        return
    trade_id = trade_id[0]

    # Fetch the trader_id from the traders table
    cursor.execute("SELECT id FROM traders WHERE discord_id=?", (str(ctx.author.id),))
    trader_id = cursor.fetchone()
    if not trader_id:
        await ctx.send("No trader associated with this user!")
        conn.close()
        return
    trader_id = trader_id[0]

    # Inserting the items into the items table
    for i in range(0, len(args), 2):
        item_image_url = args[i]
        info_image_url = args[i + 1]
        cursor.execute(
            "INSERT INTO items (trade_id, trader_id, item_image_url, info_image_url) VALUES (?, ?, ?, ?)",
            (trade_id, trader_id, item_image_url, info_image_url),
        )

    conn.commit()
    conn.close()

    await ctx.send(f"Added {len(args)//2} item(s) to this trade!")


@bot.command(name="lock-trade")
async def lock_trade(ctx):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return

    # Check if the trade is canceled
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    cursor.execute(
        """
        SELECT status, locked
        FROM trades
        WHERE channel_id = ?
        """,
        (ctx.channel.id),
    )
    (status, locked) = cursor.fetchall()

    if status == "canceled":
        await ctx.send("The trade been canceled.")
        return
    elif locked:
        await ctx.send(
            "The trade is already locked."
        )
        return
    
    cursor.execute(
        """
        UPDATE trades
        SET locked = 1
        WHERE channel_id = ?
        """,
        (ctx.channel.id),
    )
    await ctx.send(
        "Trade has been locked!"
    )
    return

@bot.command(name="unlock-trade")
async def request_unlock(ctx, channel_id: str):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return
    
    discord_id = str(ctx.author.id) # Discord ID of the user who initiated the request
    # Check if there is already an unlock request for this channel
    if channel_id in unlock_requests:
        # Add the user to the unlock request if they are not already in it
        if discord_id not in unlock_requests[channel_id]:
            unlock_requests[channel_id].append(discord_id)
            # If both traders have requested unlock, perform the unlock
            if len(unlock_requests[channel_id]) == 2:
                conn = sqlite3.connect("trading_bot.db")
                cursor = conn.cursor()
                # Unlock the trade by setting the 'locked' field to 0 (False)
                cursor.execute("UPDATE trades SET locked = 0 WHERE channel_id = ?", (channel_id,))
                conn.commit()
                conn.close()
                await ctx.send("Trade has been unlocked!")
                # Clear the unlock request as it is no longer needed
                del unlock_requests[channel_id]
            else:
                await ctx.send("Unlock request has been noted. Waiting for the other trader.")
    else:
        # Start a new unlock request with the current user
        unlock_requests[channel_id] = [discord_id]
        await ctx.send("Unlock request has been initiated. Waiting for the other trader.")


@bot.command(name="end-trade")
async def end_trade(ctx):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return
    
    result = end_trade_check(ctx.channel.id)

    if not result:
        await ctx.send("Trade cannot end at this current time. Do !show-trade to check if any gold or items are pending to be traded.")
        return

    # Archive trade
    from helpers.archive import archive_trades_by_channel

    archive_trades_by_channel(ctx.channel.id)

    # Delete from active database
    delete_records_by_channel(ctx.channel.id)

    channels_to_be_deleted[ctx.channel.id] = time.time() + 1 * 60 * 60

    await ctx.send("The trade has now been ended Successfully!\nThis channel will be deleted in exactly one hour from now.\nIf you wish to reset this timer do !reset-deletion")


@bot.command(name="reset-deletion")
async def reset_deletion(ctx):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return
    
    if ctx.channel.id in channels_to_be_deleted.keys():
        await ctx.send("You cannot reset the deletion time for this trade.")
        return

    channels_to_be_deleted[ctx.channel.id] = time.time() + 1 * 60 * 60

    await ctx.send("This channel's deletion schedule has been reset to one hour from now!")


@bot.command(name="pay-fee")
async def pay_fee(ctx, in_game_id: str):
    trade_queue.put(pay_fee_real(ctx, in_game_id))


async def pay_fee_real(ctx, in_game_id: str):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return

    # Check if the trade is canceled
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    cursor.execute(
        """
        SELECT status, locked
        FROM trades
        WHERE channel_id = ?
        """,
        (ctx.channel.id),
    )

    (status, locked) = cursor.fetchall()

    if status == "canceled":
        await ctx.send("The trade been canceled.")
        return
    elif not locked:
        await ctx.send(
            "The trade is not locked. In order to continue the trade a trader has to do !lock-trade in order for the trade to continue."
        )
        return

    from helpers.has_paid_gold_fee import has_user_paid_fee

    if has_user_paid_fee(ctx.author.id, ctx.channel.id):
        await ctx.send("You have already paid the gold fee.")
        return
    # Construct the API endpoint URL
    api_endpoint = (
        f"http://127.0.0.1:8051/gold_fee/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"
    )

    # Make the API request
    response = requests.get(api_endpoint)
    if response.status_code != 200:
        await ctx.send(
            f"Failed to complete the trade. Error {response.status_code}: {response.text}"
        )
        return

    await ctx.send(response.text)

    path_to_monitor = "shared/ipc_communication.txt"
    polling_interval = 1  # seconds

    # Every time the data in ipc_communication.txt is changed this will run again.
    # It will run forever untill stopped.
    for data in monitor_file_changes(path_to_monitor, polling_interval):
        try:
            print(data)
            if "Successfully collected fee!" == data:
                await ctx.send(
                    f"TradeBot successfully collected fee from {in_game_id}!"
                )
                return
            else:
                await ctx.send(data)
                return
        except Exception as e:
            print(e)
            await ctx.send("Unexpected error occurred. Please message @asdgew")

        print("Test6")
    return


@bot.command(name="deposit")
async def deposit(ctx, in_game_id: str):
    trade_queue.put(deposit_real(ctx, in_game_id))


async def deposit_real(ctx, in_game_id: str):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return

    # Check if the trade is canceled
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    cursor.execute(
        """
        SELECT status, locked
        FROM trades
        WHERE channel_id = ?
        """,
        (ctx.channel.id),
    )

    (status, locked) = cursor.fetchall()

    if status == "canceled":
        await ctx.send("The trade been canceled.")
        return
    elif not locked:
        await ctx.send(
            "The trade is not locked. In order to continue the trade a trader has to do !lock-trade in order for the trade to continue."
        )
        return

    from helpers.has_paid_gold_fee import has_user_paid_fee

    if not has_user_paid_fee(ctx.author.id, ctx.channel.id):
        await ctx.send(
            "You have not paid the gold fee yet. Do !pay-fee to pay the trading fee."
        )
        return

    # Check if there are still items marked as "not traded"
    from helpers.escrow_status import has_untraded_items

    if has_untraded_items(ctx.author.id, ctx.channel.id):
        print("There are items tagged 'not traded'.")
    else:
        await ctx.send(
            "All current items have been traded to the bot. Do `add-items` if you wish to add more items to the trade."
        )
        return

    # Construct the API endpoint URL
    api_endpoint = (
        f"http://127.0.0.1:8051/deposit/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"
    )

    # Make the API request
    response = requests.get(api_endpoint)
    if response.status_code != 200:
        await ctx.send(
            f"Failed to complete the trade. Error {response.status_code}: {response.text}"
        )
        return

    await ctx.send(response.text)

    path_to_monitor = "shared/ipc_communication.txt"
    polling_interval = 1  # seconds

    # Every time the data in ipc_communication.txt is changed this will run again.
    # It will run forever untill stopped.
    for data in monitor_file_changes(path_to_monitor, polling_interval):
        try:
            print(data)
            if "Trade successful" == data:
                await ctx.send(f"Items from trader {ctx.author.name}, are now stored!")
                return
            else:
                await ctx.send(data)
                return
        except Exception as e:
            print(e)
            await ctx.send("Unexpected error occurred. Please message @asdgew")

        print("Test6")
    return


@bot.command(name="claim-items")
async def claim_items(ctx, in_game_id: str):
    trade_queue.put(claim_items_real(ctx, in_game_id))


async def claim_items_real(ctx, in_game_id: str):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return

    # Check if the trade is canceled
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    cursor.execute(
        """
        SELECT status, locked
        FROM trades
        WHERE channel_id = ?
        """,
        (ctx.channel.id),
    )

    (status, locked) = cursor.fetchall()

    if status == "canceled":
        await ctx.send("The trade been canceled.")
        return
    elif not locked:
        await ctx.send(
            "The trade is not locked. In order to continue the trade a trader has to do !lock-trade in order for the trade to continue."
        )
        return

    # Check if all the gold in the trade has been traded to the bot or claimed by trader.
    from helpers.traded_gold_match import check_gold

    result = check_gold(ctx.channel.id)
    # Check the status of the result
    has_enough_gold, traders_missing = result

    if traders_missing is None:
        print("Both traders have paid.")
    elif isinstance(traders_missing, str):  # It's an error message
        print(traders_missing)
    else:  # It's a list of discord IDs
        if len(traders_missing) == 1:
            # Fetch the user
            try:
                user = await bot.fetch_user(int(traders_missing[0]))
                user_name = user.name
            except discord.NotFound:
                user_name = "Unknown User"
            print(f"Trader {user_name}, has not traded all their gold yet.")
            await ctx.send(f"Trader {user_name}, has not traded all their gold yet.")
            return
        else:
            trader_names = []
            for i, trader in enumerate(traders_missing):
                # Fetch the user
                try:
                    user = await bot.fetch_user(int(traders_missing[i]))
                    user_name = user.name
                except discord.NotFound:
                    user_name = "Unknown User"

                trader_names.append(user_name)

            print(
                f"Traders {', '.join(trader_names)}, have not traded all their gold yet."
            )
            await ctx.send(
                f"Traders {', '.join(trader_names)}, have not traded all their gold yet."
            )
            return

    from helpers.escrow_status import all_items_traded

    if all_items_traded(ctx.channel.id):
        print("All items are either traded or in escrow")
    else:
        await ctx.send(
            "Trade is not ready. Some items in the trade have not been traded to the bot yet."
        )
        return

    # Check if there are any items from the oppisite trader with status "in escrow"
    from helpers.escrow_status import has_other_trader_escrow_items

    if has_other_trader_escrow_items(ctx.author.id, ctx.channel.id):
        print("All items are in escrow.")
        await ctx.send("Items are ready to be sent!")
    else:
        await ctx.send(
            "There are no more items to claim. If you want to claim your gold then write: `claim-gold {In-game player name}`"
        )
        return
    # Construct the API endpoint URL
    api_endpoint = f"http://127.0.0.1:8051/claim_items/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"

    # Make the API request
    response = requests.get(api_endpoint)
    if response.status_code != 200:
        await ctx.send(
            f"Failed to complete the trade. Error {response.status_code}: {response.text}"
        )
        return

    await ctx.send(response.text)

    path_to_monitor = "shared/ipc_communication.txt"
    polling_interval = 1  # seconds
    # Every time the data in ipc_communication.txt is changed this will run again.
    # It will run forever untill stopped.
    for data in monitor_file_changes(path_to_monitor, polling_interval):
        try:
            print(data)
            if "Trade successful" == data:
                await ctx.send(
                    f"TradeBot successfully traded items to {ctx.author.name}!"
                )

                result = end_trade_check(ctx.channel.id)
                if result == True:
                    await ctx.send("This trade looks to be complete!\nDo !end-trade in order to end this trade.")

                return
            else:
                await ctx.send(data)
                return
        except Exception as e:
            print(e)
            await ctx.send("Unexpected error occurred. Please message @asdgew")

        print("Test6")
    return


@bot.command(name="claim-gold")
async def claim_gold(ctx, in_game_id: str):
    trade_queue.put(claim_gold_real(ctx, in_game_id))


async def claim_gold_real(ctx, in_game_id: str):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return

    # Check if the trade is canceled
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    cursor.execute(
        """
        SELECT status, locked
        FROM trades
        WHERE channel_id = ?
        """,
        (ctx.channel.id),
    )

    (status, locked) = cursor.fetchall()

    if status == "canceled":
        await ctx.send("The trade been canceled.")
        return
    elif not locked:
        await ctx.send(
            "The trade is not locked. In order to continue the trade a trader has to do !lock-trade in order for the trade to continue."
        )
        return

    # Check if all the gold in the trade has been traded
    from helpers.traded_gold_match import check_gold

    result = check_gold(ctx.channel.id)
    # Check the status of the result
    has_enough_gold, traders_missing = result

    if traders_missing is None:
        print("Both traders have paid.")
    elif isinstance(traders_missing, str):  # It's an error message
        print(traders_missing)
    else:  # It's a list of discord IDs
        if len(traders_missing) == 1:
            # Fetch the user
            try:
                user = await bot.fetch_user(int(traders_missing[0]))
                user_name = user.name
            except discord.NotFound:
                user_name = "Unknown User"
            print(f"Trader {user_name}, has not traded all their gold yet.")
            await ctx.send(f"Trader {user_name}, has not traded all their gold yet.")
            return
        else:
            trader_names = []
            for i, trader in enumerate(traders_missing):
                # Fetch the user
                try:
                    user = await bot.fetch_user(int(traders_missing[i]))
                    user_name = user.name
                except discord.NotFound:
                    user_name = "Unknown User"

                trader_names.append(user_name)

            print(
                f"Traders {', '.join(trader_names)}, have not traded all their gold yet."
            )
            await ctx.send(
                f"Traders {', '.join(trader_names)}, have not traded all their gold yet."
            )
            return

    # Check if there are any items with status "not traded"
    # If so then you cannot collect your gold before that
    from helpers.escrow_status import all_items_traded

    if all_items_traded(ctx.channel.id):
        print("All items are either traded or in escrow")
    else:
        await ctx.send(
            "Trade is not ready. Some items in the trade have not been traded to the bot yet."
        )
        return

    # Check if the gold is in escrow or not.
    from helpers.other_trader_gold_left_in_escrow_check import (
        has_other_trader_gold_left,
    )

    if has_other_trader_gold_left(ctx.author.id, ctx.channel.id):
        print("Other trader still has gold left to be claimed by trader.")
        await ctx.send(
            "Gold is ready to be sent! Hop into the bard trading channel to collect your gold."
        )
    else:
        await ctx.send(
            "No more gold available to claim. If you want to claim your items then write: `claim-items {In-game player name}`"
        )
        return

    # Construct the API endpoint URL
    api_endpoint = f"http://127.0.0.1:8051/claim_gold/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"

    # Make the API request
    response = requests.get(api_endpoint)
    if response.status_code != 200:
        await ctx.send(
            f"Failed to complete the trade. Error {response.status_code}: {response.text}"
        )
        return

    await ctx.send(response.text)

    path_to_monitor = "shared/ipc_communication.txt"
    polling_interval = 1  # seconds

    # Every time the data in ipc_communication.txt is changed this will run again.
    # It will run forever untill stopped.
    for data in monitor_file_changes(path_to_monitor, polling_interval):
        try:
            print(data)
            if "Trade successful" == data:
                await ctx.send(
                    f"TradeBot successfully traded gold to {ctx.author.name}!"
                )

                result = end_trade_check(ctx.channel.id)
                if result == True:
                    await ctx.send("This trade looks to be complete!\nDo !end-trade in order to end this trade.")

                return
            else:
                await ctx.send(data)
                return
        except Exception as e:
            print(e)
            await ctx.send("Unexpected error occurred. Please message @asdgew")

        print("Test6")
    return


@bot.command(name="return-gold")
async def return_gold(ctx, in_game_id: str):
    trade_queue.put(return_gold_real(ctx, in_game_id))


async def return_gold_real(ctx, in_game_id: str):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return
    # Check if the trade is canceled
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    cursor.execute(
        """
        SELECT status
        FROM trades
        WHERE channel_id = ?
        """,
        (ctx.channel.id),
    )

    status = cursor.fetchone()

    if status != "canceled":
        await ctx.send(
            "The trade has not yet been canceled. Do !cancel-trade first and try again."
        )
        return

    # Construct the API endpoint URL
    api_endpoint = f"http://127.0.0.1:8051/return_gold/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"

    # Make the API request
    response = requests.get(api_endpoint)
    if response.status_code != 200:
        await ctx.send(
            f"Failed to complete the trade. Error {response.status_code}: {response.text}"
        )
        return

    await ctx.send(response.text)

    path_to_monitor = "shared/ipc_communication.txt"
    polling_interval = 1  # seconds

    # Every time the data in ipc_communication.txt is changed this will run again.
    # It will run forever untill stopped.
    for data in monitor_file_changes(path_to_monitor, polling_interval):
        try:
            print(data)
            if "Trade successful" == data:
                await ctx.send(
                    f"TradeBot successfully returned gold to {ctx.author.name}!"
                )

                result = end_trade_check(ctx.channel.id)
                if result == True:
                    await ctx.send("This trade looks to be complete!\nDo !end-trade in order to end this trade.")

                return
            else:
                await ctx.send(data)
                return
        except Exception as e:
            print(e)
            await ctx.send("Unexpected error occurred. Please message @asdgew")

        print("Test6")
    return


@bot.command(name="return-items")
async def return_items(ctx, in_game_id: str):
    trade_queue.put(return_items_real(ctx, in_game_id))


async def return_items_real(ctx, in_game_id: str):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return
    # Check if the trade is canceled
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    cursor.execute(
        """
        SELECT status
        FROM trades
        WHERE channel_id = ?
        """,
        (ctx.channel.id),
    )

    status = cursor.fetchone()

    if status != "canceled":
        await ctx.send(
            "The trade has not yet been canceled. Do !cancel-trade first and try again."
        )
        return

    # Construct the API endpoint URL
    api_endpoint = f"http://127.0.0.1:8051/return_items/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"

    # Make the API request
    response = requests.get(api_endpoint)
    if response.status_code != 200:
        await ctx.send(
            f"Failed to complete the trade. Error {response.status_code}: {response.text}"
        )
        return

    await ctx.send(response.text)

    path_to_monitor = "shared/ipc_communication.txt"
    polling_interval = 1  # seconds

    # Every time the data in ipc_communication.txt is changed this will run again.
    # It will run forever untill stopped.
    for data in monitor_file_changes(path_to_monitor, polling_interval):
        try:
            print(data)
            if "Trade successful" == data:
                await ctx.send(
                    f"TradeBot successfully returned items to {ctx.author.name}!"
                )

                result = end_trade_check(ctx.channel.id)
                if result == True:
                    await ctx.send("This trade looks to be complete!\nDo !end-trade in order to end this trade.")

                return
            else:
                await ctx.send(data)
                return
        except Exception as e:
            print(e)
            await ctx.send("Unexpected error occurred. Please message @asdgew")
    return


# Returns true if the trade can be canceled, return false if not
def cancel_trade_check(discord_id, channel_id) -> bool:
    trader_1_or_2 = True

    gold_in_escrow = False
    items_in_escrow = False

    # Connect to the database
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    # Find out if trader_id is trader1 or trader2
    # Get trader's internal ID using discord_id (user_id)
    cursor.execute(
        """
        SELECT id
        FROM traders
        WHERE discord_id = ?
        """,
        (discord_id,),
    )
    trader_id = cursor.fetchone()

    cursor.execute(
        """
        SELECT trader1_id, trader2_id
        FROM trades
        WHERE channel_id = ?
    """,
        (channel_id,),
    )

    trader1_id, trader2_id = cursor.fetchall()

    if trader_id == trader1_id:
        trader_1_or_2 = True
    elif trader_id == trader2_id:
        trader_1_or_2 = False

    # Check if any gold has been claimed and
    # Check if there is any gold left to
    cursor.execute(
        """
        SELECT trader1_gold_received, trader2_gold_received, trader1_gold_traded, trader2_gold_traded
        FROM trades
        WHERE channel_id = ?
        """,
        (channel_id,),
    )

    (
        trader1_gold_claimed,
        trader2_gold_claimed,
        trader1_gold_traded,
        trader2_gold_traded,
    ) = cursor.fetchall()

    if trader_1_or_2 and trader1_gold_traded > 30:
        gold_in_escrow = True
    elif not trader_1_or_2 and trader2_gold_traded > 30:
        gold_in_escrow = True

    if trader1_gold_claimed > 30 or trader2_gold_claimed > 30:
        return False

    # Check if at least one item or over 30 gold have been traded to the bot
    # Else there is no reason to send anything back because there is nothing to send

    cursor.execute(
        """
        SELECT COUNT(*)
        FROM items
        JOIN trades ON items.trade_id = trades.id
        JOIN traders ON items.trader_id = traders.id
        WHERE items.status = 'in_escrow'
        AND trades.channel_id = ?1
        AND traders.discord_id = ?2,
        """,
        (channel_id, trader1_id),
    )
    trader1_item_escrow_count = cursor.fetchall()

    cursor.execute(
        """
        SELECT COUNT(*)
        FROM items
        JOIN trades ON items.trade_id = trades.id
        JOIN traders ON items.trader_id = traders.id
        WHERE items.status = 'in_escrow'
        AND trades.channel_id = ?1
        AND traders.discord_id = ?2,
        """,
        (channel_id, trader2_id),
    )
    trader2_item_escrow_count = cursor.fetchall()

    if trader_1_or_2 and trader1_item_escrow_count > 0:
        items_in_escrow = True
    elif not trader_1_or_2 and trader2_item_escrow_count > 0:
        items_in_escrow = True

    cursor.execute(
        """
        SELECT COUNT(*)
        FROM items
        JOIN trades ON items.trade_id = trades.id
        JOIN traders ON items.trader_id = traders.id
        WHERE items.status = 'traded'
        AND trades.channel_id = ?1
        AND traders.discord_id = ?2,
        """,
        (channel_id, trader1_id),
    )
    trader1_count = cursor.fetchall()
    cursor.execute(
        """
        SELECT COUNT(*)
        FROM items
        JOIN trades ON items.trade_id = trades.id
        JOIN traders ON items.trader_id = traders.id
        WHERE items.status = 'traded'
        AND trades.channel_id = ?1
        AND traders.discord_id = ?2,
        """,
        (channel_id, trader2_id),
    )
    trader2_count = cursor.fetchall()
    if trader1_count > 0 or trader2_count > 0:
        return False

    if not gold_in_escrow or not items_in_escrow:
        return False
    return True


# Deletes all records that have anything to do with that channel. (Keeps users)
def delete_records_by_channel(channel_id):
    # Connect to the database
    conn = sqlite3.connect("trading_bot.db")
    cursor = conn.cursor()

    # First, fetch the trade id(s) for the given channel_id
    cursor.execute("SELECT id FROM trades WHERE channel_id=?", (channel_id,))
    trade_ids = cursor.fetchall()

    # Delete records in items table that match the trade_id(s)
    for trade_id in trade_ids:
        cursor.execute("DELETE FROM items WHERE trade_id=?", (trade_id[0],))

    # Delete records in trades table that match the channel_id
    cursor.execute("DELETE FROM trades WHERE channel_id=?", (channel_id,))

    # Commit changes and close the connection
    conn.commit()
    conn.close()

    print(f"Records associated with channel_id {channel_id} deleted successfully!")


# Returns False if trade cannot be closed and True if it can be
def end_trade_check(channel_id) -> bool:
    try:
        # Connect to the database
        conn = sqlite3.connect("trading_bot.db")
        cursor = conn.cursor()

        # Item status check
        cursor.execute("SELECT id FROM trades WHERE channel_id=?", (channel_id,))
        trade_id = cursor.fetchone()

        cursor.execute("SELECT status FROM items WHERE trade_id = ?", (trade_id,))

        item_statuses = cursor.fetchall()

        for status in item_statuses:
            if status == "ongoing" or status == "in escrow":
                return False

        # Gold status check
        cursor.execute("SELECT trader1_gold, trader2_gold, trader1_gold_traded, trader2_gold_traded FROM trades WHERE channel_id = ?", (channel_id,))

        (trader1_gold, trader2_gold, trader1_gold_traded, trader2_gold_traded) = cursor.fetchone()

        if trader1_gold > 30 or trader2_gold > 30 or trader1_gold_traded > 30 or trader2_gold_traded > 30:
            return False
    except:
        return False
    return True



response_file_path = "shared/ipc_communication.txt"


# Clear the contents of the file
with open(response_file_path, "w"):
    pass


bot.run(TOKEN)
