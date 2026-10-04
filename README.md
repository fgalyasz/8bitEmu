# Scanline

Cross-platform Spectrum and C64 player. The window runs a small Z80 program in 48K RAM. It draws three bright lines through the Spectrum display file and cycles the first cell. There is no ROM. Integer scaling and the three looks stay.

```
cargo test
cargo run -p scanline-app
```

Keys: `1` Sharp, `2` Soft edge, `3` Temporal color, Escape quits.
