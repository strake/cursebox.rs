# cursebox

A cell-grid TTY UI library

Goals:
* Simple API
* Minimal dynamic allocation
* Little code size

Evolved from termbox

Note:
So far (as of rust 1.28), you must use nightly.
Building on stable is a goal — we are waiting for `const_fn` and `untagged_unions`.
