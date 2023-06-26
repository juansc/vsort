use core::cmp::{Ordering, PartialOrd};

use regex::Regex;

// VersionSortChunkIterator that iterates over a &str and returns a tuple
// of (&str, u64).
struct VersionSortChunkIterator<'a> {
    remainder: &'a str,
}

impl<'a> VersionSortChunkIterator<'a> {
    fn new(s: &'a str) -> Self {
        Self { remainder: s }
    }
}

// Implement an iterator for VersionSortChunkIterator
impl<'a> Iterator for VersionSortChunkIterator<'a> {
    type Item = (&'a str, u64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.remainder.is_empty() {
            return None;
        }
        let (non_digit_part, out) = non_digit_seq(self.remainder);
        let (digit_part, out) = digit_seq(out);
        // According to the original spec a missing numerical part also counts as zero.
        let digits = digit_part.parse::<u64>().unwrap_or_default();

        self.remainder = out;
        Some((non_digit_part, digits))
    }
}

fn with_iterator(a: &str, b: &str) -> Ordering {
    let mut a_iter = VersionSortChunkIterator::new(a);
    let mut b_iter = VersionSortChunkIterator::new(b);
    loop {
        let a_chunk = a_iter.next();
        let b_chunk = b_iter.next();
        // If both are empty they are equal.
        if a_chunk.is_none() && b_chunk.is_none() {
            return Ordering::Equal;
        }
        // We can't exit early because "~" will beat out the empty string. In this case we
        // create a default value for each chunk.
        let (a_str, a_digits) = a_chunk.map_or(("", 0), |a_out| a_out);
        let (b_str, b_digits) = b_chunk.map_or(("", 0), |b_out| b_out);

        let cmp = compare_non_digit_seq(a_str, b_str);
        if cmp != Ordering::Equal {
            return cmp;
        }
        let cmp = a_digits.cmp(&b_digits);
        if cmp != Ordering::Equal {
            return cmp;
        }
    }
}

/*
fn original_implementation(a: &str, b: &str) -> Ordering {
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
 */

fn compare_version_sort(a: &str, b: &str) -> Ordering {
    with_iterator(a, b)
}

pub fn sort(arr: &mut [&str]) {
    arr.sort_by(|a, b| compare(a, b));
}

/// compare implements GNU version-sort.
pub fn compare(a: &str, b: &str) -> Ordering {
    // Compare without the file extensions
    let cmp = compare_version_sort(split_extension(a).0, split_extension(b).0);
    if cmp != Ordering::Equal {
        return cmp;
    }
    // Compare the original strings with the file extensions
    let cmp = compare_version_sort(a, b);
    if cmp != Ordering::Equal {
        return cmp;
    }
    // At this point the file extensions are the same, so we compare the full strings.
    // this helps with cases like a0001 and a1 so that they have a consistent ordering.
    a.cmp(b)
}

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

#[derive(Eq)]
struct VersionSortChar(Option<char>);

