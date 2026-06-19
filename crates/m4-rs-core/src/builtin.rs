// m4-rs builtin registry.
//
// This module registers all GNU m4 builtin macros with the expansion engine.
// Each builtin is admitted separately with its own receipt court.
//
// Builtins are registered at engine initialization. User definitions
// may shadow them; `builtin` and `indir` can bypass the shadowing.

use crate::expansion::ExpansionEngine;

/// Register all standard GNU m4 builtins.
///
/// Each builtin is marked with whether it is "blind" (doesn't take
/// arguments and doesn't need `()` to trigger).
pub fn register_all(engine: &mut ExpansionEngine) {
    // Blind builtins (no arguments, expand immediately)
    engine.macro_table.register_builtin(b"dnl", true);

    // Primary definition builtins
    engine.macro_table.register_builtin(b"define", false);
    engine.macro_table.register_builtin(b"undefine", false);
    engine.macro_table.register_builtin(b"defn", false);
    engine.macro_table.register_builtin(b"pushdef", false);
    engine.macro_table.register_builtin(b"popdef", false);
    engine.macro_table.register_builtin(b"indir", false);
    engine.macro_table.register_builtin(b"builtin", false);

    // Quoting and comments
    engine.macro_table.register_builtin(b"changequote", false);
    engine.macro_table.register_builtin(b"changecom", false);
    // changeword — often disabled; register but mark as potentially unsupported

    // Conditionals
    engine.macro_table.register_builtin(b"ifdef", false);
    engine.macro_table.register_builtin(b"ifelse", false);

    // Loops and shifts
    engine.macro_table.register_builtin(b"shift", false);

    // Arithmetic
    engine.macro_table.register_builtin(b"eval", false);
    engine.macro_table.register_builtin(b"incr", false);
    engine.macro_table.register_builtin(b"decr", false);

    // String operations
    engine.macro_table.register_builtin(b"len", false);
    engine.macro_table.register_builtin(b"index", false);
    engine.macro_table.register_builtin(b"substr", false);
    engine.macro_table.register_builtin(b"translit", false);
    engine.macro_table.register_builtin(b"regexp", false);
    engine.macro_table.register_builtin(b"patsubst", false);

    // Formatting
    engine.macro_table.register_builtin(b"format", false);

    // File inclusion
    engine.macro_table.register_builtin(b"include", false);
    engine.macro_table.register_builtin(b"sinclude", false);

    // Diversions
    engine.macro_table.register_builtin(b"divert", false);
    engine.macro_table.register_builtin(b"undivert", false);
    engine.macro_table.register_builtin(b"divnum", false);

    // Shell execution
    engine.macro_table.register_builtin(b"syscmd", false);
    engine.macro_table.register_builtin(b"esyscmd", false);
    engine.macro_table.register_builtin(b"sysval", false);

    // Debugging and tracing
    engine.macro_table.register_builtin(b"debugmode", false);
    engine.macro_table.register_builtin(b"debugfile", false);
    engine.macro_table.register_builtin(b"traceon", false);
    engine.macro_table.register_builtin(b"traceoff", false);
    engine.macro_table.register_builtin(b"dumpdef", false);

    // Diagnostic and exit
    engine.macro_table.register_builtin(b"errprint", false);
    engine.macro_table.register_builtin(b"__file__", true);
    engine.macro_table.register_builtin(b"__line__", true);
    engine.macro_table.register_builtin(b"m4exit", false);

    // Temporary files
    engine.macro_table.register_builtin(b"maketemp", false);
    engine.macro_table.register_builtin(b"mkstemp", false);

    // Platform detection macros (register as blind — they expand to empty string).
    // GNU m4 defines __gnu__ and __unix__ (or __windows__/__os2__) at startup.
    // These are conditionally defined; we register them unconditionally since
    // m4-rs always runs with GNU extensions on a Unix-like system.
    engine.macro_table.register_builtin(b"__gnu__", true);
    engine.macro_table.register_builtin(b"__unix__", true);
}
