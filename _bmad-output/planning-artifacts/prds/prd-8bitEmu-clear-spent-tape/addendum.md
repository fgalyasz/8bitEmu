# Addendum — Clear spent tape

`Player::spent` = `!at_start() && !playing()`.

`Machine::drop_spent_tape` clears `player` when spent (call after frame advance).

`Machine::reset`: if player present and not `at_start`, set `player = None`; else rewind (noop at start). Presenter `arm_prompt` already keys off `has_tape()`.
