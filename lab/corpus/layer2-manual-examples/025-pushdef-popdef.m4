dnl M4.MANUAL.EXAMPLES.1 — pushdef/popdef
define(`foo', `Expansion one.')
pushdef(`foo', `Expansion two.')
foo
popdef(`foo')
foo
