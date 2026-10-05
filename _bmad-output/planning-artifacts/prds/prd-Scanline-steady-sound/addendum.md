# Addendum — Steady sound

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`emit` returns the held sample, updated from the front of the queue, while `live` is false and fewer than `LEAD` (4096) samples are queued. The first read at `LEAD` sets `live` and pops. Later reads use `next_sample`, which repeats the previous sample when the queue is empty.
