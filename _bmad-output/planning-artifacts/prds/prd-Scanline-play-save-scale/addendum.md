# Addendum: Play, save, and scale

```
cargo run -p scanline-app -- --rom roms/48.rom --tap game.tap
cargo run -p scanline-app -- --model 128 --rom roms/128.rom --tzx game.tzx
```

F5 125%, F6 150%, F7 175%, F8 200%. Dragging the window fits the picture. F9 resumes a stopped tape. F11 writes `scanline.sna`. A standard-speed SAVE writes `scanline.tap`. Arrows and Right Alt are Kempston.
