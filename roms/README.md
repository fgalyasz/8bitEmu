# System ROMs

8bitEmu does not ship ROM images. Put your own files in this folder.

## ZX Spectrum

Amstrad allows these images with an emulator if the copyright text stays unchanged and the image itself is not sold. The copyright remains Amstrad's.

Download the Debian `spectrum-roms` source archive:

https://deb.debian.org/debian/pool/non-free/s/spectrum-roms/spectrum-roms_20081224.orig.tar.gz

Package page: https://packages.debian.org/stable/otherosfs/spectrum-roms

Expected files here:

- `spectrum-48.rom` — 16384 bytes (48K; from upstream `48.rom`)
- `spectrum-128.rom` — 32768 bytes (editor ROM + 48K BASIC, in that order)

```
cat 128-0.rom 128-1.rom > spectrum-128.rom
cp 48.rom spectrum-48.rom
```

SHA1 sums from https://sinclair.wiki.zxnet.co.uk/wiki/ROM_images :

- `48.rom` / `spectrum-48.rom` `5ea7c2b824672e914525d1d5c419d71b84a426a2`
- `128-0.rom` `4f4b11ec22326280bdb96e3baf9db4b4cb1d02c5`
- `128-1.rom` `80080644289ed93d71a1103992a154cc9802b2fa`

The +2A and +3 images in that archive are not for this machine yet.

## Commodore 64

Commodore's original KERNAL, BASIC, and character ROMs are still copyrighted.
Do not commit those dumps. 8bitEmu loads whatever you place in the expected
filenames below (gitignored).

### Stock ROMs (best compatibility)

For `PRINT`, `LOAD`, and most software you want the real images (e.g. from
[C64 Forever](https://www.c64forever.com/) / Cloanto, or a dump from hardware
you own). Rename them to the expected names in this folder:

- `c64-kernal.rom` — 8192 bytes (901227-03)
- `c64-basic.rom` — 8192 bytes (901226-01)
- `c64-chargen.rom` — 4096 bytes (901225-01)

VICE publishes these SHA1 sums for identification (not redistribution):

- KERNAL 901227-03 `1d503e56df85a62fee696e7618dc5b4e781df1bb`
- BASIC 901226-01 `79015323128650c742a3694c9429aa91f355905e`
- CHARGEN 901225-01 `adc7c31e18c7c96429e4a4c502e945c9321b7fb9`

Keep Open ROMs copies aside if you still want them (e.g. `c64-*-openroms.rom`).

### Open ROMs (freely redistributable, incomplete BASIC)

Use the clean-room **Open ROMs** set for a legal default boot
(LGPL-3.0 / GPL-3.0). Upstream:

https://github.com/MEGA65/open-roms

Debian ships the same project as `open-roms` in main:

https://packages.debian.org/stable/otherosfs/open-roms

Latest orig tarball (sid):

https://deb.debian.org/debian/pool/main/o/open-roms/open-roms_0.0~git20260509.93b6a24.orig.tar.xz

After install, the package places:

- `/usr/share/open-roms/C64/kernal`
- `/usr/share/open-roms/C64/basic`
- `/usr/share/open-roms/C64/chargen`

Or copy the matched generic pair from upstream `bin/`:

- `kernal_generic.rom` — 8192 bytes
- `basic_generic.rom` — 8192 bytes
- `chargen_openroms.rom` — 4096 bytes

Expected files here:

- `c64-kernal.rom` — 8192 bytes (from `kernal_generic.rom`)
- `c64-basic.rom` — 8192 bytes (from `basic_generic.rom`)
- `c64-chargen.rom` — 4096 bytes (from `chargen_openroms.rom`)

SHA1 sums from MEGA65/open-roms `master` `bin/` (generic set):

- `kernal_generic.rom` `2ecf1bf1553bf77ba8a007a18a1a7c291d215b33`
- `basic_generic.rom` `4468852d4fde3394ca750f45d1c83914e218dac5`
- `chargen_openroms.rom` `466e399a5e994b52f6cef6019b32b9f6b504cd14`

Use `kernal_generic` with `basic_generic` from the same build. Do not mix with
Commodore dumps or Ultimate64-specific images. Open ROMs is not bit-identical to
the stock C64; some software may behave differently.
