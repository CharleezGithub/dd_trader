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

@bot.event
async def on_message(message):
    if message.author == bot.user:
        return
    if bot.user.mentioned_in(message):
        # Store the message and user ID when the bot is mentioned
        cursor.execute("INSERT INTO messages (user_id, content) VALUES (?, ?)", (message.author.id, message.content))
        conn.commit()
        await message.channel.send(f"Stored message: {message.content} from user ID: {message.author.id}")
    await bot.process_commands(message)

@bot.command()
async def retrieve(ctx, user_id: int):
    cursor.execute("SELECT content FROM messages WHERE user_id = ?", (user_id,))
    messages = cursor.fetchall()
    if not messages:
        await ctx.send(f"No messages found for user ID: {user_id}")
    else:
        await ctx.send(f"Messages for user ID {user_id}: {', '.join([m[0] for m in messages])}")

bot.run(TOKEN)
