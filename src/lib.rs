use core::cmp::{Ordering, PartialOrd};

/// sort will sort the given array in place using GNU version sort.
/// # Examples
/// ```
/// use vsort::sort;
///
/// fn main() {
///     let mut file_names = vec![
///        "a.txt",
///        "b 1.txt",
///        "b 10.txt",
///        "b 11.txt",
///        "b 5.txt",
///        "Ssm.txt",
///     ];
///
///     sort( & mut file_names);
///     assert_eq!(
///         file_names,
///         vec!["Ssm.txt", "a.txt", "b 1.txt", "b 5.txt", "b 10.txt", "b 11.txt"]
///     );
/// }
/// ```
pub fn sort(arr: &mut [&str]) {
    arr.sort_by(|a, b| compare(a, b));
}

/// compare implements GNU version sort.
/// # Examples
/// ```
/// use vsort::compare;
///
/// fn main() {
///     let mut file_names = vec![
///         "a.txt",
///         "b 1.txt",
///         "b 10.txt",
///         "b 11.txt",
///         "b 5.txt",
///         "Ssm.txt",
///     ];
///
///     // Pass to sort_by
///     file_names.sort_by(|a, b| compare(a, b));
///     assert_eq!(
///         file_names,
///         vec!["Ssm.txt", "a.txt", "b 1.txt", "b 5.txt", "b 10.txt", "b 11.txt"]
///     );
/// }
/// ```
pub fn compare(a: &str, b: &str) -> Ordering {
    // Let's shadow the inputs for easy reference.
    let mut a = a;
    let mut b = b;

    // The spec says that the following have special priority and sort before
    // all other strings, in the listed order: ("", ".", "..").
    // https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi#L532-L569
    if let Some(cmp) = match (a, b) {
        ("", "") | (".", ".") | ("..", "..") => Some(Ordering::Equal),
        ("", _) => Some(Ordering::Less),
        (_, "") => Some(Ordering::Greater),
        (".", _) => Some(Ordering::Less),
        (_, ".") => Some(Ordering::Greater),
        ("..", _) => Some(Ordering::Less),
        (_, "..") => Some(Ordering::Greater),
        _ => None,
    } {
        return cmp;
    };

    // Hidden files get priority. If both files are hidden then we remove the leading period
    // and compare.
    match (a.starts_with('.'), b.starts_with('.')) {
        (true, false) => return Ordering::Less,
        (false, true) => return Ordering::Greater,
        (false, false) => {}
        (true, true) => {
            a = if a.len() == 1 { "" } else { &a[1..] };
            b = if b.len() == 1 { "" } else { &b[1..] };
        }
    }

    // Compare without the file extensions
    let cmp = sequence_cmp(split_extension(a).0, split_extension(b).0);
    if cmp != Ordering::Equal {
        return cmp;
    }
    // Compare the original strings with the file extensions
    let cmp = sequence_cmp(a, b);
    if cmp != Ordering::Equal {
        return cmp;
    }
    // At this point the file extensions are the same, so we compare the full strings.
    // this helps with cases like a0001 and a1 so that they have a consistent ordering.
    a.cmp(b)
}

/// sequence_cmp extracts non-digit and digit sequences from the two strings and compares the
/// sequences until an ordering is determined.
fn sequence_cmp(a: &str, b: &str) -> Ordering {
    let mut a_str = a;
    let mut b_str = b;
    loop {
        let (a_non_digit_part, remaining_a) = non_digit_seq(a_str);
        let (b_non_digit_part, remaining_b) = non_digit_seq(b_str);
        let cmp = compare_non_digit_seq(a_non_digit_part, b_non_digit_part);
        if cmp != Ordering::Equal {
            return cmp;
        }
        let (a_digit_part, remaining_a) = digit_seq(remaining_a);
        let (b_digit_part, remaining_b) = digit_seq(remaining_b);

        // According to the docs, a missing numerical part also counts as zero.
        let a_digits = a_digit_part.parse::<u64>().unwrap_or_default();
        let b_digits = b_digit_part.parse::<u64>().unwrap_or_default();
        let cmp = a_digits.cmp(&b_digits);
        if cmp != Ordering::Equal {
            return cmp;
        }

        a_str = remaining_a;
        b_str = remaining_b;

        // If any or both strings have been exhausted we can determine the ordering.
        if a_str.is_empty() && b_str.is_empty() {
            return Ordering::Equal;
        }
    }
}

