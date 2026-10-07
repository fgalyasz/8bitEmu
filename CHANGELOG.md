# Changelog

## 0.9.13 — 2026-10-07

- Empty launch opens a shared shell window. Machine (Spectrum 48K/128K or C64) and ROM paths live under the menu and persist; Start boots that machine in the same window. CLI boot args still skip the launcher.

## 0.9.12 — 2026-10-06

- C64 undocumented NOP opcodes (`$1A`, `$80`, `$FC`, …) execute instead of trapping.

## 0.9.11 — 2026-10-06

- C64 sprites respect VIC priority: lower sprite index draws in front.

## 0.9.10 — 2026-10-06

- C64 SID audio reaches the speaker. Multiplexed multicolor sprites keep `$D025`/`$D026` and the sprite color from the latch line (Monty’s gray figure).

## 0.9.9 — 2026-10-06

- C64 sprites latch X/pointer/color when the raster hits their Y, so multiplexed title figures (e.g. Monty) draw correctly instead of end-of-frame snow.

## 0.9.8 — 2026-10-06

- C64 VIC bitmap modes paint, and the raster compare IRQ fires. Bitmap title screens no longer look like letter soup or hang waiting for `$D019`.

## 0.9.7 — 2026-10-06

- C64 hardware sprites draw over the text screen: hires, X/Y expand, and multicolor.

## 0.9.6 — 2026-10-06

- The C64 SID plays three voices (triangle, saw, pulse, noise) with ADSR and master volume. The filter still waits for a later pass.

## 0.9.5 — 2026-10-06

- Soft-load a Commodore `.prg` with `--prg` or File → Open… on a C64 session. BASIC programs at `$0801` link and `RUN`; other load addresses jump to the start.

## 0.9.4 — 2026-10-06

- `roms/README.md` documents placing stock C64 ROMs you own (C64 Forever / hardware dump) under `c64-*.rom`, with VICE SHA1s for verification. Open ROMs remain the redistributable default.

## 0.9.3 — 2026-10-06

- C64 keys follow the KERNAL matrix. Typing A no longer inserts R.

## 0.9.2 — 2026-10-06

- C64 `SBC` no longer flips carry before the add. Open ROMs boot text reads correctly (`READY.`, not a shifted garble).

## 0.9.1 — 2026-10-06

- The C64 cold start finishes. The VIC raster line and CIA-1 Timer A run, so Open ROMs can leave the sync wait and show the boot banner.

## 0.9.0 — 2026-10-06

- Boot a C64 with `--machine c64` from Open ROMs in `roms/c64-*.rom`. VIC text mode, Colodore colours, and the CIA keyboard run. Tape waits for a later pass.

## 0.8.14 — 2026-10-06

- The product is 8bitEmu. Spectrum today, C64 next, and room for other 8-bit machines.

## 0.8.13 — 2026-10-05

- A tape loads at turbo by default. Turn Loading sound on to hear the border tone.

## 0.8.12 — 2026-10-05

- Saboteur II and other 128K tunes write the AY through OUTD. That instruction is in place, so the music plays.

## 0.8.11 — 2026-10-05

- AY volumes follow the chip's measured steps, so a tune keeps its shape. The envelope also steps at the chip's rate.

## 0.8.10 — 2026-10-05

- Sustained notes stay steady. The speaker keeps a short lead, so a gap between frames does not wobble the pitch.

## 0.8.9 — 2026-10-05

- AY notes play an octave higher, in the chip's real pitch range.

## 0.8.8 — 2026-10-05

- A quiet screen stays quiet. A short gap in the sound no longer becomes a click.

## 0.8.7 — 2026-10-05

- The 128K menu step clicks once. It no longer keeps knocking after a game starts.

## 0.8.6 — 2026-10-05

- With the loading sound off, a tape finishes in a few seconds. The loading picture stays on screen.

## 0.8.5 — 2026-10-05

- 128K starts Tape Loader with Enter instead of typing `LOAD ""`. A tape that stops continues when the loader is still reading, and again after a quiet gap.

## 0.8.4 — 2026-10-05

- AY noise is the chip's shift register, so the noise channel hisses instead of playing a musical tone.

## 0.8.3 — 2026-10-05

- Escape no longer closes the window. Close it from the close button or the 8bitEmu menu.

## 0.8.2 — 2026-10-04

- Turbo load draws every refresh. The clock runs ahead for a short slice of each refresh instead of jumping to a single later frame.

## 0.8.1 — 2026-10-04

- The machine keeps Spectrum speed on a faster display. The 128K chip plays at its real pitch. A normal load is heard from the tape, and turbo stays silent.

## 0.8.0 — 2026-10-04

- `./scripts/build-release.sh` writes a release executable for the host: macOS, Linux, or Windows.

## 0.7.1 — 2026-10-04

- The border follows the beam. A load draws stripes, and a border set once stays one color.

## 0.7.0 — 2026-10-04

- Sound → Loading sound, and F4, switch the loading beeps. Off, the load runs ahead of the picture and those beeps are dropped. Game audio returns when the tape stops.

## 0.6.1 — 2026-10-04

- Once the loader is reading the ear, the tape keeps playing through the leader wait. The pause after a block no longer swallows the last data pulse.

## 0.6.0 — 2026-10-04

- View sets the picture to fit, 125%, 150%, 175%, or 200%. File saves a snapshot, or a standard-speed recording as TAP or TZX.

## 0.5.1 — 2026-10-04

- The load prompt types the LOAD keyword with J, then the quotes. The tape stays at the start until the loader is reading the ear.

## 0.5.0 — 2026-10-04

- File → Open… picks a ROM, snapshot, TAP, or TZX. Command-O and Ctrl-O open the same dialog. A tape opened on a booted machine types `LOAD ""` again.

## 0.4.0 — 2026-10-04

- Load a TAP or TZX through the ear bit, type `LOAD ""`, and resume a stopped tape with F9. F11 writes `8bitemu.sna`. A standard-speed SAVE writes `8bitemu.tap`. Kempston is port `0x1F`. The picture fits the window or snaps to 125%, 150%, 175%, and 200%.

## 0.3.0 — 2026-10-04

- Boot a 48K or basic 128K Spectrum from a user-supplied ROM or SNA. The documented Z80 set, the frame interrupt, the keyboard matrix, 128K paging, the beeper, and the AY run. Looks move to F1, F2, and F3. No ROM is included.

## 0.2.0 — 2026-10-04

- The window shows a Z80 program writing the Spectrum display file: blue border, three bright lines, and a cell that steps through the ink colors. No ROM is included.

## 0.1.0 — 2026-10-04

- Open 8bitEmu on a stable, sharp ROM-free test pattern. Keys 1, 2, and 3 switch Sharp, Soft edge, and Temporal color. A 60 Hz panel holds five source frames across six refreshes.
