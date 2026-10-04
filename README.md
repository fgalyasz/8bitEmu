# Scanline

Cross-platform Spectrum and C64 player. With no arguments the window runs a small Z80 program in 48K RAM: blue border, three bright lines, and a cell that steps through the ink colors. Pass a ROM or an SNA the user already has to boot a 48K or basic 128K Spectrum. A TAP or TZX plays through the ear bit, and a booted machine types `LOAD ""`. The picture fits the window, or snaps to 125%, 150%, 175%, and 200%. The border changes with the beam, so a load is striped. No ROM is included.

```
cargo test
./scripts/build-release.sh
cargo run -p scanline-app
cargo run -p scanline-app -- --rom image.rom
cargo run -p scanline-app -- --model 128 --rom image.rom
cargo run -p scanline-app -- --rom image.rom --sna game.sna
cargo run -p scanline-app -- --rom image.rom --tap game.tap
cargo run -p scanline-app -- --rom image.rom --tzx game.tzx
```

File → Open… chooses a `.rom`, `.sna`, `.tap`, or `.tzx`. Command-O on macOS and Ctrl-O elsewhere open the same dialog. A tape opened on a booted machine types `LOAD ""`. The tape stays parked until that loader is reading the ear, then keeps playing through the leader. File → Save Snapshot… writes an `.sna`. Save Tape as TAP… and Save Tape as TZX… write a standard-speed recording. Command-S, Command-Shift-T, and Command-Shift-Z do the same (Ctrl on the other platforms). View sets Fit, 125%, 150%, 175%, or 200%. The picture stays at Spectrum speed, including on a faster display. Sound → Loading sound is on by default, and the beep is the tape itself. Turning it off drops those beeps and runs the clock ahead while the picture still updates every refresh. The game is heard again when the tape stops.

Keys: F1 Sharp, F2 Soft edge, F3 Temporal color, F4 loading sound, F5 125%, F6 150%, F7 175%, F8 200%, F9 resume the tape, F11 save `scanline.sna`, F12 reset, Escape quits. Dragging the window fits the picture. Digit keys and the letter rows go to the Spectrum matrix. Left Shift is Caps Shift. Right Shift and Left Control are Symbol Shift. Arrows and Right Alt are the Kempston joystick. A standard-speed `SAVE` also writes `scanline.tap` in the working directory.

## Release build

`./scripts/build-release.sh` writes a release executable for the computer it runs on. macOS lands in `dist/scanline-macos-<arch>/scanline`, Linux in `dist/scanline-linux-<arch>/scanline`, and Windows in `dist/scanline-windows-<arch>/scanline.exe`. Run it on each system. No ROM is copied into `dist/`.

## System ROM

No ROM is shipped. Amstrad allows these images to be used with an emulator if the copyright text stays unchanged and the image itself is not sold. The copyright remains Amstrad's.

Debian packages them as [`spectrum-roms`](https://packages.debian.org/stable/otherosfs/spectrum-roms) in non-free. The upstream archive is [`spectrum-roms_20081224.orig.tar.gz`](https://deb.debian.org/debian/pool/non-free/s/spectrum-roms/spectrum-roms_20081224.orig.tar.gz).

`48.rom` is 16384 bytes and boots the 48K machine. For 128K, join the editor and the 48K BASIC half in that order. The result is 32768 bytes:

```
cat 128-0.rom 128-1.rom > 128.rom
```

SHA1 sums, as listed on the [Sinclair ROM images](https://sinclair.wiki.zxnet.co.uk/wiki/ROM_images) page:

- `48.rom` `5ea7c2b824672e914525d1d5c419d71b84a426a2`
- `128-0.rom` `4f4b11ec22326280bdb96e3baf9db4b4cb1d02c5`
- `128-1.rom` `80080644289ed93d71a1103992a154cc9802b2fa`

The +2A and +3 images in that archive are not for this machine. A game snapshot is a separate copyright from the system ROM.
