dnl M4.MANUAL.EXAMPLES.1 — ifdef
define(`foo', `bar')
ifdef(`foo', ``foo' is defined', ``foo' is not defined')
ifdef(`no_such', ``no_such' is defined', ``no_such' is not defined')
