# Addendum — Quiet output

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`next_sample` returns the queued value and remembers it. An empty queue returns that remembered value. `mix` divides cycle counts by `FRAME_CYCLES * 50`, so 69888 T-states become 960 samples at 48000 Hz.
