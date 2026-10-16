//! ID formats, shared by `check` and the commands that create IDs.

/// A rule file prefix: `^[A-Z][A-Z0-9]*$`.
pub fn is_prefix(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

/// A rule ID, `PREFIX-NNN`, split into its prefix and number.
pub fn parse_rule_id(s: &str) -> Option<(&str, u16)> {
    let (prefix, number) = s.split_once('-')?;
    let well_formed =
        is_prefix(prefix) && number.len() == 3 && number.bytes().all(|b| b.is_ascii_digit());
    well_formed.then(|| (prefix, number.parse().unwrap_or(0)))
}

/// The characters of the random part of decision and gap IDs.
pub const RANDOM_ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

/// A decision or gap ID: the marker, the UTC date as `YYMMDD`, `-`, and five characters from
/// `RANDOM_ALPHABET`.
pub fn is_dated_id(s: &str, marker: char) -> bool {
    let Some((date, random)) = s.strip_prefix(marker).and_then(|rest| rest.split_once('-')) else {
        return false;
    };
    date.len() == 6
        && date.bytes().all(|b| b.is_ascii_digit())
        && random.len() == 5
        && random.bytes().all(|b| RANDOM_ALPHABET.contains(&b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes() {
        assert!(is_prefix("ATTR") && is_prefix("A2"));
        assert!(!is_prefix("2A") && !is_prefix("attr") && !is_prefix(""));
    }

    #[test]
    fn rule_ids() {
        assert_eq!(parse_rule_id("ATTR-003"), Some(("ATTR", 3)));
        assert_eq!(parse_rule_id("ATTR-03"), None);
        assert_eq!(parse_rule_id("ATTR-1000"), None);
        assert_eq!(parse_rule_id("attr-003"), None);
        assert_eq!(parse_rule_id("ATTR003"), None);
    }

    #[test]
    fn dated_ids() {
        assert!(is_dated_id("D261015-k7m2q", 'D'));
        assert!(is_dated_id("G261015-x4p9t", 'G'));
        assert!(!is_dated_id("D261015-k7m2", 'D'));
        assert!(!is_dated_id("D261015-k7m2qq", 'D'));
        assert!(!is_dated_id("D261015-K7M2Q", 'D'));
        assert!(!is_dated_id("D26101-k7m2q", 'D'));
        assert!(!is_dated_id("G261015-k7m2q", 'D'));
    }
}
