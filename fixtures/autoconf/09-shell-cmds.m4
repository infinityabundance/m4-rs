dnl ============================================================================
dnl Autoconf Seed Fixture 9: Shell command integration
dnl
dnl Tests syscmd/esyscmd/sysval patterns:
dnl   - esyscmd to capture command output
dnl   - sysval to check exit status
dnl   - maketemp/mkstemp for temp files
dnl ============================================================================
define(`HOSTNAME', esyscmd(`hostname'))dnl
esyscmd(`echo hello from shell')dnl
syscmd(`true')sysval
dnl Should output: 0
syscmd(`false')sysval
dnl Should output: 1 (on most systems)
