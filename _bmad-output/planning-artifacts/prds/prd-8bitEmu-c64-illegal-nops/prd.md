---
title: C64 illegal NOP opcodes
status: done
created: 2026-10-06
updated: 2026-10-06
parent_issue: pending
---

# PRD: C64 illegal NOP opcodes

Hobby/solo. Fix: games and cracktros use undocumented NOPs (`$FC`, `$1A`, …); treating them as fatal stopped execution.

## Features

#### FR-1: Undocumented NOPs execute

Implied, immediate, zp, zp,X, abs, and abs,X NOP illegals advance PC and consume cycles instead of returning `CoreError::Opcode`.

**Consequences:**
- `$FC` abs,X NOP unit test passes; `$02` (KIL) still errors.
