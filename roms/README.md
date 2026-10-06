# System ROMs

8bitEmu does not ship ROM images. Put your own files in this folder.

## ZX Spectrum

Amstrad allows these images with an emulator if the copyright text stays unchanged and the image itself is not sold. The copyright remains Amstrad's.

Download the Debian `spectrum-roms` source archive:

https://deb.debian.org/debian/pool/non-free/s/spectrum-roms/spectrum-roms_20081224.orig.tar.gz

Package page: https://packages.debian.org/stable/otherosfs/spectrum-roms

Expected files here:

- `48.rom` — 16384 bytes (48K)
- `128.rom` — 32768 bytes (editor ROM + 48K BASIC, in that order)

```
cat 128-0.rom 128-1.rom > 128.rom
```

SHA1 sums from https://sinclair.wiki.zxnet.co.uk/wiki/ROM_images :

- `48.rom` `5ea7c2b824672e914525d1d5c419d71b84a426a2`
- `128-0.rom` `4f4b11ec22326280bdb96e3baf9db4b4cb1d02c5`
- `128-1.rom` `80080644289ed93d71a1103992a154cc9802b2fa`

The +2A and +3 images in that archive are not for this machine yet.
