# Addendum — Real time

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`due_ticks` divides lateness by 1/60 s and caps it. The window advances `next_tick` by that many steps. Hitting the cap sets `next_tick` to now. The picture from the last step is kept for redraws that are not due.

## FR-2

`Ay::tick` receives CPU cycles / 2. Tone and noise periods are the register, at least 1, times 16. The envelope period is times 256.

## FR-3

`ear_level` is ±0.3 only when loading sound is on and the loader is active. The speaker pops one queue sample per frame and copies it to each channel.
