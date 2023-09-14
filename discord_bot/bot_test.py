import discord
from discord.ext import commands
import sqlite3

TOKEN = "MTE1MTQ0MTUwMjk1NDg0ODMwNw.GtRAIh.YBUChh8QJi3Cs8jeFbuE18kRJYrAwiCpcxcnz8"


# SQLite setup
conn = sqlite3.connect('database/discord_messages.db')
cursor = conn.cursor()

def create_table():
    cursor.execute('''
    CREATE TABLE IF NOT EXISTS messages (
        user_id INTEGER NOT NULL,
        content TEXT NOT NULL
    )
    ''')
    conn.commit()

create_table()

bot = commands.Bot(command_prefix='!')

@bot.event
async def on_ready():
    print(f'Logged in as {bot.user.name} ({bot.user.id})')
    await bot.change_presence(activity=discord.Game(name="!help - Keeping the trading community happy and healthy"))



@bot.remove_command("help")  # Remove the default help command
@bot.command()
async def help(ctx):
    embed = discord.Embed(title="Bot Commands", description="Here are the available commands:", color=0x00ff00)

    embed.add_field(name="!trader-register <message>", value="Stores your provided message with your user ID.", inline=False)
    embed.add_field(name="!retrieve <user_id>", value="Retrieves all messages associated with the provided user ID.", inline=False)
    embed.add_field(name="!help", value="Displays the help message.", inline=False)

    await ctx.send(embed=embed)


@bot.command(name="trader-register")
async def trader_register(ctx, *, content: str = None):
    if not content:
        await ctx.send("Please provide the content/message you want to register.")
        return
    cursor.execute("INSERT INTO messages (user_id, content) VALUES (?, ?)", (ctx.author.id, content))
    conn.commit()
    await ctx.send(f"Stored message: {content} from user ID: {ctx.author.id}")


@bot.command()
async def retrieve(ctx, user_id: int):
    cursor.execute("SELECT content FROM messages WHERE user_id = ?", (user_id,))
    messages = cursor.fetchall()
    if not messages:
        await ctx.send(f"No messages found for user ID: {user_id}")
    else:
        await ctx.send(f"Messages for user ID {user_id}: {', '.join([m[0] for m in messages])}")

bot.run(TOKEN)
