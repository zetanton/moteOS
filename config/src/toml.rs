extern crate alloc;

use crate::error::ConfigError;
use alloc::{
    collections::BTreeMap,
    fmt,
    string::{String, ToString},
    vec::Vec,
};

/// TOML value representation
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Table(BTreeMap<String, Value>),
}

/// Minimal TOML parser (no_std compatible)
pub struct TomlParser;

impl TomlParser {
    /// Parse a TOML string into a Value
    pub fn parse(data: &str) -> Result<Value, ConfigError> {
        let mut parser = Parser::new(data);
        parser.parse()
    }

    /// Serialize a Value to TOML string
    pub fn serialize(value: &Value) -> Result<String, ConfigError> {
        let mut serializer = Serializer::new();
        serializer.serialize(value)?;
        Ok(serializer.output)
    }
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse(&mut self) -> Result<Value, ConfigError> {
        let mut root = BTreeMap::new();

        while !self.is_eof() {
            self.skip_whitespace();
            if self.is_eof() {
                break;
            }

            // Skip comments
            if self.peek() == Some('#') {
                self.skip_line();
                continue;
            }

            // Parse table header or key-value pair
            if self.peek() == Some('[') {
                // Table header - we'll handle nested tables by path
                let path = self.parse_table_header()?;
                self.skip_whitespace();

                // Parse key-value pairs until next table or EOF
                let mut table = BTreeMap::new();
                while !self.is_eof() && self.peek() != Some('[') {
                    self.skip_whitespace();
                    if self.is_eof() || self.peek() == Some('[') {
                        break;
                    }
                    if self.peek() == Some('#') {
                        self.skip_line();
                        continue;
                    }

                    let (key, value) = self.parse_key_value()?;
                    table.insert(key, value);
                    self.skip_whitespace();
                }

                // Insert nested table
                self.insert_nested(&mut root, &path, Value::Table(table))?;
            } else {
                // Root-level key-value pair
                let (key, value) = self.parse_key_value()?;
                root.insert(key, value);
            }
        }

        Ok(Value::Table(root))
    }

    fn parse_table_header(&mut self) -> Result<Vec<String>, ConfigError> {
        self.expect_char('[')?;

        let mut path = Vec::new();
        let mut current_key = String::new();

        loop {
            match self.peek() {
                Some(']') => {
                    if !current_key.is_empty() {
                        path.push(current_key.trim().to_string());
                    }
                    self.advance();
                    break;
                }
                Some('.') => {
                    if !current_key.is_empty() {
                        path.push(current_key.trim().to_string());
                        current_key.clear();
                    }
                    self.advance();
                }
                Some(ch) if ch.is_whitespace() => {
                    self.advance();
                }
                Some(ch) => {
                    current_key.push(ch);
                    self.advance();
                }
                None => return Err(ConfigError::parse_error("Unexpected EOF in table header")),
            }
        }

        if path.is_empty() {
            return Err(ConfigError::parse_error("Empty table header"));
        }

        Ok(path)
    }

    fn parse_key_value(&mut self) -> Result<(String, Value), ConfigError> {
        let key = self.parse_key()?;
        self.skip_whitespace();
        self.expect_char('=')?;
        self.skip_whitespace();
        let value = self.parse_value()?;
        Ok((key, value))
    }

