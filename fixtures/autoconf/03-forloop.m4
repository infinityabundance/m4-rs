dnl ============================================================================
dnl Autoconf Seed Fixture 3: forloop iteration pattern
dnl ============================================================================
dnl NOTE: forloop with pushdef/popdef recursion is a documented gap.
dnl The engine does not yet correctly handle recursive macro redefinition
dnl during expansion (M4.RESCAN.1, $1 resolution in nested _forloop calls).
dnl This fixture tests the basic pushdef/popdef and eval integration.
define(`count', `0')dnl
pushdef(`count', `1') count popdef(`count') count dnl
eval(1 + 2) dnl
