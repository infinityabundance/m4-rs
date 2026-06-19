dnl M4.MANUAL.EXAMPLES.1 — nested define
define(`foo', `define(`bar', `baz')')
foo
bar
