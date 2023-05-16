pub fn trim_ascii_whitespace(b: &[u8]) -> Option<&[u8]> {
    b.iter()
        .position(|&c| !c.is_ascii_whitespace())
        .and_then(|start| {
            b.iter()
                .rposition(|&c| !c.is_ascii_whitespace())
                .map(|end| &b[start..=end])
        })
}

pub fn check_valid_name(b: &[u8]) -> Option<&[u8]> {
    for &c in b {
        match c {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'*' => (),
            _ => return None,
        }
    }

    Some(b)
}

pub fn find_skip_quotes(s: &[u8], c: u8) -> Option<usize> {
    let mut escape = false;
    let mut in_quotes = false;

    for (i, &b) in s.iter().enumerate() {
        match b {
            b'\'' if !escape && !in_quotes => in_quotes = true,
            b'\'' if !escape && in_quotes => in_quotes = false,
            b'\\' if !escape => escape = true,
            _ if !in_quotes && b == c => return Some(i),
            _ => escape = false,
        }
    }

    None
}
