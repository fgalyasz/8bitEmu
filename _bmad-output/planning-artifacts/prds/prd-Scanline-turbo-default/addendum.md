# Addendum — Turbo load by default

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`Machine::new` sets `loading_sound` to false. `loading_sound_item` builds the check menu with `checked` false. Existing turbo and beep tests keep setting the flag explicitly.