    fn parse_key(&mut self) -> Result<String, ConfigError> {
        let mut key = String::new();

        // Keys can be bare (alphanumeric, underscore, dash) or quoted
        if self.peek() == Some('"') || self.peek() == Some('\'') {
            return self.parse_string();
        }

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                key.push(ch);
                self.advance();
            } else if ch.is_whitespace() || ch == '=' {
                break;
            } else {
                let msg = fmt::format(format_args!("Invalid character in key: {}", ch));
                return Err(ConfigError::parse_error(&msg));
            }
        }

        if key.is_empty() {
            return Err(ConfigError::parse_error("Empty key"));
        }

        Ok(key)
    }

    fn parse_value(&mut self) -> Result<Value, ConfigError> {
        self.skip_whitespace();

        match self.peek() {
            Some('"') | Some('\'') => {
                let s = self.parse_string()?;
                Ok(Value::String(s))
            }
            Some('[') => self.parse_array(),
            Some('{') => {
                // Inline table - not required for minimal parser, but we can support it
                self.parse_inline_table()
            }
            Some(ch) if ch.is_ascii_digit() || ch == '-' || ch == '+' => self.parse_number(),
            Some('t') | Some('f') => self.parse_boolean(),
            _ => {
                let msg = fmt::format(format_args!("Unexpected character: {:?}", self.peek()));
                Err(ConfigError::parse_error(&msg))
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, ConfigError> {
        let quote = self.peek().ok_or(ConfigError::UnexpectedEof)?;
        if quote != '"' && quote != '\'' {
            return Err(ConfigError::parse_error("Expected string quote"));
        }
        self.advance();

        let mut result = String::new();
        let mut escaped = false;

        loop {
            match self.peek() {
                Some(ch) if escaped => {
                    match ch {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        '\'' => result.push('\''),
                        _ => result.push(ch),
                    }
                    escaped = false;
                    self.advance();
                }
                Some('\\') => {
                    escaped = true;
                    self.advance();
                }
                Some(ch) if ch == quote => {
                    self.advance();
                    break;
                }
                Some(ch) => {
                    result.push(ch);
                    self.advance();
                }
                None => return Err(ConfigError::parse_error("Unterminated string")),
            }
        }

        Ok(result)
    }

    fn parse_number(&mut self) -> Result<Value, ConfigError> {
        let start = self.pos;
        let mut is_float = false;

        // Optional sign
        if self.peek() == Some('-') || self.peek() == Some('+') {
            self.advance();
        }

        // Integer part
        if !self.peek().map_or(false, |c| c.is_ascii_digit()) {
            return Err(ConfigError::parse_error("Invalid number"));
        }

        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.advance();
        }

        // Optional fractional part
        if self.peek() == Some('.') {
            is_float = true;
            self.advance();
            if !self.peek().map_or(false, |c| c.is_ascii_digit()) {
                return Err(ConfigError::parse_error("Invalid float"));
            }
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.advance();
            }
        }

        // Optional exponent
        if self.peek() == Some('e') || self.peek() == Some('E') {
            is_float = true;
            self.advance();
            if self.peek() == Some('-') || self.peek() == Some('+') {
                self.advance();
            }
            if !self.peek().map_or(false, |c| c.is_ascii_digit()) {
                return Err(ConfigError::parse_error("Invalid exponent"));
            }
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.advance();
            }
        }

        let num_str = &self.input[start..self.pos];

        if is_float {
            num_str
                .parse::<f64>()
                .map(Value::Float)
                .map_err(|_| ConfigError::InvalidNumber(num_str.to_string()))
        } else {
            num_str
                .parse::<i64>()
                .map(Value::Integer)
                .map_err(|_| ConfigError::InvalidNumber(num_str.to_string()))
        }
    }

    fn parse_boolean(&mut self) -> Result<Value, ConfigError> {
        if self.consume("true") {
            Ok(Value::Boolean(true))
        } else if self.consume("false") {
            Ok(Value::Boolean(false))
        } else {
            Err(ConfigError::parse_error("Expected boolean"))
        }
    }

    fn parse_array(&mut self) -> Result<Value, ConfigError> {
        self.expect_char('[')?;
        self.skip_whitespace();

        let mut array = Vec::new();

        // Empty array
        if self.peek() == Some(']') {
            self.advance();
            return Ok(Value::Array(array));
        }

        loop {
            self.skip_whitespace();

            // Skip comments in array
            if self.peek() == Some('#') {
                self.skip_line();
                self.skip_whitespace();
                if self.peek() == Some(']') {
                    break;
                }
                continue;
            }

            let value = self.parse_value()?;
            array.push(value);

            self.skip_whitespace();

            // Skip comments after value
            if self.peek() == Some('#') {
                self.skip_line();
                self.skip_whitespace();
            }

            match self.peek() {
                Some(',') => {
                    self.advance();
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.advance();
                    break;
                }
                _ => return Err(ConfigError::parse_error("Expected ',' or ']' in array")),
            }
        }

        Ok(Value::Array(array))
    }

    fn parse_inline_table(&mut self) -> Result<Value, ConfigError> {
        self.expect_char('{')?;
        self.skip_whitespace();

        let mut table = BTreeMap::new();

        // Empty table
        if self.peek() == Some('}') {
            self.advance();
            return Ok(Value::Table(table));
        }

        loop {
            self.skip_whitespace();
            let (key, value) = self.parse_key_value()?;
            table.insert(key, value);

            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.advance();
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(ConfigError::parse_error(
                        "Expected ',' or '}' in inline table",
                    ))
                }
            }
        }

        Ok(Value::Table(table))
    }

    fn insert_nested(
        &self,
        root: &mut BTreeMap<String, Value>,
        path: &[String],
        value: Value,
    ) -> Result<(), ConfigError> {
        if path.is_empty() {
            return Err(ConfigError::parse_error("Empty table path"));
        }

        if path.len() == 1 {
            root.insert(path[0].clone(), value);
            return Ok(());
        }

        // Navigate/create nested structure
        let mut current = root;
        for key in path.iter().take(path.len() - 1) {
            let needs_insert = !current.contains_key(key);
            if needs_insert {
                let new_table = BTreeMap::new();
                current.insert(key.clone(), Value::Table(new_table));
            }

            match current.get_mut(key) {
                Some(Value::Table(ref mut table)) => {
                    current = table;
                }
                Some(_) => {
                    let msg =
                        fmt::format(format_args!("Key '{}' already exists as non-table", key));
                    return Err(ConfigError::parse_error(&msg));
                }
                None => {
                    // This should never happen since we just inserted it
                    unreachable!();
                }
            }
        }

        // Insert at final level
        let final_key = &path[path.len() - 1];
        current.insert(final_key.clone(), value);

        Ok(())
    }

    // Utility methods
    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) {
        if let Some(ch) = self.peek() {
            self.pos += ch.len_utf8();
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn expect_char(&mut self, expected: char) -> Result<(), ConfigError> {
        match self.peek() {
            Some(ch) if ch == expected => {
                self.advance();
                Ok(())
            }
            Some(ch) => {
                let msg = fmt::format(format_args!("Expected '{}', found '{}'", expected, ch));
                Err(ConfigError::parse_error(&msg))
            }
            None => Err(ConfigError::UnexpectedEof),
        }
    }

    fn consume(&mut self, s: &str) -> bool {
        if self.input[self.pos..].starts_with(s) {
            // Check that it's not part of a larger identifier
            let end_pos = self.pos + s.len();
            if end_pos < self.input.len() {
                let next_char = self.input[end_pos..].chars().next();
                if let Some(ch) = next_char {
                    if ch.is_alphanumeric() || ch == '_' {
                        return false;
                    }
                }
            }
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                self.advance();
                break;
            }
            self.advance();
        }
    }
}

