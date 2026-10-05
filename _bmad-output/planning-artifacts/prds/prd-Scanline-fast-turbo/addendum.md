# Addendum — Fast turbo

Mechanism for implementers. FR IDs refer to `prd.md`.

## FR-1

`Machine::advance` sets `silent`, runs one CPU frame, then clears `silent`. While `silent` is set, `mix` ticks the AY and does not queue samples. `Presenter::rush_cpu` calls `advance` only when the prompt is finished and `turbo_budget` is 1. `paint_latest` builds one frame from memory and the border rows after the hidden frames. The window calls `rush_cpu` until `TURBO_SLICE` (48 ms) elapses, then `paint_latest` once. `rush` remains one hidden frame plus one paint, for the existing quiet-load test.
