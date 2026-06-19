dnl ============================================================================
dnl Autoconf Seed Fixture 5: Quote manipulation patterns
dnl ============================================================================
changequote([, ])dnl
define([bracket_macro], [[$1] in brackets])dnl
bracket_macro([hello])dnl
changequote(`'', `'')dnl
define(`backtick_macro', `$1 in backticks')dnl
backtick_macro(`world')dnl