impl From<Option<char>> for VersionSortChar {
    fn from(c: Option<char>) -> Self {
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
                if a == '~' {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
            (None, Some(b)) => {
                if b == '~' {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            }
            (Some(a), Some(b)) => {
                if a == b {
                    return Some(Ordering::Equal);
                }
                if a == '~' {
                    return Some(Ordering::Less);
                }
                if b == '~' {
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
    let mut a_chars = a.chars();
    let mut b_chars = b.chars();
    loop {
        let a_char = a_chars.next();
        let b_char = b_chars.next();
        if a_char.is_none() && b_char.is_none() {
            return Ordering::Equal;
        }
        let cmp = VersionSortChar::from(a_char)
            .partial_cmp(&VersionSortChar::from(b_char))
            .unwrap();
        if cmp == Ordering::Equal {
            continue;
        }
        return cmp;
    }
}

fn non_digit_seq(a: &str) -> (&str, &str) {
    a.char_indices()
        .find(|(_, c)| c.is_ascii_digit())
        .map(|(index, _)| a.split_at(index))
        .unwrap_or((a, ""))
}
fn digit_seq(a: &str) -> (&str, &str) {
    a.char_indices()
        .find(|(_, c)| !c.is_ascii_digit())
        .map(|(index, _)| a.split_at(index))
        .unwrap_or((a, ""))
}

#[cfg(test)]
mod test {
    use test_case::test_case;

    use super::*;


    #[test_case(vec!["", "~"], vec!["~", ""] ; "in sorted order")]
    #[test_case(vec!["~", ""], vec!["~", ""] ; "in reversed order")]
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
                // Absolute shortest comes first
                "a", // Tilde comes before empty string
                "aa~", "aa", // ASCII letters come before other bytes
                "aaa", "aab", "aa&", "aa_",
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

    #[test]
    fn test_small_list() {
        let mut list = vec![
            "file_1.txt",
            "file_10.txt",
            "file_2.txt",
            "file_20.txt",
            "file_11.txt",
            "file_1a.txt",
            "file_1B.txt",
            "file_a1.txt",
            "file_A1.txt",
            "file_001.txt",
        ];
        sort(&mut list);

        assert_eq!(
            list,
            vec![
                "file_001.txt",
                "file_1.txt",
                "file_1B.txt",
                "file_1a.txt",
                "file_2.txt",
                "file_10.txt",
                "file_11.txt",
                "file_20.txt",
                "file_A1.txt",
                "file_a1.txt",
            ]
        );
    }

    #[test]
    fn test_underscores_with_numbers() {
        let mut original_list = vec![
            "data_1.txt",
            "data_10.txt",
            "data_11.txt",
            "data_12.txt",
            "data_13.txt",
            "data_14.txt",
            "data_15.txt",
            "data_16.txt",
            "data_17.txt",
            "data_18.txt",
            "data_19.txt",
            "data_2.txt",
            "data_20.txt",
            "data_21.txt",
            "data_22.txt",
            "data_23.txt",
            "data_24.txt",
            "data_25.txt",
            "data_26.txt",
            "data_27.txt",
            "data_28.txt",
            "data_29.txt",
            "data_3.txt",
            "data_30.txt",
            "data_4.txt",
            "data_5.txt",
            "data_6.txt",
            "data_7.txt",
            "data_8.txt",
            "data_9.txt",
        ];
        sort(&mut original_list);
        assert_eq!(
            original_list,
            vec![
                "data_1.txt",
                "data_2.txt",
                "data_3.txt",
                "data_4.txt",
                "data_5.txt",
                "data_6.txt",
                "data_7.txt",
                "data_8.txt",
                "data_9.txt",
                "data_10.txt",
                "data_11.txt",
                "data_12.txt",
                "data_13.txt",
                "data_14.txt",
                "data_15.txt",
                "data_16.txt",
                "data_17.txt",
                "data_18.txt",
                "data_19.txt",
                "data_20.txt",
                "data_21.txt",
                "data_22.txt",
                "data_23.txt",
                "data_24.txt",
                "data_25.txt",
                "data_26.txt",
                "data_27.txt",
                "data_28.txt",
                "data_29.txt",
                "data_30.txt",
            ],
        );
    }

    #[test]
    fn test_large_list() {
        let mut original_list = vec![
            "file1.txt",
            "file2.txt",
            "file3.txt",
            "file10.txt",
            "file10a.txt",
            "file10b.txt",
            "file10c.txt",
            "file11.txt",
            "file12.txt",
            "file1a.txt",
            "file1b.txt",
            "file1c.txt",
            "file20.txt",
            "file200.txt",
            "file2000.txt",
            "file2001.txt",
            "file201.txt",
            "file21.txt",
            "file22.txt",
            "file100.txt",
            "file1000.txt",
            "file101.txt",
            "file1002.txt",
            "file102.txt",
            "file2002.txt",
            "file202.txt",
            "file1001.txt",
            "fileA.txt",
            "fileB.txt",
            "fileC.txt",
            "filea1.txt",
            "filea2.txt",
            "filea3.txt",
            "filea10.txt",
            "filea10b.txt",
            "filea10c.txt",
            "filea12.txt",
            "filea20.txt",
            "filea100.txt",
            "filea200.txt",
            "filea1000.txt",
            "filea1001.txt",
            "filea101.txt",
            "filea1002.txt",
            "filea102.txt",
            "filea10a.txt",
            "filea11.txt",
            "filea1a.txt",
            "filea1b.txt",
            "filea1c.txt",
            "filea2000.txt",
            "filea2001.txt",
            "filea201.txt",
            "filea21.txt",
            "filea2002.txt",
            "filea202.txt",
            "filea22.txt",
            "fileaA.txt",
            "fileaB.txt",
            "fileaC.txt",
            "fileb1.txt",
            "fileb2.txt",
            "fileb3.txt",
            "fileb10.txt",
            "fileb100.txt",
            "fileb101.txt",
            "fileb102.txt",
            "fileb10a.txt",
            "fileb10b.txt",
            "fileb10c.txt",
            "fileb11.txt",
            "fileb12.txt",
            "fileb20.txt",
            "fileb200.txt",
            "fileb1001.txt",
            "fileb2000.txt",
            "fileb2001.txt",
            "fileb201.txt",
            "fileb21.txt",
            "fileb22.txt",
            "fileb1000.txt",
            "fileb2002.txt",
            "fileb202.txt",
            "fileb1002.txt",
        ];
        sort(&mut original_list);
        assert_eq!(
            original_list,
            vec![
                "file1.txt",
                "file1a.txt",
                "file1b.txt",
                "file1c.txt",
                "file2.txt",
                "file3.txt",
                "file10.txt",
                "file10a.txt",
                "file10b.txt",
                "file10c.txt",
                "file11.txt",
                "file12.txt",
                "file20.txt",
                "file21.txt",
                "file22.txt",
                "file100.txt",
                "file101.txt",
                "file102.txt",
                "file200.txt",
                "file201.txt",
                "file202.txt",
                "file1000.txt",
                "file1001.txt",
                "file1002.txt",
                "file2000.txt",
                "file2001.txt",
                "file2002.txt",
                "fileA.txt",
                "fileB.txt",
                "fileC.txt",
                "filea1.txt",
                "filea1a.txt",
                "filea1b.txt",
                "filea1c.txt",
                "filea2.txt",
                "filea3.txt",
                "filea10.txt",
                "filea10a.txt",
                "filea10b.txt",
                "filea10c.txt",
                "filea11.txt",
                "filea12.txt",
                "filea20.txt",
                "filea21.txt",
                "filea22.txt",
                "filea100.txt",
                "filea101.txt",
                "filea102.txt",
                "filea200.txt",
                "filea201.txt",
                "filea202.txt",
                "filea1000.txt",
                "filea1001.txt",
                "filea1002.txt",
                "filea2000.txt",
                "filea2001.txt",
                "filea2002.txt",
                "fileaA.txt",
                "fileaB.txt",
                "fileaC.txt",
                "fileb1.txt",
                "fileb2.txt",
                "fileb3.txt",
                "fileb10.txt",
                "fileb10a.txt",
                "fileb10b.txt",
                "fileb10c.txt",
                "fileb11.txt",
                "fileb12.txt",
                "fileb20.txt",
                "fileb21.txt",
                "fileb22.txt",
                "fileb100.txt",
                "fileb101.txt",
                "fileb102.txt",
                "fileb200.txt",
                "fileb201.txt",
                "fileb202.txt",
                "fileb1000.txt",
                "fileb1001.txt",
                "fileb1002.txt",
                "fileb2000.txt",
                "fileb2001.txt",
                "fileb2002.txt",
            ],
        );
    }

    // This tests that the implementation can handle characters that are longer than a single byte.
    #[test_case(
      vec!["αβγ2.txt", "αβγ1.txt", "1αβγ.txt", "2αβγ.txt"],
      vec!["1αβγ.txt", "2αβγ.txt", "αβγ1.txt", "αβγ2.txt"] ;
      "test_with_non_ascii"
    )]
    fn test_with_non_utf8(original: Vec<&str>, expected: Vec<&str>) {
        let mut list = original;
        sort(&mut list);
        assert_eq!(list, expected);
    }

    #[test]
    fn test_chunk_iterator() {
        let mut iter = VersionSortChunkIterator::new("a1b2c3d");
        assert_eq!(iter.next(), Some(("a", 1)));
        assert_eq!(iter.next(), Some(("b", 2)));
        assert_eq!(iter.next(), Some(("c", 3)));
        assert_eq!(iter.next(), Some(("d", 0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_missing_number_part() {
        let mut original_list = vec!["file.txt", "file0.txt"];
        sort(&mut original_list);

        assert_eq!(original_list, vec!["file0.txt", "file.txt"],);

        let mut original_list = vec!["file0.txt", "file.txt"];
        sort(&mut original_list);

        assert_eq!(original_list, vec!["file0.txt", "file.txt"],);
    }

    #[test_case(
      vec!["aa", "az", "aα", "a%"],
      vec!["aa", "az", "a%", "aα"] ;
      "sorts by byte value"
    )]
    fn test_byte_by_byte_comparison_from_docs(original: Vec<&str>, expected: Vec<&str>) {
        let mut list = original;
        sort(&mut list);
        assert_eq!(list, expected);
    }

    // Coreutils Tests
    // These tests are lifted from https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi
    // They are used in the spec to clarify some sorting rules. They seemed useful enough to add here.

    // https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi#L265-L285
    #[test_case(
      vec!["8.10", "8.5", "8.1", "8.01", "8.010", "8.100", "8.49"],
      vec!["8.01", "8.1", "8.5", "8.010", "8.10", "8.49", "8.100"] ;
      "sort with numbers"
    )]
    fn test_version_sort_with_numbers(original: Vec<&str>, expected: Vec<&str>) {
        let mut list = original;
        sort(&mut list);
        assert_eq!(list, expected);
    }

    fn test_punctuation_sort(original: Vec<&str>, expected: Vec<&str>) {
    }

    // Examples from https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi#L608-L634
    #[test_case("hello-8.txt",  ("hello-8", ".txt") ; "basic")]
    #[test_case("hello-8.2.txt",  ("hello-8.2", ".txt") ; "with major and minor")]
    #[test_case("hello-8.0.12.tar.gz", ("hello-8.0.12", ".tar.gz") ; "with extension")]
    #[test_case("hello-8.2",  ("hello-8.2", "") ; "without extension")]
    #[test_case("hello.foobar65", ("hello", ".foobar65") ; "with long extension")]
    #[test_case(
      "gcc-c++-10.8.12-0.7rc2.fc9.tar.bz2",
      ("gcc-c++-10.8.12-0.7rc2", ".fc9.tar.bz2") ;
      "with multiple extensions"
    )]
    #[test_case(".autom4te.cfg", ("", ".autom4te.cfg") ; "empty name with extension")]
    fn test_split_extension(input: &str, split: (&str, &str)) {
        assert_eq!(split_extension(input), split);
    }
}
