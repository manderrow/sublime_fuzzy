use bumpalo::{Bump, vec};
use hashbrown::HashMap;

use crate::matching::Match;
use crate::parsing::{Occurrence, Occurrences, Query, build_occurrences};
use crate::scoring::{DEFAULT_SCORING, Scoring};

/// Finds the best match of the query in the target string.
///
/// Always tries to match the _full_ pattern. A partial match is considered
/// invalid and will return [`None`]. Will also return [`None`] in case the query or
/// target string are empty.
pub fn best_match<'bump>(
    bump: &'bump Bump,
    query: &Query<'bump>,
    scoring: &Scoring,
    haystack: &str,
) -> Option<&'bump Match<'bump>> {
    if query.is_empty() || haystack.len() == 0 {
        return None;
    }

    let occurrences = build_occurrences(&bump, query, haystack);

    let searcher = FuzzySearcher::new(bump, &query, scoring);

    searcher.best_match(&occurrences)
}

struct FuzzySearcher<'a, 'bump> {
    bump: &'bump Bump,
    // TODO: should we store this as a &str instead?
    query: &'a Query<'bump>,
    scoring: &'a Scoring,
    match_cache: HashMap<
        (u32, u32, u32),
        Option<&'bump Match<'bump>>,
        hashbrown::DefaultHashBuilder,
        &'bump Bump,
    >,
}

impl<'a, 'bump> FuzzySearcher<'a, 'bump> {
    fn new(bump: &'bump Bump, query: &'a Query<'bump>, scoring: &'a Scoring) -> Self {
        FuzzySearcher {
            bump,
            match_cache: HashMap::with_capacity_in(query.query.len() * query.query.len(), bump),
            query,
            scoring,
        }
    }

    fn best_match(mut self, occurrences: &Occurrences) -> Option<&'bump Match<'bump>> {
        let qc = self.query.query.get(0)?;

        occurrences
            .get(qc)?
            .iter()
            .filter_map(|o| self.match_(1, o, 0, &occurrences))
            .max()
    }

    fn match_(
        &mut self,
        query_idx: u32,
        occurrence: &Occurrence,
        consecutive: u32,
        occurrences: &Occurrences,
    ) -> Option<&'bump Match<'bump>> {
        let this_key = (query_idx, occurrence.target_idx, consecutive);

        // Already scored sub-tree
        if let Some(cached) = self.match_cache.get(&this_key) {
            return *cached;
        }

        let Some(next_char) = self.query.query.get(query_idx as usize) else {
            // Successfully matched all query chars

            let this_match = self.bump.alloc(Match::with_matched(
                self.match_calc_score(consecutive),
                consecutive,
                vec![in &self.bump; occurrence.target_idx],
            ));

            self.insert_match(this_key, Some(this_match));

            return Some(this_match);
        };

        let Some(occs) = occurrences.get(next_char) else {
            // Reached end of target without matching all query chars

            self.insert_match(this_key, None);

            return None;
        };

        let best_match = occs
            .iter()
            .filter(|&o| o.target_idx > occurrence.target_idx)
            .filter_map(|o| {
                let distance = o.target_idx - occurrence.target_idx;

                let new_consecutive = if distance == 1 { consecutive + 1 } else { 0 };

                self.match_(query_idx + 1, o, new_consecutive, occurrences)
            })
            .max()
            .map(|m| {
                let m = self.bump.alloc((*m).clone());
                m.prepend(
                    self.match_calc_score(consecutive),
                    consecutive,
                    &[occurrence.target_idx],
                    &self.scoring,
                );
                &*m
            });

        self.insert_match(this_key, best_match);

        best_match
    }

    fn insert_match(&mut self, key: (u32, u32, u32), m: Option<&'bump Match<'bump>>) {
        //assert!(self.match_cache.capacity() > self.match_cache.len());
        self.match_cache.insert(key, m);
    }

    fn match_calc_score(&self, consecutive: u32) -> isize {
        consecutive as isize * self.scoring.bonus_consecutive
    }
}
