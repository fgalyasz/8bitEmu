# Addendum — Border stripes

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

A 48K line is 224 T-states. The paper starts on line 64. The window shows 16 rows of border above it, so visible row 0 is line 48. `run_cpu_frame` paints a row when the beam reaches that line and does not repaint it. The unbooted program still uses one border color.
