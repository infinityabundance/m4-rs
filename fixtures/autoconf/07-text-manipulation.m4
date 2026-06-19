dnl ============================================================================
dnl Autoconf Seed Fixture 7: Text manipulation (translit, regexp, patsubst)
dnl
dnl Tests string operations used in autoconf for text transformation:
dnl   - translit for character mapping
dnl   - regexp for pattern matching
dnl   - patsubst for substitution
dnl ============================================================================
translit(`Hello World', `A-Z', `a-z')dnl
dnl Should output: hello world
translit(`abcdef', `abc', `123')dnl
dnl Should output: 123def
regexp(`hello world', `w..ld')dnl
dnl Should output: 6 (position)
patsubst(`hello world', `([a-z]+) ([a-z]+)', `\2 \1')dnl
dnl Should output: world hello
