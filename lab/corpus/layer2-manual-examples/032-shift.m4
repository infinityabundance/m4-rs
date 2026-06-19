dnl M4.MANUAL.EXAMPLES.1 — shift
define(`foo', `$1:$2:$#')
foo(`a', `b', `c')
shift(foo(`a', `b', `c'))
