dnl M4.POSIX.PROFILE.1 — POSIX m4 specification examples
dnl These are derived from the POSIX m4 utility specification.
dnl They test the portable baseline, excluding GNU extensions.

dnl POSIX 001: basic macro expansion
define(`hello', `world')hello

dnl POSIX 002: define with arguments (POSIX: up to 9 positional params)
define(`swap', `$2 $1')
swap(`one', `two')

dnl POSIX 003: undefine
define(`temp', `value')
temp
undefine(`temp')
temp

dnl POSIX 004: ifdef (POSIX requires ifdef)
define(`x', `defined')
ifdef(`x', `yes', `no')
ifdef(`y', `yes', `no')

dnl POSIX 005: ifelse (POSIX requires ifelse)
ifelse(`a', `a', `match', `no match')
ifelse(`a', `b', `match', `no match')

dnl POSIX 006: dnl
define(`x', `hello')dnl
x

dnl POSIX 007: shift
define(`args', `$1:$2:$#')
shift(args(`a', `b', `c', `d'))

dnl POSIX 008: changequote
changequote([,])define([x], [inside brackets])x

dnl POSIX 009: changecom
changecom(#)text # comment
more text

dnl POSIX 010: include (requires file to exist — skip if no file)
dnl sinclude(nonexistent_file) is POSIX-safe

dnl POSIX 011: divert and undivert
divert(1)
hidden in diversion 1
divert
undivert(1)

dnl POSIX 012: eval (POSIX requires eval with basic operators)
eval(3 + 4)
eval(10 / 3)
eval(5 % 3)

dnl POSIX 013: incr and decr
incr(10)
decr(10)

dnl POSIX 014: len
len(`hello world')

dnl POSIX 015: index
index(`hello', `ll')

dnl POSIX 016: substr
substr(`hello', 1, 3)

dnl POSIX 017: translit
translit(`hello', `elo', `ipg')

dnl POSIX 018: sinclude with missing file (no error)
sinclude(`nonexistent_file_xyz')

dnl POSIX 019: nested quoting
define(`outer', `define(`inner', `nested')')
outer
inner

dnl POSIX 020: empty arguments
define(`count', `$#')
count(,,)
