import discord
from discord.ext import commands
import sqlite3

from PIL import Image
from PIL import ImageDraw, ImageFont
import requests
from io import BytesIO

TOKEN = "MTE1MTQ0MTUwMjk1NDg0ODMwNw.GtRAIh.YBUChh8QJi3Cs8jeFbuE18kRJYrAwiCpcxcnz8"

bot = commands.Bot(command_prefix="!")

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
            name="!trade-complete [In-game player-id]",
            value="TO DO!",
            # value="Complete the trade by trading your items to the middleman bot in the game using this command. Once both players have traded their items to the middleman, the players can do !collect in order to collect the items they traded for.",
            inline=False,
        )

        embed.add_field(
            name="!collect",
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

        elif command_name.lower() in ["trade-complete", "!trade-complete"]:
            embed = discord.Embed(
                title="!trade-complete [In-game player-id]",
                description="Trade your items to the middleman bot in the game.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!trade-complete [In-game player-id]")
            embed.add_field(
                name="Notes",
                value="Once both players have traded their items to the middleman bot, you and the other player can use the !collect command to retrieve the items you traded for.",
            )
            await ctx.send(embed=embed)

        elif command_name.lower() in ["collect", "!collect"]:
            embed = discord.Embed(
                title="!collect",
                description="Collect the items that you traded for.",
                color=0x55A7F7,
            )
            embed.add_field(name="Usage", value="!collect")
            embed.add_field(
                name="Notes",
                value="This command allows you to collect the items you've acquired after completing a trade. Both parties are required to trade their items with the middleman bot before being able to collect the items.",
            )
            await ctx.send(embed=embed)

        else:
            await ctx.send(f"I couldn't find any help related to `{command_name}`.")


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
        print(f"Unexpected error: {error}")


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
        await ctx.send(
            f"{ctx.author.mention} has accepted the trade request from {user.mention}!"
        )

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
        cursor.execute("SELECT id, trader1_id, trader2_id FROM trades WHERE channel_id = ? AND status = 'ongoing'", (channel_id,))
        trade = cursor.fetchone()
        
        if not trade:
            await ctx.send("No ongoing trade found in this channel.")
            return
        
        # Update the appropriate gold amount
        if trade[1] == trader_id:  # If the trader is trader1
            cursor.execute("UPDATE trades SET trader1_gold = trader1_gold + ? WHERE id = ?", (gold, trade[0]))
        elif trade[2] == trader_id:  # If the trader is trader2
            cursor.execute("UPDATE trades SET trader2_gold = trader2_gold + ? WHERE id = ?", (gold, trade[0]))
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
async def add_items(ctx, *links: str):
    """Add item image links to a specific trade."""

    # Ensure the command is used in the "Middleman Trades" category
    if ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
        )
        return

    # Ensure the links provided are valid URLs
    for link in links:
        if not link.startswith("http"):
            await ctx.send(
                f"The link `{link}` seems invalid. Make sure to provide valid URLs!"
            )
            return

    conn = sqlite3.connect("trades.db")
    cursor = conn.cursor()

    # Ensure 'trade_items' table exists to list all channels
    cursor.execute(
        "CREATE TABLE IF NOT EXISTS trade_items (channel_id INTEGER PRIMARY KEY)"
    )
    # Insert the channel ID into trade_items if it doesn't already exist
    cursor.execute(
        "INSERT OR IGNORE INTO trade_items (channel_id) VALUES (?)", (ctx.channel.id,)
    )

    # Ensure the 'trade_data' table exists to store the trade data
    cursor.execute(
        """CREATE TABLE IF NOT EXISTS trade_data (
            id INTEGER PRIMARY KEY,
            channel_id INTEGER REFERENCES trade_items(channel_id),
            user_id INTEGER,
            item_link TEXT
        )"""
    )

    # Insert the data into 'trade_data'
    for link in links:
        cursor.execute(
            "INSERT OR IGNORE INTO trade_data (channel_id, user_id, item_link) VALUES (?, ?, ?)",
            (ctx.channel.id, ctx.author.id, link),
        )

    conn.commit()
    conn.close()

    await ctx.send(f"Added {len(links)} item(s) to this trade!")


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
        
        # Fetch the trade and traders' info associated with the channel from the 'trades' table
        cursor.execute("""
            SELECT id, trader1_id, trader2_id, trader1_gold, trader2_gold 
            FROM trades 
            WHERE channel_id = ? AND status = 'ongoing'
        """, (channel_id,))
        trade = cursor.fetchone()

        if not trade:
            await ctx.send("No ongoing trade found in this channel.")
            return

        # Fetch the items associated with the trade from the 'items' table
        cursor.execute("SELECT trader_id, item_image_url FROM items WHERE trade_id = ?", (trade[0],))
        rows = cursor.fetchall()

        # Organize items by trader
        trade_data = {}
        for trader_id, item_link in rows:
            if trader_id not in trade_data:
                trade_data[trader_id] = []
            trade_data[trader_id].append(item_link)

        user_items = trade_data.get(trade[1], [])
        if trade[3] != None:
            user_gold = trade[3]  # trader1_gold
        else:
            user_gold = "No gold"  # trader1_gold
        
        # Determine the other user in the trade
        other_user_id = trade[2]
        other_user = await bot.fetch_user(other_user_id)
        other_user_name = other_user.name if other_user else "Failed to Retrieve User"
        other_user_items = trade_data.get(other_user_id, [])
        if trade[4] != None:
            other_user_gold = trade[4]  # trader2_gold
        else:
            other_user_gold = "No gold"  # trader2_gold
        
        # Display the items and gold
        embed = discord.Embed(
            title="Items and Gold for Trade",
            description=f"Trade between {ctx.author.name} and {other_user_name}",
            color=0x55A7F7,
        )
        
        user_items_value = "\n".join([f"[Item {i+1}]({link})" for i, link in enumerate(user_items)]) if user_items else "No items added."
        other_user_items_value = "\n".join([f"[Item {i+1}]({link})" for i, link in enumerate(other_user_items)]) if other_user_items else "No items added."
        
        embed.add_field(name=f"{ctx.author.name}'s Items and Gold", value=f"{user_items_value}\nGold: {user_gold}", inline=True)
        embed.add_field(name=f"{other_user_name}'s Items and Gold", value=f"{other_user_items_value}\nGold: {other_user_gold}", inline=True)
        
        buffer = await stitch_images(user_items, other_user_items)
        embed.set_image(url="attachment://items.png")
        
        await ctx.send(embed=embed, file=discord.File(buffer, filename="items.png"))
        
    except sqlite3.Error as e:
        await ctx.send(f"An error occurred: {e}")
    finally:
        conn.close()


