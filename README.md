# SD Save Loader mod for Xenoblade 3
This mod allows saving and loading Xenoblade Chronicles 3 save data to/from a location on the SD card. `/xc3-saves`

This can be useful if you would like to save round trips to a save manager when needing to do quick adjustments to a save. You can also update the save via FTP and return to the title screen to load the updated save without having to shutdown and restart the game.

> **Important**: Use at your own risk! I am not responsible for anything that could happen to your saves, game, console, account, etc.

## Usage

### Compatibility
* Required XC3 version: **2.1.1**
* This will work with other skyline NRO plugins for [skyline with TCP logger disabled](https://github.com/RoccoDev/skyline/releases/tag/cross-game-local-logging). e.g.,
  * [xc3-file-loader](https://github.com/RoccoDev/xc3-file-loader)
  * [xc3-voice-liberator](https://github.com/wildfirekithara/xc3-voice-liberator)

### Switch
1. Download the latest version of the mod from _**TODO**_.
2. Extract the archive to root of your SD card.
3. Add an `/xc3-saves` folder to the root of your SD card, and copy the [examples/allow-list.txt](examples/allow-list.txt) file into it. Edit it to enable exactly which save slots you want to enable this functionality for.

You can also skip step 3 and proceed to run the game. The `xc3-saves` folder will automatically be created, and the default `allow-list.txt` will be created, but with no files enabled. You'll have to update the `allow-list.txt`` and restart the game.

While updating allow-listed saves do not require a game restart (simply returning to title screen and loading the save again should work), updating the allow list itself requires a game restart.

Logs are written to `/xc3-saves/log.txt`. Though I'll probably disable this for Release version.

### Emulators
You can technically run this on an emulator however it isn't as useful since you can already directly read/write into the game's save data without having to shut down the game.

## Build instructions
To build the project, install [Rust](https://rustup.rs/) and run
```sh
./build.sh
```

## Credits
* Scaffolding for this mod comes from [xc3-file-loader](https://github.com/RoccoDev/xc3-file-loader):
  * Xenoblade Chronicles 3 patched `npdm` file 
  * Build script that packages with [skyline with TCP logger disabled](https://github.com/RoccoDev/skyline/releases/tag/cross-game-local-logging)

## License
This mod is distributed under the terms of the [GPLv3](https://www.gnu.org/licenses/gpl-3.0.html). See [COPYING](COPYING) for details.
