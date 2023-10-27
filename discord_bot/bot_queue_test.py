import discord
from discord.ext import commands

import queue
import asyncio


TOKEN = "MTE1MTQ0MTUwMjk1NDg0ODMwNw.GtRAIh.YBUChh8QJi3Cs8jeFbuE18kRJYrAwiCpcxcnz8"

intents = discord.Intents.default()
intents.members = True
intents.messages = True

bot = commands.Bot(command_prefix="!", intents=intents)

# Step 1: Instantiate the queue
task_queue = queue.Queue()

@bot.event
async def on_ready():
    print(f'Logged in as {bot.user.name}')
    while True:
        if not task_queue.empty():
            task = task_queue.get()
            await task  # Await the coroutine object directly
        await asyncio.sleep(1)  # Use asyncio.sleep to not block the event loop

@bot.command()
async def test(ctx, test2):
    # Step 2: Add the coroutine object to the queue
    task_queue.put(test_real(ctx, test2))
    task_queue.put()

async def test_real(ctx, test2):
    await asyncio.sleep(3)
    print(ctx.author.id, test2)
    await ctx.send(f"{ctx.author.id} {test2}")

bot.run(TOKEN)
