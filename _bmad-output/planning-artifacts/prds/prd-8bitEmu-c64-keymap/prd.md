---
title: C64 keymap
status: done
created: 2026-10-06
updated: 2026-10-06
parent_issue: 80
---

# PRD: C64 keymap

Hobby/solo. Fix: host keys were wired to the transpose of the KERNAL matrix, so `A` typed as `R`.

## 1. Vision

Letter and digit keys match Open ROMs / stock KERNAL `SCNKEY` table: CIA1 `$DC00` bit selects the matrix line, `$DC01` bit is the key within that line.

## 2. User journeys

- **UJ-1.** Anna boots C64, presses `A`, and sees `A` at the cursor.

## 3. Features

#### FR-1: KERNAL matrix order

`c64_key` returns `(PRB bit, PRA bit)` matching Open ROMs `kb_matrix` / `kb_matrix_row_keys` (not the PRG diagram axes).

**Consequences:**
- `KeyA` maps to `(2, 1)`.
- Holding that pair and running frames after READY writes screen code `1` (`A`).

## 4. Non-goals

Joysticks on CIA1, C128 extra keys, key repeat tuning.

## 5. Success metrics

- **SM-1**: `cargo test` passes.
- **SM-C1**: The mapping and on-screen `A` assertions pass.