/*
fn split_extension(s: &str) -> (&str, &str) {
    // According to GNU sort, an extension is defined as a dot, followed by an
    // ASCII letter or tilde, followed by zero or more ASCII letters, digits,
    // or tildes; all repeated zero or more times, and ending at string end.
    // The regex is from https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi#L584-L591
    let re = Regex::new(r"(\.[A-Za-z~][A-Za-z0-9~]*)*$").unwrap();

    re.find(s).map_or((s, ""), |m| {
        let (a, b) = s.split_at(m.start());
        (a, b)
    })
}
 */

fn split_extension(s: &str) -> (&str, &str) {
    // According to GNU sort, an extension is defined as a dot, followed by an
    // ASCII letter or tilde, followed by zero or more ASCII letters, digits,
    // or tildes; all repeated zero or more times, and ending at string end.
    // The regex is from https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi#L584-L591
    let mut split_ind: Option<usize> = None;
    let mut last_char: Option<char> = None;
    for (i, c) in s.char_indices().rev() {
        // If we have found a period
        if c == '.' {
            match last_char {
                // We found a period as our last character. Exit with no extension
                None => return (s, ""),
                Some(prev_char) => {
                    // If the previous character wasn't alphanumeric this isn't a valid
                    if prev_char.is_ascii_alphabetic() || prev_char == '~' {
                        split_ind = Some(i);
                    } else {
                        break;
                    }
                }
            }
        } else if !(c.is_ascii_alphanumeric() || c == '~') {
            break;
        }
        // Update the last char for inspection
        last_char = Some(c);
    }

    split_ind.map_or((s, ""), |ind| s.split_at(ind))
}

#[derive(Eq)]
struct VersionSortChar(Option<u8>);

impl From<Option<u8>> for VersionSortChar {
    fn from(c: Option<u8>) -> Self {
        Self(c)
    }
}

impl PartialOrd for VersionSortChar {
    // Based on https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi
    // For non-digit characters, we apply the following rules:
    //   ~(tilde) comes before all other strings, even the empty string.
    //   ASCII letters sort before other bytes.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.0, other.0) {
            (None, None) => Some(Ordering::Equal),
            (Some(a), None) => {
                if a == b'~' {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
            (None, Some(b)) => {
                if b == b'~' {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            }
            (Some(a), Some(b)) => {
                if a == b {
                    return Some(Ordering::Equal);
                }
                if a == b'~' {
                    return Some(Ordering::Less);
                }
                if b == b'~' {
                    return Some(Ordering::Greater);
                }
                match (a.is_ascii_alphabetic(), b.is_ascii_alphabetic()) {
                    // ASCII letters sort before other bytes. If they are both ASCII
                    // or both are not ASCII sort normally.
                    (true, true) | (false, false) => Some(a.cmp(&b)),
                    (true, false) => Some(Ordering::Less),
                    (false, true) => Some(Ordering::Greater),
                }
            }
        }
    }
}

impl PartialEq for VersionSortChar {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

fn compare_non_digit_seq(a: &str, b: &str) -> Ordering {
    let mut a_bytes = a.bytes();
    let mut b_bytes = b.bytes();
    loop {
        let a_byte = a_bytes.next();
        let b_byte = b_bytes.next();
        if a_byte.is_none() && b_byte.is_none() {
            return Ordering::Equal;
        }
        let cmp = VersionSortChar::from(a_byte)
            .partial_cmp(&VersionSortChar::from(b_byte))
            .unwrap();
        if cmp == Ordering::Equal {
            continue;
        }
        return cmp;
    }
}

fn non_digit_seq(a: &str) -> (&str, &str) {
    a.bytes()
        .enumerate()
        .find(|(_, c)| c.is_ascii_digit())
        .map_or((a, ""), |(index, _)| a.split_at(index))
}

fn digit_seq(a: &str) -> (&str, &str) {
    a.bytes()
        .enumerate()
        .find(|(_, c)| !c.is_ascii_digit())
        .map_or((a, ""), |(index, _)| a.split_at(index))
}

#[cfg(test)]
mod test {
    use test_case::test_case;

    use super::*;

    #[test_case(vec!["", "~"], vec!["", "~"]; "in sorted order")]
    #[test_case(vec!["~", ""], vec!["", "~"]; "in reversed order")]
    fn test_empty_string_vs_tilde(original: Vec<&str>, expected: Vec<&str>) {
        let mut list = original;
        sort(&mut list);
        assert_eq!(list, expected);
    }

    #[test]
    fn test_non_digit_sorting() {
        let mut list = vec!["aaa", "aa", "aab", "aa&", "aa_", "aa~", "a"];
        list.sort_by(|a, b| compare_non_digit_seq(a, b));

        assert_eq!(
            list,
            vec![
                "a", // Absolute shortest comes first
                "aa~", "aa", // Tilde comes before empty string
                "aaa", "aab", "aa&", "aa_", // ASCII letters come before other bytes
            ]
        );
    }

