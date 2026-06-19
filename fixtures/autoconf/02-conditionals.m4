dnl ============================================================================
dnl Autoconf Seed Fixture 2: Conditional expansion (ifdef/ifelse)
dnl ============================================================================
define(`feature_enabled', `yes')dnl
ifdef(`feature_enabled', `Feature is enabled', `Feature is disabled')dnl
ifelse(`a', `a', `match', `no match')dnl
ifelse(`a', `b', `match', `no match')dnl
