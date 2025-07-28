use bumpalo::{
    Bump,
    collections::{CollectIn, Vec},
};
use hashbrown::{HashMap, HashSet};

pub type CharSet<'bump> = HashSet<char, hashbrown::DefaultHashBuilder, &'bump Bump>;
pub type Occurrences<'bump> =
    HashMap<char, Vec<'bump, Occurrence>, hashbrown::DefaultHashBuilder, &'bump Bump>;

#[derive(Debug, Clone)]
pub struct Query<'bump> {
    pub(crate) query: &'bump [char],
    pub(crate) chars: CharSet<'bump>,
}

impl<'bump> Query<'bump> {
    pub fn new(bump: &'bump Bump, query: &str) -> Self {
        let query = query
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.to_lowercase().next().unwrap())
            .collect_in::<Vec<_>>(bump)
            .into_bump_slice();
        let mut chars = CharSet::<'bump>::new_in(bump);
        chars.extend(query.iter().copied());
        Self { query, chars }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.chars.len() == 0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Occurrence {
    pub target_idx: u32,
}

pub fn build_occurrences<'bump>(
    bump: &'bump Bump,
    query: &Query,
    string: &str,
) -> Occurrences<'bump> {
    assert!(string.len() <= u32::MAX as usize);

    let mut occurrences = HashMap::new_in(bump);

    for (i, original_c) in string.chars().enumerate() {
        let lower_c = original_c.to_lowercase().next().unwrap();

        let key_char = lower_c;

        if query.chars.contains(&key_char) {
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

#[cfg(test)]
mod tests {
    use bumpalo::{
        Bump,
        collections::{CollectIn, Vec},
        vec,
    };

    use crate::parsing::{Occurrence, Query};

    use super::build_occurrences;

    #[test]
    fn query_processing() {
        let bump = Bump::new();

        let mut set = Query::new(&bump, "a b c")
            .chars
            .into_iter()
            .collect_in::<Vec<_>>(&bump);
        set.sort();
        assert_eq!(
            vec![in &bump;
                'a',
                'b',
                'c'
            ],
            set,
            "Whitespace not removed"
        );

        let mut set = Query::new(&bump, "ABC")
            .chars
            .into_iter()
            .collect_in::<Vec<_>>(&bump);
        set.sort();
        assert_eq!(
            vec![in &bump;
                'a',
                'b',
                'c'
            ],
            set
        );
    }

    #[test]
    fn occurrences() {
        let t = "SoccerCartoonController";

        let bump = Bump::new();
        let mut occs = build_occurrences(&bump, &Query::new(&bump, "scc"), t);

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
        let mut occs = build_occurrences(&bump, &Query::new(&bump, "scc"), t);

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
