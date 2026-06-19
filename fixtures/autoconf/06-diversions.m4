dnl ============================================================================
dnl Autoconf Seed Fixture 6: Diversion usage (M4sh pattern)
dnl
dnl Tests diversion patterns used in m4sh:
dnl   - divert to defer output
dnl   - undivert to reorder output
dnl   - divnum for current diversion tracking
dnl ============================================================================
divert(1)dnl
This text is diverted to diversion 1.
It appears after diversion 0 text.
divert(0)dnl
This text appears first (diversion 0).
undivert(1)dnl
dnl Auto-undivert of remaining diversions at EOF
