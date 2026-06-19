// m4-rs input handling.
//
// GNU m4 reads input from files or stdin, respecting option ordering.
// The input system uses an input stack: when a macro expands and produces
// new text, that text is pushed back onto the input and rescanned.
//
// Key behaviors:
//
// 1. **File ordering**: `m4 file1 file2` concatenates file1 + file2.
//    Each file is processed in sequence, with EOF triggering undivert
//    and then the next file.
//
// 2. **Stdin**: `m4` with no files reads stdin. `m4 - file` reads stdin
//    then file. Multiple `-` each read stdin once (GNU m4 reads stdin
//    each time `-` appears).
//
// 3. **Option ordering**: GNU m4 processes options in order.
//    `m4 -Dfoo=bar file` defines `foo` then processes `file`.
//    `m4 file -Dfoo=bar` processes `file` then defines `foo`.
//    (This matters for `-D`/`-U`/`-I` relative to file arguments.)
//
// 4. **Include search path**: `include(file)` searches:
//    1. Current working directory
//    2. Directories specified by `-I` flags (in order)
//    3. The directory of the file that did the include (GNU extension)
//
// 5. **Synclines**: With `-s`, GNU m4 emits `#line` directives to help
//    C preprocessors track original source locations.
//
// Reference: GNU M4 manual, Sections 2.1–2.2, 7.1 (include)

use std::io::{self, Read};
use std::path::PathBuf;

/// A source of input bytes for the m4 processor.
///
/// Implementations include:
/// - File input
/// - Stdin
/// - String input (for tests)
/// - Macro expansion rescan input
pub trait InputProvider {
    /// Read the next chunk of input.
    /// Returns None when input is exhausted.
    fn read(&mut self) -> io::Result<Option<Vec<u8>>>;

    /// Get the name of this input source for diagnostics.
    fn source_name(&self) -> &str;

    /// Get the current line number within this source.
    fn line_number(&self) -> usize;

    /// True if this is stdin (affects some behaviors).
    fn is_stdin(&self) -> bool;
}

/// Input from a string (for testing and macro expansion rescan).
pub struct StringInput {
    pub data: Vec<u8>,
    pub position: usize,
    pub name: String,
    pub line: usize,
}

impl StringInput {
    pub fn new(data: &[u8], name: &str) -> Self {
        Self {
            data: data.to_vec(),
            position: 0,
            name: name.to_string(),
            line: 1,
        }
    }

    pub fn from_str(data: &str, name: &str) -> Self {
        Self::new(data.as_bytes(), name)
    }
}

impl InputProvider for StringInput {
    fn read(&mut self) -> io::Result<Option<Vec<u8>>> {
        if self.position >= self.data.len() {
            return Ok(None);
        }
        // Return remaining data as one chunk
        let chunk = self.data[self.position..].to_vec();
        self.position = self.data.len();
        Ok(Some(chunk))
    }

    fn source_name(&self) -> &str {
        &self.name
    }

    fn line_number(&self) -> usize {
        self.line
    }

    fn is_stdin(&self) -> bool {
        self.name == "<stdin>"
    }
}

/// Input from a file.
pub struct FileInput {
    pub path: PathBuf,
    pub name: String,
    pub line: usize,
    remaining: Option<Vec<u8>>,
}

impl FileInput {
    pub fn open(path: PathBuf) -> io::Result<Self> {
        let name = path.to_string_lossy().to_string();
        let mut file = std::fs::File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(Self {
            path,
            name,
            line: 1,
            remaining: Some(data),
        })
    }
}

impl InputProvider for FileInput {
    fn read(&mut self) -> io::Result<Option<Vec<u8>>> {
        Ok(self.remaining.take())
    }

    fn source_name(&self) -> &str {
        &self.name
    }

    fn line_number(&self) -> usize {
        self.line
    }

    fn is_stdin(&self) -> bool {
        false
    }
}

/// Input from stdin.
pub struct StdinInput {
    pub line: usize,
    data: Option<Vec<u8>>,
}

impl StdinInput {
    pub fn new() -> io::Result<Self> {
        let mut data = Vec::new();
        io::stdin().lock().read_to_end(&mut data)?;
        Ok(Self {
            line: 1,
            data: Some(data),
        })
    }
}

impl InputProvider for StdinInput {
    fn read(&mut self) -> io::Result<Option<Vec<u8>>> {
        Ok(self.data.take())
    }

    fn source_name(&self) -> &str {
        "<stdin>"
    }

    fn line_number(&self) -> usize {
        self.line
    }

    fn is_stdin(&self) -> bool {
        true
    }
}

/// The input stack for the m4 processor.
///
/// Input sources are pushed onto a stack. When the current source is
/// exhausted, we pop and continue with the next source. This handles:
/// - Multiple input files
/// - `include` pushing new files
/// - Macro expansion pushing rescan text
/// - Diversion undivert pushing saved text
pub struct InputStack {
    pub stack: Vec<Box<dyn InputProvider>>,
}

impl InputStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Push an input source onto the stack.
    pub fn push(&mut self, source: Box<dyn InputProvider>) {
        self.stack.push(source);
    }

    /// Pop the current input source.
    pub fn pop(&mut self) -> Option<Box<dyn InputProvider>> {
        self.stack.pop()
    }

    /// Check if the input stack is empty.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Read from the current input source. If exhausted, pop and try the next.
    pub fn read(&mut self) -> io::Result<Option<Vec<u8>>> {
        loop {
            let top = match self.stack.last_mut() {
                Some(s) => s,
                None => return Ok(None),
            };
            match top.read()? {
                Some(data) => return Ok(Some(data)),
                None => {
                    self.stack.pop();
                    // Continue loop to try next source
                }
            }
        }
    }

    /// Get the name of the current input source for diagnostics.
    pub fn current_source(&self) -> &str {
        self.stack
            .last()
            .map(|s| s.source_name())
            .unwrap_or("<unknown>")
    }

    /// Get the current line number for diagnostics.
    pub fn current_line(&self) -> usize {
        self.stack.last().map(|s| s.line_number()).unwrap_or(0)
    }

    /// True if the current source is stdin.
    pub fn is_stdin(&self) -> bool {
        self.stack.last().map(|s| s.is_stdin()).unwrap_or(false)
    }
}

impl Default for InputStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_input() {
        let mut input = StringInput::from_str("hello world", "test");
        let data = input.read().unwrap().unwrap();
        assert_eq!(data, b"hello world");
        assert!(input.read().unwrap().is_none());
    }

    #[test]
    fn test_input_stack() {
        let mut stack = InputStack::new();
        stack.push(Box::new(StringInput::from_str("first", "a")));
        stack.push(Box::new(StringInput::from_str("second", "b")));

        // Should read from top of stack first
        let data = stack.read().unwrap().unwrap();
        assert_eq!(data, b"second");

        // Then from next source
        let data = stack.read().unwrap().unwrap();
        assert_eq!(data, b"first");

        // Then empty
        assert!(stack.read().unwrap().is_none());
    }
}
