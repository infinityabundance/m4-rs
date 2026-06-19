dnl M4.MANUAL.EXAMPLES.1 — foreach
define(`foreach', `pushdef(`$1')_foreach(`$1', `$2', `$3')popdef(`$1')')
define(`_arg1', `$1')
define(`_foreach', `ifelse(`$2', `()', , `define(`$1', _arg1$2)$3`'_foreach(`$1', (shift$2), `$3')')')
foreach(`x', `(a, b, c, d)', `element x
')
