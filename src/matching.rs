use std::cmp::Ordering;

use bumpalo::collections::Vec;

use crate::Scoring;

type Uint = u32;

/// A (possible partial) match of query within the target string. Matched chars
/// are stored as indices into the target string.
///
/// The score is not clamped to any range and can be negative.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Match<'bump> {
    /// Accumulative score
    score: isize,
    /// Count of current consecutive matched chars
    consecutive: Uint,
    /// Matched char indices. Filled from back to front.
    //matched: Box<[Uint]>,
    /// Index of the first match index in the matched array.
    //matched_start: Uint,
    /// Matched char indices. Elements are in reverse order.
    matched: Vec<'bump, Uint>,
}

impl<'bump> Match<'bump> {
    /// Creates a new match with the given scoring and matched indices.
    ///
    /// Panics if `matched` is empty.
    #[inline]
    pub(crate) fn with_matched(
        score: isize,
        consecutive: Uint,
        mut matched: Vec<'bump, Uint>,
    ) -> Self {
        matched.reverse();
        Match {
            score,
            consecutive,
            // FIXME: error on overflow
            //matched: vec![matched; matched as usize + 1].into_boxed_slice(),
            //matched_start: matched,
            matched,
        }
    }

    /// Returns the accumulative score for this match.
    pub fn score(&self) -> isize {
        self.score
    }

    /// Returns an iterator over the matched char indices.
    pub fn matched_indices(&self) -> std::iter::Rev<std::slice::Iter<'_, Uint>> {
        //self.matched[self.matched_start as usize..].iter()
        self.matched.iter().rev()
    }

    /// Returns an iterator that groups the individual char matches into groups.
    pub fn continuous_matches(&self) -> ContinuousMatches {
        ContinuousMatches {
            //matched: &self.matched[self.matched_start as usize..],
            matched: &self.matched,
            current: 0,
        }
    }

    /// Extends this match with `other`.
    pub fn prepend(&mut self, score: isize, consecutive: Uint, indices: &[u32], scoring: &Scoring) {
        self.score += score;
        self.consecutive += consecutive;

        // remember, these arrays are in reverse order.
        let (last, first) = (indices[0], self.matched.last().unwrap());
        let distance = first - last;

        match distance {
            0 => {}
            1 => {
                self.consecutive += 1;
                self.score += self.consecutive as isize * scoring.bonus_consecutive;
            }
            _ => {
                self.consecutive = 0;
                let penalty = (distance as isize - 1) * scoring.penalty_distance;
                self.score -= penalty;
            }
        }

        self.matched.extend_from_slice(indices);
    }

    pub fn prepend_match(&mut self, other: &Match, scoring: &Scoring) {
        self.prepend(other.score, other.consecutive, &other.matched, scoring);
    }

    pub fn extend_with(&mut self, mut other: Match<'bump>, scoring: &Scoring) {
        other.prepend_match(self, scoring);
        *self = other;
    }
}

impl Ord for Match<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for Match<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Match<'_> {}

impl PartialEq for Match<'_> {
    fn eq(&self, other: &Match) -> bool {
        self.score == other.score
    }
}

/// Describes a continuous group of char indices
#[derive(Debug, Clone, Copy)]
pub struct ContinuousMatch {
    start: Uint,
    len: Uint,
}

impl ContinuousMatch {
    pub(crate) fn new(start: Uint, len: Uint) -> Self {
        ContinuousMatch { start, len }
    }

    /// Returns the start index of this group.
    pub fn start(&self) -> Uint {
        self.start
    }

    /// Returns the length of this group.
    pub fn len(&self) -> Uint {
        self.len
    }
}

impl Eq for ContinuousMatch {}

impl PartialEq for ContinuousMatch {
    fn eq(&self, other: &ContinuousMatch) -> bool {
        self.start == other.start && self.len == other.len
    }
}

/// Iterator returning [`ContinuousMatch`]es from the matched char indices in a [`Match`]
pub struct ContinuousMatches<'a> {
    matched: &'a [Uint],
    current: usize,
}

impl<'a> Iterator for ContinuousMatches<'_> {
    type Item = ContinuousMatch;

    fn next(&mut self) -> Option<ContinuousMatch> {
        let mut start = None;
        let mut len = 0;

        let mut last_idx = None;

        for &idx in self.matched.iter().rev().skip(self.current) {
            start = start.or(Some(idx));

            if last_idx.is_some() && (idx - last_idx.unwrap() != 1) {
                return Some(ContinuousMatch::new(start.unwrap(), len));
            }

            self.current += 1;
            len += 1;
            last_idx = Some(idx);
        }

        if last_idx.is_some() {
            return Some(ContinuousMatch::new(start.unwrap(), len));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use bumpalo::collections::{CollectIn, Vec};
    use bumpalo::{Bump, vec};

    use crate::Scoring;

    use super::{ContinuousMatch, Match};

    #[test]
    fn continuous() {
        let bump = Bump::new();
        let m = Match::with_matched(0, 0, vec![in &bump; 0, 1, 2, 5, 6, 10]);

        assert_eq!(
            m.continuous_matches()
                .collect_in::<Vec<ContinuousMatch>>(&bump),
            vec![in &bump;
                ContinuousMatch { start: 0, len: 3 },
                ContinuousMatch { start: 5, len: 2 },
                ContinuousMatch { start: 10, len: 1 },
            ]
        )
    }

    #[test]
    fn extend_match() {
        let bump = Bump::new();
        let mut a = Match::with_matched(16, 3, vec![in &bump; 1, 2, 3]);
        let b = Match::with_matched(8, 3, vec![in &bump; 5, 6, 7]);

        let s = Scoring::default();

        a.extend_with(b, &s);

        assert_eq!(a.score(), 24 - s.penalty_distance);
        assert_eq!(a.consecutive, 0);
        assert_eq!(a.matched_indices().len(), 6);
    }

    #[test]
    fn extend_match_cont() {
        let bump = Bump::new();
        let mut a = Match::with_matched(16, 3, vec![in &bump; 1, 2, 3]);
        let b = Match::with_matched(8, 3, vec![in &bump; 4, 5, 6]);

        let s = Scoring::default();

        a.extend_with(b, &s);

        assert_eq!(a.score(), 16 + 8 + (3 + 3 + 1) * s.bonus_consecutive);
        assert_eq!(a.consecutive, 3 + 3 + 1);
        assert_eq!(a.matched_indices().len(), 6);
    }
}
