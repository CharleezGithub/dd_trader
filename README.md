# Dark and Darker Trading Bot

## Description
A sophisticated trading bot for the game Dark and Darker, featuring a split architecture with a Discord bot for communication and an in-game bot for game interaction. Utilizes IPC connections and a local API for seamless operation between the two components.

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
Envirement variables are to be used for the discord bot but not yet implemented

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
This project is licensed under the GNU General Public License - see the [LICENSE.md](LICENSE.md) file for details
