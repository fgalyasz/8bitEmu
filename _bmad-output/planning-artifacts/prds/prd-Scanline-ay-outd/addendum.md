# Addendum — AY OUTD

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`extended::block` handles `0xA2`–`0xBB` I/O forms. `B` decrements first, then the transfer uses the updated `BC`. `HL` steps up for INI/OUTI and down for IND/OUTD. Repeat forms rewind `PC` by two while `B` is not zero.
