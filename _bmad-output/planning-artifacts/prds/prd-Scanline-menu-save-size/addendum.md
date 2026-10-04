# Addendum — Save and size from the menu

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

Menu ids: `size-fit`, `size-125`, `size-150`, `size-175`, `size-200`. The percentage items show F5 through F8. Choosing one sets the same window size as the matching key. Fit sets the percentage to none and requests 960 by 720 logical pixels.

## FR-2

Menu ids: `save-sna`, `save-tap`, `save-tzx`. The snapshot dialog starts at `scanline.sna`. The tape dialogs start at `scanline.tap` and `scanline.tzx`. The bytes come from the recorder's finished blocks, not from the player. TZX is `ZXTape!`, version 1.20, then one `0x10` block per recorded block with a 1000 ms pause. `take_tap` still clears the dirty flag for the automatic `scanline.tap` write. A later menu save reads the blocks that remain.
