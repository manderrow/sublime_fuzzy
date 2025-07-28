use bumpalo::{
    Bump,
    collections::{CollectIn, Vec},
};
use hashbrown::{HashMap, HashSet};

pub type CharSet<'bump> = HashSet<char, hashbrown::DefaultHashBuilder, &'bump Bump>;
pub type Occurrences<'bump> =
    HashMap<char, Vec<'bump, Occurrence>, hashbrown::DefaultHashBuilder, &'bump Bump>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Occurrence {
    pub target_idx: u32,
}

pub fn build_occurrences<'bump>(
    bump: &'bump Bump,
    query: &QueryChars,
    string: &str,
) -> Occurrences<'bump> {
    assert!(string.len() <= u32::MAX as usize);

    let mut query_chars = CharSet::<'bump>::new_in(bump);
    query_chars.extend(query.iter().map(|qc| qc.lower));

    let mut occurrences = HashMap::new_in(bump);

    for (i, original_c) in string.chars().enumerate() {
        let lower_c = original_c.to_lowercase().next().unwrap();

        let key_char = lower_c;

        if query_chars.contains(&key_char) {
            occurrences
                .entry(key_char)
                .or_insert(Vec::new_in(bump))
                .push(Occurrence {
                    target_idx: i as u32,
                });
        }
    }

    occurrences
}

pub type QueryChars<'bump> = Vec<'bump, QueryChar>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryChar {
    pub lower: char,
}

pub fn process_query<'bump>(bump: &'bump Bump, query: &str) -> QueryChars<'bump> {
    query
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| QueryChar {
            lower: c.to_lowercase().next().unwrap(),
        })
        .collect_in::<Vec<QueryChar>>(bump)
}

#[cfg(test)]
mod tests {
    use bumpalo::{Bump, vec};

    use crate::parsing::Occurrence;

    use super::{QueryChar, build_occurrences, process_query};

    #[test]
    fn query_processing() {
        let bump = Bump::new();

        assert_eq!(
            vec![in &bump;
                QueryChar { lower: 'a' },
                QueryChar { lower: 'b' },
                QueryChar { lower: 'c' }
            ],
            process_query(&bump, "a b c"),
            "Whitespace not removed"
        );

        assert_eq!(
            vec![in &bump;
                QueryChar { lower: 'a' },
                QueryChar { lower: 'b' },
                QueryChar { lower: 'c' }
            ],
            process_query(&bump, "ABC")
        );
    }

    #[test]
    fn occurrences() {
        let t = "SoccerCartoonController";

        let bump = Bump::new();
        let mut occs = build_occurrences(&bump, &process_query(&bump, "scc"), t);

        assert_eq!(occs.len(), 2);

        let s = occs.remove(&'s').expect("Missing s occurrences");

        assert_eq!(s, vec![in &bump; Occurrence { target_idx: 0 }]);

        let c = occs.remove(&'c').expect("Missing c occurrences");

        assert_eq!(
            c,
            vec![in &bump;
                Occurrence { target_idx: 2 },
                Occurrence { target_idx: 3 },
                Occurrence { target_idx: 6 },
                Occurrence { target_idx: 13 },
            ]
        );
    }

    #[test]
    fn occurrences_2() {
        let t = "SccsCoolController";

        let bump = Bump::new();
        let mut occs = build_occurrences(&bump, &process_query(&bump, "scc"), t);

        assert_eq!(occs.len(), 2);

        let s = occs.remove(&'s').expect("Missing s occurrences");

        assert_eq!(
            s,
            vec![in &bump; Occurrence { target_idx: 0 }, Occurrence { target_idx: 3 }]
        );

        let c = occs.remove(&'c').expect("Missing c occurrences");

        assert_eq!(
            c,
            vec![in &bump;
                Occurrence { target_idx: 1 },
                Occurrence { target_idx: 2 },
                Occurrence { target_idx: 4 },
                Occurrence { target_idx: 8 },
            ]
        );
    }
}
