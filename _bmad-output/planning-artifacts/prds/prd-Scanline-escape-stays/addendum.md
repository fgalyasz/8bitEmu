# Addendum — Escape stays

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`quits` is false for every key, including Escape. `Window::key` exits only when `quits` is true. `WindowEvent::CloseRequested` still calls `exit`. The Scanline menu keeps the predefined Quit item.
