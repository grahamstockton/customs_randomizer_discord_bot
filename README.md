# Champion Randomizer Discord Bot
This is a bot that you can run on your computer and add to your discord channel. Given player names, it will look up which league of legends champions those players can play and create a randomized team for those players. It will also ensure that each team has a jungle.

## Setup
1. Clone this repository onto your desktop
2. Register a new discord bot using the discord developer website
3. Register with the riot API and get an API token. Unfortunately, Riot doesn't trust us, so you will have to rotate this token every day you use the bot
4. Create a file in the directory `customs_randomizer_discord_bot` called `Secrets.toml`. This file should look like:
```
DISCORD_TOKEN = "YOUR DISCORD TOKEN"
RIOT_API_TOKEN = "YOUR RIOT API TOKEN"
```
3. If you would like to add pseudonyms for common players (rather than playerName#tag, which can be cumbersome), add a key value pair `"pseudonym": "playerName#tag"` to `pseudonyms.json`
4. Invite the discord bot to your server using a permissions link generated via the discord developer website
5. To run the bot, open a terminal instance in the directory `customs_randomizer_discord_bot` and use `cargo shuttle run`. You may have to install some rust/cargo/shuttle related dependencies for this to work.

## Usage
`new_game`: The first time you use this bot for any particular set of players, you should run this command. It will do all of the heavy lifting required to look up the champions available to each player. The input is of the format `summ1#NA1,summ2#NA1,summ3#JP1,summ4#YAY,summ5#h/summ6#NA1,summ7#B,summ8#1,summ9#NA1,summ10#NA1`. If you have a pseudonym set up for a particular player, you can use that in place of `playerName#tag`.

`reroll`: If you have already run `new_game` and don't need to change the teams, you can run reroll, which is much faster, simpler, and won't spam the riot API with unnecessary calls. This command will create a whole new set of champion assignments for each team. Unlike the single-champion reroll in ARAM, this reroll can be done any number of times and doesn't consider the output of any previous `new_game` or `reroll` calls.

`help`: prints a basic help message
