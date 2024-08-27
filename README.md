<div align="center">
  <img src="https://cloud-d99wdskiv-hack-club-bot.vercel.app/0image.png" />
  <h1>Sprig Arcade</h1>
  A viewer for your <a href="https://hackclub.com">Hack Club</a> <a href="https://hackclub.com/arcade">Arcade</a> progress. 
</div>
<br />

This is my second program for the [Sprig](https://sprig.hackclub.com) and it uses the Pico W to wirelessly and automatically update your arcade stats and put it on a secondary screen for you to track and see at all times. 

## Screens
| Overall Progress | Stats | Current Session |
| :--------------: | :---: | :-------------: |
| ![](https://cloud-lji7ovis0-hack-club-bot.vercel.app/0image.png) | ![](https://cloud-4hhk34mts-hack-club-bot.vercel.app/0image.png) | ![](https://cloud-hipdkzx4o-hack-club-bot.vercel.app/0image.png) |

## Features
- [x] A working display
- [x] Uses the Arcade API to update stats
- [x] Real Time Clock updated on startup
- [x] User Input
- [x] Navigation bar
- [x] Home Screen
  - [x] Second nav bar
  - [x] Overall Progress
    - [x] Progress bar
      - [x] Shows current progress
      - [x] Shows ideal progress
    - [x] Ticket count
    - [x] Key for bar
    - [x] Arcade logo
  - [x] Stats
    - [x] hrs/day on average
    - [x] ideal daily tickets
    - [x] days left
    - [x] hrs/day to get on track
- [x] Session Screen
  - [x] Remaining time
  - [x] Progress bar
  - [x] Goal
  - [x] Ticket no
  - [x] Current Status (in progress/paused/finished)
  - [ ] Session controls (using API)
- [ ] Leaderboard Screen
- [ ] Projects List
- [ ] Todo List
- [ ] Shop Screen
- [ ] Errors output
- [ ] Settings Screen

## Building
Sprig Arcade is built using [Rust](https://rust-lang.org), and is therefore a requirement for building and running. 

First, Sprig Arcade needs configuring. 
Open up `.cargo/config.toml`. Under `[env]` there will be the following info:
```toml
[env]
DEFMT_LOG = "debug"

WIFI_NETWORK = "SSID"
WIFI_PASSWD = "PASSWORD"
SLACK_ID = "SLACK_ID"
API_TOKEN = "UUID"
```
Update the `WIFI_NETWORK` and `WIFI_PASSWD` to the correct details for your network.
Then, update the `SLACK_ID` with your id from #what-is-my-slack-id and the `API_TOKEN` with your key from the /api command. 
Both of these things can be found in the [Hack Club Slack](https://hackclub.com/slack). 

### Running on the Sprig
Simply run the command below while having your Sprig plugged in on USB Boot mode, and EGB will boot up. 
```
cargo run --target thumbv6m-none-eabi
```

## License
Sprig Arcade is licensed under Mozilla Public License 2.0 unless otherwise stated. 
THe file `assets/font.raw` is licensed under CC0 and is from Pico8. 
