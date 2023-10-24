import discord
from discord.ext import commands
import sqlite3

from PIL import Image
from PIL import ImageDraw, ImageFont
import requests
from io import BytesIO

from helpers.stitching import stitch_images

TOKEN = "MTE1MTQ0MTUwMjk1NDg0ODMwNw.GtRAIh.YBUChh8QJi3Cs8jeFbuE18kRJYrAwiCpcxcnz8"

intents = discord.Intents.default()
intents.members = True
intents.messages = True

bot = commands.Bot(command_prefix="!", intents=intents)

# Dictionary to store trade requests. Format: {requester_id: requestee_id}
trade_requests = {}


@bot.event
async def on_ready():
    print(f"Logged in as {bot.user.name} ({bot.user.id})")
    await bot.change_presence(
        activity=discord.Game(
            name="!help - Keeping the trading community happy and healthy"
        )
    )


# Disable the default help command
bot.remove_command("help")


@bot.command(name="help", aliases=["h"])
async def custom_help(ctx, *, command_name=None):
    """Displays help information for available commands."""

    if command_name is None:
        embed = discord.Embed(
            title="Help", description="List of available commands:", color=0x55A7F7
        )

        embed.add_field(
            name="!trade @user",
            value="Send a trade request to a user. You cannot trade with bots or yourself.",
            inline=False,
        )

        embed.add_field(
            name="!trade-accept @user",
            value="Accept a trade request from a user. You can only accept trade requests from users who have sent you a trade invitation.",
            inline=False,
        )

        embed.add_field(
            name="!items-add [link 1] [link 2] [link 3]...",
            value="Add the items that you want to trade for something else.",
            inline=False,
        )

        embed.add_field(
            name="!show-trade",
            value="Shows a visualization of what the trade looks like",
            inline=False,
        )
        embed.add_field(
            name="!deposit [In-game player-id]",
            value="TO DO!",
            # value="Complete the trade by trading your items to the middleman bot in the game using this command. Once both players have traded their items to the middleman, the players can do !claim-gold and claim-items in order to collect the items they traded for.",
            inline=False,
        )

        embed.add_field(
            name="!claim-items",
            value="TO DO!",
            # value="Collect the items that you traded for.",
            inline=False,
        )
        embed.add_field(
            name="!claim-gold",
            value="TO DO!",
            # value="Collect the items that you traded for.",
            inline=False,
        )

        embed.add_field(
            name="!help [command]",
            value="Get detailed help on a specific command.",
            inline=False,
        )

        await ctx.send(embed=embed)
    else:
        if command_name.lower() in ["trade", "!trade"]:
            embed = discord.Embed(
                title="!trade @user",
                description="Send a trade request to a user.",
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
                description="Accept a trade request from a user.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!trade-accept @user")
            embed.add_field(
                name="Notes",
                value="You can only accept trade requests from users who have sent you a trade invitation.",
            )
            await ctx.send(embed=embed)
        elif command_name.lower() in ["items-add", "!items-add"]:
            embed = discord.Embed(
                title="!items-add [link 1] [link 2] [link 3]...",
                description="Add the items that you want to trade for something else.",
                color=0x55A7F7,
            )
            embed.add_field(
                name="Usage", value="!items-add [link 1] [link 2] [link 3]..."
            )
            embed.add_field(
                name="Notes",
                value="This command allows you to add multiple item links that you are willing to trade. Ensure that the links provided are valid.",
            )
            await ctx.send(embed=embed)

        elif command_name.lower() in ["deposit", "!deposit"]:
            embed = discord.Embed(
                title="!deposit [In-game player-id]",
                description="Trade your items to the middleman bot in the game.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!deposit [In-game player-id]")
            embed.add_field(
                name="Notes",
                value="Once both players have traded their items to the middleman bot, you and the other player can use the !claim-items and claim-gold command to retrieve the items you traded for.",
            )
            await ctx.send(embed=embed)

        elif command_name.lower() in ["claim-items", "!claim-items"]:
            embed = discord.Embed(
                title="!claim-items",
                description="Collect the items that you traded for.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!claim-items")
            embed.add_field(
                name="Notes",
                value="This command allows you to collect the items you've acquired after completing a trade. Both parties are required to trade their items with the middleman bot before being able to collect the items.",
            )
            await ctx.send(embed=embed)

        else:
            await ctx.send(f"I couldn't find any help related to `{command_name}`.")


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
            SELECT trades.id, t1.discord_id, t2.discord_id, trader1_gold, trader2_gold, trader1_gold_traded, trader2_gold_traded
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
        greenCircle = '🟢'
        yellowCircle = '🟡'
        redCircle = '🔴'

        trade_data = {}
        for discord_id, info_image_url, status in rows:
            if discord_id not in trade_data:
                trade_data[discord_id] = []
            
            emoji_status = greenCircle if status == "traded" else (yellowCircle if status == "in escrow" else redCircle)
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
            "\n".join([f"{status} [Item {i + 1}]({link})" for i, (link, status) in enumerate(user_items)])
            if user_items
            else "No items added."
        )
        other_user_items_value = (
            "\n".join([f"{status} [Item {i + 1}]({link})" for i, (link, status) in enumerate(other_user_items)])
            if other_user_items
            else "No items added."
        )

        embed.add_field(
            name=f"{user_name}'s Items and Gold",
            value=f"{user_items_value}\nGold: {user_gold}\nClaimed Gold: {traded_gold}",
            inline=True,
        )
        embed.add_field(
            name=f"{other_user_name}'s Items and Gold",
            value=f"{other_user_items_value}\nGold: {other_user_gold}\nClaimed Gold: {other_traded_gold}",
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


@bot.command(name="pay-fee")
async def pay_fee(ctx, in_game_id: str):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return
    from helpers.has_paid_gold_fee import has_user_paid_fee

    if has_user_paid_fee(ctx.author.id, ctx.channel.id):
        await ctx.send("You have already paid the gold fee.")
        return
    try:
        print(
            f"http://127.0.0.1:8051/gold_fee/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"
        )
        # Construct the API endpoint URL
        api_endpoint = f"http://127.0.0.1:8051/gold_fee/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"

        # Make the API request
        response = requests.get(api_endpoint)
        print("response:", response.status_code)
        print(response.status_code == 200)
        print("response:", response.text)

        # Check if the request was successful
        if response.status_code == 200:
            data = response.text
            if "TradeBot ready" == data:
                await ctx.send(
                    'Going into "The Bard' + "'s" + 'Theater #1"'
                )  # Send the message from the API response, if provided
            else:
                await ctx.send(
                    "TradeBot is not ready. Wait 3 minutes and try again. Message @asdgew if this problem persists."
                )
        else:
            await ctx.send(
                f"Failed to complete the trade. Trading bot is not online. Please message @asdgew."
            )
            await ctx.send(
                f"Failed to complete the trade. Error {response.status_code}: {response.text}"
            )

    except Exception as e:
        print(e)
        await ctx.send(f"Unexpected error occurred. Please message @asdgew")
        # await ctx.send(f"Unexpected error occurred: {str(e)}")


@bot.command(name="deposit")
async def deposit(ctx, in_game_id: str):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
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

    try:
        print(
            f"http://127.0.0.1:8051/trade_request/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"
        )
        # Construct the API endpoint URL
        api_endpoint = f"http://127.0.0.1:8051/trade_request/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"

        # Make the API request
        response = requests.get(api_endpoint)
        print("response:", response.status_code)
        print(response.status_code == 200)
        print("response:", response.text)

        # Check if the request was successful
        if response.status_code == 200:
            data = response.text
            if "TradeBot ready" == data:
                await ctx.send(
                    'Going into "The Bard' + "'s" + 'Theater #1"'
                )  # Send the message from the API response, if provided
            else:
                await ctx.send(
                    "TradeBot is not ready. Wait 3 minutes and try again. Message @asdgew if this problem persists."
                )
        else:
            await ctx.send(
                f"Failed to complete the trade. Trading bot is not online. Please message @asdgew."
            )
            await ctx.send(
                f"Failed to complete the trade. Error {response.status_code}: {response.text}"
            )

    except Exception as e:
        print(e)
        await ctx.send(f"Unexpected error occurred. Please message @asdgew")
        # await ctx.send(f"Unexpected error occurred: {str(e)}")


@bot.command(name="claim-items")
async def claim_items(ctx, in_game_id: str):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return

    # Check if all the gold in the trade has been traded to the bot.
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
        await ctx.send(
            "Items are ready to be sent! Hop into the bard trading channel to collect your items."
        )
    else:
        await ctx.send("There are no more items to claim. If you want to claim your gold then write: `claim-gold {In-game player name}`")
        return

    try:
        print(
            f"http://127.0.0.1:8051/claim_items/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"
        )
        # Construct the API endpoint URL
        api_endpoint = f"http://127.0.0.1:8051/claim_items/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"

        # Make the API request
        response = requests.get(api_endpoint)
        print("response:", response.status_code)
        print(response.status_code == 200)
        print("response:", response.text)

        # Check if the request was successful
        if response.status_code == 200:
            data = response.text
            if "TradeBot ready" == data:
                await ctx.send(
                    'Going into "The Bard' + "'s" + 'Theater #1"'
                )  # Send the message from the API response, if provided
            else:
                await ctx.send(
                    "TradeBot is not ready. Wait 3 minutes and try again. Message @asdgew if this problem persists."
                )
        else:
            await ctx.send(
                f"Failed to complete the trade. Trading bot is not online. Please message @asdgew."
            )
            # Remove later. For debugging only
            await ctx.send(
                f"Failed to complete the trade. Error {response.status_code}: {response.text}"
            )

    except Exception as e:
        print(e)
        await ctx.send(f"Unexpected error occurred. Please message @asdgew")
        # await ctx.send(f"Unexpected error occurred: {str(e)}")


@bot.command(name="claim-gold")
async def claim_gold(ctx, in_game_id: str):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
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

    try:
        print(
            f"http://127.0.0.1:8051/claim_gold/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"
        )
        # Construct the API endpoint URL
        api_endpoint = f"http://127.0.0.1:8051/claim_gold/{in_game_id}/{ctx.channel.id}/{ctx.author.id}"

        # Make the API request
        response = requests.get(api_endpoint)
        print("response:", response.status_code)
        print(response.status_code == 200)
        print("response:", response.text)

        # Check if the request was successful
        if response.status_code == 200:
            data = response.text
            if "TradeBot ready" == data:
                await ctx.send(
                    'Going into "The Bard' + "'s" + 'Theater #1"'
                )  # Send the message from the API response, if provided
            else:
                await ctx.send(
                    "TradeBot is not ready. Wait 3 minutes and try again. Message @asdgew if this problem persists."
                )
        else:
            await ctx.send(
                f"Failed to complete the trade. Trading bot is not online. Please message @asdgew."
            )
            # Remove later. For debugging only
            await ctx.send(
                f"Failed to complete the trade. Error {response.status_code}: {response.text}"
            )

    except Exception as e:
        print(e)
        await ctx.send(f"Unexpected error occurred. Please message @asdgew")
        # await ctx.send(f"Unexpected error occurred: {str(e)}")


bot.run(TOKEN)
