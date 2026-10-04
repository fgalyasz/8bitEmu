# Addendum — Smooth turbo

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`Presenter::on_display_tick` does not rush. `Presenter::rush` runs one frame when `turbo_budget` is 1 and `prompt_at` is clear, drops that frame's audio, and returns the picture. The window calls it until 12 ms have passed. `turbo_budget` is 1 only when the loading sound is off, the ear is live, and the tape is still playing.