/// TOML serializer
struct Serializer {
    output: String,
    indent: usize,
}

impl Serializer {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    fn serialize(&mut self, value: &Value) -> Result<(), ConfigError> {
        match value {
            Value::Table(table) => {
                self.serialize_table(table, false)?;
            }
            _ => {
                return Err(ConfigError::parse_error("Root value must be a table"));
            }
        }
        Ok(())
    }

    fn serialize_table(
        &mut self,
        table: &BTreeMap<String, Value>,
        inline: bool,
    ) -> Result<(), ConfigError> {
        if inline {
            self.output.push('{');
        }

        let mut first = true;
        for (key, value) in table {
            if !first {
                if inline {
                    self.output.push_str(", ");
                } else {
                    self.output.push('\n');
                }
            }
            first = false;

            // Write key
            if self.needs_quotes(key) {
                self.serialize_string(key)?;
            } else {
                self.output.push_str(key);
            }

            self.output.push_str(" = ");
            self.serialize_value(value)?;
        }

        if inline {
            self.output.push('}');
        }

        Ok(())
    }

    fn serialize_value(&mut self, value: &Value) -> Result<(), ConfigError> {
        match value {
            Value::String(s) => {
                self.serialize_string(s)?;
            }
            Value::Integer(i) => {
                self.output.push_str(&i.to_string());
            }
            Value::Float(f) => {
                // Format float with proper precision
                let s = format!("{}", f);
                self.output.push_str(&s);
            }
            Value::Boolean(b) => {
                self.output.push_str(if *b { "true" } else { "false" });
            }
            Value::Array(arr) => {
                self.output.push('[');
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.serialize_value(item)?;
                }
                self.output.push(']');
            }
            Value::Table(table) => {
                // Inline table
                self.serialize_table(table, true)?;
            }
        }
        Ok(())
    }

    fn serialize_string(&mut self, s: &str) -> Result<(), ConfigError> {
        self.output.push('"');
        for ch in s.chars() {
            match ch {
                '"' => self.output.push_str("\\\""),
                '\\' => self.output.push_str("\\\\"),
                '\n' => self.output.push_str("\\n"),
                '\r' => self.output.push_str("\\r"),
                '\t' => self.output.push_str("\\t"),
                ch if ch.is_control() => {
                    // Escape other control characters
                    let code = ch as u32;
                    self.output.push_str(&format!("\\u{:04x}", code));
                }
                ch => self.output.push(ch),
            }
        }
        self.output.push('"');
        Ok(())
    }

    fn needs_quotes(&self, s: &str) -> bool {
        // Check if key needs quotes
        if s.is_empty() {
            return true;
        }
        for ch in s.chars() {
            if !(ch.is_alphanumeric() || ch == '_' || ch == '-') {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn test_simple_key_value() {
        let toml = r#"key = "value""#;
        let result = TomlParser::parse(toml).unwrap();
        if let Value::Table(map) = result {
            assert_eq!(map.get("key"), Some(&Value::String(String::from("value"))));
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_integer() {
        let toml = r#"number = 42"#;
        let result = TomlParser::parse(toml).unwrap();
        if let Value::Table(map) = result {
            assert_eq!(map.get("number"), Some(&Value::Integer(42)));
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_float() {
        let toml = r#"pi = 3.14"#;
        let result = TomlParser::parse(toml).unwrap();
        if let Value::Table(map) = result {
            if let Some(Value::Float(f)) = map.get("pi") {
                assert!((f - 3.14).abs() < 0.01);
            } else {
                panic!("Expected float");
            }
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_boolean() {
        let toml = r#"enabled = true
disabled = false"#;
        let result = TomlParser::parse(toml).unwrap();
        if let Value::Table(map) = result {
            assert_eq!(map.get("enabled"), Some(&Value::Boolean(true)));
            assert_eq!(map.get("disabled"), Some(&Value::Boolean(false)));
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_array() {
        let toml = r#"items = [1, 2, 3]"#;
        let result = TomlParser::parse(toml).unwrap();
        if let Value::Table(map) = result {
            if let Some(Value::Array(arr)) = map.get("items") {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Integer(1));
                assert_eq!(arr[1], Value::Integer(2));
                assert_eq!(arr[2], Value::Integer(3));
            } else {
                panic!("Expected array");
            }
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_nested_table() {
        let toml = r#"
[table]
key = "value"
"#;
        let result = TomlParser::parse(toml).unwrap();
        if let Value::Table(root) = result {
            if let Some(Value::Table(table)) = root.get("table") {
                assert_eq!(
                    table.get("key"),
                    Some(&Value::String(String::from("value")))
                );
            } else {
                panic!("Expected nested table");
            }
        } else {
            panic!("Expected root table");
        }
    }

    #[test]
    fn test_deeply_nested_table() {
        let toml = r#"
[table.subtable]
key = "value"
"#;
        let result = TomlParser::parse(toml).unwrap();
        if let Value::Table(root) = result {
            if let Some(Value::Table(table)) = root.get("table") {
                if let Some(Value::Table(subtable)) = table.get("subtable") {
                    assert_eq!(
                        subtable.get("key"),
                        Some(&Value::String(String::from("value")))
                    );
                } else {
                    panic!("Expected subtable");
                }
            } else {
                panic!("Expected table");
            }
        } else {
            panic!("Expected root table");
        }
    }

    #[test]
    fn test_string_escaping() {
        let toml = r#"message = "Hello\nWorld""#;
        let result = TomlParser::parse(toml).unwrap();
        if let Value::Table(map) = result {
            if let Some(Value::String(s)) = map.get("message") {
                assert_eq!(s, "Hello\nWorld");
            } else {
                panic!("Expected string");
            }
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_serialize_simple() {
        let mut map = BTreeMap::new();
        map.insert(String::from("key"), Value::String(String::from("value")));
        let value = Value::Table(map);
        let serialized = TomlParser::serialize(&value).unwrap();
        assert!(serialized.contains("key = \"value\""));
    }

    #[test]
    fn test_serialize_roundtrip() {
        let toml = r#"key = "value"
number = 42
enabled = true"#;
        let parsed = TomlParser::parse(toml).unwrap();
        let serialized = TomlParser::serialize(&parsed).unwrap();
        // Parse again to verify roundtrip
        let reparsed = TomlParser::parse(&serialized).unwrap();
        if let (Value::Table(orig), Value::Table(reparsed)) = (&parsed, &reparsed) {
            assert_eq!(orig.get("key"), reparsed.get("key"));
            assert_eq!(orig.get("number"), reparsed.get("number"));
            assert_eq!(orig.get("enabled"), reparsed.get("enabled"));
        } else {
            panic!("Expected tables");
        }
    }
}
