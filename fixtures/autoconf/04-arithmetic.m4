dnl ============================================================================
dnl Autoconf Seed Fixture 4: Arithmetic expressions (eval)
dnl ============================================================================
dnl Basic comparison: eval returns 1 for true, 0 for false
eval(3 > 5) eval(5 > 3) dnl
eval(0x10 + 020) eval(1 << 4) dnl
incr(5) decr(5) dnl
