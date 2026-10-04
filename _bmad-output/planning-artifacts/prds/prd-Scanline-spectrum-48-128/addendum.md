# Addendum — Spectrum 48K and 128K

## Launch

```
cargo run -p scanline-app -- --rom image.rom
cargo run -p scanline-app -- --model 128 --rom image.rom
cargo run -p scanline-app -- --rom image.rom --sna game.sna
```

`--model` is `48` or `128`. A 128K SNA forces the 128K port decode even if the flag was 48.

## Memory

Eight 16K RAM banks. CPU windows: bank 5 at `0x4000`, bank 2 at `0x8000`, paged bank at `0xC000`. ROM is two 16K images. Port `0x7FFD` bits: 0–2 bank, 3 screen (0 bank 5, 1 bank 7), 4 ROM half, 5 lock. The displayed bitmap is the screen bank, not necessarily the bytes currently at `0x4000`.

48K ignores `0x7FFD`. A 16K ROM copied into 128K mode fills both halves.

## Frame

`FRAME_CYCLES = 69888`. `HALT` burns 4 T-states until the next frame interrupt. `EI` sets `IFF1` after the following instruction retires. Interrupt mode 2 vector is `I * 256 + 0xFF`. Accepting an interrupt clears `IFF1` and `IFF2`.

## Keyboard

Half-rows, bit 0 is the outer key:

| High bit clear | Keys |
| --- | --- |
| 0 | Caps Shift, Z, X, C, V |
| 1 | A, S, D, F, G |
| 2 | Q, W, E, R, T |
| 3 | 1, 2, 3, 4, 5 |
| 4 | 0, 9, 8, 7, 6 |
| 5 | P, O, I, U, Y |
| 6 | Enter, L, K, J, H |
| 7 | Space, Symbol Shift, M, N, B |

Host: Left Shift = Caps Shift, Right Shift and Left Control = Symbol Shift. F1/F2/F3 looks. F12 reset. Escape quits.

## SNA

48K is 27 + 49152 bytes. `PC` is popped from `SP`. 128K is 131103 bytes. The first 48K of RAM is banks 5, 2, and the bank selected by the trailer port. The other banks follow in ascending order. The trailer is `PC` low, `PC` high, port `0x7FFD`, TR-DOS byte. TR-DOS is ignored. The 128K `PC` is not popped.

## Sound

Beeper is `0xFE` bit 4, held for the following T-states. AY select is `0xFFFD`, data is `0xBFFD`, decoded as A15=1 and A1=0, with A14 choosing select versus data. AY clock steps at half the CPU T-states. The buffer is 48 kHz.
