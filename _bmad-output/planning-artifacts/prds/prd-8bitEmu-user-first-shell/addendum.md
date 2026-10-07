# Addendum — User-first shell

## ROM resolution

`resolve_launch(config) -> Launch` fills concrete existing paths.

Standard names: `spectrum-48.rom`, `spectrum-128.rom`, `c64-kernal.rom`, `c64-basic.rom`, `c64-chargen.rom`.

Search roots: config `rom_folder` if set; else `roms` under cwd, next to the executable, and under the platform config dir.

Overrides (`spectrum_rom`, `kernal`, …) win when the path exists (absolute or relative).

## Open → machine

| Kind | Machine |
| ---- | ------- |
| `.prg` | C64 |
| `.tap` / `.tzx` / `.sna` | Spectrum (SNA size may set 128) |
| `.rom` | Spectrum (32 KiB → 128) |

## Menu

- Machine items start immediately.
- Settings → ROM Folder… primary; per-ROM items remain overrides.
- Start = restart current config.
