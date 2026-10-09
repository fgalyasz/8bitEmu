# 8bitEmu

Cross-platform Spectrum and C64 player. With no arguments the window runs a small Z80 program in 48K RAM: blue border, three bright lines, and a cell that steps through the ink colors. Pass a ROM or an SNA the user already has to boot a 48K or basic 128K Spectrum. A TAP or TZX plays through the ear bit. A booted 48K machine types `LOAD ""`. A booted 128K machine presses Enter for Tape Loader. `--machine c64` boots a Commodore 64 from ROMs in `roms/c64-*.rom` (or `--kernal` / `--basic` / `--chargen`). `--prg file.prg` selects C64, waits for boot, soft-loads the PRG, and auto-starts it. File → Open… also accepts `.prg` on a C64 session. The C64 shows VIC text and bitmap modes with sprites, the Colodore palette, a CIA keyboard, and SID tones (no filter yet). The picture fits the window, or snaps to 125%, 150%, 175%, and 200%. No ROM is included in git.

```
cargo test
./scripts/build-release.sh
cargo run -p eightbit-emu-app
cargo run -p eightbit-emu-app -- --rom image.rom
cargo run -p eightbit-emu-app -- --model 128 --rom image.rom
cargo run -p eightbit-emu-app -- --rom image.rom --sna game.sna
cargo run -p eightbit-emu-app -- --rom image.rom --tap game.tap
cargo run -p eightbit-emu-app -- --rom image.rom --tzx game.tzx
cargo run -p eightbit-emu-app -- --machine c64
```

File → Open… chooses a `.rom`, `.sna`, `.tap`, or `.tzx`. Command-O on macOS and Ctrl-O elsewhere open the same dialog. A tape opened on a booted 48K machine types `LOAD ""`. On 128K it presses Enter for Tape Loader. The tape stays parked until that loader is reading the ear, then keeps playing through the leader. File → Save Snapshot… writes an `.sna`. Save Tape as TAP… and Save Tape as TZX… write a standard-speed recording. Command-S, Command-Shift-T, and Command-Shift-Z do the same (Ctrl on the other platforms). View sets Fit, 125%, 150%, 175%, or 200%. The picture stays at Spectrum speed, including on a faster display. A tape finishes in a few seconds by default. Sound → Loading sound, and F4, turn the border tone on and load in real time. The loading picture stays on screen either way. The game is heard when the tape stops.

Keys: F1 Sharp, F2 Soft edge, F3 Temporal color, F4 loading sound, F5 125%, F6 150%, F7 175%, F8 200%, F9 resume the tape, F11 save `8bitemu.sna`, F12 reset. Escape does nothing. Close the window from its close button or the 8bitEmu menu. Dragging the window fits the picture. Digit keys and the letter rows go to the Spectrum matrix. Left Shift is Caps Shift. Right Shift and Left Control are Symbol Shift. Arrows and Right Alt are the Kempston joystick. A standard-speed `SAVE` also writes `8bitemu.tap` in the working directory.

## Release build

`./scripts/build-release.sh` writes a release executable for the computer it runs on. macOS lands in `dist/eightbit-emu-macos-<arch>/eightbit-emu`, Linux in `dist/eightbit-emu-linux-<arch>/eightbit-emu`, and Windows in `dist/eightbit-emu-windows-<arch>/eightbit-emu.exe`. Run it on each system. No ROM is copied into `dist/`.

## System ROM

No ROM is shipped. Amstrad allows these images to be used with an emulator if the copyright text stays unchanged and the image itself is not sold. The copyright remains Amstrad's.

Debian packages them as [`spectrum-roms`](https://packages.debian.org/stable/otherosfs/spectrum-roms) in non-free. The upstream archive is [`spectrum-roms_20081224.orig.tar.gz`](https://deb.debian.org/debian/pool/non-free/s/spectrum-roms/spectrum-roms_20081224.orig.tar.gz).

Put the 48K image here as `spectrum-48.rom` (16384 bytes). For 128K, join the editor and the 48K BASIC half in that order into `spectrum-128.rom` (32768 bytes):

```
cat 128-0.rom 128-1.rom > spectrum-128.rom
cp 48.rom spectrum-48.rom
```

SHA1 sums, as listed on the [Sinclair ROM images](https://sinclair.wiki.zxnet.co.uk/wiki/ROM_images) page:

- `48.rom` / `spectrum-48.rom` `5ea7c2b824672e914525d1d5c419d71b84a426a2`
- `128-0.rom` `4f4b11ec22326280bdb96e3baf9db4b4cb1d02c5`
- `128-1.rom` `80080644289ed93d71a1103992a154cc9802b2fa`

The +2A and +3 images in that archive are not for this machine. A game snapshot is a separate copyright from the system ROM.

## Commodore 64 ROM

See `roms/README.md`. Defaults are `roms/c64-kernal.rom`, `c64-basic.rom`, and
`c64-chargen.rom` (gitignored). Open ROMs boots legally but its BASIC is incomplete.
For real `PRINT` / software compatibility, place stock images you own (e.g. C64 Forever)
under those names. Stock dumps are never committed.

## License

MIT. See [LICENSE](LICENSE). System ROMs and game images are not covered and are not included.
