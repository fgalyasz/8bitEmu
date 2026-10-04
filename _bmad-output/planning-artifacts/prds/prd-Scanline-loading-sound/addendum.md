# Addendum — Loading sound and turbo load

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

Menu id `loading-sound`. The check starts set. macOS and Windows take the menu event, which muda toggles before the event. Linux uses F4 because it has no menu bar. `Machine::turbo_budget` is 32 only when the sound is off, `ear_live` is set, and the player is still playing.

## FR-2

`Presenter::on_display_tick` calls `warp_load` after the source frame. Warp does nothing while `prompt_at` is set. Each extra frame replaces the displayed frame and drops that frame's audio. The window pushes audio only when `hears_loading` is true.
