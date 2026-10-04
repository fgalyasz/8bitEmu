# Addendum — Spectrum 48K display

## Memory

ROM window: `0x0000`–`0x3FFF`, reads `0xFF`, writes ignored. RAM: 48 KB at `0x4000`. Program entry: `0x8000`.

## Display file

```
bitmap = 0x4000 | ((y & 0xC0) << 5) | ((y & 0x07) << 8) | ((y & 0x38) << 2) | (x >> 3)
attribute = 0x5800 + (y / 8) * 32 + (x / 8)
```

The frame attribute plane is per scanline (`32 * 192`). Each character-row byte is repeated for eight rows. Border width stays 16. Palette stays ULA. No sprites.

Attribute byte: bits 0–2 ink, 3–5 paper, 6 bright, 7 flash.

## Frame loop

`run_until_halt` executes at most 100000 instructions. `HALT` leaves `PC` on the halt opcode. The next frame clears halt and adds 1 to `PC`, then runs until the next halt.

`OUT (0xFE)` stores `A & 7` as the border. Other OUT ports are ignored. `IN A,(n)` sets `A` to `0xFF`.

## Picture

The program paints bitmap rows 0, 64, and 128 with `0xFF`, fills attributes with `0x47` (bright white ink, black paper), then once per frame stores `(ink + 1) & 7` at `0x5800`. The attribute fill keeps `A` in a `DJNZ` loop so the color byte is not replaced by the counter. A clear bit on `0x47` is bright black, palette index 8.

## Flags

`S=0x80`, `Z=0x40`, `H=0x10`, `PV=0x04`, `N=0x02`, `C=0x01`. Bits 3 and 5 are written as 0. `INC r` / `DEC r` copy the previous `C`.
