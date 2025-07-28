use bumpalo::{Bump, vec};
use hashbrown::HashMap;

use crate::matching::Match;
use crate::parsing::{
    Occurrence, Occurrences, QueryChar, QueryChars, build_occurrences, process_query,
};
use crate::scoring::{DEFAULT_SCORING, Scoring};

/// Describes a fuzzy search. Alternative to [`best_match`](crate::best_match) which allows for more configuration.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use sublime_fuzzy::{FuzzySearch, Scoring};
///
/// let scoring = Scoring::emphasize_word_starts();
///
/// let result = FuzzySearch::new("something", "Some Search Thing")
///     .score_with(&scoring)
///     .case_insensitive()
///     .best_match();
///
/// assert!(result.is_some());
/// ```
pub struct FuzzySearch<'a> {
    query: &'a str,
    target: &'a str,
    scoring: Option<&'a Scoring>,
    case_insensitive: bool,
}

impl<'a> FuzzySearch<'a> {
    /// Creates a new search to match `query` in `target`.
    ///
    /// Note that whitespace in query will be _ignored_.
    pub fn new(query: &'a str, target: &'a str) -> Self {
        FuzzySearch {
            query,
            target,
            scoring: None,
            case_insensitive: true,
        }
    }

    /// Use custom scoring values.
    ///
    /// If not specified will use `Scoring::default()`.
    pub fn score_with(mut self, scoring: &'a Scoring) -> Self {
        self.scoring = Some(scoring);

        self
    }

    /// Only match query chars in the target string if case matches.
    ///
    /// [`Scoring::bonus_match_case`] will not be applied if this is set (because a char match will
    /// always also be a case match).
    pub fn case_sensitive(mut self) -> Self {
        self.case_insensitive = false;

        self
    }

    /// Ignore case when matching query chars in the target string.
    ///
    /// If not only the char but also the case matches, [`Scoring::bonus_match_case`] will be added to
    /// the score. If that behavior is not wanted the bonus can be set to 0 with custom scoring.
    pub fn case_insensitive(mut self) -> Self {
        self.case_insensitive = true;

        self
    }

    /// Finds the best match of the query in the target string.
    ///
    /// Always tries to match the _full_ pattern. A partial match is considered
    /// invalid and will return [`None`]. Will also return [`None`] in case the query or
    /// target string are empty.
    pub fn best_match<'bump>(self, bump: &'bump Bump) -> Option<&'bump Match<'bump>> {
        let processed_query = process_query(self.query);

        if processed_query.len() == 0 || self.target.len() == 0 {
            return None;
        }

        let occurrences = build_occurrences(&processed_query, self.target, self.case_insensitive);

        let searcher = FuzzySearcher::new(
            bump,
            processed_query,
            self.scoring.unwrap_or(&DEFAULT_SCORING),
            self.case_insensitive,
            self.target.len(),
        );

        searcher.best_match(&occurrences)
    }
}

struct FuzzySearcher<'a, 'bump> {
    bump: &'bump Bump,
    query: QueryChars,
    scoring: &'a Scoring,
    match_cache: HashMap<
        (u32, u32, u32),
        Option<&'bump Match<'bump>>,
        hashbrown::DefaultHashBuilder,
        &'bump Bump,
    >,
    case_insensitive: bool,
}

//pub static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
//pub static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);

impl<'a, 'bump> FuzzySearcher<'a, 'bump> {
    fn new(
        bump: &'bump Bump,
        query: QueryChars,
        scoring: &'a Scoring,
        case_insensitive: bool,
        haystack_len: usize,
    ) -> Self {
        FuzzySearcher {
            bump,
            // match_cache: HashMap::with_capacity_in(query.len() * query.len() * haystack_len, bump),
            match_cache: HashMap::with_capacity_in(query.len() * query.len(), bump),
            query,
            scoring,
            case_insensitive,
        }
    }

    #[inline(always)]
    fn queried_char(&self, qc: &QueryChar) -> char {
        if self.case_insensitive {
            qc.lower
        } else {
            qc.original
        }
    }

    #[inline(always)]
    fn case_bonus(&self, query_idx: u32, occurrence: &Occurrence) -> isize {
        if self.case_insensitive {
            self.query
                .get(query_idx as usize)
                .map_or(0, |c| (c.original == occurrence.char()) as isize)
                * self.scoring.bonus_match_case
        } else {
            0
        }
    }

    fn best_match(mut self, occurrences: &Occurrences) -> Option<&'bump Match<'bump>> {
        let qc = self.query.get(0)?;

        occurrences
            .get(&self.queried_char(qc))?
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
            //CACHE_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return *cached;
        } else {
            //CACHE_MISSES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        let Some(next_char) = self.query.get(query_idx as usize) else {
            // Successfully matched all query chars

            let this_match = self.bump.alloc(Match::with_matched(
                self.match_calc_score(query_idx, occurrence, consecutive),
                consecutive,
                vec![in &self.bump; occurrence.target_idx],
            ));

            self.insert_match(this_key, Some(this_match));

            return Some(this_match);
        };

        let Some(occs) = occurrences.get(&self.queried_char(next_char)) else {
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
                    self.match_calc_score(query_idx, occurrence, consecutive),
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

    fn match_calc_score(&self, query_idx: u32, occurrence: &Occurrence, consecutive: u32) -> isize {
        consecutive as isize * self.scoring.bonus_consecutive
            + occurrence.is_start() as isize * self.scoring.bonus_word_start
            + self.case_bonus(query_idx - 1, occurrence)
    }
}
