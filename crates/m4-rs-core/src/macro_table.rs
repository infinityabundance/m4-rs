// m4-rs macro table — the symbol table for defined macros.
//
// GNU m4's macro symbol table manages:
// 1. User-defined macros (via `define`, `pushdef`, `popdef`)
// 2. Builtin macros (recognized by the engine)
// 3. Per-name definition stacks (pushdef/popdef)
// 4. Lookup during expansion
//
// Key behaviors:
//
// 1. **Case sensitivity**: GNU m4 is case-sensitive. `define` and `DEFINE`
//    are different names. This matches UNIX tradition.
//
// 2. **`define`**: Creates or replaces the current definition for a name.
//    Replaces the top of the stack without affecting lower entries.
//
// 3. **`pushdef`**: Pushes a new definition onto the stack for a name.
//    The previous definition remains underneath.
//
// 4. **`popdef`**: Pops the top definition from the stack. If the stack
//    becomes empty, the name is undefined. Calling `popdef` on an undefined
//    name is not an error (in GNU m4 it silently does nothing).
//
// 5. **`undefine`**: Clears the entire definition stack for a name,
//    including all pushdef'd entries.
//
// 6. **`defn`**: Copies the definition text (with pushdef stacking
//    semantics) and returns it as a quoted string. `defn` of a builtin
//    copies the builtin's implementation reference, allowing builtins
//    to be renamed.
//
// 7. **Builtin shadowing**: User-defined macros shadow builtins of the
//    same name. The `builtin` macro can access the shadowed builtin.
//    The `-P`/`--prefix-builtins` flag forces builtins to have the
//    `m4_` prefix (e.g., `m4_define` instead of `define`).
//
// 8. **Macro names colliding with builtins**: When a user defines a
//    macro with the same name as a builtin, the user's definition
//    takes precedence. The builtin can still be accessed via `builtin`
//    or via the prefix if `-P` is enabled.
//
// Reference: GNU M4 manual, Sections 5.1–5.2, 6.4 (builtin), 7.7 (indir)

use std::collections::HashMap;

/// A macro definition.
///
/// Each definition carries:
/// - The expansion text (raw bytes)
/// - Whether this is a builtin (immutable, cannot be redefined except via renaming)
/// - Source location where defined (for diagnostics)
/// - Whether this definition was copied from a builtin via defn
#[derive(Debug, Clone)]
pub struct MacroDef {
    /// The expansion text as raw bytes.
    /// For user-defined macros, this is the body from `define`.
    /// For builtins, this may be empty (builtins are handled by code).
    pub text: Vec<u8>,
    /// Whether this is a builtin macro.
    pub is_builtin: bool,
    /// Source location where this macro was defined.
    pub defined_at: Option<crate::token::SourceLocation>,
    /// True if this is a blind builtin (no arguments, like `dnl`).
    pub is_blind: bool,
    /// True if this was copied from a builtin via `defn`.
    /// The expansion engine treats these as builtin references.
    pub copied_builtin: Option<String>,
}

impl MacroDef {
    pub fn new(text: Vec<u8>) -> Self {
        Self {
            text,
            is_builtin: false,
            defined_at: None,
            is_blind: false,
            copied_builtin: None,
        }
    }

    pub fn builtin(is_blind: bool) -> Self {
        Self {
            text: Vec::new(),
            is_builtin: true,
            defined_at: None,
            is_blind,
            copied_builtin: None,
        }
    }

    pub fn with_location(text: Vec<u8>, location: crate::token::SourceLocation) -> Self {
        Self {
            text,
            is_builtin: false,
            defined_at: Some(location),
            is_blind: false,
            copied_builtin: None,
        }
    }

    /// Create a definition that is a copy of a builtin via defn.
    pub fn builtin_copy(name: &str) -> Self {
        Self {
            text: Vec::new(),
            is_builtin: true,
            defined_at: None,
            is_blind: false,
            copied_builtin: Some(name.to_string()),
        }
    }
}

/// Macro definition stack for a single name.
///
/// `pushdef` pushes onto the stack, `popdef` pops off,
/// `define` replaces the top, `undefine` clears the stack.
#[derive(Debug, Clone)]
pub struct MacroStack {
    /// Stack of definitions, most recent first.
    pub defs: Vec<MacroDef>,
}

impl MacroStack {
    pub fn new(def: MacroDef) -> Self {
        Self { defs: vec![def] }
    }

    /// Get the current (topmost) definition.
    pub fn current(&self) -> Option<&MacroDef> {
        self.defs.last()
    }

    /// Push a new definition onto the stack.
    pub fn push(&mut self, def: MacroDef) {
        self.defs.push(def);
    }

    /// Pop the topmost definition.
    pub fn pop(&mut self) -> Option<MacroDef> {
        self.defs.pop()
    }

    /// Replace the topmost definition (like `define`).
    pub fn replace_top(&mut self, def: MacroDef) {
        if self.defs.is_empty() {
            self.defs.push(def);
        } else {
            *self.defs.last_mut().unwrap() = def;
        }
    }

    /// Clear all definitions for this name (like `undefine`).
    pub fn clear(&mut self) {
        self.defs.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }
}

