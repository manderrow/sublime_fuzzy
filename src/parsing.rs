use std::collections::{HashMap, HashSet};

pub type CharSet = HashSet<char>;
pub type Occurrences = HashMap<char, Vec<Occurrence>>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(align(8))]
pub struct Occurrence {
    pub target_idx: u32,
    data: u32,
}

impl Occurrence {
    #[inline]
    pub fn new(target_idx: u32, is_start: bool, char: char) -> Self {
        assert_eq!(char as u32 & (1 << 31), 0);
        Self {
            target_idx,
            data: ((is_start as u32) << 31) | char as u32,
        }
    }

    #[inline]
    pub fn is_start(self) -> bool {
        (self.data >> 31) != 0
    }

    #[inline]
    pub fn char(self) -> char {
        unsafe { char::from_u32_unchecked(self.data & !(1 << 31)) }
    }
}

pub fn build_occurrences(query: &QueryChars, string: &str, case_insensitive: bool) -> Occurrences {
    assert!(string.len() <= u32::MAX as usize);

    let query_chars = condense(query, case_insensitive);

    let mut occurrences = HashMap::new();

    let mut prev_is_upper = false;
    let mut prev_is_sep = true;
    let mut prev_is_start = false;

    for (i, original_c) in string.chars().enumerate() {
        let lower_c = original_c.to_lowercase().next().unwrap();

        let mut is_start = false;
        let is_sep = is_word_sep(original_c);
        let is_upper = original_c.is_uppercase();

        let key_char = if case_insensitive {
            lower_c
        } else {
            original_c
        };

        if is_sep {
            prev_is_upper = false;
            prev_is_sep = true;
            prev_is_start = false;

            if query_chars.contains(&key_char) {
                occurrences
                    .entry(key_char)
                    .or_insert(Vec::new())
                    .push(Occurrence::new(i as u32, is_start, original_c));
            }

            continue;
        }

        if prev_is_sep {
            is_start = true;
        } else {
            if !prev_is_start && (prev_is_upper != is_upper) {
                is_start = true;
            }
        }

        if query_chars.contains(&key_char) {
            occurrences
                .entry(key_char)
                .or_insert(Vec::new())
                .push(Occurrence::new(i as u32, is_start, original_c));
        }

        prev_is_start = is_start;
        prev_is_sep = is_sep;
        prev_is_upper = is_upper;
    }

    occurrences
}

fn is_word_sep(c: char) -> bool {
    !c.is_alphanumeric()
}

fn condense(s: &QueryChars, case_insensitive: bool) -> CharSet {
    s.iter()
        .map(|qc| {
            if case_insensitive {
                qc.lower
            } else {
                qc.original
            }
        })
        .collect()
}

pub type QueryChars = Vec<QueryChar>;

#[derive(Clone, Debug)]
pub struct QueryChar {
    pub original: char,
    pub lower: char,
}

impl Eq for QueryChar {}

impl PartialEq for QueryChar {
    fn eq(&self, other: &QueryChar) -> bool {
        self.original == other.original && self.lower == other.lower
    }
}

pub fn process_query(query: &str) -> QueryChars {
    query
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| QueryChar {
            original: c,
            lower: c.to_lowercase().next().unwrap(),
        })
        .collect::<Vec<QueryChar>>()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::iter::FromIterator;

    use super::{Occurrence, QueryChar, build_occurrences, condense, is_word_sep, process_query};

    #[test]
    fn occurrence_repr() {
        for c in char::MIN..=char::MAX {
            let o = Occurrence::new(0, false, c);
            assert!(!o.is_start());
            assert_eq!(o.char(), c);

            let o = Occurrence::new(0, true, c);
            assert!(o.is_start());
            assert_eq!(o.char(), c);
        }
    }

    #[test]
    fn word_seps() {
        let seps: Vec<char> = vec![
            '/', '\\', '|', '_', '-', ' ', '\t', ':', '.', ',', '~', '>', '<',
        ];

        assert!(seps.into_iter().all(|s| is_word_sep(s)));
    }

    #[test]
    fn condense_casing() {
        assert_eq!(
            condense(&process_query("SCC"), true),
            HashSet::from_iter(vec!['s', 'c']),
            "Query chars not lowercased"
        );
        assert_eq!(
            condense(&process_query("SCC"), false),
            HashSet::from_iter(vec!['S', 'C']),
            "Query chars not matching original case"
        );
    }

    #[test]
    fn query_processing() {
        assert_eq!(
            vec![
                QueryChar {
                    lower: 'a',
                    original: 'a'
                },
                QueryChar {
                    lower: 'b',
                    original: 'b'
                },
                QueryChar {
                    lower: 'c',
                    original: 'c'
                }
            ],
            process_query("a b c"),
            "Whitespace not removed"
        );

        assert_eq!(
            vec![
                QueryChar {
                    lower: 'a',
                    original: 'A'
                },
                QueryChar {
                    lower: 'b',
                    original: 'B'
                },
                QueryChar {
                    lower: 'c',
                    original: 'C'
                }
            ],
            process_query("ABC")
        );
    }

    #[test]
    fn occurrences() {
        let t = "SoccerCartoonController";

        let mut occs = build_occurrences(&process_query("scc"), t, true);

        assert_eq!(occs.len(), 2);

        let s = occs.remove(&'s').expect("Missing s occurrences");

        assert_eq!(s, vec![Occurrence::new(0, true, 'S')]);

        let c = occs.remove(&'c').expect("Missing c occurrences");

        assert_eq!(
            c,
            vec![
                Occurrence::new(2, false, 'c'),
                Occurrence::new(3, false, 'c'),
                Occurrence::new(6, true, 'C'),
                Occurrence::new(13, true, 'C'),
            ]
        );
    }

    #[test]
    fn occurrences_2() {
        let t = "SccsCoolController";

        let mut occs = build_occurrences(&process_query("scc"), t, true);

        assert_eq!(occs.len(), 2);

        let s = occs.remove(&'s').expect("Missing s occurrences");

        assert_eq!(
            s,
            vec![
                Occurrence::new(0, true, 'S'),
                Occurrence::new(3, false, 's')
            ]
        );

        let c = occs.remove(&'c').expect("Missing c occurrences");

        assert_eq!(
            c,
            vec![
                Occurrence::new(1, false, 'c'),
                Occurrence::new(2, false, 'c'),
                Occurrence::new(4, true, 'C'),
                Occurrence::new(8, true, 'C'),
            ]
        );
    }
}
