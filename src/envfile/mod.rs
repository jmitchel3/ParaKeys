//! Placeholder `.env` read/write helpers.

use std::fs;
use std::path::Path;

use crate::error::ParaKeysError;

pub const PLACEHOLDER_SET: &str = "<set in parakeys>";
pub const PLACEHOLDER_NOT_SET: &str = "<not set in parakeys>";

/// One logical line in a dotenv-style file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvLine {
    /// Blank or comment (including empty lines).
    Raw(String),
    /// `KEY=value` assignment (value may be empty).
    Assignment { key: String, value: String, export: bool },
}

/// Parsed `.env` document preserving order and comments.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvFile {
    pub lines: Vec<EnvLine>,
}

impl EnvFile {
    pub fn parse(text: &str) -> Self {
        let mut lines = Vec::new();
        for line in text.lines() {
            lines.push(parse_line(line));
        }
        // Preserve trailing newline semantics loosely: if original ended with newline,
        // writers always end with newline when non-empty.
        Self { lines }
    }

    pub fn assignments(&self) -> impl Iterator<Item = (&str, &str)> {
        self.lines.iter().filter_map(|l| match l {
            EnvLine::Assignment { key, value, .. } => Some((key.as_str(), value.as_str())),
            EnvLine::Raw(_) => None,
        })
    }

    /// Collect KEY -> value for non-placeholder assignments (import candidates).
    pub fn secret_candidates(&self) -> Vec<(String, String)> {
        self.assignments()
            .filter(|(_, v)| !is_placeholder(v))
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// Rewrite assignment values that were imported to the set placeholder.
    /// Keys listed in `imported` become `<set in parakeys>`.
    pub fn rewrite_placeholders(&mut self, imported: &[String]) {
        for line in &mut self.lines {
            if let EnvLine::Assignment { key, value, .. } = line {
                if imported.iter().any(|k| k == key) {
                    *value = PLACEHOLDER_SET.to_string();
                }
            }
        }
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            match line {
                EnvLine::Raw(s) => out.push_str(s),
                EnvLine::Assignment { key, value, export } => {
                    if *export {
                        out.push_str("export ");
                    }
                    out.push_str(key);
                    out.push('=');
                    out.push_str(&format_value(value));
                }
            }
        }
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out
    }
}

pub fn is_placeholder(value: &str) -> bool {
    let v = value.trim();
    v == PLACEHOLDER_SET || v == PLACEHOLDER_NOT_SET
}

pub fn load_env_file(path: &Path) -> Result<EnvFile, ParaKeysError> {
    let text = fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ParaKeysError::InvalidVault(format!("env file not found: {}", path.display()))
        } else {
            ParaKeysError::Io(e)
        }
    })?;
    Ok(EnvFile::parse(&text))
}

pub fn save_env_file(path: &Path, file: &EnvFile) -> Result<(), ParaKeysError> {
    fs::write(path, file.render()).map_err(ParaKeysError::Io)
}

fn parse_line(line: &str) -> EnvLine {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return EnvLine::Raw(line.to_string());
    }

    let (export, rest) = if let Some(rest) = trimmed.strip_prefix("export ") {
        (true, rest.trim_start())
    } else {
        (false, trimmed)
    };

    let Some((key, value_part)) = rest.split_once('=') else {
        return EnvLine::Raw(line.to_string());
    };
    let key = key.trim();
    if key.is_empty() || key.contains(char::is_whitespace) {
        return EnvLine::Raw(line.to_string());
    }
    let value = unquote(value_part.trim());
    EnvLine::Assignment {
        key: key.to_string(),
        value,
        export,
    }
}

fn unquote(raw: &str) -> String {
    let b = raw.as_bytes();
    if b.len() >= 2 {
        let first = b[0];
        let last = b[b.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return raw[1..raw.len() - 1].to_string();
        }
    }
    // Strip inline comment for unquoted values: KEY=value # comment
    if let Some((v, _)) = raw.split_once(" #") {
        return v.trim_end().to_string();
    }
    raw.to_string()
}

fn format_value(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    if is_placeholder(value) {
        return value.to_string();
    }
    if value.chars().any(|c| c.is_whitespace() || matches!(c, '#' | '"' | '\'')) {
        let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
        return format!("\"{escaped}\"");
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_rewrite_placeholders() {
        let text = "\
# demo
DATABASE_URL=postgres://secret
OPENAI_API_KEY=sk-test
DEBUG=true
";
        let mut file = EnvFile::parse(text);
        let keys: Vec<_> = file
            .secret_candidates()
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert_eq!(keys.len(), 3);
        file.rewrite_placeholders(&keys);
        let out = file.render();
        assert!(out.contains(&format!("DATABASE_URL={PLACEHOLDER_SET}")));
        assert!(out.contains(&format!("OPENAI_API_KEY={PLACEHOLDER_SET}")));
        assert!(out.contains(&format!("DEBUG={PLACEHOLDER_SET}")));
        assert!(out.contains("# demo"));
        assert!(!out.contains("postgres://secret"));
        assert!(!out.contains("sk-test"));
    }

    #[test]
    fn skip_existing_placeholders_on_import_candidates() {
        let text = format!("A={PLACEHOLDER_SET}\nB=real\n");
        let file = EnvFile::parse(&text);
        let c = file.secret_candidates();
        assert_eq!(c, vec![("B".into(), "real".into())]);
    }

    #[test]
    fn export_and_quotes() {
        let file = EnvFile::parse("export FOO=\"bar baz\"\n");
        match &file.lines[0] {
            EnvLine::Assignment { key, value, export } => {
                assert_eq!(key, "FOO");
                assert_eq!(value, "bar baz");
                assert!(*export);
            }
            _ => panic!("expected assignment"),
        }
    }
}