/// The macro symbol table.
///
/// Maps macro names (as byte strings) to definition stacks.
/// Also tracks whether prefix mode (`-P`) is enabled.
#[derive(Debug, Clone)]
pub struct MacroTable {
    /// Macro definitions, keyed by name (as bytes).
    pub table: HashMap<Vec<u8>, MacroStack>,
    /// Whether `-P`/`--prefix-builtins` is enabled.
    pub prefix_builtins: bool,
}

impl MacroTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            prefix_builtins: false,
        }
    }

    /// Register a builtin macro.
    pub fn register_builtin(&mut self, name: &[u8], is_blind: bool) {
        let def = MacroDef::builtin(is_blind);
        self.table
            .entry(name.to_vec())
            .or_insert_with(|| MacroStack::new(def.clone()))
            .replace_top(def);
    }

    /// Define a user macro (like `define`).
    ///
    /// Uses `get_mut` first to avoid cloning the name for already-existing
    /// keys. In workloads with repeated definitions of the same name
    /// (e.g., 10k-defines benchmark), this avoids 9,999 unnecessary Vec<u8>
    /// allocations for the lookup key.
    pub fn define(&mut self, name: &[u8], text: &[u8]) {
        let def = MacroDef::new(text.to_vec());
        if let Some(stack) = self.table.get_mut(name) {
            stack.replace_top(def);
        } else {
            self.table.insert(name.to_vec(), MacroStack::new(def));
        }
    }

    /// Push a definition (like `pushdef`).
    pub fn pushdef(&mut self, name: &[u8], text: &[u8]) {
        let def = MacroDef::new(text.to_vec());
        if let Some(stack) = self.table.get_mut(name) {
            stack.push(def);
        } else {
            self.table.insert(name.to_vec(), MacroStack::new(def));
        }
    }

    /// Pop a definition (like `popdef`).
    pub fn popdef(&mut self, name: &[u8]) {
        if let Some(stack) = self.table.get_mut(name) {
            stack.pop();
            if stack.is_empty() {
                self.table.remove(name);
            }
        }
    }

    /// Remove all definitions for a name (like `undefine`).
    pub fn undefine(&mut self, name: &[u8]) {
        self.table.remove(name);
    }

    /// Look up the current definition for a name.
    ///
    /// Checks the user-defined table first. If the name has a
    /// definition, it's returned. Otherwise, if the name is a known
    /// builtin, the builtin definition is returned.
    ///
    /// In prefix mode (`-P`), only `m4_`-prefixed names are checked
    /// against builtins. Non-prefixed names only match user definitions.
    /// Prefixed names (e.g., `m4_define`) are looked up by stripping the
    /// prefix and checking the builtin table.
    pub fn lookup(&self, name: &[u8]) -> Option<&MacroDef> {
        if let Some(stack) = self.table.get(name) {
            return stack.current();
        }

        // In prefix mode, non-prefixed names don't match builtins
        if self.prefix_builtins {
            // Prefixed names: strip "m4_" and look up the bare builtin
            if name.starts_with(b"m4_") && name.len() > 3 {
                let bare = &name[3..];
                if let Some(stack) = self.table.get(bare) {
                    return stack.current();
                }
            }
            // Non-prefixed names (except m4exit, m4wrap): don't match builtins
            if !name.starts_with(b"m4_") && name != b"m4exit" && name != b"m4wrap" {
                return None;
            }
        }

        // Not in table — might be a builtin recognized by the engine.
        None
    }

    /// Check if a name is defined (has any entry in the table).
    pub fn is_defined(&self, name: &[u8]) -> bool {
        self.table.contains_key(name)
    }

    /// Get the definition stack depth for a name.
    pub fn stack_depth(&self, name: &[u8]) -> usize {
        self.table.get(name).map(|s| s.len()).unwrap_or(0)
    }

    /// Get a list of all defined names (as String for diagnostics).
    pub fn defined_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .table
            .keys()
            .map(|k| String::from_utf8_lossy(k).to_string())
            .collect();
        names.sort();
        names
    }
}

impl Default for MacroTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_lookup() {
        let mut table = MacroTable::new();
        table.define(b"foo", b"bar");
        let def = table.lookup(b"foo").unwrap();
        assert_eq!(def.text, b"bar");
        assert!(!def.is_builtin);
    }

    #[test]
    fn test_pushdef_popdef() {
        let mut table = MacroTable::new();
        table.pushdef(b"foo", b"first");
        table.pushdef(b"foo", b"second");
        assert_eq!(table.lookup(b"foo").unwrap().text, b"second");
        table.popdef(b"foo");
        assert_eq!(table.lookup(b"foo").unwrap().text, b"first");
        table.popdef(b"foo");
        assert!(table.lookup(b"foo").is_none());
    }

    #[test]
    fn test_undefine() {
        let mut table = MacroTable::new();
        table.define(b"foo", b"bar");
        assert!(table.is_defined(b"foo"));
        table.undefine(b"foo");
        assert!(!table.is_defined(b"foo"));
    }

    #[test]
    fn test_builtin_registration() {
        let mut table = MacroTable::new();
        table.register_builtin(b"dnl", true);
        let def = table.lookup(b"dnl").unwrap();
        assert!(def.is_builtin);
        assert!(def.is_blind);
    }
}
