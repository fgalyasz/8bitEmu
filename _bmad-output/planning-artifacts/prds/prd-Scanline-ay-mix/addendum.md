# Addendum — AY mix

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`amplitude` maps a level 0–15 through the measured curve used by Fuse (Matthew Westcott's steps, scaled so 15 is 1). `channel` returns that value. `sample` still averages the three channels.

## FR-2

`envelope_period` multiplies the register, at least 1, by 16. A step of the 16-deep envelope is one of those periods, so a full cycle is 256 times the register.
