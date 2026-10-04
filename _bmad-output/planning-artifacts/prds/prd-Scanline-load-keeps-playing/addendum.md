# Addendum — Keep the tape playing through a load

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`ear_live` becomes true when `fe_reads` reaches 64 and stays true until `load_rom`, `load_tape`, `load_sna`, or `reset`. Quiet frames, including the ROM delay at `$0574`, keep calling `player.advance`.

## FR-2

`append_pause` pushes an edge whose level is the level the next pulse would have used, then toggles that level. A zero pause is still the stop marker.
