# Source Archaeology

## Clean-Room Design

`m4-rs` is a **clean-room behavioral reconstruction**. This means:

1. **No GNU m4 source code is consulted.** The implementation is based solely on:
   - The GNU m4 manual (black-box behavior description)
   - The POSIX m4 specification (where applicable)
   - Black-box oracle interrogation (running GNU m4 and observing output)
   - Published academic/tutorial material about m4 semantics

2. **Behavioral witnesses.** We treat the GNU m4 binary as an external oracle. When in doubt about how something works, we interrogate the oracle rather than reading its source code.

3. **No GPL entanglement.** Because we do not copy, translate, or derive from GNU m4 source, `m4-rs` can be licensed independently (MIT OR Apache-2.0).

## Historical Context

### Origins of m4

m4 was created by Brian Kernighan and Dennis Ritchie at Bell Labs in 1977. It was designed as a macro preprocessor for languages that lacked their own preprocessing capabilities. Key design decisions:

- **`\`` and `'` as quote delimiters** — chosen because they rarely appear in typical source code, minimizing conflicts.
- **`#` and newline as comment delimiters** — following the convention of many scripting languages of the era.
- **Byte orientation** — m4 operates on raw bytes, making it encoding-agnostic. This was important in the pre-Unicode era.
- **Diversions** — inspired by the `divert` mechanism in the `troff` typesetting system.

### GNU m4 Extensions

The GNU project adopted m4 and extended it significantly:

- **Frozen files** (`-F`/`-R`) — allow saving and restoring macro definitions, avoiding re-parsing of large standard libraries.
- **Regexp support** — `regexp` and `patsubst` builtins using Emacs-style regular expressions.
- **`esyscmd`** — captures command output as a string (vs `syscmd` which passes it through).
- **`mkstemp`** — safely creates temporary files.
- **`debugmode`/`debugfile`** — fine-grained control over debugging output.
- **`-d` flags** — rich debug tracing options.
- **Synclines (`-s`)** — emit `#line` directives for C preprocessor coordination.

### POSIX m4

POSIX.1-2024 specifies a minimal m4 with only a subset of GNU m4's features. GNU m4 with `-G` (traditional mode) aims for POSIX compatibility but is not identical.

## Surprising Behaviors

Some GNU m4 behaviors that are frequently misunderstood:

1. **`$10` is `$1` followed by `0`**, not the 10th argument. There is no way to access arguments beyond `$9` directly; use `shift`.

2. **Quotes are stripped, not passed through.** `\`hello'` produces `hello`, not `\`hello'`.

3. **Comments consume the newline.** A `# comment` line produces no blank line in output. This is why `dnl` exists — to suppress newlines in macros.

4. **`include` rescans, `undivert` copies.** `include(filename)` reads and expands macros in the file. `undivert(filename)` copies the file contents without expansion. `undivert` of a diversion number is different from `undivert` of a filename.

5. **Diversion 0 is special.** Output to diversion 0 goes to stdout immediately. Other diversions are buffered and output when explicitly undiverted or at EOF.

6. **Builtins can be shadowed and restored.** User-defined macros shadow builtins of the same name. `builtin(define, ...)` calls the original builtin even if `define` has been redefined.

## Implementation Notes

### Why not regex-based?

Many "m4-like" tools use regular expressions for macro substitution. This fails for GNU m4 because:

- Nested quotes require balancing, which regex can't do
- Argument collection requires tracking nested parentheses
- Rescanning order is sensitive to intermediate state
- Quote delimiters can change at runtime

Instead, `m4-rs` uses a proper tokenizer and pushdown automaton.

### Why bytes, not characters?

GNU m4 is explicitly byte-oriented (eight-bit-clean except NUL). Multi-byte encodings (UTF-8) are not special-cased. `m4-rs` follows this design:

- Core engine operates on `Vec<u8>` and `&[u8]`
- String conversion is only done at CLI/report boundaries
- All comparisons are byte-level

### Why safe Rust?

The initial implementation forbids `unsafe`. If `unsafe` ever becomes necessary for performance reasons, it would require a separate `M4.UNSAFE.ADMISSION.1` receipt with line-by-line justification.