async def stitch_images(user1_urls, user2_urls):
    """Stitch together images from the provided URLs for both users."""
    user1_images = [
        Image.open(BytesIO(requests.get(url).content)) for url in user1_urls
    ]
    user2_images = [
        Image.open(BytesIO(requests.get(url).content)) for url in user2_urls
    ]

    # Padding values (change these to adjust the space)
    arrow_padding = 50  # Added space on each side of the arrow
    side_padding = 20  # Added space on each side of the items

    # Determine max width and total height for each user's images
    max_width_user1 = max(img.width for img in user1_images) + 2 * side_padding
    max_width_user2 = max(img.width for img in user2_images) + 2 * side_padding
    total_height_user1 = sum(img.height for img in user1_images)
    total_height_user2 = sum(img.height for img in user2_images)

    # Create an arrow image with added padding and the same height as the tallest column
    max_height = max(total_height_user1, total_height_user2)
    arrow_image_width = 50 + 2 * arrow_padding
    arrow_image = Image.new("RGB", (arrow_image_width, max_height), color="white")
    draw = ImageDraw.Draw(arrow_image)
    font = ImageFont.truetype("arial.ttf", 50)
    draw.text((arrow_padding, (max_height - 35) // 2), "<--->", font=font, fill="black")

    # Create the final stitched image
    total_width = max_width_user1 + arrow_image.width + max_width_user2
    new_image = Image.new("RGB", (total_width, max_height), color="white")

    # Paste user1 images with side padding
    y_offset = (max_height - total_height_user1) // 2
    for img in user1_images:
        x_offset = side_padding
        new_image.paste(img, (x_offset, y_offset))
        y_offset += img.height

    # Paste arrow
    new_image.paste(arrow_image, (max_width_user1, 0))

    # Paste user2 images with side padding
    y_offset = (max_height - total_height_user2) // 2
    for img in user2_images:
        x_offset = max_width_user1 + arrow_image.width + side_padding
        new_image.paste(img, (x_offset, y_offset))
        y_offset += img.height

    buffer = BytesIO()
    new_image.save(buffer, "PNG")
    buffer.seek(0)

    return buffer


@bot.command(name="complete-trade")
async def complete_trade(ctx, in_game_id: str):
    if not ctx.channel.category or ctx.channel.category.name != "Middleman Trades":
        await ctx.send(
            "This command can only be used within the 'Middleman Trades' category!"
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
        await ctx.send(f"Unexpected error occurred: {str(e)}")


bot.run(TOKEN)
