# Addendum: Open a file from the menu

File → Open… uses the system file dialog. Command-O on macOS and Ctrl-O elsewhere open that dialog.

`.rom` calls the ROM loader. A 32768-byte image forces the 128K machine. A 16384-byte image keeps the machine from `--model` or from the last image. `.sna` calls the snapshot loader. `.tap` and `.tzx` call the tape loader. On a booted machine the tape path then resets, so `LOAD ""` runs again.

The command line is unchanged:

```
cargo run -p scanline-app -- --rom roms/48.rom
cargo run -p scanline-app -- --rom roms/48.rom --tzx game.tzx
```
