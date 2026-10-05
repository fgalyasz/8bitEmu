# Addendum — AY noise

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`Ay` keeps one 17-bit `rng`, starting at 1. `tick_noise` adds the clocks, divides by `max(register 6, 1) * 16`, and shifts that many times. Each shift is `rng = (rng >> 1) | (((rng ^ (rng >> 3)) & 1) << 16)`. The output bit is `rng & 1`. `audible` is `tone_open && noise_open`. A mixer bit of 0 enables that generator; a set bit forces the gate high.
