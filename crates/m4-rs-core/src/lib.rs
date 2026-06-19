// ============================================================================
// m4-rs-core: Core macro expansion engine for forensic-parity GNU m4.
// ============================================================================
//
// WHO:   infinityabundance, following the forensic-parity methodology
//        established in gnucobol-rs, zic-rs, chrony-rs, and ncurses-native.
//        GNU m4 was originally created by Brian Kernighan and Dennis Ritchie
//        at Bell Labs in 1977. The GNU project extended it significantly
//        beginning in the early 1990s by François Pinard, René Seindal, and
//        later Gary V. Vaughan and Eric Blake.
//
// WHAT:  This crate implements the core macro-processing engine of m4:
//        - Lexer: converts raw input bytes into a token stream
//        - Macro table: symbol table with pushdef/popdef/define/undefine
//        - Expansion engine: recognizes macros, collects arguments, expands,
//          and rescans output
//        - Quote/comment system: configurable delimiters with nesting
//        - All standard GNU m4 builtins (builtin.rs registers them)
//        - Input stack: files, stdin, string input, rescan pushback
//
// WHEN:  This is Phase 0-3 implementation work. The engine is the nucleus
//        of m4-rs. Everything else — diversions, frozen files, shell
//        execution, Autoconf compatibility — hangs off this core.
//
// WHERE: crates/m4-rs-core/ — the heart of the m4-rs workspace.
//        Depended on by m4-rs-cli (the binary) and tested by xtask.
//
// WHY:   GNU m4 is not just a parser or a regex-substitution system. It is
//        a byte-oriented macro-expansion machine with quoting, comments,
//        recursive rescanning, diversions, include/search paths, shell
//        execution, debug traces, frozen-state files, POSIX/GNU mode
//        differences, and Autoconf compatibility pressure. A faithful
//        reproduction requires a proper tokenizer and pushdown automaton,
//        not regex substitution.
//
//        The forensic-parity approach means:
//        - Every behavior is verified against a pinned GNU m4 oracle
//        - No behavior is "correct" unless the oracle agrees
//        - Unimplemented surfaces are explicit non-claims
//        - The manual is a witness, not an authority
//
// HOW:   Architecture:
//
//          Input → Lexer → Token Stream → Expansion Engine → Output
//                       ↑                    ↑
//                 Quote/Comment         Macro Table
//                 Delimiters            (define/pushdef/popdef)
//
//        The data flow is:
//        1. Input bytes arrive from files, stdin, or macro expansion rescan
//        2. The Lexer tokenizes bytes into Text, Name, ParenOpen, ParenClose,
//           Comma, QuoteOpen, and QuoteClose tokens
//        3. The Expansion Engine consumes tokens:
//           a. Text tokens → copied to output
//           b. Name tokens → looked up in macro table
//              - If defined: the macro is expanded
//              - If undefined: the name is copied to output as text
//           c. ParenOpen after a Name → trigger argument collection
//           d. The macro body text has $n placeholders substituted
//           e. The resulting text is rescanned (fed back to the lexer)
//        4. Output is collected and eventually written to stdout or diversion
//
//        Side effects are abstracted through traits (to be fully implemented):
//        - InputProvider  — source of input bytes
//        - OutputSink     — destination for output bytes
//        - FileSystem     — filesystem operations (include search, frozen files)
//        - CommandRunner  — subprocess execution (syscmd/esyscmd)
//        - DiagnosticSink — error/warning/trace output
// ============================================================================

pub mod args;
pub mod builtin;
pub mod comment;
pub mod diagnostics;
pub mod diversion;
pub mod eval;
pub mod expansion;
pub mod format;
pub mod frozen;
pub mod include_;
pub mod input;
pub mod lexer;
pub mod macro_table;
pub mod profile;
pub mod quote;
pub mod regexp;
pub mod stack;
pub mod token;
pub mod trace;

// Re-export the key public types for convenience.
// External users (m4-rs-cli, xtask) use these to interact with the engine.
pub use lexer::Lexer;
pub use macro_table::MacroTable;
pub use quote::QuoteState;
pub use stack::{get_recursion_limit, probe_stack, set_recursion_limit, DEFAULT_STACK_SIZE};
pub use token::Token;
pub use token::TokenKind;
