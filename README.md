# Dark and Darker Trading Bot

## Description
A sophisticated trading bot for the game Dark and Darker:
It Currently acts as an escrow bot keeping traders items safe doing bigger trades or trades where both traders cannot be online at the same time.

- **Discord Bot Features:**
  - Provides a help guide with `!help`.
  - Offers a tutorial for new traders using `!tutorial`.
  - Ability to restart both bots if needed with `!restart-bot`.
  - Initiates trade requests with `!trade`.
  - Accepts incoming trade requests via `!trade-accept`.
  - Locks and unlocks trades for security and management using `!lock-trade` and `!unlock-trade`.
  - Displays current trade status and details with `!show-trade`.
  - Adds gold to trades using `!add-gold`.
  - Includes items in trades with `!add-items`.
  - Concludes trades with `!end-trade`.
  - Cancels trades when necessary with `!cancel-trade`.

- **In-game Bot Features:**
  - Collects processin fee in gold with `!pay-fee`.
  - Handles the deposit of items or gold with `!deposit`.
  - Allows players to claim their traded items or gold in-game through `!claim-items` and `!claim-gold`.
  - Allows the return of items or gold with `!return-items` and `!return-gold`.

### Todo
- [x] Escrow style middleman trading bot
- [ ] Trading between own accounts
- [ ] Automatic fencing feature
- [ ] Automatic discord auctions for items that have been sitting in escrow bot inventory / stash for too long.


## Table of Contents
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [Credits](#credits)
- [License](#license)

## Installation
Install python and rustup
Install python packages
Install tesseract
Link: https://tesseract-ocr.github.io/tessdoc/Installation.html


## Usage
Run the process_manager.py process.
The bot will automatically check if it is in the main lobby of the game and if not it will try to start the game.
So make sure that you are in the lobby when starting the script and that the terminal window is not covering up the play tab at the top of the game screen.

## Configuration
Envirement variables are used for the discord bot token. Configuration of this can be found in the process manager script.

## Inter-component Communication
### IPC Connections
In the shared folder there are two text files. ipc_restart.txt is only used for restarting the entire bot.
ipc_communication.txt is used for communicating from the in-game (rust) bot to the discord (python) bot.
It's main purpose is to update the discord bot at the end of the trade. When the trade is done, successful or failed, the bot will send an update through the connection and the discord bot will for the most part relay that message to the user.

### Local API
The local API is on the rust side a rocket webserver. Most of what happens on the rust side / in-game bot side is triggered through the websever by the discord bot.
It returns instantly, therefore providing an emidiate response to the discord bot.
Then it spawns a new task that will do the actual trading. Once that task returns, the in-game bot will update the discord bot through the ipc_connection.text.

## Contributing
Please read [CONTRIBUTING.md](#) for details on our code of conduct, and the process for submitting pull requests to us.

## Credits
CharleezGithub

## License
This project is licensed under the CC0-1.0 - see the [LICENSE](LICENSE) file for details