    #[test]
    fn test_non_digit_seq() {
        let a = "file_1.txt";
        let (seq, remainder) = non_digit_seq(a);
        assert_eq!(seq, "file_");
        assert_eq!(remainder, "1.txt");

        let (seq, remainder) = non_digit_seq(&a[5..]);
        assert_eq!(seq, "");
        assert_eq!(remainder, "1.txt");

        let (seq, remainder) = non_digit_seq(&a[6..]);
        assert_eq!(seq, ".txt");
        assert_eq!(remainder, "");
    }

    #[test]
    fn test_unusual_test_case() {
        let mut list = vec![
            "a.txt", "b 1.txt", "b 10.txt", "b 11.txt", "b 5.txt", "Ssm.txt",
        ];
        sort(&mut list);

        assert_eq!(
            list,
            vec!["Ssm.txt", "a.txt", "b 1.txt", "b 5.txt", "b 10.txt", "b 11.txt"]
        );
    }

    // This tests that the implementation can handle characters that are longer than a single byte.
    #[test_case(
      vec!["αβγ2.txt", "αβγ1.txt", "1αβγ.txt", "2αβγ.txt"],
      vec!["1αβγ.txt", "2αβγ.txt", "αβγ1.txt", "αβγ2.txt"];
      "test_with_non_ascii"
    )]
    fn test_with_non_utf8(original: Vec<&str>, expected: Vec<&str>) {
        let mut list = original;
        sort(&mut list);
        assert_eq!(list, expected);
    }

    #[test]
    fn test_missing_number_part() {
        let mut original_list = vec!["file.txt", "file0.txt"];
        sort(&mut original_list);

        assert_eq!(original_list, vec!["file0.txt", "file.txt"]);

        let mut original_list = vec!["file0.txt", "file.txt"];
        sort(&mut original_list);

        assert_eq!(original_list, vec!["file0.txt", "file.txt"]);
    }

    // Coreutils Tests
    // These tests are lifted from https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi
    // They are used in the spec to clarify some sorting rules. They seemed useful enough to add here.

