# Addendum — AY envelope

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`Ay` starts held and silent. `write` of register 13 calls `retrigger_envelope`: index 0, the phase counter 0, and the four shape bits from the register. Continue is bit 3, attack is bit 2, alternate is bit 1, hold is bit 0. Each envelope step increments the index. At 16 the cycle ends: without continue, or with hold, it stays at the held level. Otherwise the index returns to 0, and alternate flips the direction. Decay outputs `15 - index`. Attack outputs the index. The held level is 0 when continue is clear. When continue and hold are set, the level stays at 15 only when attack and alternate differ.
