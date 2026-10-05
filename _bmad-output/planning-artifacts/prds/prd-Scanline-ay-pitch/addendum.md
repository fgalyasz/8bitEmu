# Addendum — AY pitch

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`period_of` multiplies the 12-bit tone period by 8. The tone output flips each time that many AY clocks elapse, so the square wave repeats every 16 times the period. Noise stays at 16 times its period per shift. The envelope stays at 256 times its period per step.
