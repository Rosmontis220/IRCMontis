//! IRC wire protocol parsing (RFC 1459 + IRCv3 message tags). Pure functions.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prefix {
    pub raw: String,
}

impl Prefix {
    /// Nick part of `nick!user@host`, or the whole raw value for server prefixes.
    pub fn nick(&self) -> &str {
        self.raw.split('!').next().unwrap_or(&self.raw)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub tags: Vec<(String, String)>,
    pub prefix: Option<Prefix>,
    pub command: String,
    pub params: Vec<String>,
}

impl Message {
    pub fn tag(&self, key: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn trailing(&self) -> Option<&str> {
        self.params.last().map(|s| s.as_str())
    }
}

fn unescape_tag(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some(':') => out.push(';'),
            Some('s') => out.push(' '),
            Some('\\') => out.push('\\'),
            Some('r') => out.push('\r'),
            Some('n') => out.push('\n'),
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

/// Parse one IRC line (without CRLF). Returns None for empty/blank lines.
pub fn parse(line: &str) -> Option<Message> {
    let line = line.trim_end_matches(['\r', '\n']);
    if line.is_empty() {
        return None;
    }
    let mut rest = line;
    let mut tags = Vec::new();
    if let Some(after_at) = rest.strip_prefix('@') {
        let (tag_part, remainder) = split_once_space(after_at);
        for item in tag_part.split(';') {
            if item.is_empty() {
                continue;
            }
            let (key, value) = match item.split_once('=') {
                Some((k, v)) => (k, unescape_tag(v)),
                None => (item, String::new()),
            };
            tags.push((key.to_string(), value));
        }
        rest = remainder;
    }
    let mut prefix = None;
    if let Some(after_colon) = rest.strip_prefix(':') {
        let (prefix_part, remainder) = split_once_space(after_colon);
        prefix = Some(Prefix {
            raw: prefix_part.to_string(),
        });
        rest = remainder;
    }
    if rest.is_empty() {
        return None;
    }
    let (command, mut remainder) = split_once_space(rest);
    if command.is_empty() {
        return None;
    }
    let mut params = Vec::new();
    while !remainder.is_empty() {
        if let Some(trailing) = remainder.strip_prefix(':') {
            params.push(trailing.to_string());
            break;
        }
        let (param, next) = split_once_space(remainder);
        params.push(param.to_string());
        remainder = next;
    }
    Some(Message {
        tags,
        prefix,
        command: command.to_ascii_uppercase(),
        params,
    })
}

/// Split at the first run of spaces; returns (token, remainder with leading spaces skipped).
fn split_once_space(s: &str) -> (&str, &str) {
    match s.find(' ') {
        Some(idx) => (&s[..idx], s[idx..].trim_start_matches(' ')),
        None => (s, ""),
    }
}

/// Split text into chunks of at most `max_bytes` bytes, never cutting a UTF-8 character.
pub fn split_message(text: &str, max_bytes: usize) -> Vec<String> {
    assert!(max_bytes > 0);
    let mut chunks = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if current.len() + ch.len_utf8() > max_bytes && !current.is_empty() {
            chunks.push(std::mem::take(&mut current));
        }
        current.push(ch);
    }
    if !current.is_empty() || chunks.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Extract CTCP ACTION content (`\x01ACTION text\x01`), if present.
pub fn ctcp_action(text: &str) -> Option<&str> {
    let inner = text.strip_prefix('\x01')?.strip_suffix('\x01')?;
    inner.strip_prefix("ACTION ")
}

/// Strip mIRC formatting control codes and CTCP wrappers, for previews/search.
pub fn plain_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\x02' | '\x1d' | '\x1f' | '\x1e' | '\x11' | '\x16' | '\x0f' | '\x01' => {}
            '\x03' => {
                // color: up to 2 digits, optional comma + up to 2 digits
                let mut digits = 0;
                while digits < 2 && chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                    chars.next();
                    digits += 1;
                }
                if digits > 0 && chars.peek() == Some(&',') {
                    chars.next();
                    let mut bg = 0;
                    while bg < 2 && chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                        chars.next();
                        bg += 1;
                    }
                }
            }
            '\x04' => {
                // hex color: up to 6 hex digits, optional comma + 6
                let mut digits = 0;
                while digits < 6 && chars.peek().is_some_and(|c| c.is_ascii_hexdigit()) {
                    chars.next();
                    digits += 1;
                }
                if digits > 0 && chars.peek() == Some(&',') {
                    chars.next();
                    let mut bg = 0;
                    while bg < 6 && chars.peek().is_some_and(|c| c.is_ascii_hexdigit()) {
                        chars.next();
                        bg += 1;
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_privmsg_with_prefix() {
        let m = parse(":nick!user@host PRIVMSG #channel :hello world").unwrap();
        assert_eq!(m.command, "PRIVMSG");
        assert_eq!(m.prefix.as_ref().unwrap().nick(), "nick");
        assert_eq!(m.params, vec!["#channel", "hello world"]);
    }

    #[test]
    fn parses_ping_without_prefix() {
        let m = parse("PING :12345").unwrap();
        assert_eq!(m.command, "PING");
        assert!(m.prefix.is_none());
        assert_eq!(m.trailing(), Some("12345"));
    }

    #[test]
    fn parses_numeric() {
        let m = parse(":cadmium.libera.chat 001 mynick :Welcome to Libera").unwrap();
        assert_eq!(m.command, "001");
        assert_eq!(m.params[0], "mynick");
        assert_eq!(m.trailing(), Some("Welcome to Libera"));
    }

    #[test]
    fn parses_tags_with_escapes() {
        let m = parse("@time=2020-01-01T00:00:00.000Z;id=a\\sb\\:c :n PRIVMSG #c :hi").unwrap();
        assert_eq!(m.tag("time"), Some("2020-01-01T00:00:00.000Z"));
        assert_eq!(m.tag("id"), Some("a b;c"));
        assert_eq!(m.params, vec!["#c", "hi"]);
    }

    #[test]
    fn parses_valueless_tag_and_empty_trailing() {
        let m = parse("@draft :x PRIVMSG #c :").unwrap();
        assert_eq!(m.tag("draft"), Some(""));
        assert_eq!(m.trailing(), Some(""));
    }

    #[test]
    fn parses_cap_ls() {
        let m = parse("CAP * LS :multi-prefix sasl").unwrap();
        assert_eq!(m.command, "CAP");
        assert_eq!(m.params, vec!["*", "LS", "multi-prefix sasl"]);
    }

    #[test]
    fn parses_names_reply() {
        let m = parse(":server 353 me = #c :@op +voice plain").unwrap();
        assert_eq!(m.params, vec!["me", "=", "#c", "@op +voice plain"]);
    }

    #[test]
    fn rejects_blank_lines() {
        assert!(parse("").is_none());
        assert!(parse("\r\n").is_none());
        assert!(parse("   ").is_none());
    }

    #[test]
    fn crlf_tolerated() {
        let m = parse(":n QUIT :Quit: bye\r\n").unwrap();
        assert_eq!(m.command, "QUIT");
        assert_eq!(m.trailing(), Some("Quit: bye"));
    }

    #[test]
    fn splits_ascii_on_budget() {
        assert_eq!(split_message("abcdefgh", 3), vec!["abc", "def", "gh"]);
    }

    #[test]
    fn splits_multibyte_on_char_boundary() {
        let chunks = split_message("你好世界聊天", 6);
        assert_eq!(chunks, vec!["你好", "世界", "聊天"]);
        for c in &chunks {
            assert!(c.len() <= 6);
        }
    }

    #[test]
    fn split_empty_yields_one_empty_chunk() {
        assert_eq!(split_message("", 10), vec![""]);
    }

    #[test]
    fn detects_ctcp_action() {
        assert_eq!(ctcp_action("\x01ACTION waves\x01"), Some("waves"));
        assert_eq!(ctcp_action("hello"), None);
        assert_eq!(ctcp_action("\x01VERSION\x01"), None);
    }

    #[test]
    fn strips_mirc_formatting() {
        assert_eq!(plain_text("\x0304hello\x03 world"), "hello world");
        assert_eq!(
            plain_text("\x02bold\x02 and \x1ditalic\x1d"),
            "bold and italic"
        );
        assert_eq!(plain_text("\x0304,02colored\x0f plain"), "colored plain");
        assert_eq!(plain_text("\x04ff0000red\x04"), "red");
        assert_eq!(plain_text("no codes"), "no codes");
    }
}
