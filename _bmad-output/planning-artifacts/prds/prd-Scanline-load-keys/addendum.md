# Addendum — Load keys

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`load_prompt(false)` is the 48K sequence: 100 quiet frames, J (row 6 mask `0x08`), Symbol Shift+P twice, Enter. `load_prompt(true)` is 80 quiet frames, then Enter (row 6 mask `0x01`). `Presenter::arm_prompt` passes `Machine::is_128`.

## FR-2

`resume_after_stop(phase, polling)` :

- phase 0, any poll: phase 1, do not resume
- phase 1 and polling: phase 0, resume
- not polling: phase 2, do not resume
- phase 2 and polling: phase 0, resume

`Machine` calls it from `finish_ear` only while `Player::stopped` is true. Polling means 64 or more port `FE` reads in the frame.
