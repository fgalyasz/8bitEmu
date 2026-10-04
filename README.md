# Scanline

Cross-platform Spectrum and C64 player. With no arguments the window runs a small Z80 program in 48K RAM: blue border, three bright lines, and a cell that steps through the ink colors. Pass a ROM or an SNA the user already has to boot a 48K or basic 128K Spectrum. Integer scaling and the three looks stay. No ROM is included.

```
cargo test
cargo run -p scanline-app
cargo run -p scanline-app -- --rom image.rom
cargo run -p scanline-app -- --model 128 --rom image.rom
cargo run -p scanline-app -- --rom image.rom --sna game.sna
```

Keys: F1 Sharp, F2 Soft edge, F3 Temporal color, F12 reset, Escape quits. Digit keys and the letter rows go to the Spectrum matrix. Left Shift is Caps Shift. Right Shift and Left Control are Symbol Shift.
