// m4-rs eval module — GNU m4-compatible arithmetic expression evaluator.
//
// WHO:   infinityabundance. Original eval engine by GNU m4 authors.
// WHAT:  A recursive-descent parser for C-like integer expressions matching
//        GNU m4's eval() builtin semantics. Supports all operators, radix,
//        and width control.
// WHEN:  Called by the expansion engine when eval/incr/decr builtins are invoked.
// WHERE: crates/m4-rs-core/src/eval.rs
// WHY:   eval is fundamental to ifelse comparisons, forloop counters, and
//        virtually all Autoconf macros. It's the #1 biggest mover in the
//        needle report at weight 8.
// HOW:   Recursive descent with operator precedence climbing. The grammar:
//
//        expr       → cond (',' cond)*
//        cond       → lor ('?' cond ':' cond)?
//        lor        → land ('||' land)*
//        land       → bwor ('&&' bwor)*
//        bwor       → bxor ('|' bxor)*
//        bxor       → band ('^' band)*
//        band       → eq ('&' eq)*
//        eq         → rel (('==' | '!=') rel)*
//        rel        → shift (('<' | '<=' | '>' | '>=') shift)*
//        shift      → add (('<<' | '>>') add)*
//        add        → mul (('+' | '-') mul)*
//        mul        → unary (('*' | '/' | '%') unary)*
//        unary      → ('+' | '-' | '~' | '!') unary | pow
//        pow        → atom ('**' | '^') pow | atom
//        atom       → NUMBER | '(' expr ')' | NAME
//
//        Numbers support decimal, octal (0 prefix), hex (0x prefix).
//        Result is a 32-bit signed integer (matching typical GNU m4).

/// Evaluate an m4 arithmetic expression and return the result as a 32-bit signed integer.
///
/// `expr`: the expression bytes
/// `radix`: output radix (2-36), default 10
/// `width`: if present, result is truncated to this many bits (1-32)
///
/// Returns the result as a string in the specified radix.
pub fn eval_expression(expr: &[u8], radix: u32, width: Option<u32>) -> Result<String, String> {
    let input = String::from_utf8_lossy(expr).to_string();
    let parser = ExprParser::new(&input);
    let result = parser.parse()?;

    // Apply width truncation
    let result = if let Some(w) = width {
        if w > 0 && w < 32 {
            let mask = (1i64 << w) - 1;
            // Sign-extend from width bits
            let sign_bit = 1i64 << (w - 1);
            let mut val = result & mask;
            if (val & sign_bit) != 0 {
                val |= !mask;
            }
            val
        } else {
            result
        }
    } else {
        result
    };

    // Format in specified radix
    let radix = if !(2..=36).contains(&radix) {
        10
    } else {
        radix
    };
    Ok(format_radix(result, radix))
}

struct ExprParser {
    input: Vec<char>,
    pos: usize,
}

impl ExprParser {
    fn new(s: &str) -> Self {
        Self {
            input: s.chars().collect(),
            pos: 0,
        }
    }

    fn parse(mut self) -> Result<i64, String> {
        let result = self.expr()?;
        self.skip_ws();
        if self.pos < self.input.len() {
            return Err(format!("unexpected character: '{}'", self.input[self.pos]));
        }
        Ok(result)
    }