    // https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi#L265-L285
    #[test_case(
      vec!["8.10", "8.5", "8.1", "8.01", "8.010", "8.100", "8.49"],
      vec!["8.01", "8.1", "8.5", "8.010", "8.10", "8.49", "8.100"];
      "sort with numbers"
    )]
    // https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi#L316-L335
    #[test_case(
      vec!["1.0_src.tar.gz", "1.0.5_src.tar.gz"],
      vec!["1.0.5_src.tar.gz", "1.0_src.tar.gz"];
      "period is before underscore"
    )]
    // https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi#L353-L363
    #[test_case(
      vec!["3.0/", "3.0.5"],
      vec!["3.0.5", "3.0/"];
      "period is before forward slash"
    )]
    // https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi#L372-L379
    #[test_case(
      vec!["a%", "az"],
      vec!["az", "a%"];
      "letters before non-letters"
    )]
    // https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi#L400-L413
    #[test_case(
      vec!["1", "1%", "1.2", "1~", "~"],
      vec!["~", "1~", "1", "1%", "1.2"];
      "tilde before all others strings"
    )]
    // https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi#L451-L456
    #[test_case(
      vec!["aa", "az", "a%", "aα"],
      vec!["aa", "az", "a%", "aα"];
      "sort ignores locale"
    )]
    // https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi#L551-L560
    #[test_case(
      vec!["a", "b", ".", "c", "..", ".d20", ".d3"],
      vec![".", "..", ".d3", ".d20", "a", "b", "c"];
      "special directories and hidden files are sorted first"
    )]
    fn test_basic_tests(original: Vec<&str>, expected: Vec<&str>) {
        let mut list = original;
        sort(&mut list);
        assert_eq!(list, expected);
    }

    // Examples from https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi#L608-L634
    #[test_case("hello-8.txt", ("hello-8", ".txt"); "basic")]
    #[test_case("hello-8.2.txt", ("hello-8.2", ".txt"); "with major and minor")]
    #[test_case("hello-8.0.12.tar.gz", ("hello-8.0.12", ".tar.gz"); "with extension")]
    #[test_case("hello-8.2", ("hello-8.2", ""); "without extension")]
    #[test_case("hello.foobar65", ("hello", ".foobar65"); "with long extension")]
    #[test_case(
      "gcc-c++-10.8.12-0.7rc2.fc9.tar.bz2",
      ("gcc-c++-10.8.12-0.7rc2", ".fc9.tar.bz2");
      "with multiple extensions"
    )]
    #[test_case(".autom4te.cfg", ("", ".autom4te.cfg"); "empty name with extension")]
    #[test_case("a.#$%", ("a.#$%", ""); "no extension present")]
    #[test_case("a.#$%.txt", ("a.#$%", ".txt"); "extension stops at non-alphanumeric characters")]
    fn test_split_extension(input: &str, split: (&str, &str)) {
        assert_eq!(split_extension(input), split);
    }

    // This list is pulled from
    // https://github.com/coreutils/gnulib/blob/master/tests/test-filevercmp.c#L26-L102
    #[test]
    fn test_long_sorted_list() {
        let expected = vec![
            "",
            ".",
            "..",
            ".0",
            ".9",
            ".A",
            ".Z",
            ".a~",
            ".a",
            ".b~",
            ".b",
            ".z",
            ".zz~",
            ".zz",
            ".zz.~1~",
            ".zz.0",
            ".\u{1}",
            ".\u{1}.txt",
            ".\u{1}x",
            ".\u{1}x\u{1}",
            ".\u{1}.0",
            "0",
            "9",
            "A",
            "Z",
            "a~",
            "a",
            "a.b~",
            "a.b",
            "a.bc~",
            "a.bc",
            "a+",
            "a.",
            "a..a",
            "a.+",
            "b~",
            "b",
            "gcc-c++-10.fc9.tar.gz",
            "gcc-c++-10.fc9.tar.gz.~1~",
            "gcc-c++-10.fc9.tar.gz.~2~",
            "gcc-c++-10.8.12-0.7rc2.fc9.tar.bz2",
            "gcc-c++-10.8.12-0.7rc2.fc9.tar.bz2.~1~",
            "glibc-2-0.1.beta1.fc10.rpm",
            "glibc-common-5-0.2.beta2.fc9.ebuild",
            "glibc-common-5-0.2b.deb",
            "glibc-common-11b.ebuild",
            "glibc-common-11-0.6rc2.ebuild",
            "libstdc++-0.5.8.11-0.7rc2.fc10.tar.gz",
            "libstdc++-4a.fc8.tar.gz",
            "libstdc++-4.10.4.20040204svn.rpm",
            "libstdc++-devel-3.fc8.ebuild",
            "libstdc++-devel-3a.fc9.tar.gz",
            "libstdc++-devel-8.fc8.deb",
            "libstdc++-devel-8.6.2-0.4b.fc8",
            "nss_ldap-1-0.2b.fc9.tar.bz2",
            "nss_ldap-1-0.6rc2.fc8.tar.gz",
            "nss_ldap-1.0-0.1a.tar.gz",
            "nss_ldap-10beta1.fc8.tar.gz",
            "nss_ldap-10.11.8.6.20040204cvs.fc10.ebuild",
            "z",
            "zz~",
            "zz",
            "zz.~1~",
            "zz.0",
            "zz.0.txt",
            "\u{1}",
            "\u{1}.txt",
            "\u{1}x",
            "\u{1}x\u{1}",
            "\u{1}.0",
            "#\u{1}.b#",
            "#.b#",
        ];
        let mut list = expected.clone();
        list.reverse();
        assert_ne!(list, expected);
        sort(&mut list);
        assert_eq!(list, expected);
    }

    // These tests are lifted from
    // https://github.com/coreutils/gnulib/blob/master/tests/test-filevercmp.c
    #[test_case(vec!["a", "a0", "a0000"]; "zeros are the same as empty string")]
    #[test_case(vec!["a\u{1}c-27.txt", "a\u{1}c-027.txt", "a\u{1}c-00000000000000000000000000000000000000000000000000000027.txt",]; "non-ascii")]
    #[test_case(vec![".a\u{1}c-27.txt", ".a\u{1}c-027.txt", ".a\u{1}c-00000000000000000000000000000000000000000000000000000027.txt",]; "non-ascii with leading period")]
    #[test_case(vec!["a\u{1}c-", "a\u{1}c-0", "a\u{1}c-00",]; "non-ascii without extension")]
    #[test_case(vec![".a\u{1}c-", ".a\u{1}c-0", ".a\u{1}c-00",]; "non-ascii without extension and leading period")]
    #[test_case(vec!["a\u{1}c-0.txt", "a\u{1}c-00.txt"]; "non-ascii with trailing zeros")]
    #[test_case(vec![".a\u{1}c-1\u{1}.txt", ".a\u{1}c-001\u{1}.txt"]; "non-ascii with leading zeros before a number")]
    fn test_strings_cmp_equal(list: Vec<&str>) {
        let end = list.len();
        for i in 0..end {
            for j in (i + 1)..end {
                assert_eq!(sequence_cmp(list[i], list[j]), Ordering::Equal);
            }
        }
    }
}
