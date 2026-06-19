dnl ============================================================================
dnl Autoconf Seed Fixture 1: Basic argument-based macros
dnl ============================================================================
define(`FOO', `bar')dnl
define(`greet', `hello $1')dnl
FOO greet(`world')dnl
