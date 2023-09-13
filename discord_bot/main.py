import asyncio

import discord
from discord.ext import commands, tasks

TOKEN = "YOUR_BOT_TOKEN"

intents = (
    discord.Intents.default()
)  # Use this if you want to access certain privileged events
intents.messages = True

bot = commands.Bot(command_prefix="!", intents=intents)


@bot.event
async def on_ready():
    print(f"Bot is logged in as {bot.user.name} - {bot.user.id}")
    change_status.start()


@tasks.loop(seconds=10)  # Looping background task
async def change_status():
    await bot.change_presence(activity=discord.Game(name="First Status"))
    await asyncio.sleep(10)
    await bot.change_presence(activity=discord.Game(name="Second Status"))


@bot.event
async def on_member_join(member):
    channel = member.guild.system_channel
    if channel:
        await channel.send(f"Welcome {member.mention} to {member.guild.name}!")


@bot.event
async def on_message(message):
    if message.author == bot.user:
        return
    if "hello" in message.content.lower():
        await message.channel.send("Hello!")
    await bot.process_commands(
        message
    )  # To process commands alongside the on_message event


@bot.command()
async def ping(ctx):
    await ctx.send(f"Pong! {round(bot.latency * 1000)}ms")


@bot.command()
async def echo(ctx, *, content: str):
    await ctx.send(content)


@bot.command()
async def info(ctx, user: discord.Member = None):
    if user is None:
        user = ctx.author
    embed = discord.Embed(title=f"{user.name}'s Info", color=user.color)
    embed.add_field(name="Username:", value=user.name, inline=False)
    embed.add_field(name="ID:", value=user.id, inline=False)
    embed.set_thumbnail(url=user.avatar_url)
    await ctx.send(embed=embed)


bot.run(TOKEN)
