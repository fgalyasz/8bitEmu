# Changelog

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

- Load a TAP or TZX through the ear bit, type `LOAD ""`, and resume a stopped tape with F9. F11 writes `scanline.sna`. A standard-speed SAVE writes `scanline.tap`. Kempston is port `0x1F`. The picture fits the window or snaps to 125%, 150%, 175%, and 200%.

## 0.3.0 — 2026-10-04

- Boot a 48K or basic 128K Spectrum from a user-supplied ROM or SNA. The documented Z80 set, the frame interrupt, the keyboard matrix, 128K paging, the beeper, and the AY run. Looks move to F1, F2, and F3. No ROM is included.

## 0.2.0 — 2026-10-04

- The window shows a Z80 program writing the Spectrum display file: blue border, three bright lines, and a cell that steps through the ink colors. No ROM is included.

## 0.1.0 — 2026-10-04

- Open Scanline on a stable, sharp ROM-free test pattern. Keys 1, 2, and 3 switch Sharp, Soft edge, and Temporal color. A 60 Hz panel holds five source frames across six refreshes.