    fn skip_ws(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    /// expr → cond (',' cond)* — returns the last cond value (GNU m4 comma operator)
    fn expr(&mut self) -> Result<i64, String> {
        self.skip_ws();
        let mut val = self.cond()?;
        self.skip_ws();
        while self.peek() == Some(',') {
            self.advance(); // consume ','
            val = self.cond()?;
            self.skip_ws();
        }
        Ok(val)
    }

    /// cond → lor ('?' cond ':' cond)?
    fn cond(&mut self) -> Result<i64, String> {
        let val = self.lor()?;
        self.skip_ws();
        if self.peek() == Some('?') {
            self.advance(); // '?'
            let then_val = self.cond()?;
            self.skip_ws();
            if self.peek() != Some(':') {
                return Err("expected ':' in ternary".to_string());
            }
            self.advance(); // ':'
            let else_val = self.cond()?;
            Ok(if val != 0 { then_val } else { else_val })
        } else {
            Ok(val)
        }
    }

    /// lor → land ('||' land)*
    fn lor(&mut self) -> Result<i64, String> {
        let mut val = self.land()?;
        self.skip_ws();
        while self.pos + 1 < self.input.len()
            && self.input[self.pos] == '|'
            && self.input[self.pos + 1] == '|'
        {
            self.pos += 2;
            let rhs = self.land()?;
            val = if val != 0 || rhs != 0 { 1 } else { 0 };
            self.skip_ws();
        }
        Ok(val)
    }

    /// land → bwor ('&&' bwor)*
    fn land(&mut self) -> Result<i64, String> {
        let mut val = self.bwor()?;
        self.skip_ws();
        while self.pos + 1 < self.input.len()
            && self.input[self.pos] == '&'
            && self.input[self.pos + 1] == '&'
        {
            self.pos += 2;
            let rhs = self.bwor()?;
            val = if val != 0 && rhs != 0 { 1 } else { 0 };
            self.skip_ws();
        }
        Ok(val)
    }

    /// bwor → bxor ('|' bxor)*
    fn bwor(&mut self) -> Result<i64, String> {
        let mut val = self.bxor()?;
        self.skip_ws();
        while self.peek() == Some('|') && !self.is_double('|') {
            self.advance();
            val |= self.bxor()?;
            self.skip_ws();
        }
        Ok(val)
    }

    /// bxor → band ('^' band)*
    fn bxor(&mut self) -> Result<i64, String> {
        let mut val = self.band()?;
        self.skip_ws();
        while self.peek() == Some('^') {
            self.advance();
            val ^= self.band()?;
            self.skip_ws();
        }
        Ok(val)
    }

    /// band → eq ('&' eq)*
    fn band(&mut self) -> Result<i64, String> {
        let mut val = self.eq()?;
        self.skip_ws();
        while self.peek() == Some('&') && !self.is_double('&') {
            self.advance();
            val &= self.eq()?;
            self.skip_ws();
        }
        Ok(val)
    }

    /// eq → rel (('==' | '!=') rel)*
    fn eq(&mut self) -> Result<i64, String> {
        let mut val = self.rel()?;
        self.skip_ws();
        loop {
            if self.pos + 1 < self.input.len()
                && self.input[self.pos] == '='
                && self.input[self.pos + 1] == '='
            {
                self.pos += 2;
                let rhs = self.rel()?;
                val = if val == rhs { 1 } else { 0 };
                self.skip_ws();
            } else if self.pos + 1 < self.input.len()
                && self.input[self.pos] == '!'
                && self.input[self.pos + 1] == '='
            {
                self.pos += 2;
                let rhs = self.rel()?;
                val = if val != rhs { 1 } else { 0 };
                self.skip_ws();
            } else {
                break;
            }
        }
        Ok(val)
    }

    /// rel → shift (('<' | '<=' | '>' | '>=') shift)*
    fn rel(&mut self) -> Result<i64, String> {
        let mut val = self.shift()?;
        self.skip_ws();
        loop {
            if self.pos + 1 < self.input.len()
                && self.input[self.pos] == '<'
                && self.input[self.pos + 1] == '='
            {
                self.pos += 2;
                let rhs = self.shift()?;
                val = if val <= rhs { 1 } else { 0 };
                self.skip_ws();
            } else if self.pos + 1 < self.input.len()
                && self.input[self.pos] == '>'
                && self.input[self.pos + 1] == '='
            {
                self.pos += 2;
                let rhs = self.shift()?;
                val = if val >= rhs { 1 } else { 0 };
                self.skip_ws();
            } else if self.peek() == Some('<') {
                self.advance();
                let rhs = self.shift()?;
                val = if val < rhs { 1 } else { 0 };
                self.skip_ws();
            } else if self.peek() == Some('>') {
                self.advance();
                let rhs = self.shift()?;
                val = if val > rhs { 1 } else { 0 };
                self.skip_ws();
            } else {
                break;
            }
        }
        Ok(val)
    }

    /// shift → add (('<<' | '>>') add)*
    fn shift(&mut self) -> Result<i64, String> {
        let mut val = self.add()?;
        self.skip_ws();
        while self.pos + 1 < self.input.len()
            && (self.input[self.pos] == '<' || self.input[self.pos] == '>')
            && self.input[self.pos + 1] == self.input[self.pos]
        {
            let is_left = self.input[self.pos] == '<';
            self.pos += 2;
            let rhs = self.add()?;
            val = if is_left {
                (val as i32).wrapping_shl(rhs as u32) as i64
            } else {
                (val as i32).wrapping_shr(rhs as u32) as i64
            };
            self.skip_ws();
        }
        Ok(val)
    }

    /// add → mul (('+' | '-') mul)*
    fn add(&mut self) -> Result<i64, String> {
        let mut val = self.mul()?;
        self.skip_ws();
        loop {
            if self.peek() == Some('+') {
                self.advance();
                val = (val as i32).wrapping_add(self.mul()? as i32) as i64;
                self.skip_ws();
            } else if self.peek() == Some('-') {
                self.advance();
                val = (val as i32).wrapping_sub(self.mul()? as i32) as i64;
                self.skip_ws();
            } else {
                break;
            }
        }
        Ok(val)
    }

    /// mul → unary (('*' | '/' | '%') unary)*
    fn mul(&mut self) -> Result<i64, String> {
        let mut val = self.unary()?;
        self.skip_ws();
        loop {
            if self.peek() == Some('*') && !self.is_pow() {
                self.advance();
                val = (val as i32).wrapping_mul(self.unary()? as i32) as i64;
                self.skip_ws();
            } else if self.peek() == Some('/') {
                self.advance();
                let rhs = self.unary()?;
                if rhs == 0 {
                    return Err("division by zero".to_string());
                }
                val = (val as i32).wrapping_div(rhs as i32) as i64;
                self.skip_ws();
            } else if self.peek() == Some('%') {
                self.advance();
                let rhs = self.unary()?;
                if rhs == 0 {
                    return Err("modulo by zero".to_string());
                }
                val = (val as i32).wrapping_rem(rhs as i32) as i64;
                self.skip_ws();
            } else {
                break;
            }
        }
        Ok(val)
    }

    /// unary → ('+' | '-' | '~' | '!') unary | pow
    fn unary(&mut self) -> Result<i64, String> {
        self.skip_ws();
        if self.peek() == Some('+') {
            self.advance();
            return self.unary();
        }
        if self.peek() == Some('-') {
            self.advance();
            return Ok(-self.unary()?);
        }
        if self.peek() == Some('~') {
            self.advance();
            return Ok(!self.unary()?);
        }
        if self.peek() == Some('!') {
            self.advance();
            let v = self.unary()?;
            return Ok(if v == 0 { 1 } else { 0 });
        }
        self.pow()
    }

    /// pow → atom ('**' | '^') pow | atom
    fn pow(&mut self) -> Result<i64, String> {
        let val = self.atom()?;
        self.skip_ws();
        if self.is_pow() {
            let is_starstar = true;
            if is_starstar {
                self.pos += 2; // skip '**'
            } else {
                self.advance(); // skip '^'
            }
            let rhs = self.pow()?;
            if rhs < 0 {
                // Negative exponent → 0 in integer arithmetic (GNU m4 behavior)
                return Ok(0);
            }
            let mut result: i64 = 1;
            for _ in 0..rhs {
                result = (result as i32).wrapping_mul(val as i32) as i64;
            }
            return Ok(result);
        }
        Ok(val)
    }

    /// atom → NUMBER | '(' expr ')' | NAME (treated as 0 for undefined)
    fn atom(&mut self) -> Result<i64, String> {
        self.skip_ws();
        if self.peek() == Some('(') {
            self.advance();
            let val = self.expr()?;
            self.skip_ws();
            if self.peek() != Some(')') {
                return Err("expected ')'".to_string());
            }
            self.advance();
            return Ok(val);
        }
        if let Some(c) = self.peek() {
            if c.is_ascii_digit()
                || (c == '0' && self.pos + 1 < self.input.len() && self.input[self.pos + 1] == 'x')
            {
                return self.parse_number();
            }
            if c.is_alphabetic() || c == '_' {
                // Name — in GNU m4, undefined names in eval are treated as 0
                self.advance();
                while self.pos < self.input.len()
                    && (self.input[self.pos].is_alphanumeric() || self.input[self.pos] == '_')
                {
                    self.pos += 1;
                }
                return Ok(0);
            }
        }
        Err(format!("unexpected character: {:?}", self.peek()))
    }

    fn parse_number(&mut self) -> Result<i64, String> {
        // Check for 0x hex prefix
        if self.pos + 1 < self.input.len()
            && self.input[self.pos] == '0'
            && (self.input[self.pos + 1] == 'x' || self.input[self.pos + 1] == 'X')
        {
            self.pos += 2; // skip 0x
            let start = self.pos;
            while self.pos < self.input.len() && self.input[self.pos].is_ascii_hexdigit() {
                self.pos += 1;
            }
            let hex_str: String = self.input[start..self.pos].iter().collect();
            return i64::from_str_radix(&hex_str, 16).map_err(|e| format!("bad hex: {}", e));
        }

        // Check for octal (leading 0)
        let is_octal = self.input[self.pos] == '0'
            && self.pos + 1 < self.input.len()
            && self.input[self.pos + 1].is_ascii_digit();

        let radix = if is_octal { 8 } else { 10 };
        let start = if is_octal { self.pos + 1 } else { self.pos };

        if is_octal {
            self.pos += 1; // skip leading 0
        }

        while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        let num_str: String = self.input[start..self.pos].iter().collect();
        if num_str.is_empty() && is_octal {
            return Ok(0); // just "0"
        }
        i64::from_str_radix(&num_str, radix).map_err(|e| format!("bad number: {}", e))
    }

    fn is_double(&self, c: char) -> bool {
        self.pos + 1 < self.input.len()
            && self.input[self.pos] == c
            && self.input[self.pos + 1] == c
    }

    fn is_pow(&self) -> bool {
        self.is_double('*')
    }
}

/// Format a number in the given radix (2-36).
fn format_radix(n: i64, radix: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let digits = "0123456789abcdefghijklmnopqrstuvwxyz";
    let mut result = String::new();
    let is_neg = n < 0;
    let mut abs = if is_neg {
        (n as i32).wrapping_neg() as u32 as u64
    } else {
        n as u64
    };

    while abs > 0 {
        let digit = (abs % radix as u64) as usize;
        result.push(digits.chars().nth(digit).unwrap_or('?'));
        abs /= radix as u64;
    }

    if is_neg {
        result.push('-');
    }

    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_arithmetic() {
        assert_eq!(eval_expression(b"1 + 2", 10, None).unwrap(), "3");
        assert_eq!(eval_expression(b"10 - 3", 10, None).unwrap(), "7");
        assert_eq!(eval_expression(b"4 * 5", 10, None).unwrap(), "20");
        assert_eq!(eval_expression(b"20 / 4", 10, None).unwrap(), "5");
        assert_eq!(eval_expression(b"10 % 3", 10, None).unwrap(), "1");
    }

    #[test]
    fn test_bitwise() {
        assert_eq!(eval_expression(b"5 & 3", 10, None).unwrap(), "1");
        assert_eq!(eval_expression(b"5 | 3", 10, None).unwrap(), "7");
        assert_eq!(eval_expression(b"5 ^ 3", 10, None).unwrap(), "6");
        assert_eq!(eval_expression(b"~0", 10, None).unwrap(), "-1");
        assert_eq!(eval_expression(b"1 << 3", 10, None).unwrap(), "8");
        assert_eq!(eval_expression(b"8 >> 2", 10, None).unwrap(), "2");
    }

    #[test]
    fn test_comparison() {
        assert_eq!(eval_expression(b"5 == 5", 10, None).unwrap(), "1");
        assert_eq!(eval_expression(b"5 != 3", 10, None).unwrap(), "1");
        assert_eq!(eval_expression(b"5 > 3", 10, None).unwrap(), "1");
        assert_eq!(eval_expression(b"3 < 5", 10, None).unwrap(), "1");
        assert_eq!(eval_expression(b"5 <= 5", 10, None).unwrap(), "1");
        assert_eq!(eval_expression(b"5 >= 5", 10, None).unwrap(), "1");
    }

    #[test]
    fn test_logical() {
        assert_eq!(eval_expression(b"1 && 1", 10, None).unwrap(), "1");
        assert_eq!(eval_expression(b"1 && 0", 10, None).unwrap(), "0");
        assert_eq!(eval_expression(b"1 || 0", 10, None).unwrap(), "1");
        assert_eq!(eval_expression(b"0 || 0", 10, None).unwrap(), "0");
    }

    #[test]
    fn test_ternary() {
        assert_eq!(eval_expression(b"1 ? 10 : 20", 10, None).unwrap(), "10");
        assert_eq!(eval_expression(b"0 ? 10 : 20", 10, None).unwrap(), "20");
    }

    #[test]
    fn test_power() {
        assert_eq!(eval_expression(b"2 ** 3", 10, None).unwrap(), "8");
        assert_eq!(eval_expression(b"2 ** 4", 10, None).unwrap(), "16");
    }

    #[test]
    fn test_precedence() {
        assert_eq!(eval_expression(b"2 + 3 * 4", 10, None).unwrap(), "14");
        assert_eq!(eval_expression(b"(2 + 3) * 4", 10, None).unwrap(), "20");
    }

    #[test]
    fn test_hex() {
        assert_eq!(eval_expression(b"0xff", 10, None).unwrap(), "255");
        assert_eq!(eval_expression(b"0x10", 10, None).unwrap(), "16");
    }

    #[test]
    fn test_octal() {
        assert_eq!(eval_expression(b"010", 10, None).unwrap(), "8");
    }

    #[test]
    fn test_radix_output() {
        assert_eq!(eval_expression(b"255", 16, None).unwrap(), "ff");
        assert_eq!(eval_expression(b"8", 2, None).unwrap(), "1000");
    }

    #[test]
    fn test_negative() {
        assert_eq!(eval_expression(b"-5", 10, None).unwrap(), "-5");
        assert_eq!(eval_expression(b"-5 + 3", 10, None).unwrap(), "-2");
    }

    #[test]
    fn test_division_by_zero() {
        assert!(eval_expression(b"1 / 0", 10, None).is_err());
    }
}
